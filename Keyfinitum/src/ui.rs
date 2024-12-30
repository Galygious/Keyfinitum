// Keyfinitum/src/ui.rs

use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::profile_manager::ProfileManager;
use crate::plugin::PluginManager;
use crate::device::{DeviceManager, DeviceType, DeviceCapabilities};

mod editor {
    use super::*;
    
    pub struct MacroEditor {
        // TODO: Implement macro editor
    }

    impl MacroEditor {
        pub fn show(&mut self, _ui: &mut egui::Ui) -> Option<()> {
            None // TODO: Implement show
        }
    }

    pub struct RemappingEditor {
        // TODO: Implement remapping editor
    }

    impl RemappingEditor {
        pub fn show(&mut self, _ui: &mut egui::Ui) -> Option<()> {
            None // TODO: Implement show
        }
    }
}

use editor::{MacroEditor, RemappingEditor};

#[derive(Clone)]
struct DeviceInfo {
    path: String,
    device_type: DeviceType,
    vendor_id: u16,
    product_id: u16,
    capabilities: DeviceCapabilities,
    current_dpi: u16,
    min_dpi: u16,
    max_dpi: u16,
    dpi_step: u16,
}

/// Main application UI
pub struct KeyfinitumApp {
    profile_manager: Arc<Mutex<ProfileManager>>,
    plugin_manager: Arc<Mutex<PluginManager>>,
    profile_switch_sender: std::sync::mpsc::Sender<usize>,
    #[allow(dead_code)]
    profile_switch_receiver: std::sync::mpsc::Receiver<usize>,
    current_profile: usize,
    show_macro_editor: bool,
    show_layer_editor: bool,
    show_app_mappings: bool,
    show_remapping_editor: bool,
    macro_editor: Option<MacroEditor>,
    remapping_editor: Option<RemappingEditor>,
    new_app_name: String,
    selected_profile: usize,
    device_manager: DeviceManager,
}

impl KeyfinitumApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let profile_manager = Arc::new(Mutex::new(ProfileManager::new()));
        let plugin_manager = Arc::new(Mutex::new(PluginManager::new()));
        Self {
            profile_manager,
            plugin_manager,
            profile_switch_sender: tx,
            profile_switch_receiver: rx,
            current_profile: 0,
            show_macro_editor: false,
            show_layer_editor: false,
            show_app_mappings: false,
            show_remapping_editor: false,
            macro_editor: None,
            remapping_editor: None,
            new_app_name: String::new(),
            selected_profile: 0,
            device_manager: DeviceManager::new(),
        }
    }
}

impl eframe::App for KeyfinitumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Devices section
            ui.collapsing("Devices", |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Connected Devices");
                    if ui.button("🔄 Refresh").clicked() {
                        if let Err(e) = self.device_manager.detect_devices() {
                            eprintln!("Failed to detect devices: {}", e);
                        }
                    }
                });

                ui.separator();

                // Clone device info first to avoid borrowing issues
                let devices_info: Vec<DeviceInfo> = self.device_manager.get_devices()
                    .iter()
                    .map(|device| DeviceInfo {
                        path: device.device_path.clone(),
                        device_type: device.device_type.clone(),
                        vendor_id: device.vendor_id,
                        product_id: device.product_id,
                        capabilities: device.capabilities.clone(),
                        current_dpi: device.current_dpi,
                        min_dpi: device.min_dpi,
                        max_dpi: device.max_dpi,
                        dpi_step: device.dpi_step,
                    })
                    .collect();

                for device in devices_info {
                    ui.group(|ui| {
                        // Device header with type icon
                        ui.horizontal(|ui| {
                            let icon = match device.device_type {
                                DeviceType::Keyboard => "⌨️",
                                DeviceType::Mouse => "🖱️",
                                DeviceType::Other => "📱",
                            };
                            ui.label(format!("{} Device ID: {:04x}:{:04x}", 
                                icon, device.vendor_id, device.product_id));
                        });

                        // Device capabilities
                        ui.indent("capabilities", |ui| {
                            if device.capabilities.has_dpi_switch {
                                ui.horizontal(|ui| {
                                    ui.label("DPI Control:");
                                    if ui.button("-").clicked() {
                                        if let Err(e) = self.device_manager.decrease_dpi(&device.path) {
                                            eprintln!("Failed to decrease DPI: {}", e);
                                        }
                                    }
                                    ui.label(format!("{} DPI", device.current_dpi));
                                    if ui.button("+").clicked() {
                                        if let Err(e) = self.device_manager.increase_dpi(&device.path) {
                                            eprintln!("Failed to increase DPI: {}", e);
                                        }
                                    }
                                    ui.label(format!("(Range: {}-{}, Step: {})",
                                        device.min_dpi, device.max_dpi, device.dpi_step));
                                });
                            }

                            if device.capabilities.has_macro_keys {
                                ui.label("✓ Macro Keys Support");
                                // TODO: Add macro key configuration
                            }

                            if device.capabilities.has_media_controls {
                                ui.label("✓ Media Controls");
                                // TODO: Add media control configuration
                            }
                        });
                    });
                    ui.add_space(8.0);
                }

                if self.device_manager.get_devices().is_empty() {
                    ui.label("No devices detected");
                }
            });
        });

        // Remapping editor window
        if self.show_remapping_editor {
            let mut editor_closed = false;
            let mut binding_created = None;

            egui::Window::new("Key Binding Editor")
                .open(&mut self.show_remapping_editor)
                .show(ctx, |ui| {
                    if let Some(editor) = &mut self.remapping_editor {
                        if let Some(binding) = editor.show(ui) {
                            binding_created = Some(binding);
                            editor_closed = true;
                        }
                    }
                });

            if editor_closed {
                if let Some(_binding) = binding_created {
                    // TODO: Add the binding to the current profile
                    self.show_remapping_editor = false;
                    self.remapping_editor = None;
                }
            }
        }
    }
}
