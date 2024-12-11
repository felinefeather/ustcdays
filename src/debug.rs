//  其实有“可以在这里写屎山”的意思在

use egui::Context;

use crate::{frontend::{DebugFromFrontend, DebugSign, FromFrontend}, game::GameDataSource, MainApp};

pub fn debug_window(app: &mut MainApp, ctx: &Context) {
    egui::Window::new("Debug").show(ctx, |ui| {
        ui.text_edit_multiline(&mut app.debug_cache.path_str);
        if ui.button("as path").clicked() {
            app.backend.sender.send(FromFrontend::Debug(
                DebugFromFrontend { sign: DebugSign::ReloadData(
                    GameDataSource::Path(app.debug_cache.path_str.clone().into())
                ) }
            )).unwrap_or_else(|_| panic!("failed to send the selection to the backend"));
        }
        if ui.button("as raw").clicked() {
            app.backend.sender.send(FromFrontend::Debug(
                DebugFromFrontend { sign: DebugSign::ReloadData(
                    GameDataSource::Raw(app.debug_cache.path_str.clone().into())
                ) }
            )).unwrap_or_else(|_| panic!("failed to send the selection to the backend"));
        }
        if ui.button("as default").clicked() {
            app.backend.sender.send(FromFrontend::Debug(
                DebugFromFrontend { sign: DebugSign::ReloadData(
                    GameDataSource::None
                ) }
            )).unwrap_or_else(|_| panic!("failed to send the selection to the backend"));
        }

        ui.text_edit_singleline(&mut app.debug_cache.attr_str);
        ui.add(egui::Slider::new(&mut app.debug_cache.value, 0..=100));
        
        if ui.button("modify").clicked() {
            app.backend.sender.send(FromFrontend::Debug(
                DebugFromFrontend { sign: DebugSign::SetAttribute(
                    app.debug_cache.attr_str.clone(),
                    app.debug_cache.value 
                ) }
            )).unwrap_or_else(|_| panic!("failed to send the selection to the backend"));
        }
    });
}