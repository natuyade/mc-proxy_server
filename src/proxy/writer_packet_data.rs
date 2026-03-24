// 非同期書きこみメソッドを使うための拡張機能
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use crate::write_varint_to_stream;

pub async fn write_packet_data(stream: &mut TcpStream, packet_data: &[u8]) -> std::io::Result<()> {

    // packet_lengthを送信
    write_varint_to_stream(stream, packet_data.len() as i32).await?;

    // length(varint)を送信出来たら次はclientから受け取ったraw dataをサーバーに送信
    stream.write_all(packet_data).await?;

    Ok(())
}