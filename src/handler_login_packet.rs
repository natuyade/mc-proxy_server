use crate::{parse_login_start_payload_with_mcid_uuid, ConnectionContext};
use std::io::{Error, ErrorKind};

pub fn handle_login_packet(packet_id: i32, payload_slice: &[u8], ctx: &ConnectionContext) -> std::io::Result<()> {

    println!("Login state packet observed: id = 0x{packet_id:02X}");
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

            let payload = parse_login_start_payload_with_mcid_uuid(payload_slice)?;

            if payload.used_bytes != payload_slice.len(){
                println!("warning:\npayload has trailing bytes\n[ total: {}, used: {} ]", payload_slice.len(), payload.used_bytes);
            }
            println!();
            println!("Login start payload");
            println!("minecraft_id: {}", payload.minecraft_id);
            println!("uuid: {}", payload.uuid);

            Ok(())
        }
        _ => {
            Err(Error::new(ErrorKind::InvalidData, format!("unexpected packet in Login state: id = 0x{packet_id:02X}")))
        }
    }
}