use std::net::IpAddr;

use bgpkit_parser::MrtRecord;
use chrono::{DateTime, Utc};

pub fn ip_address_to_v8(ip: IpAddr) -> Vec<u8> {
    match ip {
        IpAddr::V4(ip) => ip.octets().to_vec(),
        IpAddr::V6(ip) => ip.octets().to_vec(),
    }
}

/// Parse a single key-value pair
pub fn parse_key_value(s: &str) -> Result<(String, String), String>
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;

    let file = &s[pos + 1..];
    let rrc = &s[..pos];
    Ok((rrc.to_string(), file.to_string()))
}

pub fn mrt_record_ts(record: &MrtRecord) -> DateTime<Utc> {
    match record.common_header.microsecond_timestamp {
        None => DateTime::from_timestamp(record.common_header.timestamp as i64, 0).unwrap(),
        Some(us) => DateTime::from_timestamp(record.common_header.timestamp as i64, 1000*us).unwrap(),
    }
}

/// Extension trait for DateTime<Utc> that adds conversion to f64 timestamp
pub trait DateTimeExt {
    /// Convert to seconds since epoch as f64
    fn to_timestamp_f64(&self) -> f64;
}

impl DateTimeExt for DateTime<Utc> {
    fn to_timestamp_f64(&self) -> f64 {
        self.timestamp() as f64 +
        self.timestamp_subsec_nanos() as f64 / 1_000_000_000.0
    }
}

// Then use it like:
// let timestamp = some_datetime.to_timestamp_f64();
