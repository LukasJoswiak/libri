mod darwin;
mod kobo;
pub mod list;
mod usb;

use std::error::Error;
use std::path::PathBuf;

use super::Ebook;
use kobo::{Libra2, KOBO_VENDOR_ID, LIBRA_2_PRODUCT_ID};
use usb::UsbDevice;

#[derive(Debug)]
pub struct Device {
    name: String,
    manufacturer: String,
    usb_info: Box<dyn UsbDevice>,
}

impl Device {
    pub fn new(name: String, manufacturer: String, usb_info: Box<dyn UsbDevice>) -> Device {
        Device {
            name,
            manufacturer,
            usb_info,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }

    pub fn upload_ebook(&self, ebook: &Ebook, dry_run: bool) -> Result<(), Box<dyn Error>> {
        self.usb_info.upload_ebook(&ebook, dry_run)
    }
}

/// Low-level information about a mounted USB device. Other modules should use the specific device
/// struct which implements the UsbDevice trait to interact with eReaders.
#[derive(Debug, Default)]
pub struct MountedDevice {
    mount_point: PathBuf,
    manufacturer: String,
    name: String,
    vendor_id: u16,
    product_id: u16,
}

/// Returns a list of mounted devices (macOS specific).
#[cfg(target_os = "macos")]
fn mounted_devices() -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    darwin::usb_devices()
}

/// Returns a list of mounted devices (Linux specific).
#[cfg(target_os = "linux")]
fn mounted_devices() -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    // TODO: Implement
    panic!("device recognition not yet implemented for Linux");
}

/// Returns a list of mounted devices (Windows specific).
#[cfg(target_os = "windows")]
fn mounted_devices() -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    // TODO: Implement
    panic!("device recognition not yet implemented for Windows");
}

// TODO: Add support for other OS's (the BSDs)

/// Filters the list of mounted devices and returns a list of supported eReaders.
fn filter(devices: Vec<MountedDevice>) -> Vec<Device> {
    let mut available_devices: Vec<Device> = Vec::new();
    for device in devices {
        match (device.vendor_id, device.product_id) {
            (KOBO_VENDOR_ID, LIBRA_2_PRODUCT_ID) => {
                available_devices.push(Device::new(
                    device.name,
                    device.manufacturer,
                    Box::new(Libra2::new(device.mount_point)),
                ));
            }
            _ => {}
        }
    }
    available_devices
}

pub fn available_devices() -> Result<Vec<Device>, Box<dyn Error>> {
    let devices = mounted_devices()?;
    Ok(filter(devices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")] // TODO: Can run against all platforms once support is added
    fn filter_devices() {
        let devices = vec![
            MountedDevice {
                mount_point: PathBuf::from("/path/to/libra2"),
                manufacturer: "Kobo".to_string(),
                name: "Libra 2".to_string(),
                vendor_id: KOBO_VENDOR_ID,
                product_id: LIBRA_2_PRODUCT_ID,
            },
            MountedDevice {
                mount_point: PathBuf::from("/path/to/other"),
                manufacturer: "Unknown".to_string(),
                name: "Name".to_string(),
                vendor_id: 1,
                product_id: 2,
            },
        ];
        let available_devices = filter(devices);
        assert_eq!(available_devices.len(), 1);

        let device = &available_devices[0];
        assert_eq!(device.usb_info.vendor_id(), KOBO_VENDOR_ID);
        assert_eq!(device.usb_info.product_id(), LIBRA_2_PRODUCT_ID);
    }
}
