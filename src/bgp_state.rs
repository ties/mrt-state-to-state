use std::collections::HashMap;
use std::net::IpAddr;
use std::time::SystemTime;

/// Represents the state of a BGP connection
#[derive(Debug, Clone)]
pub struct BgpState {
    /// The current state of the BGP connection (e.g. Established, Active, etc.)
    pub connection_state: ConnectionState,
    /// Timestamp of the last received message
    pub last_message_timestamp: SystemTime,
    /// Map from IP prefix to the last announcement for that prefix
    pub prefix_announcements: HashMap<IpPrefix, Announcement>,
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

/// Represents an IP prefix (address + prefix length)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IpPrefix {
    pub address: IpAddr,
    pub prefix_length: u8,
}

/// Represents a BGP route announcement
#[derive(Debug, Clone)]
pub struct Announcement {
    /// Timestamp when the announcement was received
    pub timestamp: SystemTime,
    /// AS path for this route
    pub as_path: Vec<u32>,
    /// Next hop for this route
    pub next_hop: IpAddr,
    /// Additional BGP attributes
    pub attributes: HashMap<String, String>,
}

impl BgpState {
    /// Creates a new BGP state with default values
    pub fn new() -> Self {
        BgpState {
            connection_state: ConnectionState::Idle,
            last_message_timestamp: SystemTime::now(),
            prefix_announcements: HashMap::new(),
        }
    }

    /// Updates the connection state and timestamp
    pub fn update_connection_state(&mut self, new_state: ConnectionState) {
        self.connection_state = new_state;
        self.last_message_timestamp = SystemTime::now();
    }

    /// Adds or updates an announcement for a prefix
    pub fn update_prefix(&mut self, prefix: IpPrefix, announcement: Announcement) {
        self.prefix_announcements.insert(prefix, announcement);
        self.last_message_timestamp = SystemTime::now();
    }
}