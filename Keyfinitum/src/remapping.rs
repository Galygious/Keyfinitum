// Keyfinitum/src/remapping.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_EXTENDEDKEY,
    VK_SHIFT, VK_CONTROL, VK_MENU, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_WHEEL, INPUT_MOUSE, MOUSEINPUT,
};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRemapping {
    layers: Vec<Layer>,
    active_layer_index: usize,
    #[serde(skip)]
    modifier_state: ModifierState,
}

#[derive(Debug, Clone, Default)]
struct ModifierState {
    shift: bool,
    ctrl: bool,
    alt: bool,
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
    KeyCombination(Vec<u32>),
    SystemCommand(String),
    MacroTrigger(String),
    LayerSwitch(usize),
    MouseButton(MouseButton),
    MouseMove { dx: i32, dy: i32 },
    MouseWheel(i32),
    MediaControl(MediaAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaAction {
    PlayPause,
    NextTrack,
    PrevTrack,
    VolumeUp,
    VolumeDown,
    Mute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: u32,
    pub modifiers: KeyModifiers,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

#[derive(Debug)]
pub enum KeyCodeError {
    InvalidKeyCode(u32),
    InvalidLayerIndex(usize),
    SystemCommandFailed,
    FileError(String),
}

impl KeyRemapping {
    pub fn new() -> Self {
        let default_layer = Layer {
            name: String::from("Default"),
            mappings: HashMap::new(),
        };
        
        Self {
            layers: vec![default_layer],
            active_layer_index: 0,
            modifier_state: ModifierState::default(),
        }
    }

    /// Load remapping configuration from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, KeyCodeError> {
        let content = fs::read_to_string(path)
            .map_err(|e| KeyCodeError::FileError(e.to_string()))?;
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| KeyCodeError::FileError(e.to_string()))?;
        Ok(config)
    }

    /// Save remapping configuration to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), KeyCodeError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| KeyCodeError::FileError(e.to_string()))?;
        fs::write(path, content)
            .map_err(|e| KeyCodeError::FileError(e.to_string()))?;
        Ok(())
    }

    /// Add a new key binding with modifiers
    pub fn add_binding(&mut self, binding: KeyBinding) -> Result<(), KeyCodeError> {
        let active_layer_index = self.active_layer_index;
        let modified_key = self.create_modifier_key(binding.key, &binding.modifiers);
        
        if let Some(layer) = self.layers.get_mut(active_layer_index) {
            layer.mappings.insert(modified_key, binding.action);
            Ok(())
        } else {
            Err(KeyCodeError::InvalidKeyCode(binding.key))
        }
    }

    /// Create a unique key that includes modifier information
    fn create_modifier_key(&self, key: u32, modifiers: &KeyModifiers) -> u32 {
        let mut modified_key = key;
        if modifiers.shift { modified_key |= 0x01000000; }
        if modifiers.ctrl { modified_key |= 0x02000000; }
        if modifiers.alt { modified_key |= 0x04000000; }
        modified_key
    }

    /// Update modifier state
    pub fn update_modifier(&mut self, key: u32, pressed: bool) {
        match key as i32 {
            VK_SHIFT => self.modifier_state.shift = pressed,
            VK_CONTROL => self.modifier_state.ctrl = pressed,
            VK_MENU => self.modifier_state.alt = pressed,
            _ => {}
        }
    }

    /// Handle key press event
    pub fn handle_key_press(&mut self, key: u32) -> Result<(), KeyCodeError> {
        self.update_modifier(key, true);

        let modified_key = self.create_modifier_key(key, &KeyModifiers {
            shift: self.modifier_state.shift,
            ctrl: self.modifier_state.ctrl,
            alt: self.modifier_state.alt,
        });

        if let Some(layer) = self.layers.get(self.active_layer_index) {
            if let Some(action) = layer.mappings.get(&modified_key) {
                match action {
                    Action::KeyPress(target_key) => self.send_key_event(*target_key, false),
                    Action::KeySequence(keys) => {
                        for key in keys {
                            self.send_key_event(*key, false)?;
                            self.send_key_event(*key, true)?;
                        }
                        Ok(())
                    },
                    Action::KeyCombination(keys) => {
                        for key in keys {
                            self.send_key_event(*key, false)?;
                        }
                        for key in keys.iter().rev() {
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
                    Action::MacroTrigger(_name) => {
                        // TODO: Integrate with macro system
                        Ok(())
                    },
                    Action::LayerSwitch(layer_index) => {
                        if *layer_index < self.layers.len() {
                            self.active_layer_index = *layer_index;
                            Ok(())
                        } else {
                            Err(KeyCodeError::InvalidLayerIndex(*layer_index))
                        }
                    },
                    Action::MouseButton(button) => {
                        self.send_mouse_button(button, false)
                    },
                    Action::MouseMove { dx, dy } => {
                        self.send_mouse_move(*dx, *dy)
                    },
                    Action::MouseWheel(delta) => {
                        self.send_mouse_wheel(*delta)
                    },
                    Action::MediaControl(action) => {
                        self.send_media_control(action)
                    },
                }
            } else {
                self.send_key_event(key, false)
            }
        } else {
            Err(KeyCodeError::InvalidKeyCode(key))
        }
    }

    /// Handle key release event
    pub fn handle_key_release(&mut self, key: u32) -> Result<(), KeyCodeError> {
        self.update_modifier(key, false);

        let modified_key = self.create_modifier_key(key, &KeyModifiers {
            shift: self.modifier_state.shift,
            ctrl: self.modifier_state.ctrl,
            alt: self.modifier_state.alt,
        });

        if let Some(layer) = self.layers.get(self.active_layer_index) {
            if let Some(action) = layer.mappings.get(&modified_key) {
                match action {
                    Action::KeyPress(target_key) => self.send_key_event(*target_key, true),
                    Action::MouseButton(button) => self.send_mouse_button(button, true),
                    _ => Ok(()), // Other actions don't need release handling
                }
            } else {
                self.send_key_event(key, true)
            }
        } else {
            Err(KeyCodeError::InvalidKeyCode(key))
        }
    }

    /// Send a key event using Windows API
    fn send_key_event(&self, key: u32, key_up: bool) -> Result<(), KeyCodeError> {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_KEYBOARD,
                u: std::mem::zeroed(),
            };
            
            let keyboard_input = KEYBDINPUT {
                wVk: key as u16,
                wScan: 0,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 } | KEYEVENTF_EXTENDEDKEY,
                time: 0,
                dwExtraInfo: 0,
            };
            
            *input.u.ki_mut() = keyboard_input;
            
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            Ok(())
        }
    }

    /// Send a mouse button event
    fn send_mouse_button(&self, button: &MouseButton, up: bool) -> Result<(), KeyCodeError> {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_MOUSE,
                u: std::mem::zeroed(),
            };

            let flags = match (button, up) {
                (MouseButton::Left, false) => MOUSEEVENTF_LEFTDOWN,
                (MouseButton::Left, true) => MOUSEEVENTF_LEFTUP,
                (MouseButton::Right, false) => MOUSEEVENTF_RIGHTDOWN,
                (MouseButton::Right, true) => MOUSEEVENTF_RIGHTUP,
                (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEDOWN,
                (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEUP,
                _ => return Ok(()), // Other buttons not implemented yet
            };

            let mouse_input = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            };

            *input.u.mi_mut() = mouse_input;
            
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            Ok(())
        }
    }

    /// Send a mouse move event
    fn send_mouse_move(&self, dx: i32, dy: i32) -> Result<(), KeyCodeError> {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_MOUSE,
                u: std::mem::zeroed(),
            };

            let mouse_input = MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: 0x0001, // MOUSEEVENTF_MOVE
                time: 0,
                dwExtraInfo: 0,
            };

            *input.u.mi_mut() = mouse_input;
            
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            Ok(())
        }
    }

    /// Send a mouse wheel event
    fn send_mouse_wheel(&self, delta: i32) -> Result<(), KeyCodeError> {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_MOUSE,
                u: std::mem::zeroed(),
            };

            let mouse_input = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: (delta * 120) as u32, // Convert to wheel delta
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            };

            *input.u.mi_mut() = mouse_input;
            
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            Ok(())
        }
    }

    /// Send a media control event
    fn send_media_control(&self, _action: &MediaAction) -> Result<(), KeyCodeError> {
        // TODO: Implement media control using multimedia keys or system commands
        Ok(())
    }
}
