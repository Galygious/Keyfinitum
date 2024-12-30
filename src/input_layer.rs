// Keyfinitum/src/input_layers.rs

use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputLayer {
    pub name: String,
    pub modifier_key: u32,
    pub key_mappings: Vec<(u32, u32)>, // (from_key, to_key)
}

#[allow(dead_code)]
impl InputLayer {
    pub fn new(name: &str, modifier_key: u32) -> Self {
        InputLayer {
            name: name.to_string(),
            modifier_key,
            key_mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, from: u32, to: u32) {
        self.key_mappings.push((from, to));
    }

    pub fn remove_mapping(&mut self, from: u32) -> Option<u32> {
        if let Some(index) = self.key_mappings.iter().position(|(f, _)| *f == from) {
            Some(self.key_mappings.remove(index).1)
        } else {
            None
        }
    }
}
