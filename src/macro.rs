// Keyfinitum/src/macro.rs

use std::time::{Duration, Instant};
use std::thread;
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    INPUT_MOUSE, MOUSEINPUT, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON,
    VK_SPACE, VK_RETURN, VK_SHIFT, VK_CONTROL, VK_MENU, VK_ESCAPE
};
use serde::{Serialize, Deserialize};

// Define virtual key codes for letters and numbers
#[allow(dead_code)]
const VK_A: i32 = 0x41;
#[allow(dead_code)]
const VK_B: i32 = 0x42;
#[allow(dead_code)]
const VK_C: i32 = 0x43;
#[allow(dead_code)]
const VK_D: i32 = 0x44;
#[allow(dead_code)]
const VK_E: i32 = 0x45;
#[allow(dead_code)]
const VK_F: i32 = 0x46;
#[allow(dead_code)]
const VK_G: i32 = 0x47;
#[allow(dead_code)]
const VK_H: i32 = 0x48;
#[allow(dead_code)]
const VK_I: i32 = 0x49;
#[allow(dead_code)]
const VK_J: i32 = 0x4A;
#[allow(dead_code)]
const VK_K: i32 = 0x4B;
#[allow(dead_code)]
const VK_L: i32 = 0x4C;
#[allow(dead_code)]
const VK_M: i32 = 0x4D;
#[allow(dead_code)]
const VK_N: i32 = 0x4E;
#[allow(dead_code)]
const VK_O: i32 = 0x4F;
#[allow(dead_code)]
const VK_P: i32 = 0x50;
#[allow(dead_code)]
const VK_Q: i32 = 0x51;
#[allow(dead_code)]
const VK_R: i32 = 0x52;
#[allow(dead_code)]
const VK_S: i32 = 0x53;
#[allow(dead_code)]
const VK_T: i32 = 0x54;
#[allow(dead_code)]
const VK_U: i32 = 0x55;
#[allow(dead_code)]
const VK_V: i32 = 0x56;
#[allow(dead_code)]
const VK_W: i32 = 0x57;
#[allow(dead_code)]
const VK_X: i32 = 0x58;
#[allow(dead_code)]
const VK_Y: i32 = 0x59;
#[allow(dead_code)]
const VK_Z: i32 = 0x5A;
#[allow(dead_code)]
const VK_0: i32 = 0x30;
#[allow(dead_code)]
const VK_1: i32 = 0x31;
#[allow(dead_code)]
const VK_2: i32 = 0x32;
#[allow(dead_code)]
const VK_3: i32 = 0x33;
#[allow(dead_code)]
const VK_4: i32 = 0x34;
#[allow(dead_code)]
const VK_5: i32 = 0x35;
#[allow(dead_code)]
const VK_6: i32 = 0x36;
#[allow(dead_code)]
const VK_7: i32 = 0x37;
#[allow(dead_code)]
const VK_8: i32 = 0x38;
#[allow(dead_code)]
const VK_9: i32 = 0x39;

/// Records user inputs and converts them into MacroActions
#[allow(dead_code)]
pub struct MacroRecorder {
    recording: bool,
    start_time: Option<Instant>,
    actions: Vec<MacroAction>,
    modifier_states: ModifierStates,
}

#[derive(Default)]
#[allow(dead_code)]
struct ModifierStates {
    shift: bool,
    control: bool,
    alt: bool,
    windows: bool,
}

impl ModifierStates {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            shift: false,
            control: false,
            alt: false,
            windows: false,
        }
    }
}

impl MacroRecorder {
    /// Create a new MacroRecorder
    #[allow(dead_code)]
    pub fn new() -> Self {
        MacroRecorder {
            modifier_states: ModifierStates::new(),
            recording: false,
            start_time: None,
            actions: Vec::new(),
        }
    }

    /// Start recording user inputs
    #[allow(dead_code)]
    pub fn start_recording(&mut self) {
        self.recording = true;
        self.start_time = Some(Instant::now());
        self.actions.clear();
    }

    /// Stop recording and return the recorded macro
    #[allow(dead_code)]
    pub fn stop_recording(&mut self) -> Macro {
        self.recording = false;
        Macro {
            name: "Recorded Macro".to_string(),
            actions: self.actions.clone(),
        }
    }

    /// Update the recorder state (should be called periodically)
    #[allow(dead_code)]
    pub fn update(&mut self) {
        if !self.recording {
            return;
        }

        // Record mouse clicks
        self.record_mouse_click(VK_LBUTTON, 0);
        self.record_mouse_click(VK_RBUTTON, 1);

        // Record keyboard inputs
        for key in [
            VK_A, VK_B, VK_C, VK_D, VK_E, VK_F, VK_G, VK_H, VK_I, VK_J, VK_K, VK_L, VK_M,
            VK_N, VK_O, VK_P, VK_Q, VK_R, VK_S, VK_T, VK_U, VK_V, VK_W, VK_X, VK_Y, VK_Z,
            VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9,
            VK_SPACE, VK_RETURN, VK_SHIFT, VK_CONTROL, VK_MENU, VK_ESCAPE
        ].iter() {
            self.record_key_press(*key);
        }
    }

    fn record_mouse_click(&mut self, button: i32, button_id: u32) {
        unsafe {
            let state = GetAsyncKeyState(button);
            if state & 0x8000u16 as i16 != 0 {
                // Button is pressed
                if !self.actions.iter().any(|a| matches!(a, MacroAction::MousePress(btn) if *btn == button_id)) {
                    self.actions.push(MacroAction::MousePress(button_id));
                }
            } else {
                // Button is released
                if self.actions.iter().any(|a| matches!(a, MacroAction::MousePress(btn) if *btn == button_id)) {
                    self.actions.push(MacroAction::MouseRelease(button_id));
                }
            }
        }
    }

    fn record_key_press(&mut self, key: i32) {
        unsafe {
            let state = GetAsyncKeyState(key);
            if state & 0x8000u16 as i16 != 0 {
                // Key is pressed
                if !self.actions.iter().any(|a| matches!(a, MacroAction::KeyPress(k) if *k == key as u32)) {
                    // Update modifier states
                    match key {
                        VK_SHIFT => self.modifier_states.shift = true,
                        VK_CONTROL => self.modifier_states.control = true,
                        VK_MENU => self.modifier_states.alt = true,
                        _ => {}
                    }
                    
                    // Record the key press with modifier states
                    self.actions.push(MacroAction::KeyPress(key as u32));
                    
                    // If this is a regular key (not modifier), record the current modifier combination
                    if ![VK_SHIFT, VK_CONTROL, VK_MENU].contains(&key) {
                        self.record_modifier_combination();
                    }
                }
            } else {
                // Key is released
                if self.actions.iter().any(|a| matches!(a, MacroAction::KeyPress(k) if *k == key as u32)) {
                    // Update modifier states
                    match key {
                        VK_SHIFT => self.modifier_states.shift = false,
                        VK_CONTROL => self.modifier_states.control = false,
                        VK_MENU => self.modifier_states.alt = false,
                        _ => {}
                    }
                    
                    self.actions.push(MacroAction::KeyRelease(key as u32));
                }
            }
        }
    }

    fn record_modifier_combination(&mut self) {
        let mut modifiers = Vec::new();
        if self.modifier_states.shift {
            modifiers.push(VK_SHIFT as u32);
        }
        if self.modifier_states.control {
            modifiers.push(VK_CONTROL as u32);
        }
        if self.modifier_states.alt {
            modifiers.push(VK_MENU as u32);
        }
        
        if !modifiers.is_empty() {
            for modifier in modifiers {
                if !self.actions.iter().any(|a| matches!(a, MacroAction::KeyPress(k) if *k == modifier)) {
                    self.actions.push(MacroAction::KeyPress(modifier));
                }
            }
        }
    }
}

/// Represents a single macro action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroAction {
    KeyPress(u32),
    KeyRelease(u32),
    MousePress(u32),
    MouseRelease(u32),
    Delay(Duration),
}

/// Represents a complete macro sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub actions: Vec<MacroAction>,
}

impl Macro {
    /// Create a new empty macro
    pub fn new(name: &str) -> Self {
        Macro {
            name: name.to_string(),
            actions: Vec::new(),
        }
    }

    /// Add an action to the macro
    pub fn add_action(&mut self, action: MacroAction) {
        self.actions.push(action);
    }

    /// Execute the macro
    pub fn execute(&self) {
        for action in &self.actions {
            match action {
                MacroAction::KeyPress(key) => self.send_key_event(*key, false),
                MacroAction::KeyRelease(key) => self.send_key_event(*key, true),
                MacroAction::MousePress(button) => self.send_mouse_event(*button, false),
                MacroAction::MouseRelease(button) => self.send_mouse_event(*button, true),
                MacroAction::Delay(duration) => thread::sleep(*duration),
            }
        }
    }

    fn send_key_event(&self, key: u32, key_up: bool) {
        unsafe {
            // Create an array to hold all inputs (main key + modifiers)
            let mut inputs = Vec::new();
            
            // Check if this is a modifier key
            let is_modifier = match key as i32 {
                VK_SHIFT | VK_CONTROL | VK_MENU => true,
                _ => false,
            };
            
            // If this is a regular key press (not modifier), send modifier keys first
            if !key_up && !is_modifier {
                // Check which modifiers are active
                let mut modifiers = Vec::new();
                if GetAsyncKeyState(VK_SHIFT) & 0x8000u16 as i16 != 0 {
                    modifiers.push(VK_SHIFT as u32);
                }
                if GetAsyncKeyState(VK_CONTROL) & 0x8000u16 as i16 != 0 {
                    modifiers.push(VK_CONTROL as u32);
                }
                if GetAsyncKeyState(VK_MENU) & 0x8000u16 as i16 != 0 {
                    modifiers.push(VK_MENU as u32);
                }
                
                // Send modifier key presses
                for modifier in modifiers {
                    let mut input = INPUT {
                        type_: INPUT_KEYBOARD,
                        u: std::mem::zeroed(),
                    };
                    
                    let keyboard_input = KEYBDINPUT {
                        wVk: modifier as u16,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    
                    *input.u.ki_mut() = keyboard_input;
                    inputs.push(input);
                }
            }
            
            // Send the main key event
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
            inputs.push(input);
            
            // Send all inputs at once
            SendInput(inputs.len() as u32, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn send_mouse_event(&self, button: u32, button_up: bool) {
        unsafe {
            let mut input = INPUT {
                type_: INPUT_MOUSE,
                u: std::mem::zeroed(),
            };
            
            let mouse_input = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: match button {
                    0 => if button_up { MOUSEEVENTF_LEFTUP } else { MOUSEEVENTF_LEFTDOWN },
                    _ => 0,
                },
                time: 0,
                dwExtraInfo: 0,
            };
            
            *input.u.mi_mut() = mouse_input;
            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        }
    }
}
