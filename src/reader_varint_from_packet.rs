use std::io::{Error, ErrorKind};

pub fn read_varint_from_packet(data: &[u8]) -> std::io::Result<(i32, usize)> {

    let mut result = 0i32;
    let mut used = 0usize;

    for i in 0..5 {

        if data.len() <= i {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Incomplete VarInt"))
        }

        let byte = data[i];

        let value = byte & 0b0111_1111;

        result |= (value as i32) << (7 * i);

        used += 1;

        if byte & 0b1000_0000 == 0 {
            return Ok((result, used))
        }
    }

    Err(Error::new(ErrorKind::InvalidData, "Too Long VarInt"))
}