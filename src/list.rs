use std::cmp;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

use chrono::{DateTime, Local};
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

/// Returns a string representation of the date the ebook was added to the library. The returned
/// string is suitable for display to the user.
fn created(ebook: &Ebook) -> String {
    let metadata = ebook.path.metadata().expect("failed to read file metadata");
    let created: DateTime<Local> = DateTime::from(
        metadata
            .created()
            .expect("failed to read file creation date"),
    );
    format!("{}", created.format("%B %d, %Y"))
}

pub fn run(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let ebooks = get_ebooks(&config.library)?;

    // Calculate the maximum length of each category, in order to determine the correct number of
    // "-" characters to add below the header for each column. This is pretty inefficient, but
    // works for now.
    let mut maxlen: Vec<usize> = vec![0, 0, 0];
    for ebook in &ebooks {
        maxlen[0] = cmp::max(maxlen[0], ebook.title.len());
        maxlen[1] = cmp::max(maxlen[1], ebook.author.len());
        maxlen[2] = cmp::max(maxlen[2], created(&ebook).len());
    }

    let mut tw = TabWriter::new(io::stdout());
    write!(&mut tw, "Title\tAuthor\tCreated\n").unwrap();
    // Note: the dash character here is an en dash, to make the separating line look even and not
    // have spaces in between each dash.
    write!(
        &mut tw,
        "{}\t{}\t{}\n",
        "–".repeat(maxlen[0]),
        "–".repeat(maxlen[1]),
        "–".repeat(maxlen[2])
    )
    .unwrap();
    // TODO: Sort by date added
    for ebook in ebooks {
        write!(
            &mut tw,
            "{}\t{}\t{}\n",
            ebook.title,
            ebook.author,
            created(&ebook)
        )
        .unwrap();
    }
    tw.flush().unwrap();
    Ok(())
}
