mod reader_varint_from_stream;
mod reader_packet_data;
mod reader_varint_from_packet;
mod parser_handshake_payload;
mod parser_status_ping_payload;
mod parser_login_payload;
mod handler_status_state_packet;
mod handler_login_packet;
mod handler_handshaking_packet;
mod writer_varint_to_stream;
mod writer_packet_data;

use reader_varint_from_stream::read_varint_from_stream;
use reader_packet_data::read_packet_data;
use reader_varint_from_packet::read_varint_from_packet;
use parser_handshake_payload::parse_handshake_payload;
use parser_status_ping_payload::parse_status_ping_payload;
use parser_login_payload::parse_login_start_payload_with_mcid_uuid;
use handler_status_state_packet::handle_status_packet;
use handler_login_packet::handle_login_packet;
use handler_handshaking_packet::handle_handshaking_packet;
use writer_varint_to_stream::write_varint_to_stream;
use writer_packet_data::write_packet_data;

//mod format_to_hex;
//use format_to_hex::format_hex;

// 非同期読み書きメソッドを使うための拡張機能
use tokio::net::{TcpListener, TcpStream};
use std::io::{Error, ErrorKind};

#[derive(Debug, Copy, Clone)]
enum ConnectionState {
    HandShaking,
    Status,
    Login,
    Transfer,
    Unknown,
}

struct HandShakePayload {
    protocol_version: i32,
    server_address: String,
    server_port: u16,
    next_state: i32,
    used_bytes: usize,
}

struct LoginStatePayload {
    minecraft_id: String,
    uuid: uuid::Uuid,
    used_bytes: usize,
}

struct ConnectionContext {
    state: ConnectionState,
    protocol_version: Option<i32>,
    server_address: Option<String>,
}

const MAX_PACKET_SIZE: usize = 1024 * 1024;

// 非同期処理を行うためのtokioランタイムを作り, async fn mainを実行できる
#[tokio::main]
async fn main() -> Result<(), Error> {

    // Listener::bindで全ての接続の待ち受けアドレスを指定
    let listener = TcpListener::bind("0.0.0.0:25565").await?;

    println!("================\n System Started\n================\n");

    // サーバーは接続を待ち続ける必要があるためloop
    loop {
        println!("Prepared for next connection\n");

        // listenerが待ち受けしているbindされたアドレスへ接続された際にlistener.accept()で
        // (TcpStream:それぞれの接続元と個別で読み書きするためのハンドル(ストリーム),
        // SocketAddr:接続元のアドレス)が返される
        let (socket, addr) = listener.accept().await?;
        println!("--------------------------------");
        println!("client connected {addr}");
        println!("--------------------------------");

        // tokio::spawnは中の処理を別のタスクで継続させ, 次の処理をすぐに実行できる
        // 今回ならaccept()を受けたらその接続先との処理をspawnで継続させつつ
        // すぐに次の接続を待ち受けれる
        tokio::spawn(
            async move {
                // socketはhandleに独占させて扱うので参照させるのではなく所有させる
                match handle_client(socket).await {
                    Ok(_) => {}
                    Err(e) => println!("{e}")
                };
                println!();
                println!("client disconnected");
            }
        );
    }
}

async fn handle_client(mut client_stream: TcpStream) -> std::io::Result<()> {

    let mut ctx = ConnectionContext {
        state: ConnectionState::HandShaking,
        protocol_version: None,
        server_address: None,
    };
    let mut backend: Option<TcpStream> = None;
    let mut relay = false;

    loop {
        // loopされたときの保険
        if relay == true {
            break;
        }

        let packet_data = match read_packet_data(&mut client_stream).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                println!("connection closed normally before next packet");
                return Ok(())
            }
            Err(e) => {
                return Err(e);
            }
        };

        let (packet_id, id_used) = read_varint_from_packet(&packet_data)?;

        let payload_slice = &packet_data[id_used..];
        println!("payload: {}bytes", payload_slice.len());

        match ctx.state {

            ConnectionState::HandShaking => {
                ctx = handle_handshaking_packet(packet_id, payload_slice)?;

                if backend.is_none() {

                    // match左辺のSomeの中でstringの記述ができないため
                    // as_deref()でStringを&strに変換.
                    // as_deref()方法(そもそも左辺を元から&strで書く)でもいいが,
                    // 先に変換作業をしている場合はこれで解決する
                    let backend_address = match ctx.server_address.as_deref() {
                        Some("127.0.0.1") => "127.0.0.1:25566",
                        Some("localhost") => "127.0.0.1:25566",
                        Some("test.example.com") => "127.0.0.1:25566",
                        Some("test2.example.com") => "127.0.0.1:25567",
                        Some(_) => return Err(Error::new(ErrorKind::InvalidData, "this server_address is not allowed")),
                        None => {
                            return Err(Error::new(ErrorKind::NotFound, "server_address is missing in ConnectionContext"))
                        }
                    };
                    // upstreamへこちらからconnectする(これはまだrelayしているわけではない)
                    // Client -> Proxy(this code) -> Server へ繋げる準備
                    let server_stream = TcpStream::connect(backend_address.to_string()).await?;
                    backend = Some(server_stream);
                    println!("proxy connected to real server");
                }
                let server = match backend.as_mut() {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
                };
                // packetのraw dataを送信
                write_packet_data(server, &packet_data).await?;
                println!();
                println!("forwarded handshake packet to upstream");
            }
            // サーバー一覧での表示用
            ConnectionState::Status => {
                handle_status_packet(packet_id, payload_slice, &ctx)?;

                let server = match backend.as_mut() {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
                };
                // packetのraw dataを送信
                write_packet_data(server, &packet_data).await?;
                println!();
                println!("forwarded status packet to upstream");

                // 0x01の場合は現状relay側で実装されるので
                // 分岐処理は書く必要性は極めて低いので無し
                if packet_id == 0x00 {
                    relay = true;
                    // relayまでの条件がそろったのでbreak
                    break;
                }
            }
            // ログイン処理用
            ConnectionState::Login => {
                handle_login_packet(packet_id, payload_slice, &ctx)?;

                let server = match backend.as_mut() {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
                };
                write_packet_data(server, &packet_data).await?;
                println!();
                println!("forwarded login packet to upstream");

                if packet_id == 0x00 {
                    relay = true;
                    break;
                }
            }
            ConnectionState::Transfer => {
                println!();
                println!("Transfer state packet observed: id = 0x{packet_id:02X} packet_data_len = {}", packet_data.len());
            }
            ConnectionState::Unknown => {
                println!();
                println!("Unknown state packet observed: id = 0x{packet_id:02X} packet_data_len = {}", packet_data.len());
            }
        }
        println!();
    }

    // relay処理
    if relay == true {
        let server_stream = match backend.as_mut() {
            Some(s) => s,
            None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
        };

        // copy_bidirectionalは
        // 両方向(各stream)からのこちらで消費していないデータをそれぞれ送りあう.
        // ここの時点ではClientから送られてきたpacketデータを読んで消費してしまっているので,
        // write_packet_dataで消費した分を送信しなおしている.
        // 以降のデータを全て相合中継する
        // 片方のデータがEofした場合に反対方向にshutdown()を送る

        // from_clientとfrom_serverはそれぞれから流れたbyte数が出力されます
        let (from_client, from_server) = tokio::io::copy_bidirectional(&mut client_stream, server_stream).await?;

        println!();
        println!("relay finished:\nclient->server: {} bytes\nserver->client: {} bytes", from_client, from_server);
    }

    Ok(())
}