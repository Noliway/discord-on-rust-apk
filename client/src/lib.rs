use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver},
    Arc,
};
use std::time::Duration;

use eframe::egui;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Msg {
    id: u64,
    author: String,
    text: String,
}

enum Event {
    Msgs(Vec<Msg>),
    Status(String),
}

pub struct RustcordApp {
    server: String,
    name: String,
    channel: String,
    input: String,
    messages: Vec<Msg>,
    status: String,
    rx: Option<Receiver<Event>>,
    stop: Option<Arc<AtomicBool>>,
    ctx: egui::Context,
}

fn clean_channel(s: &str) -> String {
    let c: String = s
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if c.is_empty() { "general".into() } else { c }
}

fn base_url(s: &str) -> String {
    let s = s.trim().trim_end_matches('/');
    if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("http://{s}")
    }
}

impl RustcordApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.global_style()).clone();
        style.text_styles.iter_mut().for_each(|(_, f)| f.size *= 1.25);
        cc.egui_ctx.set_global_style(style);
        Self {
            server: "http://10.0.2.2:8080".into(),
            name: "user".into(),
            channel: "general".into(),
            input: String::new(),
            messages: Vec::new(),
            status: String::new(),
            rx: None,
            stop: None,
            ctx: cc.egui_ctx.clone(),
        }
    }

    fn connected(&self) -> bool {
        self.rx.is_some()
    }

    fn connect(&mut self) {
        self.disconnect();
        self.channel = clean_channel(&self.channel);
        let url = format!("{}/channels/{}/messages", base_url(&self.server), self.channel);
        let (tx, rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let mut last = 0u64;
            while !stop2.load(Ordering::Relaxed) {
                let res = ureq::get(&url)
                    .query("after", &last.to_string())
                    .timeout(Duration::from_secs(10))
                    .call()
                    .map_err(|e| e.to_string())
                    .and_then(|r| r.into_json::<Vec<Msg>>().map_err(|e| e.to_string()));
                let ev = match res {
                    Ok(v) => {
                        if let Some(m) = v.last() {
                            last = m.id;
                        }
                        if v.is_empty() { Event::Status(String::new()) } else { Event::Msgs(v) }
                    }
                    Err(e) => Event::Status(format!("Ошибка: {e}")),
                };
                if tx.send(ev).is_err() {
                    break;
                }
                ctx.request_repaint();
                std::thread::sleep(Duration::from_millis(1500));
            }
        });
        self.rx = Some(rx);
        self.stop = Some(stop);
        self.messages.clear();
        self.status = "Подключение…".into();
    }

    fn disconnect(&mut self) {
        if let Some(s) = self.stop.take() {
            s.store(true, Ordering::Relaxed);
        }
        self.rx = None;
    }

    fn send(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        self.input.clear();
        let url = format!("{}/channels/{}/messages", base_url(&self.server), self.channel);
        let body = serde_json::json!({ "author": self.name.trim(), "text": text });
        std::thread::spawn(move || {
            let _ = ureq::post(&url).timeout(Duration::from_secs(10)).send_json(body);
        });
    }

    fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(ev) = rx.try_recv() {
                match ev {
                    Event::Msgs(v) => {
                        self.messages.extend(v);
                        self.status.clear();
                    }
                    Event::Status(s) => self.status = s,
                }
            }
        }
    }
}

impl eframe::App for RustcordApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();

        if !self.connected() {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add_space(24.0);
                ui.heading("Rustcord");
                ui.add_space(12.0);
                ui.label("Адрес сервера");
                ui.add(egui::TextEdit::singleline(&mut self.server).desired_width(f32::INFINITY));
                ui.label("Ваше имя");
                ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(f32::INFINITY));
                ui.label("Канал");
                ui.add(egui::TextEdit::singleline(&mut self.channel).desired_width(f32::INFINITY));
                ui.add_space(12.0);
                let ok = !self.name.trim().is_empty() && !self.server.trim().is_empty();
                if ui.add_enabled(ok, egui::Button::new("Войти")).clicked() {
                    self.connect();
                }
            });
            return;
        }

        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("# {}", self.channel));
                if ui.button("Выйти").clicked() {
                    self.disconnect();
                }
                if !self.status.is_empty() {
                    ui.weak(&self.status);
                }
            });
        });

        let mut do_send = false;
        egui::Panel::bottom("input").show(ui, |ui| {
            ui.horizontal(|ui| {
                let w = ui.available_width() - 90.0;
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .desired_width(w)
                        .hint_text("Сообщение"),
                );
                let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("Отправить").clicked() || enter {
                    do_send = true;
                    r.request_focus();
                }
            });
        });
        if do_send {
            self.send();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false; 2]).show(ui, |ui| {
                for m in &self.messages {
                    ui.horizontal_wrapped(|ui| {
                        ui.strong(&m.author);
                        ui.label(&m.text);
                    });
                }
            });
        });
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Rustcord",
        options,
        Box::new(|cc| Ok(Box::new(RustcordApp::new(cc)))),
    );
}
