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

    fn upload_ebook(&self, ebook: &Path) {
        // TODO: Implement
        println!("sending {:?} to Libra 2", ebook);
    }
}
