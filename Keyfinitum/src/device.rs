// Keyfinitum/src/device.rs

use std::collections::HashMap;
use winapi::ctypes::c_void;
use winapi::shared::hidpi::{HIDP_PREPARSED_DATA, HidP_GetCaps};

/// Represents a connected input device
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InputDevice {
    pub device_type: DeviceType,
    pub vendor_id: u16,
    pub product_id: u16,
    pub capabilities: DeviceCapabilities,
    pub device_path: String,
    pub current_dpi: u16,
    pub min_dpi: u16,
    pub max_dpi: u16,
    pub dpi_step: u16,
}

/// Type of input device
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum DeviceType {
    Keyboard,
    Mouse,
    Other,
}

/// Capabilities of an input device
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeviceCapabilities {
    pub has_dpi_switch: bool,
    pub has_macro_keys: bool,
    pub has_media_controls: bool,
}

/// Manages connected input devices
#[allow(dead_code)]
pub struct DeviceManager {
    devices: HashMap<String, InputDevice>,
}

#[allow(dead_code)]
impl DeviceManager {
    /// Create a new DeviceManager instance
    pub fn new() -> Self {
        DeviceManager {
            devices: HashMap::new(),
        }
    }

    /// Detect and register connected devices
    pub fn detect_devices(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use std::mem::zeroed;
        use winapi::shared::hidsdi::{HidD_GetAttributes, HidD_GetPreparsedData, HidD_FreePreparsedData};
        use winapi::um::setupapi::{SetupDiGetClassDevsA, SetupDiEnumDeviceInterfaces, SetupDiGetDeviceInterfaceDetailA};
        use winapi::um::setupapi::{DIGCF_PRESENT, DIGCF_DEVICEINTERFACE};
        use winapi::shared::hidclass::GUID_DEVINTERFACE_HID;
        use winapi::shared::minwindef::DWORD;
        
        unsafe {
            let device_info = SetupDiGetClassDevsA(
                &GUID_DEVINTERFACE_HID,
                std::ptr::null(),
                std::ptr::null_mut(),
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            );

            let mut device_index = 0;
            let mut device_interface_data: winapi::um::setupapi::SP_DEVICE_INTERFACE_DATA = zeroed();
            device_interface_data.cbSize = std::mem::size_of::<winapi::um::setupapi::SP_DEVICE_INTERFACE_DATA>() as DWORD;

            while SetupDiEnumDeviceInterfaces(
                device_info,
                std::ptr::null_mut(),
                &GUID_DEVINTERFACE_HID,
                device_index,
                &mut device_interface_data,
            ) != 0 {
                // Get the required buffer size
                let mut required_size: DWORD = 0;
                SetupDiGetDeviceInterfaceDetailA(
                    device_info,
                    &mut device_interface_data as *mut _,
                    std::ptr::null_mut(),
                    0,
                    &mut required_size,
                    std::ptr::null_mut(),
                );

                // Allocate buffer and get device path
                let mut buffer = vec![0u8; required_size as usize];
                let p_device_interface_detail = buffer.as_mut_ptr() as *mut winapi::um::setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_A;
                (*p_device_interface_detail).cbSize = std::mem::size_of::<winapi::um::setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_A>() as DWORD;

                if SetupDiGetDeviceInterfaceDetailA(
                    device_info,
                    &mut device_interface_data as *mut _,
                    p_device_interface_detail,
                    required_size,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                ) != 0 {
                    let device_path = std::ffi::CStr::from_ptr((*p_device_interface_detail).DevicePath.as_ptr())
                        .to_string_lossy()
                        .into_owned();

                    let device_handle = winapi::um::fileapi::CreateFileA(
                        (*p_device_interface_detail).DevicePath.as_ptr(),
                        winapi::um::winnt::GENERIC_READ | winapi::um::winnt::GENERIC_WRITE,
                        winapi::um::winnt::FILE_SHARE_READ | winapi::um::winnt::FILE_SHARE_WRITE,
                        std::ptr::null_mut(),
                        winapi::um::fileapi::OPEN_EXISTING,
                        0,
                        std::ptr::null_mut(),
                    );

                    if device_handle != winapi::um::handleapi::INVALID_HANDLE_VALUE {
                        let mut attrs: winapi::shared::hidsdi::HIDD_ATTRIBUTES = zeroed();
                        attrs.Size = std::mem::size_of::<winapi::shared::hidsdi::HIDD_ATTRIBUTES>() as DWORD;

                        if HidD_GetAttributes(device_handle, &mut attrs) != 0 {
                            let mut preparsed_data: *mut HIDP_PREPARSED_DATA = std::ptr::null_mut();
                            if HidD_GetPreparsedData(device_handle, &mut preparsed_data) != 0 {
                                // Determine device type and capabilities
                                let device_type = self.determine_device_type(preparsed_data as *mut c_void);
                                let capabilities = self.determine_capabilities(preparsed_data as *mut c_void);

                                let device = InputDevice {
                                    device_type,
                                    vendor_id: attrs.VendorID,
                                    product_id: attrs.ProductID,
                                    capabilities,
                                    device_path: device_path.clone(),
                                    current_dpi: 800, // Default DPI
                                    min_dpi: 400,    // Common minimum
                                    max_dpi: 16000,  // Common maximum
                                    dpi_step: 100,   // Common step
                                };

                                self.devices.insert(device_path, device);

                                HidD_FreePreparsedData(preparsed_data);
                            }
                        }
                        winapi::um::handleapi::CloseHandle(device_handle);
                    }
                }
                device_index += 1;
            }
        }
        Ok(())
    }

    /// Determine the type of device based on its capabilities
    fn determine_device_type(&self, preparsed_data: *mut c_void) -> DeviceType {
        use winapi::shared::hidusage::{HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_GENERIC_MOUSE};
        
        unsafe {
            let mut caps: winapi::shared::hidpi::HIDP_CAPS = std::mem::zeroed();
            if HidP_GetCaps(preparsed_data as *mut HIDP_PREPARSED_DATA, &mut caps) == 0 {
                match caps.Usage {
                    HID_USAGE_GENERIC_KEYBOARD => DeviceType::Keyboard,
                    HID_USAGE_GENERIC_MOUSE => DeviceType::Mouse,
                    _ => DeviceType::Other,
                }
            } else {
                DeviceType::Other
            }
        }
    }

    /// Determine device capabilities based on its features
    fn determine_capabilities(&self, preparsed_data: *mut c_void) -> DeviceCapabilities {
        unsafe {
            let mut caps: winapi::shared::hidpi::HIDP_CAPS = std::mem::zeroed();
            if HidP_GetCaps(preparsed_data as *mut HIDP_PREPARSED_DATA, &mut caps) == 0 {
                // Check for specific capabilities based on the device's buttons and features
                DeviceCapabilities {
                    has_dpi_switch: caps.NumberInputButtonCaps > 0,
                    has_macro_keys: caps.NumberInputButtonCaps > 12, // Assume devices with many buttons support macros
                    has_media_controls: caps.NumberInputValueCaps > 0,
                }
            } else {
                DeviceCapabilities {
                    has_dpi_switch: false,
                    has_macro_keys: false,
                    has_media_controls: false,
                }
            }
        }
    }

    /// Get device by ID
    pub fn get_device(&self, device_id: &str) -> Option<&InputDevice> {
        self.devices.get(device_id)
    }

    /// Get all connected devices
    pub fn get_devices(&self) -> Vec<&InputDevice> {
        self.devices.values().collect()
    }

    /// Set DPI for a mouse device
    pub fn set_dpi(&mut self, device_id: &str, new_dpi: u16) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(device) = self.devices.get_mut(device_id) {
            if device.device_type != DeviceType::Mouse || !device.capabilities.has_dpi_switch {
                return Err("Device does not support DPI adjustment".into());
            }

            if new_dpi < device.min_dpi || new_dpi > device.max_dpi {
                return Err("DPI value out of range".into());
            }

            // Round to nearest step
            let new_dpi = (new_dpi / device.dpi_step) * device.dpi_step;

            unsafe {
                let device_handle = winapi::um::fileapi::CreateFileA(
                    std::ffi::CString::new(device.device_path.clone())?.as_ptr(),
                    winapi::um::winnt::GENERIC_READ | winapi::um::winnt::GENERIC_WRITE,
                    winapi::um::winnt::FILE_SHARE_READ | winapi::um::winnt::FILE_SHARE_WRITE,
                    std::ptr::null_mut(),
                    winapi::um::fileapi::OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                );

                if device_handle != winapi::um::handleapi::INVALID_HANDLE_VALUE {
                    // Prepare feature report
                    let mut buffer = vec![0u8; 8];
                    buffer[0] = 0x04; // Feature report ID for DPI
                    buffer[1] = (new_dpi >> 8) as u8;
                    buffer[2] = (new_dpi & 0xFF) as u8;

                    let success = winapi::shared::hidsdi::HidD_SetFeature(
                        device_handle as *mut c_void,
                        buffer.as_mut_ptr() as *mut c_void,
                        buffer.len() as u32,
                    );

                    winapi::um::handleapi::CloseHandle(device_handle);

                    if success != 0 {
                        device.current_dpi = new_dpi;
                        Ok(())
                    } else {
                        Err("Failed to set DPI".into())
                    }
                } else {
                    Err("Failed to open device".into())
                }
            }
        } else {
            Err("Device not found".into())
        }
    }

    /// Increase DPI for a mouse device
    pub fn increase_dpi(&mut self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(device) = self.devices.get(device_id) {
            let new_dpi = std::cmp::min(device.current_dpi + device.dpi_step, device.max_dpi);
            self.set_dpi(device_id, new_dpi)
        } else {
            Err("Device not found".into())
        }
    }

    /// Decrease DPI for a mouse device
    pub fn decrease_dpi(&mut self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(device) = self.devices.get(device_id) {
            let new_dpi = std::cmp::max(device.current_dpi.saturating_sub(device.dpi_step), device.min_dpi);
            self.set_dpi(device_id, new_dpi)
        } else {
            Err("Device not found".into())
        }
    }

    /// Get current DPI for a mouse device
    pub fn get_dpi(&self, device_id: &str) -> Option<u16> {
        self.devices.get(device_id).map(|device| device.current_dpi)
    }
}
