use std::{collections::HashMap, net::IpAddr};

use bgpkit_parser::models::NetworkPrefix;
use chrono::NaiveDateTime;

#[derive(Eq, Hash, PartialEq)]
struct PeerPrefix {
    peer_ip: IpAddr,
    prefix: NetworkPrefix,
}

impl PeerPrefix {
    fn new(peer_ip: IpAddr, prefix: NetworkPrefix) -> Self {
        Self {
            peer_ip,
            prefix,
        }
    }
}



#[derive(Default)]
pub struct AnnouncementTracker {
    announement_start: HashMap<PeerPrefix, NaiveDateTime>,
}


impl AnnouncementTracker {
    pub fn add_announcement(&mut self, peer_ip: IpAddr, prefix: NetworkPrefix, ts: NaiveDateTime) -> Option<NaiveDateTime> {
        // Find or create the entry for this peer/prefix pair
        let key = PeerPrefix::new(peer_ip, prefix);
        let initial: Option<NaiveDateTime> = self.announement_start.get(&key).cloned();

        self.announement_start.insert(key, ts);
        initial
    }

    pub fn withdraw_announcement(&mut self, peer_ip: IpAddr, prefix: NetworkPrefix) -> Option<NaiveDateTime> {
        self.announement_start
            .remove(&PeerPrefix::new(peer_ip, prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;
    use chrono::{DateTime, Days};

    #[test]
    fn test_announcement_tracker() {
        let mut tracker = AnnouncementTracker::default();

        // Test data
        let peer_ip = IpAddr::from_str("192.0.2.1").unwrap();
        let prefix = NetworkPrefix::from_str("192.0.2.0/24").unwrap();
        let ts_before = DateTime::from_timestamp(1600000000, 0).unwrap().naive_utc();
        let ts = ts_before.checked_add_days(Days::new(1)).unwrap();

        // Test adding announcement
        assert_eq!(tracker.add_announcement(peer_ip, prefix, ts_before), None);
        assert_eq!(tracker.announement_start.get(&PeerPrefix::new(peer_ip, prefix)), Some(&ts_before));
        // Test adding an updated timestamp
        assert_eq!(tracker.add_announcement(peer_ip, prefix, ts), Some(ts_before));
        assert_eq!(tracker.announement_start.get(&PeerPrefix::new(peer_ip, prefix)), Some(&ts));

        // Test withdrawing announcement
        let withdrawn_ts = tracker.withdraw_announcement(peer_ip, prefix);
        assert_eq!(withdrawn_ts, Some(ts));
        assert!(tracker.announement_start.is_empty());

        // Test withdrawing non-existent announcement
        let non_existent = tracker.withdraw_announcement(peer_ip, prefix);
        assert_eq!(non_existent, None);
    }
}