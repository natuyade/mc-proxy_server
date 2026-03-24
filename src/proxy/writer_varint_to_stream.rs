use tokio::net::TcpStream;
use std::io::{Error, ErrorKind};
use tokio::io::AsyncWriteExt;

// packet_lengthを送信する
pub async fn write_varint_to_stream(stream: &mut TcpStream, length: i32) -> std::io::Result<()> {

    if length < 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "found negative VarInt",
        ));
    }

    let mut value = length;

    for _ in 0..5 {

        let mut byte = (value & 0b0111_1111) as u8;

        // varintなので7ずつright shift
        value >>= 7;

        // valueの後ろにまだ値があるかの確認, あればbyteに継続bitを渡す
        if value != 0 {
            byte |= 0b1000_0000
        }

        stream.write_all(&[byte]).await?;

        // valueが空になれば終わり
        if value == 0 {
            return Ok(())
        }
    }

    Err(Error::new(ErrorKind::InvalidData, "VarInt encoding exceeded 5 bytes"))
}