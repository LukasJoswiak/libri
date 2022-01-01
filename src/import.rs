use std::error::Error;
use std::fs;
use std::path::Path;

use crate::common;
use crate::config;
use crate::format::epub;

pub fn run(config: &config::Config, path: &Path) -> Result<(), Box<dyn Error>> {
    let ebook_paths = common::find_books(&path)?;
    for path in ebook_paths {
        let ebook = match epub::parse(path.as_path()) {
            Ok(ebook) => ebook,
            Err(e) => return Err(e),
        };

        // TODO: Ignore any already imported books.
        let mut destination = config.library.clone();
        destination.push(format!("{}/{}", ebook.author, ebook.title));
        fs::create_dir_all(&destination)?;
        destination.push(format!("{}.epub", ebook.title));
        common::sanitize(&mut destination);
        common::copy(&ebook.path, &destination)?;
        println!("imported \"{}\"", ebook.title);
    }

    Ok(())
}
