mod reader_varint_from_stream;
use reader_varint_from_stream::read_varint_from_stream;

mod reader_packet_data;
use reader_packet_data::read_packet_data;

mod reader_varint_from_packet;
use reader_varint_from_packet::read_varint_from_packet;
mod parser_handshake_payload;
use parser_handshake_payload::parse_handshake_payload;

mod parser_status_ping_payload;
use parser_status_ping_payload::parse_status_ping_payload;

mod parser_login_payload;
use parser_login_payload::parse_login_start_payload_with_mcid_uuid;

mod format_to_hex;
use format_to_hex::format_hex;

// 非同期読み書きメソッドを使うための拡張機能
use tokio::net::{TcpListener, TcpStream};

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
                handle_client(socket).await;
                println!("client disconnected");
            }
        );
    }
}

async fn handle_client(mut stream: TcpStream) {

    let mut state = ConnectionState::HandShaking;

    loop {

        let packet_data = match read_packet_data(&mut stream).await {
            Ok(Some(data)) => data,
            Ok(None) => break println!("connection closed normally before next packet"),
            Err(e) => break println!("invalid data: {e}"),
        };

        let (packet_id, id_used) = match read_varint_from_packet(&packet_data) {
            Ok(id) => id,
            Err(e) => break println!("packet id parse error {e}"),
        };

        let payload_len = packet_data.len().saturating_sub(id_used);

        match state {

            ConnectionState::HandShaking => {

                println!();
                if packet_id == 0x00 {

                    println!("Handshake!^_^@@^.^");
                    println!();
                    println!("handshake payload_len: {payload_len}");

                    let payload_slice = &packet_data[id_used..];

                    match parse_handshake_payload(payload_slice) {

                        Ok(payload) =>{

                            // 読み終わったpayloadに余剰バイトがあるときの未読領域が存在するか確認
                            if payload.used_bytes != payload_slice.len(){
                                println!("warning:\npayload has trailing bytes\n[ total: {}, used: {} ]", payload_slice.len(), payload.used_bytes);
                            }

                            println!();
                            println!("HandShake payload");
                            println!("protocol version: {}", payload.protocol_version);
                            println!("server address: {}", payload.server_address);
                            println!("server port: {}", payload.server_port);

                            state = match payload.next_state {
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
                        }

                        Err(e) => break println!("handshake parse error: {e}"),
                    };
                } else {
                    println!("unexpected packet in handshaking");
                    println!();
                    break;
                }
            }

            ConnectionState::Status => {
                println!();
                println!("Status state packet observed: id = 0x{packet_id:02X}");
                println!("login start payload_len: {payload_len}");

                match packet_id {
                    0x00 => {
                        println!("Status Request packet candidate");
                        println!();

                        let payload_slice = &packet_data[id_used..];

                        println!("payload: {}bytes", payload_slice.len());

                        if payload_slice.is_empty() {
                            println!("valid Status Request payload length")
                        } else {
                            println!("invalid Status Request payload length: expected 0, got {}", payload_slice.len());
                            break;
                        }
                    }
                    0x01 => {
                        println!("Status Ping packet candidate");
                        println!();

                        let payload_slice = &packet_data[id_used..];

                        println!("payload: {}bytes", payload_slice.len());

                        if payload_slice.len() == 8 {
                            println!("valid Status Ping payload length")
                        } else {
                            println!("invalid Status Ping payload length: expected 8, got {}", payload_slice.len());
                            break;
                        }

                        let payload = match parse_status_ping_payload(&payload_slice) {
                            Ok(p) => p,
                            Err(e) => break println!("status ping parse error: {e}"),
                        };
                        println!("{payload}")
                    }
                    _ => {
                        println!("Unknown packet in Status state");
                        println!();
                    }
                }
            }
            ConnectionState::Login => {
                println!();
                println!("Login state packet observed: id = 0x{packet_id:02X} packet_data_len = {}", packet_data.len());

                let payload_slice = &packet_data[id_used..];

                let payload_hex = format_hex(payload_slice);

                println!("payload: {payload_len}bytes");

                println!("payload_hex: {payload_hex}");

                match parse_login_start_payload_with_mcid_uuid(payload_slice) {
                    Ok(payload) => {

                        println!();
                        println!("Login start payload");
                        println!("payload_data: {}", payload.minecraft_id);
                        println!("payload_data: {}", payload.uuid);

                    }
                    Err(_) => break
                };
            }
            ConnectionState::Transfer => {
                println!();
                println!("Transfer state packet observed: id = 0x{packet_id:02X} packet_data_len = {}", packet_data.len());
                println!("payload_len: {payload_len}");
            }
            ConnectionState::Unknown => {
                println!();
                println!("Unknown state packet observed: id = 0x{packet_id:02X} packet_data_len = {}", packet_data.len());
                println!("payload_len: {payload_len}");
            }
        }
        println!();
    }
}