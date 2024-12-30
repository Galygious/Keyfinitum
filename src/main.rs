// Keyfinitum/src/main.rs

mod remapping;
mod profile;
mod r#macro;
mod input_layer;
mod ui;
mod device;
mod profile_manager;

use ui::KeyfinitumApp;
use device::DeviceManager;
use profile_manager::ProfileManager;
use std::path::PathBuf;
use crate::r#macro::{Macro, MacroAction};
use std::time::Duration;

fn main() {
    let mut profile_manager = ProfileManager::new();
    let _device_manager = DeviceManager::new();
    
    profile_manager.add_profile("Default");
    profile_manager.add_profile("Gaming");
    profile_manager.add_profile("Productivity");
    
    {
        let mut profiles = profile_manager.profiles.lock().unwrap();
        profiles[0].add_remapping_config("Work", PathBuf::from("configs/work.json"));
        profiles[1].add_remapping_config("FPS", PathBuf::from("configs/fps.json"));
        profiles[2].add_remapping_config("Coding", PathBuf::from("configs/coding.json"));
    }
    
    profile_manager.add_app_mapping("explorer.exe", 0, None);
    profile_manager.add_app_mapping("notepad.exe", 2, None);
    profile_manager.add_app_mapping("game.exe", 1, None);
    
    {
        let mut macro_seq = Macro::new("TestMacro");
        macro_seq.add_action(MacroAction::KeyPress(0x41));
        macro_seq.add_action(MacroAction::Delay(Duration::from_millis(100)));
        macro_seq.add_action(MacroAction::KeyRelease(0x41));
        profile_manager.add_macro("TestMacro", macro_seq).unwrap();
    }

    profile_manager.start_auto_switcher();
    
    let options = eframe::NativeOptions::default();
    let profile_manager = std::sync::Arc::new(std::sync::Mutex::new(profile_manager));
    let profile_manager_clone = profile_manager.clone();
    if let Err(e) = eframe::run_native(
        "Keyfinitum",
        options,
        Box::new(move |_cc| Box::new(KeyfinitumApp::new(profile_manager_clone))),
    ) {
        eprintln!("Application error: {}", e);
    }
}
