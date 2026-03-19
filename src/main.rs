// 非同期読み書きメソッドを使うための拡張機能
use tokio::io::AsyncReadExt;

use tokio::net::TcpListener;

#[derive(Debug)]
enum VarIntError {
    Incomplete,
    TooLong,
}

// 非同期処理を行うためのtokioランタイムを作り, async fn mainを実行できる
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    // Listener::bindで全ての接続の待ち受けアドレスを指定
    let listener = TcpListener::bind("0.0.0.0:25566").await?;

    // サーバーは接続を待ち続ける必要があるためloop
    loop {

        // listenerが待ち受けしているbindされたアドレスへ接続された際にlistener.accept()で
        // (TcpStream:それぞれの接続元と個別で読み書きするためのハンドル(ストリーム),
        // SocketAddr:接続元のアドレス)が返される
        let (mut socket, addr) = listener.accept().await?;

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
                        println!("client disconnected {addr}");
                    }

                    // 受信したbyteデータを表示, 先頭に1byte存在していれば16進数へ変換,表示(
                    // read(1) => 10進数=49 -> 16進数=31,
                    // read(7) => 10進数=55 -> 16進数=37
                    // ) (あくまでも流れの説明)
                    Ok(n) => {
                        println!("read {n}bytes data");
                        println!("{:?}", &buffer[0..n]);
                        let start_index = 0;

                        match read_varint_from_buffer(&buffer[0..n], start_index) {
                            Ok((value, used)) => {
                                println!("get value: {value}");
                                println!("used {used}bytes");
                            }
                            Err(VarIntError::Incomplete) => {
                                println!("varint is incomplete")
                            }
                            Err(VarIntError::TooLong) => {
                                println!("varint is too long")
                            }
                        };
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
        println!("byte{i} = {byte}");
        println!("byte{i} hex = {:02X}", byte);
        println!("start {i}byte loop");

        let value_part = byte & 0b0111_1111;
        println!("{i}byte value part: {value_part}");

        let has_next = (byte & 0b1000_0000) != 0;

        value |= (value_part as i32) << (7 * i);
        println!("value bits: {:b}", value);

        if !has_next {
            return Ok((value, i + 1));
        }
        println!("{i}byte has next");
    }

    Err(VarIntError::TooLong)
}