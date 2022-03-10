use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};

use super::config;
use super::device;
use super::list;

struct UploadStats {
    uploaded: u32,
    skipped: u32,
    elapsed: Duration,
}

impl fmt::Display for UploadStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "uploaded {}; skipped {}; finished in {:.2}s",
            self.uploaded,
            self.skipped,
            self.elapsed.as_secs_f32()
        )
    }
}

pub fn run(config: &config::Config, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let mut stats = UploadStats {
        uploaded: 0,
        skipped: 0,
        elapsed: Duration::ZERO,
    };
    let start = Instant::now();

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
        println!("{}", device.name());
        ebooks
            .iter()
            .for_each(|ebook| match device.upload_ebook(&ebook, dry_run) {
                Ok(_) => {
                    stats.uploaded += 1;
                    println!("uploaded \"{}\"", &ebook.title);
                }
                Err(e) => {
                    stats.skipped += 1;
                    println!("skipping \"{}\" -- {}", &ebook.title, e);
                }
            });
        println!();
    });
    stats.elapsed = start.elapsed();
    print!("{}", stats);
    if dry_run {
        print!("; dry run");
    }
    println!();
    Ok(())
}
