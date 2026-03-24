use crate::SharedLogs;

pub fn push_log(shared_log: &SharedLogs, repaint_ctx: &egui::Context, str: impl Into<String>) {

    let line = format!("proxy_log: {}", str.into());

    if let Ok(mut write_log) = shared_log.write() {
        write_log.log.push(line)
    };
    repaint_ctx.request_repaint()
}