use std::fmt;
use std::path::Path;

/// Types that implement this trait represent physical USB eReader hardware connected to the computer.
pub trait UsbDevice {
    /// Returns the vendor ID of the USB device.
    fn vendor_id(&self) -> u16;
    /// Returns the product ID of the USB device.
    fn product_id(&self) -> u16;

    /// Uploads the specified ebook to the correct location on the device such that it will be
    /// recognized automatically.
    fn upload_ebook(&self, source: &Path);
}

impl fmt::Debug for dyn UsbDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("dyn UsbDevice")
            .field("vendor_id", &self.vendor_id())
            .field("product_id", &self.product_id())
            .finish()
    }
}
