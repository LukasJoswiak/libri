use std::path::Path;

use super::UsbDevice;

pub const KOBO_VENDOR_ID: u16 = 0x2237;
pub const LIBRA_2_PRODUCT_ID: u16 = 0x4234;

pub struct Libra2 {}

impl UsbDevice for Libra2 {
    fn vendor_id(&self) -> u16 {
        KOBO_VENDOR_ID
    }

    fn product_id(&self) -> u16 {
        LIBRA_2_PRODUCT_ID
    }

    fn upload_ebook(&self, _source: &Path) {
        // TODO: Implement. All this method needs to do is copy the source file to the right place
        // on the device. The only logic that is expected to go in this function is device specific
        // logic relating to where books need to be stored in order to be recognized correctly, and
        // perhaps how they should be renamed. May refactor this function to take an Ebook struct,
        // or if the dependencies become circular then at least the ebook title and author to
        // assist in potential renaming.
    }
}
