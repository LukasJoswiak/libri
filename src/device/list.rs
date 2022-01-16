use std::error::Error;
use std::io::{self, Write};

use tabwriter::TabWriter;

pub fn run() -> Result<(), Box<dyn Error>> {
    let devices = super::mounted_devices()?;
    let available_devices = super::filter(devices);
    let mut tw = TabWriter::new(io::stdout());
    write!(&mut tw, "\x1b[1mUID\tName\tManufacturer\x1b[0m\n").unwrap();
    for device in available_devices {
        // This is my lazy effort to generate a UID for each device. It's probably good enough, but
        // could also probably be improved in the future.
        let device_uid = device.usb_info.vendor_id() ^ device.usb_info.product_id();
        write!(
            &mut tw,
            "{}\t{}\t{}\n",
            device_uid, device.name, device.manufacturer
        )
        .unwrap();
    }
    tw.flush().unwrap();
    Ok(())
}
