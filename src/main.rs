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

mod handler_status_state_packet;
use handler_status_state_packet::handle_status_packet;

mod handler_login_packet;
use handler_login_packet::handle_login_packet;

mod handler_handshaking_packet;
use handler_handshaking_packet::handle_handshaking_packet;

//mod format_to_hex;
//use format_to_hex::format_hex;

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

struct ConnectionContext {
    state: ConnectionState,
    protocol_version: Option<i32>,
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
                match handle_client(socket).await {
                    Ok(_) => {}
                    Err(e) => println!("{e}")
                };
                println!("client disconnected");
            }
        );
    }
}

async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {

    let mut ctx = ConnectionContext {
        state: ConnectionState::HandShaking,
        protocol_version: None,
    };

    loop {

        let packet_data = match read_packet_data(&mut stream).await {
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
            }
            ConnectionState::Status => {
                handle_status_packet(packet_id, payload_slice, &ctx)?;
            }
            ConnectionState::Login => {
                handle_login_packet(packet_id, payload_slice, &ctx)?;
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
}