use bgpkit_parser::{BgpkitParser, Elementor, ParserError};
use std::{collections::HashMap, net::IpAddr, path::Path};
use crate::bgp_state::{Announcement, BgpKitStateExt, BgpState, ConnectionState};
use crate::util::{mrt_record_ts, DateTimeExt};

/// Processor for MRT (Multi-threaded Routing Toolkit) files
pub struct MrtProcessor {
    current_state: HashMap<BgpPeer, BgpState>,
}

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

impl MrtProcessor {
    /// Create a new MRT processor
    pub fn new() -> Self {
        MrtProcessor {
            current_state: HashMap::new(),
        }
    }

    pub fn process_bview<P: AsRef<Path>>(&mut self, file_path: P) -> Result<(),  Box<dyn std::error::Error>> {
        let file_str = file_path.as_ref().display().to_string();
        println!("Processing bview: {}", file_str);

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
        println!("Processing update file: {}", file_str);

        // Create a parser for the MRT file
        let parser = BgpkitParser::new(file_path.as_ref().to_str().unwrap())?;

        // Iterate over BGP messages in the file
        for record in parser.into_record_iter() {
            let ts = mrt_record_ts(&record);

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
                                    log::info!("{}: Received open message from peer: {:?}", ts, bgp_open_message);
                                    // Move state to established.
                                    peer_state.update_connection_state(ts, ConnectionState::Established);

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
                                    log::info!("{}: Received notification message from peer: {:?}", ts, bgp_notification_message);
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

        println!("Finished processing file: {}", file_path.as_ref().display());
        Ok(())
    }

    /// Get the current BGP state
    pub fn get_current_state(&self) -> &HashMap<BgpPeer, BgpState> {
        &self.current_state
    }
}
