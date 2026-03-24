mod proxy;

use proxy::proxy_main::{ConnectionContext, ConnectionState, HandShakePayload, LoginStatePayload};
use proxy::proxy_main::MAX_PACKET_SIZE;

use proxy::proxy_main::run_proxy;
use proxy::handler_handshaking_packet::handle_handshaking_packet;
use proxy::handler_login_packet::handle_login_packet;
use proxy::handler_status_state_packet::handle_status_packet;
use proxy::parser_handshake_payload::parse_handshake_payload;
use proxy::parser_login_payload::parse_login_start_payload_with_mcid_uuid;
use proxy::parser_status_ping_payload::parse_status_ping_payload;
use proxy::reader_packet_data::read_packet_data;
use proxy::reader_varint_from_packet::read_varint_from_packet;
use proxy::reader_varint_from_stream::read_varint_from_stream;
use proxy::writer_packet_data::write_packet_data;
use proxy::writer_varint_to_stream::write_varint_to_stream;
//use proxy::format_to_hex::format_hex;

// Arc(Atomic Reference Counted)はデータの所有権を複数で持てるようにするもの
// Rustは通常一つの値の所有者は一人, Arcを使えば一つのデータを複数で共有できる.
// RwLockはそのデータを安全に複数から読み書きできるようにするもの
// 誰かが書き込みしているときは, 他からのReadを制限し安全に書き込める.
// Read自体は複数から同時にできる
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;

// typeは型の名前を決めて書きやすくするためのもの
// {let shared_rules: SharedRules}で設定された型が使える
pub type SharedRules = Arc<RwLock<Vec<RouteRule>>>;
pub type SharedLogs = Arc<RwLock<ProxyLogs>>;

// proxy側とegui側で共有したい値
#[derive(Debug)]
pub struct RouteRule {
    accept_address: String,
    backend_address: String,
    enabled: bool,
}

// error 共有
#[derive(Debug)]
pub struct ProxyLogs {
    log: Vec<String>,
}

struct MyApp {
    rules: SharedRules,
    logs: Vec<String>,
    is_running: bool,
    runtime: Arc<Runtime>,
    proxy_task: Option<tokio::task::JoinHandle<()>>,
    proxy_logs: SharedLogs,
}

fn main() -> eframe::Result<()> {

    let runtime = Arc::new(Runtime::new().expect("failed to create runtime."));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::Vec2::new(860., 480.)),
        ..Default::default()
    };

    eframe::run_native(
        "Mc proxy server",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, runtime)))),
    )
}

// startup
impl MyApp {
    fn new(_cc: &eframe::CreationContext, runtime: Arc<Runtime>) -> Self {
        Self {
            rules: Arc::new(RwLock::new(vec![RouteRule {
                accept_address: "127.0.0.1:25566".to_string(),
                backend_address: "127.0.0.1:25565".to_string(),
                enabled: true,
            }])),
            logs: vec!["App started!".to_string()],
            is_running: false,
            runtime,
            proxy_task: None,
            proxy_logs: Arc::new(RwLock::new(ProxyLogs {
                log: Vec::new(),
            })),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = egui::Color32::LIGHT_GRAY;
        visuals.override_text_color = Some(egui::Color32::DARK_GRAY);
        visuals.text_edit_bg_color = Some(egui::Color32::BLACK);

        ctx.set_visuals(visuals);

        let mut rules = match self.rules.write() {
            Ok(guard) => guard,
            Err(_) => {
                self.logs.push("failed to lock rules".to_string());
                return;
            }
        };

        egui::SidePanel::right("logs_panel")
            .resizable(false)
            .exact_width(400.)
            .show(ctx, |ui| {
                ui.heading("logs");

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in &self.logs {
                        ui.label(line);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Minecraft Proxy GUI");

                ui.horizontal(|ui| {

                    // proxyの起動ボタン
                    if ui.button("Start").clicked() {
                        if !self.is_running {
                            self.logs.push("Starting proxy server...".to_string());
                            self.is_running = true;

                            // 共有する構造体をArc::clone()
                            let share_rules = Arc::clone(&self.rules);
                            let share_proxy_logs = Arc::clone(&self.proxy_logs);
                            let share_ctx = ctx.clone();

                            // proxy本体を起動
                            let handle = self.runtime.spawn(async move {
                                run_proxy(share_rules, share_proxy_logs, share_ctx).await.expect("proxy panic!");
                            });

                            self.proxy_task = Some(handle);

                        } else {
                            self.logs.push("server is already running.".to_string());
                        }
                    }

                    // proxyの停止ボタン
                    if ui.button("Stop").clicked() {
                        if self.is_running {

                            // .take()でOptionの中身をもらった後相手のOptionを空にする(貰う)
                            if let Some(handle) = self.proxy_task.take() {
                                // JoinHandleで紐づいたtaskを.abort()で強制終了する
                                handle.abort();
                            }

                            self.logs.push("Stopping proxy Server...".to_string());
                            self.is_running = false;
                        } else {
                            self.logs.push("proxy server is not working.".to_string());
                        }
                    }

                    // taskの有無を見てステータス表示
                    if self.proxy_task.is_none() {
                        ui.label("ProxyServer: Stopped");
                    } else {
                        ui.label("ProxyServer: Started");
                    }

                    // proxy側のログをもらう
                    if let Ok(mut proxy_logs) = self.proxy_logs.write() {
                        // appendでtakeのvector版のようなことができる
                        self.logs.append(&mut proxy_logs.log);
                    } else {
                        self.logs.push("failed to lock proxy logs".to_string())
                    }
                });

                // uiを分ける線を描画
                ui.separator();

                // buttonが押されたときにrulesVecに構造体をpushする.
                // 下のforへ
                if ui.button("add rule").clicked() {
                    rules.push(RouteRule {
                        accept_address: String::new(),
                        backend_address: "127.0.0.1:25565".to_string(),
                        enabled: true,
                    });
                    self.logs.push("added extra rule".to_string());
                }

                ui.separator();

                let mut remove_index = None;

                egui::ScrollArea::vertical().show(ui, |ui| {

                    // rulesに構造体があればここで一覧表示される.
                    // .enumerate()でiterator(vec等の中身)に順番にidを振り分ける(今回なら -> (id: usize, rule: &mut RouteRule))
                    for (id, rule) in rules.iter_mut().enumerate() {
                        // 横並びに配置
                        ui.horizontal(|ui| {
                            ui.label("allow");
                            ui.checkbox(&mut rule.enabled, "");
                            ui.label("from:");
                            ui.add(
                                egui::TextEdit::singleline(&mut rule.accept_address)
                                    .text_color(egui::Color32::WHITE)
                                    .desired_width(128.),
                            );

                            ui.label("to:");
                            ui.add(
                                egui::TextEdit::singleline(&mut rule.backend_address)
                                    .text_color(egui::Color32::WHITE)
                                    .desired_width(128.),
                            );

                            // clickされた時にこの要素全体に振られたidをremove_indexに入れ
                            // 下のif letへ
                            if ui.button("-").clicked() {
                                remove_index = Some(id);
                                self.logs.push(format!(
                                    "removed rule [ from: \"{}\", to: \"{}\" ]",
                                    rule.accept_address,
                                    rule.backend_address,
                                ));
                            }
                        });
                    }
                });

                // 上で指定されたidをvectorのindexにし対応した場所を削除
                // そのまま上の表示も対応して変わる
                if let Some(n) = remove_index {
                    rules.remove(n);
                }
            });
        });
    }
}
