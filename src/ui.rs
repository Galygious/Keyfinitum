// Keyfinitum/src/ui.rs

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::r#macro::{Macro, MacroAction};
use crate::profile_manager::ProfileManager;

/// Main application UI
pub struct KeyfinitumApp {
    profile_manager: Arc<Mutex<ProfileManager>>,
    profile_switch_sender: std::sync::mpsc::Sender<usize>,
    #[allow(dead_code)]
    profile_switch_receiver: std::sync::mpsc::Receiver<usize>,
    current_profile: usize,
    show_macro_editor: bool,
    show_layer_editor: bool,
    show_app_mappings: bool,
    macro_editor: Option<MacroEditor>,
    new_app_name: String,
    selected_profile: usize,
}

/// Macro visual editor component
struct MacroEditor {
    macro_name: String,
    actions: Vec<MacroAction>,
    timeline_zoom: f32,
    selected_action: Option<usize>,
    drag_start_pos: Option<egui::Pos2>,
}

impl MacroEditor {
    fn new() -> Self {
        Self {
            macro_name: String::new(),
            actions: Vec::new(),
            timeline_zoom: 1.0,
            selected_action: None,
            drag_start_pos: None,
        }
    }

    fn from_macro(r#macro: Macro) -> Self {
        Self {
            macro_name: r#macro.name.clone(),
            actions: r#macro.actions.clone(),
            timeline_zoom: 1.0,
            selected_action: None,
            drag_start_pos: None,
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        // Macro name input
        ui.horizontal(|ui| {
            ui.label("Macro Name:");
            ui.text_edit_singleline(&mut self.macro_name);
        });

        // Zoom controls
        ui.horizontal(|ui| {
            ui.label("Zoom:");
            if ui.button("-").clicked() {
                self.timeline_zoom = (self.timeline_zoom * 0.8).max(0.1);
            }
            ui.add(egui::Slider::new(&mut self.timeline_zoom, 0.1..=2.0));
            if ui.button("+").clicked() {
                self.timeline_zoom = (self.timeline_zoom * 1.2).min(2.0);
            }
        });

        // Timeline view
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let timeline_height = 100.0;
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), timeline_height),
                egui::Sense::click_and_drag(),
            );

            // Draw timeline background
            painter.rect_filled(
                response.rect,
                0.0,
                egui::Color32::from_gray(30),
            );

            // Draw actions on timeline
            let mut x_pos = 10.0;
            // Store actions length before iteration to avoid borrowing issues
            let actions_len = self.actions.len();
            for i in 0..actions_len {
                let action_width = match &self.actions[i] {
                    MacroAction::KeyPress(_) => 50.0,
                    MacroAction::KeyRelease(_) => 50.0,
                    MacroAction::MousePress(_) => 60.0,
                    MacroAction::MouseRelease(_) => 60.0,
                    MacroAction::Delay(duration) => duration.as_secs_f32() * self.timeline_zoom * 100.0,
                };

                let action_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x_pos, response.rect.top() + 10.0),
                    egui::Vec2::new(action_width, timeline_height - 20.0),
                );

                // Draw action block
                painter.rect_filled(
                    action_rect,
                    4.0,
                    if self.selected_action == Some(i) {
                        egui::Color32::from_rgb(100, 150, 255)
                    } else {
                        egui::Color32::from_rgb(50, 100, 200)
                    },
                );

                // Handle selection and dragging
                if response.clicked() && action_rect.contains(response.interact_pointer_pos().unwrap()) {
                    self.selected_action = Some(i);
                    self.drag_start_pos = response.interact_pointer_pos();
                }

                // Handle dragging and resizing
                if let Some(selected) = self.selected_action {
                    if selected == i {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some(start_pos) = self.drag_start_pos {
                                let delta = pos.x - start_pos.x;
                                if delta.abs() > 2.0 {
                                    // Check if cursor is near edge for resizing
                                    let edge_threshold = 10.0;
                                    let near_left_edge = (pos.x - action_rect.left()).abs() < edge_threshold;
                                    let near_right_edge = (pos.x - action_rect.right()).abs() < edge_threshold;

                                    match &mut self.actions[i] {
                                        MacroAction::Delay(duration) => {
                                            if near_left_edge {
                                                // Resize from left edge
                                                let new_duration = (*duration).as_secs_f32() - (delta / (self.timeline_zoom * 100.0));
                                                *duration = Duration::from_secs_f32(new_duration.max(0.0));
                                                x_pos += delta; // Adjust position of subsequent blocks
                                            } else if near_right_edge {
                                                // Resize from right edge
                                                let new_duration = (*duration).as_secs_f32() + (delta / (self.timeline_zoom * 100.0));
                                                *duration = Duration::from_secs_f32(new_duration.max(0.0));
                                            } else {
                                                // Regular dragging
                                                let new_duration = (*duration).as_secs_f32() + (delta / (self.timeline_zoom * 100.0));
                                                *duration = Duration::from_secs_f32(new_duration.max(0.0));
                                            }
                                        }
                                        _ => {} // Other actions don't support dragging/resizing yet
                                    }
                                    self.drag_start_pos = Some(pos);
                                }
                            }
                        }
                    }
                }

                x_pos += action_width + 5.0;
            }
        });

        // Action controls
        ui.horizontal(|ui| {
            if ui.button("Add Key Press").clicked() {
                self.actions.push(MacroAction::KeyPress(0));
            }
            if ui.button("Add Mouse Click").clicked() {
                self.actions.push(MacroAction::MousePress(0));
            }
            if ui.button("Add Delay").clicked() {
                self.actions.push(MacroAction::Delay(Duration::from_millis(500)));
            }
            if ui.button("Delete Selected").clicked() {
                if let Some(index) = self.selected_action {
                    self.actions.remove(index);
                    self.selected_action = None;
                }
            }
        });
    }
}

impl KeyfinitumApp {
    /// Create a new UI instance
    pub fn new(profile_manager: Arc<Mutex<ProfileManager>>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        KeyfinitumApp {
            profile_manager,
            profile_switch_sender: sender,
            profile_switch_receiver: receiver,
            current_profile: 0,
            show_macro_editor: false,
            show_layer_editor: false,
            show_app_mappings: false,
            macro_editor: None,
            new_app_name: String::new(),
            selected_profile: 0,
        }
    }
}

impl eframe::App for KeyfinitumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Monitoring service controls
            ui.horizontal(|ui| {
                ui.label("Monitoring Service:");
                if ui.button("Start").clicked() {
                    let mut profile_manager = self.profile_manager.lock().unwrap();
                    profile_manager.start_auto_switcher();
                    ui.label("Running");
                }
                if ui.button("Stop").clicked() {
                    let mut profile_manager = self.profile_manager.lock().unwrap();
                    profile_manager.stop_auto_switcher();
                    ui.label("Stopped");
                }
            });

            // Profile selection
            ui.horizontal(|ui| {
                ui.label("Profile:");
                {
                    let profile_manager = self.profile_manager.lock().unwrap();
                    let profiles = profile_manager.profiles.lock().unwrap();
                    egui::ComboBox::from_id_source("profile_select")
                        .selected_text(&profiles[self.current_profile].name)
                        .show_ui(ui, |ui| {
                            for (i, profile) in profiles.iter().enumerate() {
                                if ui.selectable_value(&mut self.current_profile, i, &profile.name).clicked() {
                                    // Send profile switch request through channel
                                    if self.profile_switch_sender.send(i).is_err() {
                                        eprintln!("Failed to send profile switch request");
                                    }
                                }
                            }
                        });
                }
            });

            // Remapping section
            ui.collapsing("Key Remapping", |_ui| {
                // TODO: Implement remapping UI
            });

            // Macro section
            ui.collapsing("Macros", |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New Macro").clicked() {
                        self.show_macro_editor = true;
                        self.macro_editor = Some(MacroEditor::new());
                    }
                    
                    // Add recording controls
                    if ui.button("Start Recording").clicked() {
                        let mut profile_manager = self.profile_manager.lock().unwrap();
                        profile_manager.start_macro_recording();
                    }
                    
                    if ui.button("Stop Recording").clicked() {
                        let mut profile_manager = self.profile_manager.lock().unwrap();
                        if let Some(r#macro) = profile_manager.stop_macro_recording() {
                            // Add the recorded macro to the current profile
                            profile_manager.add_macro_to_current_profile(r#macro);
                        }
                    }
                    
                    if ui.button("Execute Macro").clicked() {
                        let profile_manager = self.profile_manager.lock().unwrap();
                        if let Some(r#macro) = profile_manager.get_current_macro() {
                            r#macro.execute();
                        }
                    }
                });
                
                // List existing macros
                let profile_manager = self.profile_manager.lock().unwrap();
                let profile = profile_manager.get_current_profile();
                ui.label("Current Macros:");
                for (macro_name, macro_value) in &profile.macros {
                    ui.horizontal(|ui| {
                        ui.label(&macro_value.name);
                        if ui.button("Execute").clicked() {
                            macro_value.execute();
                        }
                        if ui.button("Edit").clicked() {
                            self.show_macro_editor = true;
                            self.macro_editor = Some(MacroEditor::from_macro(macro_value.clone()));
                        }
                        if ui.button("Delete").clicked() {
                            profile_manager.remove_macro(macro_name).unwrap_or_else(|e| {
                                eprintln!("Failed to remove macro: {}", e);
                            });
                        }
                    });
                }
            });

            // Input Layers section
            ui.collapsing("Input Layers", |ui| {
                if ui.button("New Layer").clicked() {
                    self.show_layer_editor = true;
                }
                // TODO: List existing layers
            });
        });

        // Macro editor window
        if self.show_macro_editor {
            egui::Window::new("Macro Editor")
                .open(&mut self.show_macro_editor)
                .show(ctx, |ui| {
                    if let Some(editor) = &mut self.macro_editor {
                        editor.show(ui);
                    }
                });
        }

        // Layer editor window
        if self.show_layer_editor {
            egui::Window::new("Layer Editor")
                .open(&mut self.show_layer_editor)
                .show(ctx, |_ui| {
                    // TODO: Implement layer editor
                });
        }

        // Application mapping editor window
        egui::Window::new("Application Mappings")
            .open(&mut self.show_app_mappings)
            .show(ctx, |ui| {
                let profile_manager = self.profile_manager.lock().unwrap();
                let mappings = profile_manager.app_mappings.lock().unwrap();
                
                ui.heading("Current Mappings");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (app_pattern, profile_index) in mappings.iter() {
                        let profiles = profile_manager.profiles.lock().unwrap();
                        let profile_name = &profiles[*profile_index].name;
                        ui.label(format!("{} → {}", app_pattern, profile_name));
                    }
                });

                ui.separator();

                ui.heading("Add New Mapping");
                ui.horizontal(|ui| {
                    ui.label("Application:");
                    ui.text_edit_singleline(&mut self.new_app_name);
                    ui.label("Profile:");
                    let profiles = profile_manager.profiles.lock().unwrap();
                    egui::ComboBox::from_id_source("profile_select")
                        .selected_text(&profiles[self.selected_profile].name)
                        .show_ui(ui, |ui| {
                            for (i, profile) in profiles.iter().enumerate() {
                                ui.selectable_value(&mut self.selected_profile, i, &profile.name);
                            }
                        });
                });

                if ui.button("Add Mapping").clicked() {
                    let profile_manager = self.profile_manager.lock().unwrap();
                    profile_manager.add_app_mapping(
                        &self.new_app_name,
                        self.selected_profile,
                        None
                    );
                    self.new_app_name.clear();
                }
            });
    }
}
