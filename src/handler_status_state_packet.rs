use crate::{parse_status_ping_payload, ConnectionContext};
use std::io::{Error, ErrorKind};

pub fn handle_status_packet(packet_id: i32, payload_slice: &[u8], ctx: &ConnectionContext) -> std::io::Result<()> {

    println!("Status state packet observed: id = 0x{packet_id:02X}");
    println!();

    let protocol_version = match ctx.protocol_version {
        Some(v) => v,
        None => {
            return Err(Error::new(ErrorKind::NotFound, "unexpected protocol_version"))
        }
    };
    println!("protocol_version from context: {protocol_version}");
    
    match packet_id {
        0x00 => {
            println!("Status Request packet candidate");
            println!();

            if payload_slice.is_empty() {
                println!("valid Status Request payload length")
            } else {
                return Err(Error::new(ErrorKind::InvalidData, format!("invalid Status Request payload length: expected 0, got {}", payload_slice.len())))
            }
            Ok(())
        }
        0x01 => {
            println!("Status Ping packet candidate");
            println!();

            if payload_slice.len() == 8 {
                println!("valid Status Ping payload length")
            } else {
                return Err(Error::new(ErrorKind::InvalidData, format!("invalid Status Ping payload length: expected 8, got {}", payload_slice.len())))
            }

            let payload = parse_status_ping_payload(payload_slice)?;
            println!("{payload}");
            Ok(())
        }
        _ => {
            Err(Error::new(ErrorKind::InvalidData, format!("unexpected packet in Status state: id = 0x{packet_id:02X}")))
        }
    }
}