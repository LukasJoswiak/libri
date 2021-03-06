//
// This file contains macOS-specific functionality to recognize connected USB devices. It reads
// device information including the USB vendor and product IDs, and contains logic to determine the
// mount path of each USB device. The following resources were used extensively in the creation of
// this file:
//
//   * https://svartalf.info/posts/2019-05-31-poking-the-macos-io-kit-with-rust/
//   * https://github.com/svartalf/rust-battery
//   * https://github.com/jtakakura/io-kit-rs
//   * https://github.com/kovidgoyal/calibre/blob/master/src/calibre/devices/usbobserver/usbobserver.c
//

#![cfg(target_os = "macos")]
#![allow(non_camel_case_types, non_upper_case_globals)]

use std::collections::HashMap;
use std::error::Error;
use std::ffi::CStr;
use std::mem::{self, MaybeUninit};
use std::path::PathBuf;

use core_foundation::base::{
    kCFAllocatorDefault, mach_port_t, CFAllocatorRef, CFRelease, CFTypeRef, FromVoid,
};
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use libc::{c_char, c_int, getfsstat, statfs, MNT_NOWAIT};
use mach::{kern_return, port};

pub type IOOptionBits = u32;

pub const kIOMainPortDefault: mach_port_t = port::MACH_PORT_NULL;
pub const kIOServicePlane: *const c_char = "IOService\0".as_ptr() as *const c_char;
pub const kIOBSDNameKey: *const c_char = "BSD Name\0".as_ptr() as *const c_char;
pub const kUSBVendorID: *const c_char = "idVendor\0".as_ptr() as *const c_char;
pub const kUSBVendorString: *const c_char = "USB Vendor Name\0".as_ptr() as *const c_char;
pub const kUSBProductID: *const c_char = "idProduct\0".as_ptr() as *const c_char;
pub const kUSBProductString: *const c_char = "USB Product Name\0".as_ptr() as *const c_char;

pub const kIORegistryIterateRecursively: IOOptionBits = 0x00000001;
// pub const kIORegistryIterateParents: IOOptionBits = 0x00000002;

type io_object_t = mach_port_t;
type io_registry_entry_t = io_object_t;
type io_iterator_t = io_object_t;
type io_name_t = *const c_char;

extern "C" {
    static mut errno: c_int;

    pub fn IOServiceMatching(name: *const c_char) -> CFMutableDictionaryRef;

    pub fn IOServiceGetMatchingServices(
        masterPort: mach_port_t,
        matching: CFDictionaryRef,
        existing: *mut io_iterator_t,
    ) -> kern_return::kern_return_t;

    pub fn IORegistryEntrySearchCFProperty(
        entry: io_registry_entry_t,
        plane: io_name_t,
        key: CFStringRef,
        allocator: CFAllocatorRef,
        options: IOOptionBits,
    ) -> CFTypeRef;

    pub fn IOIteratorNext(iterator: io_iterator_t) -> io_object_t;

    pub fn IOObjectRelease(object: io_object_t) -> kern_return::kern_return_t;

    fn __CFStringMakeConstantString(c_str: *const c_char) -> CFStringRef;
}

#[allow(non_snake_case)]
fn CFSTR(c_str: *const c_char) -> CFStringRef {
    unsafe { __CFStringMakeConstantString(c_str) }
}

trait Property {
    type Output;
    unsafe fn parse(property: CFTypeRef) -> Option<Self::Output>;
}

impl Property for i32 {
    type Output = i32;
    unsafe fn parse(property: CFTypeRef) -> Option<i32> {
        CFNumber::from_void(property).to_i32()
    }
}

impl Property for String {
    type Output = String;
    unsafe fn parse(property: CFTypeRef) -> Option<String> {
        Some(CFString::from_void(property).to_string())
    }
}

unsafe fn find_property<T>(service: io_object_t, key: CFStringRef) -> Option<T>
where
    T: Property<Output = T>,
{
    let property = IORegistryEntrySearchCFProperty(
        service,
        kIOServicePlane,
        key,
        kCFAllocatorDefault,
        kIORegistryIterateRecursively,
    );
    if property.as_ref().is_none() {
        return None;
    }

    let parsed = T::parse(property);
    CFRelease(property);
    parsed
}

fn mounted_file_systems() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut results: HashMap<String, String> = HashMap::new();

    unsafe {
        let mut num = getfsstat(std::ptr::null_mut(), 0, MNT_NOWAIT);
        if num == -1 {
            panic!(
                "failed to read the number of mounted file systems: {}",
                errno
            );
        }

        // MNT_NOWAIT causes getfsstat to immediately return instead of blocking on slow file
        // systems. Add a few extra slots in case the number of mounted file systems is too low.
        num += 5;

        let mut mounted = Vec::with_capacity(num as usize);
        num = getfsstat(
            mounted.as_mut_ptr(),
            num * mem::size_of::<statfs>() as i32,
            MNT_NOWAIT,
        );
        if num == -1 {
            panic!("failed to retrieve info on mounted file systems: {}", errno);
        }
        mounted.set_len(num as usize);

        for stat in mounted {
            let from_name_str = CStr::from_ptr(stat.f_mntfromname.as_ptr()).to_str();
            let name_str = CStr::from_ptr(stat.f_mntonname.as_ptr()).to_str();

            if let (Ok(from_name), Ok(name)) = (from_name_str, name_str) {
                results.insert(from_name.to_owned(), name.to_owned());
            }
        }
    }

    Ok(results)
}

pub fn usb_devices() -> Result<Vec<super::MountedDevice>, Box<dyn Error>> {
    let mounted = mounted_file_systems().unwrap();
    let mut devices: Vec<super::MountedDevice> = Vec::new();

    unsafe {
        let matching_dict = IOServiceMatching(b"IOUSBHostDevice\0".as_ptr() as *const c_char);
        if matching_dict.as_ref().is_none() {
            panic!("failed to create matching dictionary for USB devices");
        }

        let mut iterator: io_iterator_t = MaybeUninit::uninit().assume_init();
        let ret = IOServiceGetMatchingServices(kIOMainPortDefault, matching_dict, &mut iterator);
        if ret != kern_return::KERN_SUCCESS {
            panic!("failed to retrieve USB devices");
        }

        let mut service = IOIteratorNext(iterator);
        while service != 0 {
            let bsd_name = find_property::<String>(service, CFSTR(kIOBSDNameKey));
            let vendor_id = find_property::<i32>(service, CFSTR(kUSBVendorID));
            let vendor_name = find_property::<String>(service, CFSTR(kUSBVendorString));
            let product_id = find_property::<i32>(service, CFSTR(kUSBProductID));
            let product_name = find_property::<String>(service, CFSTR(kUSBProductString));

            if let (
                Some(mut bsd_name),
                Some(vendor_id),
                Some(vendor_name),
                Some(product_id),
                Some(product_name),
            ) = (bsd_name, vendor_id, vendor_name, product_id, product_name)
            {
                // TODO: Replace with _PATH_DEV constant from paths.h system header
                bsd_name.insert_str(0, "/dev/");
                if let Some(mount_point) = mounted.get(&bsd_name) {
                    devices.push(super::MountedDevice {
                        mount_point: PathBuf::from(mount_point),
                        manufacturer: vendor_name,
                        name: product_name,
                        vendor_id: vendor_id.try_into().unwrap(),
                        product_id: product_id.try_into().unwrap(),
                    });
                }
            }
            service = IOIteratorNext(iterator);
        }
        IOObjectRelease(iterator);
    }

    Ok(devices)
}
