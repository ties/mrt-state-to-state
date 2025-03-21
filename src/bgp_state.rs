use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use bgpkit_parser::models::{AsPath, BgpElem, MetaCommunitiesIter, MetaCommunity, NetworkPrefix, Origin};
use chrono::{DateTime, Utc};

/// Represents the state of a BGP connection
#[derive(Debug, Clone)]
pub struct BgpState {
    /// The current state of the BGP connection (e.g. Established, Active, etc.)
    pub connection_state: ConnectionState,
    /// Timestamp of the last received message
    pub last_message_timestamp: Option<DateTime<Utc>>,
    /// Map from IP prefix to the last announcement for that prefix
    pub prefix_announcements: HashMap<NetworkPrefix, Announcement>,
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
            origin: elem.origin.clone(),
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
        }
    }

    /// Updates the connection state and timestamp
    pub fn update_connection_state(&mut self, new_state: ConnectionState) {
        self.connection_state = new_state;
        panic!("Need to switch based on new state.");
    }

    fn update_last_message_timestamp(&mut self, timestamp: DateTime<Utc>) {
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

    pub fn withdraw_prefix(&mut self, elem: BgpElem) {
        self.update_last_message_timestamp(timestamp_to_datetime(elem.timestamp));
        self.prefix_announcements.remove(&elem.prefix);
    }
}