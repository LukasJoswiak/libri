use std::error::Error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "macos")]
use xml::reader::EventReader;
use xml::reader::{self, XmlEvent};

#[derive(Debug)]
pub struct Device {
    name: String,
    manufacturer: String,
    usb_device: Box<dyn UsbDevice>,
}

impl Device {
    pub fn new(name: String, manufacturer: String, usb_device: Box<dyn UsbDevice>) -> Device {
        Device {
            name,
            manufacturer,
            usb_device,
        }
    }

    pub fn upload_ebook(&self, ebook: &Path) {
        self.usb_device.upload_ebook(&ebook);
    }
}

/// Types that implement this trait represent physical USB eReader hardware connected to the computer.
pub trait UsbDevice {
    /// Returns the vendor ID of the USB device.
    fn vendor_id(&self) -> u16;
    /// Returns the product ID of the USB device.
    fn product_id(&self) -> u16;

    /// Uploads the specified ebook to the correct location on the device such that it will be
    /// recognized automatically.
    fn upload_ebook(&self, ebook: &Path);
}

impl fmt::Debug for dyn UsbDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("dyn UsbDevice")
            .field("vendor_id", &self.vendor_id())
            .field("product_id", &self.product_id())
            .finish()
    }
}

// TODO: Device specific configuration should be moved to a separate file
const KOBO_VENDOR_ID: u16 = 0x2237;
const LIBRA_2_PRODUCT_ID: u16 = 0x4234;

struct Libra2 {}

impl UsbDevice for Libra2 {
    fn vendor_id(&self) -> u16 {
        KOBO_VENDOR_ID
    }

    fn product_id(&self) -> u16 {
        LIBRA_2_PRODUCT_ID
    }

    fn upload_ebook(&self, ebook: &Path) {
        // TODO: Implement
        println!("sending {:?} to Libra 2", ebook);
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
}
