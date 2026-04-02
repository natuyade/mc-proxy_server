use crate::rewrite_path::rewrite_path;

// 終了時の処理なので一旦unwrapで書いてます
// 後々error出たらエラー内容の表示ウィンドウを生成して
// 一旦のログの保存場所をファイル参照などで入力できるようにし
// 処置できるようにしたいなーなんて
pub fn save_logs_to_file(save_dir: bool, logs: &Vec<String>) {

    let collect_logs = logs.iter().map(|log| log.to_string()).collect::<Vec<_>>().join("\n");

    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");

    if save_dir == true {

        let log_dir = format!("logs/{now}");

        let (rewrote_path_file, rewrote_path_dir) = rewrite_path(log_dir, "log").unwrap();

        match std::fs::write(&rewrote_path_file, &collect_logs) {
            Ok(()) => {}
            Err(_) => {
                std::fs::create_dir_all(&rewrote_path_dir).unwrap();
                std::fs::write(&rewrote_path_file, &collect_logs).unwrap()
            }
        }
    } else {
        let relative = format!("logs/{now}.log");

        match std::fs::write(&relative, &collect_logs) {
            Ok(()) => {}
            Err(_) => {
                std::fs::create_dir_all("logs/").unwrap();
                std::fs::write(&relative, &collect_logs).unwrap()
            }
        }
    }
}