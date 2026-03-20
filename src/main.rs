use std::io::ErrorKind;
use std::str::from_utf8;
// 非同期読み書きメソッドを使うための拡張機能
use tokio::io::AsyncReadExt;

use tokio::net::{TcpListener, TcpStream};

struct HandShakePayload {
    protocol_version: i32,
    server_address: String,
    server_port: u16,
    next_state: i32,
    used_bytes: usize,
}

const MAX_PACKET_SIZE: usize = 1024 * 1024;

// 非同期処理を行うためのtokioランタイムを作り, async fn mainを実行できる
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    // Listener::bindで全ての接続の待ち受けアドレスを指定
    let listener = TcpListener::bind("0.0.0.0:25566").await?;

    println!("================\n System Started\n================\n");

    // サーバーは接続を待ち続ける必要があるためloop
    loop {
        println!("Prepared for next connection\n");

        // listenerが待ち受けしているbindされたアドレスへ接続された際にlistener.accept()で
        // (TcpStream:それぞれの接続元と個別で読み書きするためのハンドル(ストリーム),
        // SocketAddr:接続元のアドレス)が返される
        let (mut socket, addr) = listener.accept().await?;
        println!("--------------------------------");
        println!("client connected {addr}");
        println!("--------------------------------");

        // tokio::spawnは中の処理を別のタスクで継続させ, 次の処理をすぐに実行できる
        // 今回ならaccept()を受けたらその接続先との処理をspawnで継続させつつ
        // すぐに次の接続を待ち受けれる
        tokio::spawn(
            async move {

                let packet_data = match read_packet_data(&mut socket).await {
                    Ok(d) => d,
                    Err(e) => { return println!("{e}") }
                };

                let (packet_id, id_used) = match read_varint_from_packet(&packet_data) {
                    Ok(id) => {id}
                    Err(e) => { return println!("{e}") }
                };
                println!();
                if packet_id != 0x00 {
                    println!("This is not handshake");
                    println!();
                    return;
                }
                println!("Handshake!^_^@@^.^");
                println!();

                let payload = match parse_handshake_payload(&packet_data[id_used..]) {
                    Ok(p) => p,
                    Err(e) => { return println!("{e}") }
                };

                println!("payload used {}bytes", payload.used_bytes);

                println!("protocol version: {}", payload.protocol_version);
                println!("server address: {}", payload.server_address);
                println!("server port: {}", payload.server_port);
                match payload.next_state {
                    1 => println!("next state: {} = status", payload.next_state),
                    2 => println!("next state: {} = login", payload.next_state),
                    3 => println!("next state: {} = transfer", payload.next_state),
                    _ => println!("next state: {} = unknown", payload.next_state),
                }
            }
        );
    }
}

// server間でioしているためasync化
async fn read_varint_from_stream(stream: &mut TcpStream) -> std::io::Result<i32> {

    let mut result = 0i32;

    // VarIntは最大5bytesなだけであって, 確定要素ではないためfor return
    for i in 0..5 {
        let mut part = [0u8; 1];

        stream.read_exact(&mut part).await?;
        let byte = part[0];

        let value = byte & 0b0111_1111;

        result |= (value as i32) << (7 * i);

        if byte & 0b1000_0000 == 0 {
            return Ok(result)
        }
    }
    // 6byte目に到達したならエラー
    Err(std::io::Error::new(ErrorKind::InvalidData, "Too Long VarInt"))
}

async fn read_packet_data(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {

    let packet_length = read_varint_from_stream(stream).await?;

    // 不正なpacket対策
    if packet_length < 0 {
        return Err(std::io::Error::new(ErrorKind::InvalidData, "negative packet length"))
    }
    if packet_length as usize > MAX_PACKET_SIZE {
        return Err(std::io::Error::new(ErrorKind::InvalidData, "packet too large"))
    }

    // 範囲指定でパケットを取得
    let mut packet_data: Vec<u8> = vec![0u8; packet_length as usize];
    stream.read_exact(&mut packet_data).await?;

    Ok(packet_data)
}

fn parse_handshake_payload(payload_data: &[u8]) -> std::io::Result<HandShakePayload> {

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
        return Err(std::io::Error::new(ErrorKind::InvalidData, "negative address length"))
    }
    if payload_data.len() < index + address_length as usize {
        return Err(std::io::Error::new(ErrorKind::InvalidData, "payload data(address_parts) didn't have enough space"))
    }
    let address_parts = &payload_data[index..index + address_length as usize];
    index += address_parts.len();

    let server_address = match from_utf8(address_parts) {
        Ok(s) => s.to_string(),
        Err(e) => {
            println!("{e}");
            return Err(std::io::Error::new(ErrorKind::InvalidData, "Couldn't convert to string from &str"))
        }
    };

    // server_port用範囲チェック
    if payload_data.len() < index + 2 {
        return Err(std::io::Error::new(ErrorKind::InvalidData, "payload data(server_port) didn't have enough space"))
    }
    let server_port = (payload_data[index + 1] as u16) | (payload_data[index] as u16) << 8;
    index += 2;

    // varint
    let (next_state, next_state_used) = read_varint_from_packet(&payload_data[index..])?;
    index += next_state_used;

    let packet_payload = HandShakePayload {
        protocol_version,
        server_address,
        server_port,
        next_state,
        used_bytes: index,
    };

    Ok(packet_payload)
}

fn read_varint_from_packet(data: &[u8]) -> std::io::Result<(i32, usize)> {

    let mut result = 0i32;
    let mut used = 0usize;

    for i in 0..5 {

        if data.len() <= i {
            return Err(std::io::Error::new(ErrorKind::UnexpectedEof, "Incomplete VarInt"))
        }

        let byte = data[i];

        let value = byte & 0b0111_1111;

        result |= (value as i32) << (7 * i);

        used += 1;

        if byte & 0b1000_0000 == 0 {
            return Ok((result, used))
        }
    }

    Err(std::io::Error::new(ErrorKind::InvalidData, "Too Long VarInt"))
}