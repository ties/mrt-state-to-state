use core::fmt;
use std::collections::HashMap;
use std::net::IpAddr;
use bgpkit_parser::models::{AsPath, BgpElem, BgpOpenMessage, MetaCommunity, NetworkPrefix, OptParam, Origin};
use chrono::{DateTime, Utc};

/// Represents the state of a BGP connection
#[derive(Debug, Clone)]
pub struct BgpState {
    /// The current state of the BGP connection (e.g. Established, Active, etc.)
    connection_state: ConnectionState,
    /// Timestamp of the last received message
    last_message_timestamp: Option<DateTime<Utc>>,
    /// Map from IP prefix to the last announcement for that prefix
    prefix_announcements: HashMap<NetworkPrefix, Announcement>,
    /// Hold time from last open message
    hold_time: Option<u16>,
    /// BGP options
    options: Option<Vec<OptParam>>
}

/// Represents the possible states of a BGP connection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Idle,
    Connect,
    Active,
    OpenSent,
    OpenConfirm,
    Established,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Idle => write!(f, "Idle"),
            ConnectionState::Connect => write!(f, "Connect"),
            ConnectionState::Active => write!(f, "Active"),
            ConnectionState::OpenSent => write!(f, "OpenSent"),
            ConnectionState::OpenConfirm => write!(f, "OpenConfirm"),
            ConnectionState::Established => write!(f, "Established"),
        }
    }
}


pub trait BgpKitStateExt {
    fn to_connection_state(&self) -> ConnectionState;
}

impl BgpKitStateExt for bgpkit_parser::models::BgpState {
    fn to_connection_state(&self) -> ConnectionState {
        match self {
            bgpkit_parser::models::BgpState::Idle => ConnectionState::Idle,
            bgpkit_parser::models::BgpState::Connect => ConnectionState::Connect,
            bgpkit_parser::models::BgpState::Active => ConnectionState::Active,
            bgpkit_parser::models::BgpState::OpenSent => ConnectionState::OpenSent,
            bgpkit_parser::models::BgpState::OpenConfirm => ConnectionState::OpenConfirm,
            bgpkit_parser::models::BgpState::Established => ConnectionState::Established,
        }
    }
}

/// Represents a BGP route announcement
#[derive(Debug, Clone)]
pub struct Announcement {
    /// Timestamp when the announcement was received
    pub timestamp: DateTime<Utc>,
    /// AS path for this route
    pub as_path: Option<AsPath>,
    /// The origin type
    pub origin: Option<Origin>,
    /// Local Pref
    pub local_pref: Option<u32>,
    /// Next hop for this route
    pub next_hop: Option<IpAddr>,
    /// The MED
    pub med: Option<u32>,

    /// The communities
    pub communities: Option<Vec<MetaCommunity>>,

    pub only_to_customer: Option<u32>,
}

fn timestamp_to_datetime(timestamp: f64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp as i64, (timestamp.fract() * 1_000_000_000.0) as u32).unwrap()
}

impl Announcement {
    /// Create an announcement from a BgpElem
    pub fn from_bgp_elem(elem: BgpElem) -> Result<Self, &'static str> {
        // Check if this is an announcement (not a withdrawal)
        if !elem.is_announcement() {
            return Err("Cannot create announcement from withdrawal element");
        }

        // Convert timestamp from f64 to DateTime
        let timestamp = timestamp_to_datetime(elem.timestamp);

        Ok(Announcement {
            timestamp,
            as_path: elem.as_path.clone(),
            origin: elem.origin,
            local_pref: elem.local_pref,
            next_hop: elem.next_hop,
            med: elem.med,
            communities: elem.communities.clone(),
            only_to_customer: elem.only_to_customer.map(|v| v.to_u32()),
        })
    }
}

impl BgpState {
    /// Creates a new BGP state with default values
    pub fn new() -> Self {
        BgpState {
            connection_state: ConnectionState::Idle,
            last_message_timestamp: None,
            prefix_announcements: HashMap::new(),
            hold_time: None,
            options: None,
        }
    }

    pub fn open_message(&mut self, ts: DateTime<Utc>, msg: BgpOpenMessage) {
        self.update_connection_state(ts, ConnectionState::OpenSent);
        self.hold_time = Some(msg.hold_time);
        self.options = Some(msg.opt_params);
    }

    /// Updates the connection state and timestamp
    pub fn update_connection_state(&mut self, ts: DateTime<Utc>, new_state: ConnectionState) {
        match (&self.connection_state, &new_state) {
            (ConnectionState::Established, ConnectionState::Established) => {
                log::warn!("{}: Connection state changed from Established to Established for peer.", ts);
            },
            (_, ConnectionState::Established) => {
                log::warn!("{}: Connection state changed from {} to Established for peer.", ts, self.connection_state);
                self.prefix_announcements.clear();
            },
            _ => {
                self.prefix_announcements.clear();
            },
        }

        self.prefix_announcements.clear();
        self.connection_state = new_state;
        self.last_message_timestamp = Some(ts);
    }

    pub fn update_last_message_timestamp(&mut self, timestamp: DateTime<Utc>) {
        self.last_message_timestamp = self.last_message_timestamp
            .map(|ts| ts.max(timestamp))
            .or(Some(timestamp));
    }

    /// Adds or updates an announcement for a prefix
    pub fn update_prefix(&mut self, elem: BgpElem) {
        let prefix = elem.prefix;
        let announcement = Announcement::from_bgp_elem(elem).unwrap();

        self.update_last_message_timestamp(announcement.timestamp);
        self.prefix_announcements.insert(prefix, announcement);
    }

    pub fn withdraw_prefix(&mut self, ts: f64, prefix: NetworkPrefix) {
        self.update_last_message_timestamp(timestamp_to_datetime(ts));
        self.prefix_announcements.remove(&prefix);
    }
}
