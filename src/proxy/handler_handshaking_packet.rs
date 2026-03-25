use std::io::{Error, ErrorKind};
use crate::{ConnectionContext, ConnectionState};
use crate::parse_handshake_payload;

pub fn handle_handshaking_packet(packet_id: i32, payload_slice: &[u8]) -> std::io::Result<ConnectionContext> {

    match packet_id {
        0x00 => {

            let payload = parse_handshake_payload(payload_slice)?;

            // 読み終わったpayloadに余剰バイトがあるときの未読領域が存在するか確認
            if payload.used_bytes != payload_slice.len(){
                return Err(Error::new(ErrorKind::InvalidData, format!(
                    "warning:\npayload has trailing bytes\n[ total: {}, used: {} ]",
                    payload_slice.len(), payload.used_bytes
                )));
            }

            let state = match payload.next_state {
                1 => ConnectionState::Status,
                2 => ConnectionState::Login,
                3 => ConnectionState::Transfer,
                _ => ConnectionState::Unknown,
            };

            let connection_context = ConnectionContext {
                state,
                protocol_version: Some(payload.protocol_version),
                server_address: Some(payload.server_address),
            };

            Ok(connection_context)
        }

        _ => {
            Err(Error::new(ErrorKind::InvalidData, "unexpected packet in HandShaking"))
        }
    }
}