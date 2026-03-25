use crate::SharedLogs;

pub fn push_log(shared_log: &SharedLogs, repaint_ctx: &egui::Context, str: impl Into<String>) {

    // %Y-%m-%d_%H:%M:%S
    let now = chrono::Local::now().format("%H:%M:%S");

    let line = format!("[{now}]{}", str.into());

    if let Ok(mut write_log) = shared_log.write() {
        write_log.log.push(line)
    };
    // こちらで共有した値がeguiに即時更新反映されるようにしたい場合request_repaintが使える
    repaint_ctx.request_repaint()
}