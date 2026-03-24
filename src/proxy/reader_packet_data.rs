use std::io::{Error, ErrorKind};
use tokio::net::TcpStream;
// 非同期読みこみメソッドを使うための拡張機能
use tokio::io::AsyncReadExt;
use crate::read_varint_from_stream;

use crate::MAX_PACKET_SIZE;

// 最初のpacket_lengthを読む
pub async fn read_packet_data(stream: &mut TcpStream) -> std::io::Result<Option<Vec<u8>>> {

    let packet_length = match read_varint_from_stream(stream).await? {
        Some(len) => len,
        None => {
            return Ok(None)
        },
    };

    // 不正なpacket対策
    if packet_length < 0 {
        return Err(Error::new(ErrorKind::InvalidData, "negative packet length"))
    }
    if packet_length as usize > MAX_PACKET_SIZE {
        return Err(Error::new(ErrorKind::InvalidData, "packet too large"))
    }

    // 範囲指定でパケットを取得
    let mut packet_data: Vec<u8> = vec![0u8; packet_length as usize];

    match stream.read_exact(&mut packet_data).await {
        Ok(_) => Ok(Some(packet_data)),
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => Err(Error::new(ErrorKind::UnexpectedEof, "User disconnected while reading packet data")),
        Err(e) => Err(e),
    }
}