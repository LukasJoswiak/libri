use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use tabwriter::TabWriter;

use crate::common;
use crate::config;
use crate::format::epub;

/// Returns a mapping of author to a vector of book titles. Reads ebook data from epubs in the given path.
pub fn get_books(path: &Path) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let mut ebooks = HashMap::new();
    let ebook_paths = common::find_books(&path)?;

    for path in ebook_paths {
        let ebook = match epub::parse(path.as_path()) {
            Ok(ebook) => ebook,
            Err(e) => return Err(e),
        };

        let entry = ebooks.entry(ebook.author).or_insert(Vec::new());
        entry.push(ebook.title);
    }
    Ok(ebooks)
}

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let ebooks = get_books(&config.library)?;

    let mut tw = TabWriter::new(io::stdout());
    write!(&mut tw, "Title\tAuthor\n").unwrap();
    // TODO: Sort by date added
    for (author, titles) in ebooks {
        for title in titles {
            write!(&mut tw, "{}\t{}\n", title, author).unwrap();
        }
    }
    tw.flush().unwrap();
    Ok(())
}
