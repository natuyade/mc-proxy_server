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
                // read()は一回(/= 1packet)で受け取れるbyteの数は流れによって変わる
                // 一回で1byteきたり... 一回で100bytesきたり...
                match socket.read(&mut buffer).await {

                    // 0は接続が切れた時の処理(相手が切断したり)
                    Ok(0) => {
                        println!("client disconnected {addr}");
                    }

                    // 受信したbyteデータを表示, 16進数へ変換,表示(
                    // read(1) => 10進数=49 -> 16進数=31,
                    // read(7) => 10進数=55 -> 16進数=37
                    // ) (あくまでも流れの説明)
                    Ok(n) => {
                        println!("read {n}bytes data");
                        println!("{:?}", &buffer[0..n]);

                        // hex = 16進数
                        let hex = buffer[..n]
                            .iter()
                            // mapでiterで取り出したものを文字列へ変換
                            // {:02X}の意味
                            // X=数値を16進数に大文字変換(x=小文字)
                            // 2=表示最小桁数
                            // 0=桁が最小桁数に満たなければ0で埋める
                            // これでみんながよくみる`あれ`になる.
                            // format!の場合変数が参照されたものでも扱えるが
                            // ここでは型意識のために*で参照を外し実体を扱う
                            .map(|b| format!("{:02X}", *b))
                            // mapでformatしたものをそれぞれcollect()のVecの中に集める(collect)
                            .collect::<Vec<_>>()
                            // それぞれの要素の間に" "(空白)を入れ連結
                            .join(" ");

                        println!("{hex}");

                        // buffer[0..3]とすると,受け取ったbyte列が二つの場合,
                        // 2番目(3個目)は今回受信したデータ以降の場所を見てしまう.
                        // そのため一度すべて受け取ってから読む範囲を指定する
                        // minで指定すれば3まではそのまま表示,それ以降は端折られる
                        let head_len = n.min(3);

                        let head_hex = buffer[..head_len]
                            .iter()
                            .map(|b| format!("{:02X}", *b))
                            .collect::<Vec<_>>()
                            .join(" ");

                        println!("convert {head_len}bytes");

                        println!("{head_hex}");
                    }

                    Err(e) => {
                        eprintln!("read error {addr}: {e}");
                    }
                }
            }
        );
    }
}