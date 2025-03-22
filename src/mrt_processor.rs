use bgpkit_parser::BgpkitParser;
use chrono::{DateTime, Utc};
use std::{collections::HashMap, net::IpAddr, path::Path};
use crate::bgp_state::{BgpKitStateExt, BgpState, ConnectionState};
use crate::util::{mrt_record_ts, DateTimeExt};

/// Represents an IP prefix (address + prefix length)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BgpPeer {
    /// IP address of the peer
    pub address: IpAddr,
    /// AS-number in 4 bytes
    pub peer_as: u32,
}

impl BgpPeer {
    /// From BgpElem to peer data
    pub fn from_elem(elem: &bgpkit_parser::models::BgpElem) -> Self {
        BgpPeer {
            address: elem.peer_ip,
            peer_as: elem.peer_asn.to_u32(),
        }
    }
}

/// Processor for MRT (Multi-threaded Routing Toolkit) files
pub struct MrtProcessor {
    current_state: HashMap<BgpPeer, BgpState>,
    send_hold_time_multiple: Option<u16>,
    default_hold_time: u16,
}

impl MrtProcessor {
    /// Create a new MRT processor
    pub fn new(default_hold_time: u16, send_hold_time_multiple: Option<u16>) -> Self {
        MrtProcessor {
            current_state: HashMap::new(),
            send_hold_time_multiple,
            default_hold_time
        }
    }

    pub fn default() -> Self {
        MrtProcessor::new(180, None)
    }

    pub fn process_bview<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(),  Box<dyn std::error::Error>> {
        let file_str = file_path.as_ref().display().to_string();
        log::info!("Processing bview: {}", file_str);

        // Clear the current state
        self.current_state.clear();

        let parser = BgpkitParser::new(file_path.as_ref().to_str().unwrap())?;
        for elem in parser {
            let peer = BgpPeer::from_elem(&elem);

            let peer_state = self.current_state.entry(peer).or_insert_with(BgpState::new);
            match elem.elem_type {
                bgpkit_parser::models::ElemType::ANNOUNCE => {
                    peer_state.update_prefix(elem);
                },
                bgpkit_parser::models::ElemType::WITHDRAW => {
                    peer_state.withdraw_prefix(elem.timestamp, elem.prefix);
                },
            }
        }

        Ok(())
    }

    /// Process an MRT file at the given path
    pub fn process_update_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(),  Box<dyn std::error::Error>> {
        let file_str = file_path.as_ref().display().to_string();
        log::info!("Processing update file: {}", file_str);

        // Create a parser for the MRT file
        let parser = BgpkitParser::new(file_path.as_ref().to_str().unwrap())?;

        // Last timestamp seen over all peers
        let mut last_ts: Option<DateTime<Utc>> = None;

        // Iterate over BGP messages in the file
        for record in parser.into_record_iter() {
            let ts = mrt_record_ts(&record);
            last_ts = last_ts.map(|old| old.max(ts)).or_else(|| Some(ts));

            match record.message {
                bgpkit_parser::models::MrtMessage::Bgp4Mp(msg) => {
                    match msg {
                        bgpkit_parser::models::Bgp4MpEnum::Message(msg) => {
                            let peer = BgpPeer {
                                address: msg.peer_ip,
                                peer_as: msg.peer_asn.to_u32(),
                            };
                            let peer_state = self.current_state.entry(peer).or_insert_with(BgpState::new);

                            match msg.bgp_message {
                                bgpkit_parser::models::BgpMessage::Open(bgp_open_message) => {
                                    log::debug!("{}: Received open message from peer: {:?}", ts, bgp_open_message);
                                    if !bgp_open_message.opt_params.is_empty() {
                                        log::info!("[{}/{}] OPEN: {:?}", msg.peer_ip, msg.peer_asn, bgp_open_message.opt_params);
                                    }
                                    peer_state.open_message(ts, bgp_open_message);
                                },
                                bgpkit_parser::models::BgpMessage::Update(bgp_update_message) => {
                                    // Construct the BgpElems from the BgpUpdateMessage
                                    // TODO: Construct only the updates, use the withdraws based on the information already available.
                                    let elements = bgpkit_parser::Elementor::bgp_update_to_elems(bgp_update_message, ts.to_timestamp_f64(), &msg.peer_ip, &msg.peer_asn);

                                    for elem in elements {
                                        match elem.elem_type {
                                            bgpkit_parser::models::ElemType::ANNOUNCE => {
                                                peer_state.update_prefix(elem);
                                            },
                                            bgpkit_parser::models::ElemType::WITHDRAW => {
                                                peer_state.withdraw_prefix(elem.timestamp, elem.prefix);
                                            },
                                        }
                                    }
                                },
                                bgpkit_parser::models::BgpMessage::KeepAlive => {
                                    // Update last timestamp for peer
                                    peer_state.update_last_message_timestamp(ts);
                                },
                                bgpkit_parser::models::BgpMessage::Notification(bgp_notification_message) => {
                                    log::debug!("{}: Received notification message from peer: {:?}", ts, bgp_notification_message);
                                    // Move state to idle.
                                    peer_state.update_connection_state(ts, ConnectionState::Idle);
                                }
                            }
                        },
                        bgpkit_parser::models::Bgp4MpEnum::StateChange(msg) => {
                            let peer = BgpPeer {
                                address: msg.peer_addr,
                                peer_as: msg.peer_asn.to_u32(),
                            };
                            let peer_state = self.current_state.entry(peer).or_insert_with(BgpState::new);
                            peer_state.update_connection_state(ts, msg.new_state.to_connection_state());
                        },

                    }
                },
                _ => {
                    return Err(format!("Unsupported content: Update file {file_str} might be a bview.").into());
                }
            }
        }

        // Check peers for validity
        if let Some(last_ts) = last_ts {
            for (peer, state) in self.current_state.iter_mut() {
                if state.connection_state == ConnectionState::Idle {
                    continue;
                }
                // Only for peers that have last messages
                if let Some(last_message_ts) = state.last_message_timestamp {
                    let hold_time = match state.hold_time {
                        Some(hold_time) => hold_time,
                        None => {
                            log::warn!("{}: Peer {:?} does not have a hold time - no open message.", last_ts, peer);
                            self.default_hold_time
                        }
                    };

                    let effective_hold_time = self.send_hold_time_multiple.unwrap_or(1) * hold_time;
                    let cutoff = last_ts + chrono::Duration::seconds(effective_hold_time as i64);

                    if last_message_ts < cutoff {
                        log::info!("Hold timer expired for {:?}, last message at {} (cutoff: {}), resetting state to idle.", peer, last_message_ts, cutoff);
                        state.update_connection_state(last_ts, ConnectionState::Idle);
                    }
                }
            }
        }

        log::info!("Finished processing file: {}", file_path.as_ref().display());
        Ok(())
    }

    /// Get the current BGP state
    pub fn get_current_state(&self) -> &HashMap<BgpPeer, BgpState> {
        &self.current_state
    }
}
