// Keyfinitum/src/remapping.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRemapping {
    layers: Vec<Layer>,
    active_layer_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Layer {
    name: String,
    mappings: HashMap<u32, Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    KeyPress(u32),
    KeySequence(Vec<u32>),
    SystemCommand(String),
    // CustomFunction cannot be serialized, so we'll remove it for now
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum KeyCodeError {
    InvalidKeyCode(u32),
    SystemCommandFailed,
}

impl KeyRemapping {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let default_layer = Layer {
            name: String::from("Default"),
            mappings: HashMap::new(),
        };
        
        Self {
            layers: vec![default_layer],
            active_layer_index: 0,
        }
    }

    /// Remap a key to a specific action
    #[allow(dead_code)]
    pub fn remap_key(&mut self, from: u32, to: Action) -> Result<(), KeyCodeError> {
        if let Some(layer) = self.layers.get_mut(self.active_layer_index) {
            layer.mappings.insert(from, to);
            Ok(())
        } else {
            Err(KeyCodeError::InvalidKeyCode(from))
        }
    }

    /// Handle key press event
    #[allow(dead_code)]
    pub fn handle_key_press(&self, key: u32) -> Result<(), KeyCodeError> {
        if let Some(layer) = self.layers.get(self.active_layer_index) {
            if let Some(action) = layer.mappings.get(&key) {
                match action {
                    Action::KeyPress(target_key) => self.send_key_event(*target_key, false),
                    Action::KeySequence(keys) => {
                        for key in keys {
                            self.send_key_event(*key, false)?;
                            self.send_key_event(*key, true)?;
                        }
                        Ok(())
                    },
                    Action::SystemCommand(command) => {
                        std::process::Command::new("cmd")
                            .arg("/C")
                            .arg(command)
                            .spawn()
                            .map_err(|_| KeyCodeError::SystemCommandFailed)?;
                        Ok(())
                    },
                }
            } else {
                Ok(())
            }
        } else {
            Err(KeyCodeError::InvalidKeyCode(key))
        }
    }

    /// Send a key event using Windows API
    #[allow(dead_code)]
    fn send_key_event(&self, key: u32, key_up: bool) -> Result<(), KeyCodeError> {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_KEYBOARD,
                u: std::mem::zeroed(),
            };
            
            let keyboard_input = KEYBDINPUT {
                wVk: key as u16,
                wScan: 0,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            };
            
            *input.u.ki_mut() = keyboard_input;
            
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            Ok(())
        }
    }
}
