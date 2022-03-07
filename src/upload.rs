use std::error::Error;

use super::config;
use super::device;
use super::list;

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let available_devices = device::available_devices()?;
    // TODO: Filter devices based on user predicates
    if available_devices.len() == 0 {
        println!("no devices available");
        return Ok(());
    }
    // FIXME: Modules are starting to become connected... perhaps list::get_ebooks should be moved to the
    // common module in the future.
    let ebooks = list::get_ebooks(&config.library)?;
    // TODO: Filter ebooks based on user predicates
    if ebooks.len() == 0 {
        println!("no ebooks selected");
        return Ok(());
    }
    available_devices.iter().for_each(|device| {
        ebooks.iter().for_each(|ebook| {
            device.upload_ebook(&ebook).unwrap();
            println!("uploaded {} to {}", &ebook.title, device.name());
        })
    });
    // TODO: Add statistics, similar to `libri import`
    Ok(())
}
