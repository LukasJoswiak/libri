use std::cmp;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use tabwriter::TabWriter;

use super::common;
use super::config;
use super::format::epub;
use super::Ebook;

/// Returns a vector of ebooks read from files in the given directory.
pub fn get_ebooks(path: &Path) -> Result<Vec<Ebook>, Box<dyn Error>> {
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

    // Calculate the maximum length of each category, in order to determine the correct number of
    // "-" characters to add below the header for each column. This is pretty inefficient, but
    // works for now.
    let mut maxlen: Vec<usize> = vec![0, 0];
    for ebook in &ebooks {
        maxlen[0] = cmp::max(maxlen[0], ebook.title.len());
        maxlen[1] = cmp::max(maxlen[1], ebook.author.len());
    }

    let mut tw = TabWriter::new(io::stdout());
    write!(&mut tw, "Title\tAuthor\n").unwrap();
    // Note: the dash character here is an en dash, to make the separating line look even and not
    // have spaces in between each dash.
    write!(
        &mut tw,
        "{}\t{}\n",
        "–".repeat(maxlen[0]),
        "–".repeat(maxlen[1])
    )
    .unwrap();
    // TODO: Sort by date added
    for ebook in ebooks {
        write!(&mut tw, "{}\t{}\n", ebook.title, ebook.author).unwrap();
    }
    tw.flush().unwrap();
    Ok(())
}
