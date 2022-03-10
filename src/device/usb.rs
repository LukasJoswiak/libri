use std::fmt;
use std::io;
use std::path::Path;

use super::super::Ebook;

/// Types that implement this trait represent physical USB eReader hardware connected to the computer.
pub trait UsbDevice {
    /// Returns the directory where the device is mounted.
    fn mount_dir(&self) -> &Path;

    /// Returns the vendor ID of the USB device.
    fn vendor_id(&self) -> u16;
    /// Returns the product ID of the USB device.
    fn product_id(&self) -> u16;

    /// Uploads the specified ebook to the correct location on the device such that it will be
    /// recognized automatically.
    fn upload_ebook(&self, ebook: &Ebook, dry_run: bool) -> Result<(), io::Error>;
}

impl fmt::Debug for dyn UsbDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("dyn UsbDevice")
            .field("mount_dir", &self.mount_dir())
            .field("vendor_id", &self.vendor_id())
            .field("product_id", &self.product_id())
            .finish()
    }
}
