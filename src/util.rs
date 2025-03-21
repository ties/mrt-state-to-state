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
