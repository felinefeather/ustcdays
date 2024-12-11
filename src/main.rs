#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] use frontend::{FromFrontend, ToFrontend};
// hide console window on Windows in release
use game::{Game, GameDataSource};
use eframe::egui;
use egui::FontDefinitions;
use std::{
    sync::mpsc::{Receiver, SendError, Sender},
    thread,
};

mod events;
mod frontend;
mod game;
mod player;
mod systems;
mod debug;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "USTCDAYS",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_fonts({
                let mut font = FontDefinitions::default();
                let font_data = egui::FontData::from_static(include_bytes!(
                    "../assets/SourceHanSans-Regular.otf"
                ));
                font.font_data.insert("Chinese".to_owned(), font_data);
                font.families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "Chinese".to_owned());
                font
            });
            Ok(Box::<MainApp>::default())
        }),
    )
}

struct MainApp {
    backend: Backend,
    debug_cache: DebugCache,
}

struct Backend {
    receiver: Receiver<ToFrontend>,
    sender: Sender<FromFrontend>,
    cache: ToFrontend,
}

impl Backend {
    fn send(&mut self, t: FromFrontend) -> Result<(), SendError<FromFrontend>> {
        self.sender.send(t)
    }
}

#[derive(Default)]
struct DebugCache {
    path_str: String,
    attr_str: String,
    value: i32,
    enable: bool,
}

impl Default for MainApp {
    fn default() -> Self {
        let (su, ru) = std::sync::mpsc::channel();
        let (sf, rf) = std::sync::mpsc::channel();
        thread::spawn(|| {
            Game::new(
                GameDataSource::Path(
                    r#"C:\Users\felin\ustcdays\src\data\game_data.toml"#.into(),
                ), (sf, ru),
            )
            .unwrap().run();
        });
        Self {
            backend: Backend {
                receiver: rf,
                sender: su,
                cache: ToFrontend::new(),
            },
            debug_cache: DebugCache::default()
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.try_frontend_update();
        if self.debug_cache.enable { debug::debug_window(self, ctx); }
        egui::SidePanel::left("PlayerStateBar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("It is a sunny day today.");
                ui.add(egui::Image::new(egui::include_image!(
                    "../assets/untitled.png"
                )));
                ui.add_space(16.);
                for (name, max, cur) in &self.backend.cache.player_attribute {
                    ui.add(egui::ProgressBar::new(*cur as f32 / *max as f32))
                        .labelled_by(ui.label(name).id);
                }
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("今日日程");
            ui.label(self.backend.cache.main_area.clone());

            let options = self.backend.cache.option_area.clone();
            for (id, opt) in options.into_iter().enumerate() {
                if ui.button(opt).clicked() {
                    self.backend.send(FromFrontend::Choice(id))
                        .unwrap_or_else(|_| panic!("failed to send the selection to the backend"));
                    break;
                }
            }
        });
    }
}

impl MainApp {
    pub fn try_frontend_update(&mut self) {
        if let Ok(f) = self.backend.receiver.try_recv() {
            self.backend.cache = f;
        }
    }
}