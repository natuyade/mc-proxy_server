// 非同期読み書きメソッドを使うための拡張機能
use tokio::io::AsyncReadExt;

use tokio::net::TcpListener;

#[derive(Debug)]
enum VarIntError {
    Incomplete,
    TooLong,
}

enum ParsePacketError {
    VarInt(VarIntError),
    InvalidLength,
    InvalidUtf8,
    CouldntHandShake,
}

struct PacketHead {
    packet_length: i32,
    packet_id: i32,
    length_field_used: usize,
    head_used: usize,
}

struct HandShakePayload {
    protocol_version: i32,
    server_address: String,
    server_port: u16,
    next_state: i32,
    payload_used: usize,
}

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
            // moveで使用する変数を所有権ごと受けとり処理する.
            // loopで作られた変数は進むたびに初期化されてしまい,
            // 外(socketなど)の変数を借用しつづける事ができないため
            async move {

                // TCPの世界では文字列ではなくbyte列を扱う
                let mut buffer = [0u8; 1024];

                // read(&mut buffer)で受信したbyte列を可変bufferに書き込む.
                // read()一回での返り値は,送信側のwrite単位等の
                // packet境界(1packetの含む値)とは一致しない
                // 一回で1byteきたり... 一回で100bytesきたり...
                match socket.read(&mut buffer).await {

                    // 0は接続が切れた時の処理(相手が切断したり)
                    Ok(0) => {
                        println!("--------------------------------");
                        println!("client disconnected {addr}");
                        println!("--------------------------------");
                    }

                    // 受信したbyteデータを表示, 先頭に1byte存在していれば16進数へ変換,表示(
                    // read(1) => 10進数=49 -> 16進数=31,
                    // read(7) => 10進数=55 -> 16進数=37
                    // ) (あくまでも流れの説明)
                    Ok(n) => {
                        println!("read {n}bytes data:");
                        println!("{:?}", &buffer[0..n]);

                        let buf = &buffer[0..n];

                        let head = match parse_packet_head(buf) {
                            Ok(h) => h,
                            Err(ParsePacketError::VarInt(e)) => return println!("parse packet error: {:?}", e),
                            Err(ParsePacketError::InvalidLength) => return println!("parse packet error: InvalidLength"),
                            Err(ParsePacketError::InvalidUtf8) => return println!("parse packet error: InvalidUtf8"),
                            Err(ParsePacketError::CouldntHandShake) => return println!("parse packet error: CouldntHandShake"),
                        };

                        // Packetの列の長さを表す(PacketLength自身を除いた長さ)
                        println!("packet length: {}", head.packet_length);
                        // id == 0x00: HandShake
                        // HandShake:
                        // (データを送信する前に双方ともとあるパケットをやり取りし
                        // 相互の接続テストと通信準備を整えている.
                        // その名の通り)
                        println!("packet id: {}", head.packet_id);
                        println!("used bytes: {}", head.head_used);

                        // 受信したpacket dataの長さを可視化 (buf.len - length data) = packet data len
                        let packet_data_received = buf.len().saturating_sub(head.length_field_used);

                        // lengthが負の値でも0でもないという保障
                        if head.packet_length < 0 { return println!("invalid packet length") }
                        if head.packet_length == 0 { return println!("packet data is empty") }

                        // buffer内のdataの長さと実際のdataの長さの比較
                        // (受信したデータを全て解析できたか)
                        if packet_data_received < head.packet_length as usize {
                            return println!("packet not fully received yet")
                        }

                        // payload length, payload data取得
                        //
                        //  1byte- Protocol Version: VarInt
                        //      = payload[payload_start],
                        //          1.21.1 = [134, 6](774)
                        //
                        //  2byte- ServerAddressLength: VarInt
                        //      = payload[payload_start+1],
                        //          [1, 2, 7, ., 0, ., 0, ., 1] = [9](len: 9)
                        //
                        //  3byte- ServerAddress: String
                        //      = payload[(payload_start+2)..ServerAddressLength],
                        //          Address = [49, 50, 55, 46, 48, 46, 48, 46 ,49](127.0.0.1)
                        //
                        //  4byte- ServerPort: u16
                        //      = payload[(payload_start+2)+ServerAddressLength],
                        //          Port = [99, 222](25566)
                        //
                        //  5byte- NextState: VarInt
                        //      = payload[(payload_start+2)+ServerAddressLength+1],
                        //          state = [1(status)] or [2(login)] or [3(transfer)]
                        let payload_start = head.head_used;
                        let payload_bytes = &buf[payload_start..];
                        println!("payload bytes: {:?}", payload_bytes);
                        println!("payload bytes total: {}", payload_bytes.len());

                        let payload = match parse_packet_payload(payload_bytes) {
                            Ok(p) => p,
                            Err(ParsePacketError::VarInt(e)) => return println!("parse payload error: {:?}", e),
                            Err(ParsePacketError::InvalidLength) => return println!("parse payload error: InvalidLength"),
                            Err(ParsePacketError::InvalidUtf8) => return println!("parse payload error: InvalidUtf8"),
                            Err(ParsePacketError::CouldntHandShake) => return println!("parse payload error: CouldntHandShake"),
                        };

                        println!("parse payload used bytes: {}", payload.payload_used);

                        println!("~~~~~~~~~~~~~~~~");
                        println!("・Payload Data");
                        println!("- protocol_version: {}", payload.protocol_version);
                        println!("- server_address: {}", payload.server_address);
                        println!("- server_port: {}", payload.server_port);
                        match payload.next_state {
                            1 => println!("- next_state: status"),
                            2 => println!("- next_state: login"),
                            3 => println!("- next_state: transfer"),
                            _ => println!("- next_state: unknown"),
                        }
                        println!("~~~~~~~~~~~~~~~~\n");
                    }

                    Err(e) => {
                        eprintln!("read error {addr}: {e}");
                    }
                }
            }
        );
    }
}

fn read_varint_from_buffer(buf: &[u8], index: usize) -> Result<(i32, usize), VarIntError>{

    let mut value = 0i32;

    // MinecraftJEでのVarIntはi32を可変長で表すため最大5bytesまでに制限
    for i in 0..5 {

        // byteが存在すれば参照外した実体を返す
        let byte = match buf.get(index + i) {
            Some(byte) => *byte,
            None => return Err(VarIntError::Incomplete),
        };

        let value_part = byte & 0b0111_1111;

        let has_next = (byte & 0b1000_0000) != 0;

        value |= (value_part as i32) << (7 * i);

        if !has_next {
            return Ok((value, i + 1));
        }
    }

    Err(VarIntError::TooLong)
}

fn parse_packet_head(buf: &[u8]) -> Result<PacketHead, ParsePacketError> {

    let mut index = 0;

    // match result
    let (packet_length, length_field_used) = read_varint_from_buffer(buf, index).map_err(ParsePacketError::VarInt)?;
    index += length_field_used;

    let (packet_id, id_field_used) = read_varint_from_buffer(buf, index).map_err(ParsePacketError::VarInt)?;
    index += id_field_used;

    println!();
    if packet_id != 0 {
        println!("This is not handshake");
        println!();
        return Err(ParsePacketError::CouldntHandShake)
    }
    println!("Handshake!^_^@@^.^");
    println!();

    let packet_data = PacketHead {
        packet_length,
        packet_id,
        length_field_used,
        head_used: index,
    };

    Ok(packet_data)
}

fn parse_packet_payload(payload: &[u8]) -> Result<HandShakePayload, ParsePacketError> {

    let mut index = 0;

    let (protocol_version, version_field_used) = read_varint_from_buffer(payload, index).map_err(ParsePacketError::VarInt)?;
    index += version_field_used;

    let (address_length, address_length_used) = read_varint_from_buffer(payload, index).map_err(ParsePacketError::VarInt)?;
    index += address_length_used;

    // 負の値チェック
    if address_length < 0 {
        return Err(ParsePacketError::InvalidLength)
    }
    // 範囲チェック(usize変換でのoverflow等対策になるのと
    // 複数のbyte列を読む必要があり,途中で切れてる可能性があるため)
    if payload.len() < index + address_length as usize {
        return Err(ParsePacketError::InvalidLength)
    }

    // String変換, utf8変換
    let address_bytes = &payload[index..index + address_length as usize];
    let server_address = match std::str::from_utf8(address_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => return Err(ParsePacketError::InvalidUtf8),
    };
    index += address_length as usize;

    // portは2bytes固定のUnsigned Short(u16)
    // この先2byteあるか範囲チェック(途中で切れる可能性があるため)
    if payload.len() < index + 2 {
        return Err(ParsePacketError::InvalidLength)
    }
    // big-endianなので順序は最初のbyteが上位になる
    let server_port = (payload[index + 1] as u16) | (payload[index] as u16) << 8;
    index += 2;

    let (next_state, next_state_used) = read_varint_from_buffer(payload, index).map_err(ParsePacketError::VarInt)?;
    index += next_state_used;

    let packet_payload = HandShakePayload {
        protocol_version,
        server_address,
        server_port,
        next_state,
        payload_used: index,
    };

    Ok(packet_payload)
}