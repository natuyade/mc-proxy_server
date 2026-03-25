use crate::{parse_login_start_payload_with_mcid_uuid, ConnectionContext};
use std::io::{Error, ErrorKind};
use crate::proxy::proxy_main::PlayerData;

pub fn handle_login_packet(packet_id: i32, payload_slice: &[u8], ctx: &ConnectionContext) -> std::io::Result<PlayerData> {

    match ctx.protocol_version {
        Some(v) => v,
        None => {
            return Err(Error::new(ErrorKind::NotFound, "unexpected protocol_version"))
        }
    };

    match packet_id {
        0x00 => {

            let payload = parse_login_start_payload_with_mcid_uuid(payload_slice)?;

            let mut warning = false;
            if payload.used_bytes != payload_slice.len(){
                warning = true;
            }

            let login_data = PlayerData {
                player_id: payload.minecraft_id,
                player_uuid: payload.uuid,
                payload_warning: warning,
                payload_used_bytes: payload.used_bytes,
            };
            Ok(login_data)
        }
        _ => {
            Err(Error::new(ErrorKind::InvalidData, format!("unexpected packet in Login state: id = 0x{packet_id:02X}")))
        }
    }
}