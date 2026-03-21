use std::io::{Error, ErrorKind};

pub fn parse_status_ping_payload(payload_data: &[u8]) -> std::io::Result<i64> {

    if payload_data.len() != 8 {
        return Err(Error::new(ErrorKind::InvalidData, "status ping has wrong payload"))
    }

    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&payload_data[0..8]);

    let payload = i64::from_be_bytes(bytes);

    Ok(payload)
}