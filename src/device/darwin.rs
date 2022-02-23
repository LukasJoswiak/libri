//
// This file contains macOS-specific functionality to recognize connected USB devices. It reads
// device information including the USB vendor and product IDs, and contains logic to determine the
// mount path of each USB device. The following resources were used extensively in the creation of
// this file:
//
//   * https://svartalf.info/posts/2019-05-31-poking-the-macos-io-kit-with-rust/
//   * https://github.com/svartalf/rust-battery
//   * https://github.com/jtakakura/io-kit-rs
//

#![cfg(target_os = "macos")]
#![allow(non_camel_case_types, non_upper_case_globals)]

use std::error::Error;
use std::mem::MaybeUninit;

use core_foundation::base::{
    kCFAllocatorDefault, mach_port_t, CFAllocatorRef, CFRelease, CFTypeRef, FromVoid,
};
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use libc::c_char;
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

// TODO: This is a temporary struct. Will be replaced once functionality to associate a BSD path
// with the associated mount path is complete.
#[derive(Debug)]
pub struct MountedDevice {
    bsd_path: String,
    manufacturer: String,
    name: String,
    vendor_id: u16,
    product_id: u16,
}

pub fn usb_devices() -> Result<Vec<MountedDevice>, Box<dyn Error>> {
    let mut devices: Vec<MountedDevice> = Vec::new();

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
                devices.push(MountedDevice {
                    bsd_path: bsd_name,
                    manufacturer: vendor_name,
                    name: product_name,
                    vendor_id: vendor_id.try_into().unwrap(),
                    product_id: product_id.try_into().unwrap(),
                });
            }
            service = IOIteratorNext(iterator);
        }
        IOObjectRelease(iterator);
    }

    // TODO: Connect BSD name to mount path using getfsstat
    Ok(devices)
}
