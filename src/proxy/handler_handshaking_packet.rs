use std::io::{Error, ErrorKind};
use crate::{ConnectionContext, ConnectionState};
use crate::parse_handshake_payload;

pub fn handle_handshaking_packet(packet_id: i32, payload_slice: &[u8]) -> std::io::Result<ConnectionContext> {

    println!();
    println!("Handshake packet observed: id = 0x{packet_id:02X}");

    match packet_id {
        0x00 => {
            println!();
            println!();
            println!("Handshake!^_^)🤝(^.^");
            println!();

            let payload = parse_handshake_payload(payload_slice)?;

            // 読み終わったpayloadに余剰バイトがあるときの未読領域が存在するか確認
            if payload.used_bytes != payload_slice.len(){
                println!("warning:\npayload has trailing bytes\n[ total: {}, used: {} ]", payload_slice.len(), payload.used_bytes);
            }

            println!();
            println!("HandShake payload");
            println!("protocol version: {}", payload.protocol_version);
            println!("server address: {}", payload.server_address);
            println!("server port: {}", payload.server_port);

            let state = match payload.next_state {
                1 => {
                    println!("next state: {} = status", payload.next_state);
                    ConnectionState::Status
                }
                2 => {
                    println!("next state: {} = login", payload.next_state);
                    ConnectionState::Login
                }
                3 => {
                    println!("next state: {} = transfer", payload.next_state);
                    ConnectionState::Transfer
                }
                _ => {
                    println!("next state: {} = unknown", payload.next_state);
                    ConnectionState::Unknown
                }
            };

            let connection_context = ConnectionContext {
                state,
                protocol_version: Some(payload.protocol_version),
                server_address: Some(payload.server_address),
            };

            Ok(connection_context)
        }

        _ => {
            Err(Error::new(ErrorKind::InvalidData, format!("unexpected packet in HandShaking: id = 0x{packet_id:02X}")))
        }
    }
}