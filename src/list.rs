use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use tabwriter::TabWriter;

use super::common;
use super::config;
use super::format::epub;
use super::Ebook;

/// Returns a vector of ebooks read from files in the given directory.
fn get_ebooks(path: &Path) -> Result<Vec<Ebook>, Box<dyn Error>> {
    let mut ebooks: Vec<Ebook> = Vec::new();
    let ebook_paths = common::find_ebooks(&path)?;

    for path in ebook_paths {
        let ebook = match epub::parse(path.as_path()) {
            Ok(ebook) => ebook,
            Err(e) => return Err(e),
        };
        ebooks.push(ebook);
    }
    Ok(ebooks)
}

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let ebooks = get_ebooks(&config.library)?;

    let mut tw = TabWriter::new(io::stdout());
    write!(&mut tw, "\x1b[1mTitle\tAuthor\x1b[0m\n").unwrap();
    // TODO: Sort by date added
    for ebook in ebooks {
        write!(&mut tw, "{}\t{}\n", ebook.title, ebook.author).unwrap();
    }
    tw.flush().unwrap();
    Ok(())
}
