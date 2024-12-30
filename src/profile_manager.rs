use crate::profile::Profile;
use crate::r#macro::Macro;
use active_win_pos_rs::get_active_window;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ProfileManager {
    pub(crate) profiles: Arc<Mutex<Vec<Profile>>>,
    active_profile_index: Arc<Mutex<usize>>,
    pub app_mappings: Arc<Mutex<HashMap<String, usize>>>,
    monitor_thread: Option<std::thread::JoinHandle<()>>,
    monitor_stop_signal: Arc<std::sync::atomic::AtomicBool>,
}

#[allow(dead_code)]
impl ProfileManager {
    pub fn new() -> Self {
        ProfileManager {
            profiles: Arc::new(Mutex::new(vec![Profile::new("Default")])),
            active_profile_index: Arc::new(Mutex::new(0)),
            app_mappings: Arc::new(Mutex::new(HashMap::new())),
            monitor_thread: None,
            monitor_stop_signal: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn add_profile(&self, name: &str) {
        self.profiles.lock().unwrap().push(Profile::new(name));
    }

    pub fn switch_profile(&self, index: usize) -> Result<(), String> {
        let profiles = self.profiles.lock().unwrap();
        if index >= profiles.len() {
            return Err(format!("Profile index {} out of bounds", index));
        }
        *self.active_profile_index.lock().unwrap() = index;
        Ok(())
    }

    pub fn active_profile(&self) -> Profile {
        let profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        profiles[index].clone()
    }

    pub fn add_app_mapping(&self, app_name: &str, profile_index: usize, title_pattern: Option<&str>) {
        let mut mappings = self.app_mappings.lock().unwrap();
        let key = match title_pattern {
            Some(pattern) => format!("{}:{}", app_name, pattern),
            None => app_name.to_string(),
        };
        mappings.insert(key, profile_index);
    }

    pub fn add_macro(&self, name: &str, macro_seq: Macro) -> Result<(), String> {
        let mut profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        if index >= profiles.len() {
            return Err("Invalid profile index".to_string());
        }
        profiles[index].add_macro(name, macro_seq);
        Ok(())
    }

    pub fn remove_macro(&self, name: &str) -> Result<(), String> {
        let mut profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        if index >= profiles.len() {
            return Err("Invalid profile index".to_string());
        }
        profiles[index].remove_macro(name)
            .map(|_| ())
            .ok_or_else(|| format!("Macro '{}' not found", name))
    }

    pub fn execute_macro(&self, name: &str) -> Result<(), String> {
        let profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        if index >= profiles.len() {
            return Err("Invalid profile index".to_string());
        }
        profiles[index].execute_macro(name)
    }

    pub fn start_macro_recording(&mut self) {
        let mut profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        profiles[index].start_macro_recording();
    }

    pub fn stop_macro_recording(&mut self) -> Option<Macro> {
        let mut profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        profiles[index].stop_macro_recording()
    }

    pub fn add_macro_to_current_profile(&mut self, macro_seq: Macro) {
        let mut profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        profiles[index].add_macro("Recorded Macro", macro_seq);
    }

    pub fn get_current_macro(&self) -> Option<Macro> {
        let profile = self.active_profile();
        profile.get_macro("Recorded Macro")
    }

    pub fn get_current_profile(&self) -> Profile {
        self.active_profile()
    }

    pub fn remove_macro_from_current_profile(&mut self, name: &str) {
        let mut profiles = self.profiles.lock().unwrap();
        let index = *self.active_profile_index.lock().unwrap();
        profiles[index].remove_macro(name);
    }

    pub fn start_auto_switcher(&mut self) {
        let profiles = Arc::clone(&self.profiles);
        let app_mappings = Arc::clone(&self.app_mappings);
        let active_profile_index = Arc::clone(&self.active_profile_index);
        let stop_signal = Arc::clone(&self.monitor_stop_signal);
        
        if let Some(handle) = self.monitor_thread.take() {
            stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = handle.join();
        }
        
        stop_signal.store(false, std::sync::atomic::Ordering::SeqCst);
        
        self.monitor_thread = Some(thread::spawn(move || {
            while !stop_signal.load(std::sync::atomic::Ordering::SeqCst) {
                if let Ok(active_window) = get_active_window() {
                    let mappings = app_mappings.lock().unwrap();
                    let profiles = profiles.lock().unwrap();
                    let current_index = *active_profile_index.lock().unwrap();
                    
                    // Check if there's a mapping for this window
                    let key = match Some(&active_window.title) {
                        Some(title) => format!("{}:{}", active_window.app_name, title),
                        None => active_window.app_name.clone(),
                    };
                    
                    if let Some(&profile_index) = mappings.get(&key) {
                        if current_index != profile_index {
                            *active_profile_index.lock().unwrap() = profile_index;
                            println!("Switched to profile: {}", profiles[profile_index].name);
                        }
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }
        }));
    }
    
    pub fn stop_auto_switcher(&mut self) {
        if let Some(handle) = self.monitor_thread.take() {
            self.monitor_stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = handle.join();
        }
    }
}
