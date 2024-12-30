use eframe::egui;
use crate::ui::KeyfinitumApp;

mod device;
mod input_layer;
mod r#macro;
mod profile;
mod profile_manager;
mod remapping;
mod ui;
mod plugin;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 720.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Keyfinitum",
        options,
        Box::new(|cc| Box::new(KeyfinitumApp::new(cc)))
    )
}
