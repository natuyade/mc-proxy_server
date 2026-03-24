use tokio::net::{TcpListener, TcpStream};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use crate::{read_packet_data, read_varint_from_packet, handle_handshaking_packet, write_packet_data, handle_status_packet, handle_login_packet};

use crate::{SharedRules, SharedLogs};
use crate::proxy::push_log_line::push_log;

#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    HandShaking,
    Status,
    Login,
    Transfer,
    Unknown,
}

pub struct HandShakePayload {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
    pub used_bytes: usize,
}

pub struct LoginStatePayload {
    pub minecraft_id: String,
    pub uuid: uuid::Uuid,
    pub used_bytes: usize,
}

pub struct ConnectionContext {
    pub state: ConnectionState,
    pub protocol_version: Option<i32>,
    pub server_address: Option<String>,
}

pub struct PlayerData {
    pub player_id: String,
    pub player_uuid: uuid::Uuid,
    pub protocol_version: i32,
    pub payload_warning: bool,
    pub payload_used_bytes: usize,
}

pub struct PlayerContext {
    pub player_data: Option<PlayerData>,
}

pub const MAX_PACKET_SIZE: usize = 1024 * 1024;

/*
// 非同期処理を行うためのtokioランタイムを作り, async fn mainを実行できる
#[tokio::main]
 */

pub async fn run_proxy(shared_rules: SharedRules, shared_logs: SharedLogs, ctx: egui::Context) -> Result<(), Error> {
    push_log(&shared_logs, &ctx, "Running!");

    // Listener::bindで全ての接続の待ち受けアドレスを指定
    let listener = TcpListener::bind("0.0.0.0:25566").await?;

    // サーバーは接続を待ち続ける必要があるためloop
    loop {

        // listenerが待ち受けしているbindされたアドレスへ接続された際にlistener.accept()で
        // (TcpStream:それぞれの接続元と個別で読み書きするためのハンドル(ストリーム),
        // SocketAddr:接続元のアドレス)が返される
        let (socket, addr) = listener.accept().await?;

        let rules = Arc::clone(&shared_rules);
        let logs = Arc::clone(&shared_logs);

        // こちらで共有した値がeguiに即時更新反映されるようにしたい場合request_repaintが使える
        push_log(&logs, &ctx, format!("Client connected: {addr}"));

        let move_ctx = ctx.clone();

        // tokio::spawnは中の処理を別のタスクで継続させ, 次の処理をすぐに実行できる
        // 今回ならaccept()を受けたらその接続先との処理をspawnで継続させつつ
        // すぐに次の接続を待ち受けれる
        tokio::spawn(
            async move {

                let end_log = Arc::clone(&logs);
                let ctx = move_ctx.clone();

                // socketはhandleに独占させて扱うので参照させるのではなく所有させる
                match handle_client(socket, rules, logs, move_ctx).await {
                    Ok(_) => {}
                    Err(e) => push_log(&end_log, &ctx, format!("{e}"))
                };
                push_log(&end_log, &ctx, format!("Client disconnected: {addr}"));
            }
        );
    }
}

pub async fn handle_client(mut client_stream: TcpStream, _shared_rules: SharedRules, shared_log: SharedLogs, egui_ctx: egui::Context) -> std::io::Result<()> {

    let mut connection_ctx = ConnectionContext {
        state: ConnectionState::HandShaking,
        protocol_version: None,
        server_address: None,
    };
    let mut player_ctx = PlayerContext {
        player_data: None
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
                push_log(&shared_log, &egui_ctx, "connection closed normally");
                return Ok(())
            }
            Err(e) => {
                return Err(e);
            }
        };

        let (packet_id, id_used) = read_varint_from_packet(&packet_data)?;

        let payload_slice = &packet_data[id_used..];

        match connection_ctx.state {

            ConnectionState::HandShaking => {
                connection_ctx = handle_handshaking_packet(packet_id, payload_slice)?;

                if backend.is_none() {

                    // match左辺のSomeの中でstringの記述ができないため
                    // as_deref()でStringを&strに変換.
                    // as_deref()方法(そもそも左辺を元から&strで書く)でもいいが,
                    // 先に変換作業をしている場合はこれで解決できる
                    let backend_address = match connection_ctx.server_address.as_deref() {
                        Some("127.0.0.1") => "127.0.0.1:25565",
                        Some("localhost") => "127.0.0.1:25565",
                        Some("test.example.com") => "127.0.0.1:25565",
                        Some("test2.example.com") => "127.0.0.1:25567",
                        Some(_) => return Err(Error::new(ErrorKind::InvalidData, "this server_address is not allowed")),
                        None => {
                            return Err(Error::new(ErrorKind::NotFound, "server_address is missing in ConnectionContext"))
                        }
                    };
                    // upstreamへこちらからconnectする(これはまだrelayしているわけではない)
                    // Client -> Proxy(this code) -> Server へ繋げる準備
                    let server_stream = TcpStream::connect(backend_address).await?;
                    backend = Some(server_stream);
                }
                let server = match backend.as_mut() {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
                };
                // packetのraw dataを送信
                write_packet_data(server, &packet_data).await?;
                push_log(&shared_log, &egui_ctx, "Client connected by the proxy to the server");
            }
            // サーバー一覧での表示用
            ConnectionState::Status => {
                handle_status_packet(packet_id, payload_slice, &connection_ctx)?;

                let server = match backend.as_mut() {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
                };
                // packetのraw dataを送信
                write_packet_data(server, &packet_data).await?;

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

                let player_data = handle_login_packet(packet_id, payload_slice, &connection_ctx)?;
                push_log(&shared_log, &egui_ctx, format!("[PlayerData]prot:{}, {}({})", player_data.protocol_version, player_data.player_id, player_data.player_uuid));
                if player_data.payload_warning == true {
                    push_log(&shared_log, &egui_ctx, format!(
                        "warning: payload has trailing bytes[total: {}, used: {}]",
                        payload_slice.len(),
                        player_data.payload_used_bytes
                    ));
                }
                player_ctx.player_data = Some(player_data);

                let server = match backend.as_mut() {
                    Some(s) => s,
                    None => return Err(Error::new(ErrorKind::NotConnected, "backend connect failed"))
                };
                write_packet_data(server, &packet_data).await?;
                push_log(&shared_log, &egui_ctx, "forwarded login packet to upstream");

                if packet_id == 0x00 {
                    relay = true;
                    break;
                }
            }
            ConnectionState::Transfer => {
                push_log(&shared_log, &egui_ctx, "Transfer state packet observed: Unsupported state.");
            }
            ConnectionState::Unknown => {
                push_log(&shared_log, &egui_ctx, "Unknown state packet observed: Unsupported state.");
            }
        }
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

        let player_data = match player_ctx.player_data {
            Some(d) => d,
            None => return Err(Error::new(ErrorKind::NotFound, "[what?]Not found player data"))
        };

        push_log(&shared_log, &egui_ctx, format!("Relay Finished![{}: {}]Result(client->server: {}bytes, server->client: {}bytes)", player_data.player_id, player_data.player_uuid, from_client, from_server));
    }

    Ok(())
}