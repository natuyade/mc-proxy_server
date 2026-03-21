use std::io::{Error, ErrorKind};
use std::str::from_utf8;
use uuid::Uuid;
use crate::{read_varint_from_packet, LoginStatePayload};

// parser for NameLength(VarInt) + mcid(UTF-8) + uuid(16-byte)
pub fn parse_login_start_payload_with_mcid_uuid(payload_data: &[u8]) -> std::io::Result<LoginStatePayload> {

    let mut index = 0;

    let (name_length, length_used) = read_varint_from_packet(&payload_data[index..])?;
    index += length_used;


    if name_length < 0 {
        return Err(Error::new(ErrorKind::InvalidData, "negative name length"))
    }
    // 負の値でなければ通過させusize変換
    let name_length = name_length as usize;
    if payload_data.len() < index + name_length{
        return Err(Error::new(ErrorKind::InvalidData, "payload data(login mcid) didn't have enough space"))
    }

    let id_parts = &payload_data[index..index + name_length];
    index += name_length;

    let minecraft_id = match from_utf8(id_parts) {
        Ok(id) => id.to_string(),
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, "minecraft_id is not valid UTF-8"))
    };

    // uuid = 16bytes固定, なければ弾く
    if payload_data.len() < index + 16 {
        return Err(Error::new(ErrorKind::InvalidData, "payload data(login uuid) didn't have enough space"))
    }

    if payload_data[index..].len() != 16 {
        return Err(Error::new(ErrorKind::InvalidData, "payload data(login) unexpected trailing bytes after login uuid"))
    }

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&payload_data[index..index + 16]);
    index += 16;

    let uuid = Uuid::from_bytes(bytes);

    let packet_payload = LoginStatePayload {
        minecraft_id,
        uuid,
        used_bytes: index,
    };

    Ok(packet_payload)
    // "uuid":"66567bfd-1082-476f-b001-33ffe90c222a","name":"natuyade"
}