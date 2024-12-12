#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod game;
mod time_system;
mod map_system;
mod conditions;
mod events;
mod player;
mod triggers;
mod frontend;
mod cli_frontend;

use game::Game;
use cli_frontend::CLIFrontend;
use crate::player::Attribute;


fn main() {
    let frontend = CLIFrontend::new();
    let mut game = Game::new(frontend).expect("Failed to initialize game");
    game.run().expect("Game encountered an error");
}


use eframe::egui;
use egui::{FontDefinitions, Layout, Vec2};

fn main_this_is_for_ui() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_fonts({
                let mut font = FontDefinitions::default();
                font.font_data.insert(
                    "Chinese".to_owned(), 
                    egui::FontData::from_static(
                        include_bytes!("../assets/SourceHanSans-Regular.otf")));
                font.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0,"Chinese".to_owned());
                font
            });
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("PlayerStateBar").resizable(false).show(ctx, |ui| {
            ui.heading("It is a sunny day today.");
            ui.add(egui::Image::new(egui::include_image!("../assets/untitled.png")));
            ui.add_space(16.);
            ui.add(egui::ProgressBar::new(self.age as f32/100.)).labelled_by(ui.label("GPA").id);
            ui.add(egui::ProgressBar::new(self.age as f32/100.)).labelled_by(ui.label("GPB").id);
            ui.add(egui::ProgressBar::new(self.age as f32/100.)).labelled_by(ui.label("GPC").id);
            ui.add(egui::ProgressBar::new(self.age as f32/100.)).labelled_by(ui.label("GPD").id);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("今日日程");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("GPA"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(r#"
    [0] 食堂 - - - [3] 北门 - - [12] 生科楼
    /                   |            |
[1] 西活 - - - - - [4] 西图书馆       |
    |                   |            |
[5] 西门 - [6] 三教 - - - - - - - - *13* 肥西路
    |                 |    |         |
    |           [9] 电教 [10] 力教    |
[7] 加速器 - [8] 南门 - - - - [11] 公寓区
"#).highlight();

            //ui.image(egui::include_image!(
            //    "../../../crates/egui/assets/ferris.png"
            //));
        });
    }
}
