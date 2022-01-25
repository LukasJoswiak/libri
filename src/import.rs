use std::error::Error;
use std::fs;
use std::path::Path;

use crate::common;
use crate::config;
use crate::format::epub;
use crate::list;

pub fn run(config: &config::Config, path: &Path) -> Result<(), Box<dyn Error>> {
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
            continue;
        }

        let author = common::sanitize(&ebook.author);
        let title = common::sanitize(&ebook.title);

        let mut destination = config.library.clone();
        destination.push(format!("{}/{}", author, title));
        fs::create_dir_all(&destination)?;
        destination.push(format!("{}.epub", title));
        common::copy(&ebook.path, &destination)?;
        println!("imported \"{}\"", ebook.title);
    }
    Ok(())
}
