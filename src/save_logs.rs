use std::path::PathBuf;
use crate::rewrite_path::rewrite_path;

pub fn save_logs_to_file(save_dir: bool, logs: &Vec<String>, while_error_dir: Option<PathBuf>) -> std::io::Result<()> {
    let collect_logs = logs.iter().map(|log| log.to_string()).collect::<Vec<_>>().join("\n");

    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");

    if while_error_dir.is_some() {

        if let Some(mut path) = while_error_dir {

            let log = format!("{now}.log");
            path.push(log);

            std::fs::write(&path, &collect_logs)?
        }
    } else {

        if save_dir == true {
            let log_dir = format!("logs/{now}");

            let (rewrote_path_file, rewrote_path_dir) = rewrite_path(log_dir, "log")?;

            match std::fs::write(&rewrote_path_file, &collect_logs) {
                Ok(()) => {}
                Err(_) => {
                    std::fs::create_dir_all(&rewrote_path_dir)?;
                    std::fs::write(&rewrote_path_file, &collect_logs)?
                }
            }
        } else {
            let relative = format!("logs/{now}.log");

            match std::fs::write(&relative, &collect_logs) {
                Ok(()) => {}
                Err(_) => {
                    std::fs::create_dir_all("logs/")?;
                    std::fs::write(&relative, &collect_logs)?
                }
            }
        }
    }

    Ok(())
}