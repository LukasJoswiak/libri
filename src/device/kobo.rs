use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::{common, Ebook};
use super::UsbDevice;

pub const KOBO_VENDOR_ID: u16 = 0x2237;
pub const LIBRA_2_PRODUCT_ID: u16 = 0x4234;

pub struct Libra2 {
    mount_dir: PathBuf,
}

impl Libra2 {
    pub fn new(mount_dir: PathBuf) -> Libra2 {
        Libra2 { mount_dir }
    }
}

impl UsbDevice for Libra2 {
    fn mount_dir(&self) -> &Path {
        self.mount_dir.as_path()
    }

    fn vendor_id(&self) -> u16 {
        KOBO_VENDOR_ID
    }

    fn product_id(&self) -> u16 {
        LIBRA_2_PRODUCT_ID
    }

    // TODO: Add option to auto-convert epubs to kepubs!
    fn upload_ebook(&self, ebook: &Ebook, library: &Path, dry_run: bool) -> Result<(), io::Error> {
        // TODO: Factor out any common logic that can be reused across devices
        let mut destination = self.mount_dir().to_path_buf();
        destination.push(ebook.path.strip_prefix(library).unwrap());
        if destination.exists() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "already on device").into());
        }

        if !dry_run {
            fs::create_dir_all(&destination.parent().unwrap())?;
            common::copy(&ebook.path, &destination)?;
        }
        Ok(())
    }
}
