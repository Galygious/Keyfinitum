use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::profile_manager::ProfileManager;
use crate::device::DeviceManager;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

pub trait Plugin {
    fn name(&self) -> &str;
    fn initialize(&mut self);
    fn execute(&self, context: PluginContext);
}

pub struct PluginContext {
    pub profile_manager: Arc<Mutex<ProfileManager>>,
    pub device_manager: Arc<Mutex<DeviceManager>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<(), PluginError> {
        // TODO: Implement plugin loading
        Ok(())
    }

    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        // TODO: Implement plugin unloading
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }
}

#[derive(Debug)]
pub enum PluginError {
    LoadError(String),
    UnloadError(String),
    ExecutionError(String),
}
