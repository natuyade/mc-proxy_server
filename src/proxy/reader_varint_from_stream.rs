use std::io::{Error, ErrorKind};
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;

// server間でioしているためasync化
pub async fn read_varint_from_stream(stream: &mut TcpStream) -> std::io::Result<Option<i32>> {

    let mut result = 0i32;

    // VarIntは最大5bytesなだけであって, 確定要素ではないためfor return
    for i in 0..5 {
        let mut part = [0u8; 1];

        match stream.read_exact(&mut part).await {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof && i == 0 => return Ok(None),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Err(Error::new(ErrorKind::UnexpectedEof, "User disconnected while reading varint")),
            Err(e) => return Err(e),
        };
        let byte = part[0];

        let value = byte & 0b0111_1111;

        result |= (value as i32) << (7 * i);

        if byte & 0b1000_0000 == 0 {
            return Ok(Some(result))
        }
    }
    // 6byte目に到達したならエラー
    Err(Error::new(ErrorKind::InvalidData, "Too Long VarInt"))
}