use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::common;
use crate::config;
use crate::format::epub;
use crate::list;

struct ImportStats {
    imported: u32,
    skipped: u32,
    elapsed: Duration,
}

impl fmt::Display for ImportStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "imported {}; skipped {}; finished in {:.2}s",
            self.imported,
            self.skipped,
            self.elapsed.as_secs_f32()
        )
    }
}

pub fn run(config: &config::Config, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut stats = ImportStats {
        imported: 0,
        skipped: 0,
        elapsed: Duration::ZERO,
    };
    let start = Instant::now();
    let ebooks = list::get_ebooks(&config.library)?;
    let ebook_paths = common::find_ebooks(&path)?;
    for path in ebook_paths {
        let ebook = match epub::parse(path.as_path()) {
            Ok(ebook) => ebook,
            Err(e) => return Err(e),
        };

        if ebooks
            .iter()
            .find(|e| e.identifier == ebook.identifier)
            .is_some()
        {
            println!("skipping \"{}\" -- previously imported", ebook.title);
            stats.skipped += 1;
            continue;
        }

        let author = common::sanitize(&ebook.author);
        let title = common::sanitize(&ebook.title);

        let mut destination = config.library.clone();
        destination.push(format!("{}/{}", author, title));
        fs::create_dir_all(&destination)?;
        destination.push(format!("{}.epub", title));
        common::copy(&ebook.path, &destination)?;
        stats.imported += 1;
        println!("imported \"{}\"", ebook.title);
    }
    stats.elapsed = start.elapsed();
    println!("\n{}", stats);
    Ok(())
}
