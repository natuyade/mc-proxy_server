// 非同期読み書きメソッドを使うための拡張機能
use tokio::io::AsyncReadExt;

use tokio::net::TcpListener;

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
                // read()は一回(!= 1packet)で受け取れるbyteの数は流れによって変わる
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

                        match buffer[..n].get(0) {
                            Some(first) => {
                                println!("first byte: {}", *first);
                                println!("first hex: {:02X}", *first);

                                // 0b=後に続くものが二進数であることを表す
                                // 1000_0000=二進数での128.この状態を最上位ビットが立っている状態ともいう.

                                // VarInt等の世界で8bitの最上位bitは,主にバイトの続きがあるかを表し
                                // 1で継続,0が最終バイト.を意味する
                                // 以降7bitは実データを表す

                                // 例えばfirstが1101_0011だとして&(bit AND演算)で
                                // 最上位bitだけが立ったマスク(1000_0000)と比較し,
                                // 1101_0011
                                // 1000_0000
                                // ---------
                                // 1000_0000になる

                                // 分かりやすくすると
                                // (0 & 0 = 0), (1 & 0 = 0), (1 & 1 = 1)

                                // そうすればfirstの最上位bitが立っていることが判明し
                                // 1000_0000(128) != 0 でtrueになる.
                                // 最上位bitが立っていたら(0でなければ)次のバイトが存在し継続するというboolになる
                                let has_next = (*first & 0b1000_0000) != 0;
                                println!("has next byte: {has_next}");

                                // こっちは逆に最上位以外の下位7bitを取り出し.
                                // 1101_0011
                                // 0111_1111
                                // ---------
                                // 0101_0011になる
                                // firstから値部分を取り出す
                                let value_part = *first & 0b0111_1111;
                                println!("value part: {value_part}");
                                println!("value part hex: {:02X}",value_part);
                            }
                            None => {
                                println!("no first byte")
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("read error {addr}: {e}");
                    }
                }
            }
        );
    }
}