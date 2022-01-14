mod kobo;
mod usb;

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use kobo::{Libra2, KOBO_VENDOR_ID, LIBRA_2_PRODUCT_ID};
use usb::UsbDevice;
#[cfg(target_os = "macos")]
use xml::reader::EventReader;
use xml::reader::{self, XmlEvent};

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

    pub fn upload_ebook(&self, ebook: &Path) {
        self.usb_info.upload_ebook(&ebook);
    }
}

/// Low-level information about a mounted USB device.
#[derive(Debug, Default)]
pub struct MountedDevice {
    mount_point: PathBuf,
    manufacturer: String,
    name: String,
    vendor_id: u16,
    product_id: u16,
}

/// Returns the contents of the value associated with a key in an XML dictionary. Pass an iterator
/// currently pointing to the contents of the key.
///
/// Example XML:
///
/// # Examples
///
/// ```
/// use xml::reader::{EventReader, XmlEvent};
/// use libri::device;
///
/// let xml_str = "<dict> \
///     <key>foo</key> \
///     <value>bar</value> \
///   </dict>";
/// let parser = EventReader::from_str(xml_str);
/// let mut iter = parser.into_iter();
/// while let Some(e) = iter.next() {
///     match e {
///         Ok(XmlEvent::Characters(s)) => {
///             let value = device::get_value(&mut iter).unwrap();
///             assert_eq!(value, "bar");
///         },
///         _ => {},
///     }
/// }
/// ```
pub fn get_value<R: io::Read>(iter: &mut reader::Events<R>) -> Result<String, reader::Error> {
    // Advance to the value associated with the key.
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    let value = iter.next().expect("missing value for xml key");
    match value {
        Ok(XmlEvent::Characters(s)) => return Ok(s),
        Err(e) => return Err(e),
        _ => {}
    };
    panic!("unknown error when parsing value for xml key");
}

/// Returns a list of mounted devices (macOS specific).
#[cfg(target_os = "macos")]
fn mounted_devices(data: &[u8]) -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    let mut depth = 0;
    let mut search_depth: Option<usize> = None;
    let mut device: Option<MountedDevice> = None;
    let mut devices: Vec<MountedDevice> = Vec::new();

    let parser = EventReader::new(data);
    let mut iter = parser.into_iter();

    while let Some(e) = iter.next() {
        match e {
            Ok(XmlEvent::StartElement { .. }) => depth += 1,
            Ok(XmlEvent::EndElement { .. }) => {
                depth -= 1;

                // Stop searching for a mount point and associated vendor and product data when the
                // search leaves the relevant block.
                if let Some(d) = search_depth {
                    if depth < d {
                        if let Some(device) = device {
                            if !device.mount_point.as_os_str().is_empty() {
                                devices.push(device);
                            }
                        }
                        device = None;
                        search_depth = None;
                    }
                }
            }
            Ok(XmlEvent::Characters(s)) => {
                // It seems "Media" elements will usually contain the mount path. Start the search
                // here.
                if s == "Media" {
                    device = Some(MountedDevice::default());
                    search_depth = Some(depth - 1);
                    continue;
                }

                if search_depth.is_some() {
                    // When the search is active, record the metadata of the device.
                    if s == "mount_point" {
                        let value = get_value(&mut iter)?;
                        device.as_mut().unwrap().mount_point = PathBuf::from(value);
                    } else if s == "manufacturer" {
                        let value = get_value(&mut iter)?;
                        device.as_mut().unwrap().manufacturer = value;
                    } else if s == "_name" {
                        let value = get_value(&mut iter)?;
                        device.as_mut().unwrap().name = value;
                    } else if s == "vendor_id" {
                        let value = get_value(&mut iter)?;
                        let value = value.split_whitespace().collect::<Vec<&str>>()[0]
                            .trim_start_matches("0x");
                        let vendor_id = u16::from_str_radix(value, 16)?;
                        device.as_mut().unwrap().vendor_id = vendor_id;
                    } else if s == "product_id" {
                        let value = get_value(&mut iter)?;
                        let value = value.trim_start_matches("0x");
                        let product_id = u16::from_str_radix(value, 16)?;
                        device.as_mut().unwrap().product_id = product_id;
                    }
                }
            }
            Err(e) => {
                println!("error parsing system_profiler output: {}", e);
                break;
            }
            _ => {}
        }
    }
    Ok(devices)
}

/// Returns a list of mounted devices.
#[cfg(target_os = "linux")]
fn mounted_devices(_data: &[u8]) -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    // TODO: Implement
    panic!("device recognition not yet implemented for Linux");
}

/// Returns a list of mounted devices.
#[cfg(target_os = "windows")]
fn mounted_devices(_data: &[u8]) -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    // TODO: Implement
    panic!("device recognition not yet implemented for Windows");
}

// TODO: Add support for other OS's (the BSDs)

/// Filters the list of mounted devices and returns a list of eReaders that support uploading of
/// ebooks.
fn filter(devices: Vec<MountedDevice>) -> Vec<Device> {
    let mut available_devices: Vec<Device> = Vec::new();
    for device in devices {
        match (device.vendor_id, device.product_id) {
            (KOBO_VENDOR_ID, LIBRA_2_PRODUCT_ID) => {
                available_devices.push(Device::new(
                    device.name,
                    device.manufacturer,
                    Box::new(Libra2 {}),
                ));
            }
            _ => {}
        }
    }
    available_devices
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // Parse the output of `system_profiler SPUSBDataType -xml` to read the mount point of each USB
    // device, as well as its vendor and product ID.
    let output = if cfg!(target_os = "macos") {
        Command::new("system_profiler")
            .arg("SPUSBDataType")
            .arg("-xml")
            .output()
            .expect("failed to execute system_profiler")
            .stdout
    } else if cfg!(target_os = "linux") {
        // TODO: Implement
        vec![]
    } else if cfg!(target_os = "windows") {
        // TODO: Implement
        vec![]
    } else {
        vec![]
    };

    let devices = mounted_devices(&output)?;
    let available_devices = filter(devices);
    available_devices
        .iter()
        .for_each(|device| device.upload_ebook(&PathBuf::from("")));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn parse_system_profiler() {
        let xml_str = "<dict> \
            <key>Media</key> \
            <array> \
                <dict> \
                    <key>_name</key> \
                    <string>Device Name</string> \
                    <array> \
                        <dict> \
                            <key>mount_point</key> \
                            <string>/path/to/mount/point</string> \
                        </dict> \
                    </array> \
                </dict> \
            </array> \
            <key>manufacturer</key> \
            <string>Manufacturer</string> \
            <key>product_id</key> \
            <string>0x1234</string> \
            <key>vendor_id</key> \
            <string>0x4321  (Company Name)</string> \
          </dict>";
        let devices = mounted_devices(xml_str.as_bytes()).unwrap();
        assert_eq!(devices.len(), 1);

        let device = &devices[0];
        assert_eq!(device.mount_point, PathBuf::from("/path/to/mount/point"));
        assert_eq!(device.manufacturer, "Manufacturer");
        assert_eq!(device.name, "Device Name");
        assert_eq!(device.vendor_id, 0x4321);
        assert_eq!(device.product_id, 0x1234);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn parse_unmounted_system_profiler() {
        let xml_str = "<dict> \
            <key>Media</key> \
            <array> \
                <dict> \
                    <key>_name</key> \
                    <string>Device Name</string> \
                    <array> \
                        <dict> \
                            <key>volume_uuid</key> \
                            <string>abcd</string> \
                        </dict> \
                    </array> \
                </dict> \
            </array> \
            <key>manufacturer</key> \
            <string>Manufacturer</string> \
            <key>product_id</key> \
            <string>0x1234</string> \
            <key>vendor_id</key> \
            <string>0x4321  (Company Name)</string> \
          </dict>";
        let devices = mounted_devices(xml_str.as_bytes()).unwrap();
        assert_eq!(devices.len(), 0);
    }

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
