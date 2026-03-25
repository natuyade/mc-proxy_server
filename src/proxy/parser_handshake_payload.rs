use std::io::{Error, ErrorKind};
use std::str::from_utf8;
use crate::{read_varint_from_packet, HandShakePayload};

pub fn parse_handshake_payload(payload_data: &[u8]) -> std::io::Result<HandShakePayload> {

    let mut index = 0;

    // read_varintで範囲チェックを兼用しているため,それ以外の型では範囲を先に見る
    // varint
    let (protocol_version, version_used) = read_varint_from_packet(&payload_data[index..])?;
    index += version_used;

    // varint
    let (address_length, length_used) = read_varint_from_packet(&payload_data[index..])?;
    index += length_used;

    // server_address用範囲チェック
    // usize変換するので, 負値i32だった場合のoverflowを防ぐ
    if address_length < 0 {
        return Err(Error::new(ErrorKind::InvalidData, "negative address length"))
    }
    let address_length = address_length as usize;
    if payload_data.len() < index + address_length {
        return Err(Error::new(ErrorKind::InvalidData, "payload data(status) didn't have enough space"))
    }
    let address_parts = &payload_data[index..index + address_length];
    index += address_parts.len();

    let server_address = match from_utf8(address_parts) {
        Ok(s) => s.to_string(),
        Err(_) => {
            return Err(Error::new(ErrorKind::InvalidData, "server_address is not valid UTF-8"))
        }
    };

    // server_port用範囲チェック
    if payload_data.len() < index + 2 {
        return Err(Error::new(ErrorKind::InvalidData, "payload data(status) didn't have enough space"))
    }
    let server_port = u16::from_be_bytes([payload_data[index], payload_data[index + 1]]);
    index += 2;

    // varint
    let (next_state, next_state_used) = read_varint_from_packet(&payload_data[index..])?;
    index += next_state_used;

    let packet_payload = HandShakePayload {
        protocol_version,
        server_address,
        _server_port: server_port,
        next_state,
        used_bytes: index,
    };

    Ok(packet_payload)
}