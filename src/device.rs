use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use xml::reader::{self, EventReader, XmlEvent};

/// Low-level information about a mounted USB device.
#[derive(Debug)]
pub struct UsbDevice {
    mount_point: PathBuf,
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
fn mounted_devices(data: &[u8]) -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    // TODO: Move to separate fn, have this take a vec of bytes, unit test

    let mut depth = 0;
    let mut search_depth: Option<usize> = None;
    let mut devices: Vec<UsbDevice> = Vec::new();

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
                        search_depth = None;
                    }
                }
            }
            Ok(XmlEvent::Characters(s)) => {
                // It seems "Media" elements will usually contain the mount path. Start the search
                // here.
                if s == "Media" {
                    search_depth = Some(depth - 1);
                    continue;
                }

                if search_depth.is_some() {
                    // When the search is active, record data for the mount point, vendor ID, and
                    // product ID of each device.
                    if s == "mount_point" {
                        let value = get_value(&mut iter)?;
                        devices.push(UsbDevice {
                            mount_point: PathBuf::from(value),
                            vendor_id: 0,
                            product_id: 0,
                        });
                    } else if s == "vendor_id" {
                        let value = get_value(&mut iter)?;
                        let value = value.split_whitespace().collect::<Vec<&str>>()[0]
                            .trim_start_matches("0x");
                        let vendor_id = u16::from_str_radix(value, 16)?;
                        devices.last_mut().unwrap().vendor_id = vendor_id;
                    } else if s == "product_id" {
                        let value = get_value(&mut iter)?;
                        let value = value.trim_start_matches("0x");
                        let product_id = u16::from_str_radix(value, 16)?;
                        devices.last_mut().unwrap().product_id = product_id;
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
fn mounted_devices(data: &[u8]) -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    // TODO: Implement
    panic!("device recognition not yet implemented for Linux");
}

/// Returns a list of mounted devices.
#[cfg(target_os = "windows")]
fn mounted_devices(data: &[u8]) -> Result<Vec<UsbDevice>, Box<dyn Error>> {
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

    let _devices = mounted_devices(&output)?;
    // TODO: Filter devices based on predefined, known, supported ereader vendor/product IDs

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_value() {
        let xml_str = "<dict> \
            <key>Media</key> \
            <array> \
                <dict> \
                    <array> \
                        <dict> \
                            <key>mount_point</key>
                            <string>/path/to/mount/point</string>
                        </dict> \
                    </array> \
                </dict> \
            </array> \
            <key>product_id</key> \
            <string>0x1234</string> \
            <key>vendor_id</key> \
            <string>0x4321  (Company Name)</string> \
          </dict>";
        let devices = mounted_devices(xml_str.as_bytes()).unwrap();
        assert_eq!(devices.len(), 1);

        let device = &devices[0];
        assert_eq!(device.mount_point, PathBuf::from("/path/to/mount/point"));
        assert_eq!(device.vendor_id, 0x4321);
        assert_eq!(device.product_id, 0x1234);
    }
}
