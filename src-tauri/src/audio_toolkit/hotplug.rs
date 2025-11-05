//! Hot-plug audio device detection
//!
//! Detects when audio devices are connected or disconnected and notifies the application.

use cpal::traits::{DeviceTrait, HostTrait};
use log::{debug, info, warn};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Audio device information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceInfo {
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
}

/// Hot-plug event
#[derive(Clone, Debug)]
pub enum HotplugEvent {
    /// Device was connected
    Connected(DeviceInfo),
    /// Device was disconnected
    Disconnected(DeviceInfo),
}

/// Hot-plug detector
///
/// Monitors audio devices and detects connect/disconnect events.
pub struct HotplugDetector {
    known_devices: Arc<Mutex<HashSet<DeviceInfo>>>,
    is_running: Arc<Mutex<bool>>,
    poll_interval: Duration,
}

impl HotplugDetector {
    /// Creates a new hot-plug detector
    ///
    /// # Arguments
    ///
    /// * `poll_interval` - How often to check for device changes
    pub fn new(poll_interval: Duration) -> Self {
        Self {
            known_devices: Arc::new(Mutex::new(HashSet::new())),
            is_running: Arc::new(Mutex::new(false)),
            poll_interval,
        }
    }

    /// Creates detector with default 2-second poll interval
    pub fn default() -> Self {
        Self::new(Duration::from_secs(2))
    }

    /// Starts monitoring for device changes
    ///
    /// # Arguments
    ///
    /// * `callback` - Called when devices are connected/disconnected
    pub fn start<F>(&self, callback: F)
    where
        F: Fn(HotplugEvent) + Send + 'static,
    {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            warn!("Hot-plug detector already running");
            return;
        }

        *is_running = true;
        drop(is_running);

        // Initialize known devices
        self.scan_devices();

        let known_devices = Arc::clone(&self.known_devices);
        let is_running = Arc::clone(&self.is_running);
        let poll_interval = self.poll_interval;

        thread::spawn(move || {
            info!("Hot-plug detector started");

            while *is_running.lock().unwrap() {
                let current_devices = Self::get_current_devices();
                let mut known = known_devices.lock().unwrap();

                // Find newly connected devices
                for device in &current_devices {
                    if !known.contains(device) {
                        info!("Device connected: {:?}", device.name);
                        callback(HotplugEvent::Connected(device.clone()));
                    }
                }

                // Find disconnected devices
                let disconnected: Vec<_> = known
                    .iter()
                    .filter(|d| !current_devices.contains(d))
                    .cloned()
                    .collect();

                for device in disconnected {
                    info!("Device disconnected: {:?}", device.name);
                    callback(HotplugEvent::Disconnected(device.clone()));
                }

                // Update known devices
                *known = current_devices;
                drop(known);

                thread::sleep(poll_interval);
            }

            info!("Hot-plug detector stopped");
        });
    }

    /// Stops monitoring
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }

    /// Gets a snapshot of current devices
    fn get_current_devices() -> HashSet<DeviceInfo> {
        let mut devices = HashSet::new();

        let host = cpal::default_host();

        // Get input devices
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let is_default = host
                        .default_input_device()
                        .and_then(|d| d.name().ok())
                        .map(|n| n == name)
                        .unwrap_or(false);

                    devices.insert(DeviceInfo {
                        name,
                        is_input: true,
                        is_default,
                    });
                }
            }
        }

        // Get output devices
        if let Ok(output_devices) = host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    let is_default = host
                        .default_output_device()
                        .and_then(|d| d.name().ok())
                        .map(|n| n == name)
                        .unwrap_or(false);

                    devices.insert(DeviceInfo {
                        name,
                        is_input: false,
                        is_default,
                    });
                }
            }
        }

        debug!("Found {} audio devices", devices.len());
        devices
    }

    /// Scans and initializes known devices
    fn scan_devices(&self) {
        let devices = Self::get_current_devices();
        *self.known_devices.lock().unwrap() = devices;
        info!("Initial device scan complete");
    }

    /// Gets list of currently known devices
    pub fn known_devices(&self) -> Vec<DeviceInfo> {
        self.known_devices.lock().unwrap().iter().cloned().collect()
    }

    /// Checks if detector is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_equality() {
        let dev1 = DeviceInfo {
            name: "Test".to_string(),
            is_input: true,
            is_default: false,
        };

        let dev2 = DeviceInfo {
            name: "Test".to_string(),
            is_input: true,
            is_default: false,
        };

        assert_eq!(dev1, dev2);
    }

    #[test]
    fn test_device_info_hash() {
        let mut set = HashSet::new();
        let dev = DeviceInfo {
            name: "Test".to_string(),
            is_input: true,
            is_default: false,
        };

        set.insert(dev.clone());
        assert!(set.contains(&dev));
    }

    #[test]
    fn test_detector_creation() {
        let detector = HotplugDetector::default();
        assert!(!detector.is_running());
    }

    #[test]
    fn test_detector_with_custom_interval() {
        let detector = HotplugDetector::new(Duration::from_secs(5));
        assert_eq!(detector.poll_interval, Duration::from_secs(5));
    }

    #[test]
    fn test_get_current_devices() {
        let devices = HotplugDetector::get_current_devices();
        // Should find at least some devices on most systems
        // But we can't assert specific count as it varies
        println!("Found {} devices", devices.len());
    }
}
