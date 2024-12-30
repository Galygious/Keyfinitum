// Keyfinitum/src/profile.rs

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::r#macro::Macro;
use crate::input_layer::InputLayer;

/// Represents a remapping configuration
#[derive(Serialize, Deserialize, Clone)]
pub struct RemappingConfig {
    pub name: String,
    pub path: PathBuf,
}

/// Represents a user profile containing remapping configurations
#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub remapping_config: HashMap<String, String>, // Maps profile names to their config paths
    pub macros: HashMap<String, Macro>, // Stores macros for this profile
    pub input_layers: HashMap<String, InputLayer>, // Stores input layers for this profile
    pub active_profile: String,
    pub app_mappings: HashMap<String, usize>, // Maps application patterns to profile indices
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Profile {
            name: name.to_string(),
            remapping_config: HashMap::new(),
            macros: HashMap::new(),
            input_layers: HashMap::new(),
            active_profile: "default".to_string(),
            app_mappings: HashMap::new(),
        }
    }

    pub fn add_remapping_config(&mut self, name: &str, config_path: PathBuf) {
        self.remapping_config.insert(name.to_string(), config_path.to_string_lossy().to_string());
    }

    #[allow(dead_code)]
    pub fn remove_remapping_config(&mut self, name: &str) -> Option<String> {
        self.remapping_config.remove(name)
    }

    #[allow(dead_code)]
    pub fn switch_remapping_config(&mut self, name: &str) -> Result<(), String> {
        if self.remapping_config.contains_key(name) {
            self.active_profile = name.to_string();
            Ok(())
        } else {
            Err(format!("Remapping configuration '{}' not found", name))
        }
    }

    pub fn start_macro_recording(&mut self) {
        // Implementation for starting macro recording
    }

    pub fn stop_macro_recording(&mut self) -> Option<Macro> {
        // Implementation for stopping macro recording
        None
    }

    pub fn add_macro(&mut self, name: &str, macro_seq: Macro) {
        self.macros.insert(name.to_string(), macro_seq);
    }

    pub fn get_macro(&self, name: &str) -> Option<Macro> {
        self.macros.get(name).cloned()
    }

    pub fn remove_macro(&mut self, name: &str) -> Option<Macro> {
        self.macros.remove(name)
    }

    #[allow(dead_code)]
    pub fn execute_macro(&self, name: &str) -> Result<(), String> {
        if let Some(macro_seq) = self.macros.get(name) {
            macro_seq.execute();
            Ok(())
        } else {
            Err(format!("Macro '{}' not found", name))
        }
    }

    #[allow(dead_code)]
    pub fn save_app_mappings(&self, path: &PathBuf) -> std::io::Result<()> {
        let json = serde_json::to_string(&self.app_mappings)?;
        std::fs::write(path, json)
    }

    #[allow(dead_code)]
    pub fn load_app_mappings(&mut self, path: &PathBuf) -> std::io::Result<()> {
        let data = std::fs::read_to_string(path)?;
        self.app_mappings = serde_json::from_str(&data)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn auto_switch_profile(&mut self, app_name: &str, window_title: Option<&str>) -> bool {
        if let Some(title) = window_title {
            let key = format!("{}:{}", app_name, title);
            if let Some(profile_index) = self.get_profile_for_app(&key) {
                self.active_profile = profile_index.to_string();
                return true;
            }
        }

        if let Some(profile_index) = self.get_profile_for_app(app_name) {
            self.active_profile = profile_index.to_string();
            return true;
        }

        false
    }

    #[allow(dead_code)]
    pub fn active_config_path(&self) -> Option<&String> {
        self.remapping_config.get(&self.active_profile)
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    #[allow(dead_code)]
    pub fn add_input_layer(&mut self, name: &str, modifier_key: u32) {
        self.input_layers.insert(name.to_string(), InputLayer::new(name, modifier_key));
    }

    #[allow(dead_code)]
    pub fn remove_input_layer(&mut self, name: &str) -> Option<InputLayer> {
        self.input_layers.remove(name)
    }

    #[allow(dead_code)]
    pub fn get_input_layer(&self, name: &str) -> Option<&InputLayer> {
        self.input_layers.get(name)
    }

    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[allow(dead_code)]
    pub fn get_profile_for_app(&self, app_name: &str) -> Option<usize> {
        self.app_mappings.get(app_name).copied()
    }
}
