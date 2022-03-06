use std::cmp;
use std::error::Error;
use std::fs::Metadata;
use std::io::{self, Write};
use std::path::Path;

#[cfg(target_family = "unix")]
use std::os::unix::fs::MetadataExt;

use chrono::{DateTime, Local, NaiveDateTime, Utc};
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

#[cfg(target_family = "unix")]
fn modified_datetime(metadata: &Metadata) -> DateTime<Utc> {
    let naive = NaiveDateTime::from_timestamp(metadata.mtime(), 0);
    DateTime::from_utc(naive, Utc)
}

// TODO: Support the more accurate "last modified" datetime on other platforms (this will require
// support for updating the last modified timestamp when importing books on other platforms as
// well).
#[cfg(not(target_family = "unix"))]
fn modified_datetime(metadata: &Metadata) -> DateTime<Utc> {
    DateTime::from(metadata.created().unwrap_or_default(0))
}

/// Returns a string representation of the date the ebook was added to the library. The returned
/// string is suitable for display to the user.
fn created(ebook: &Ebook) -> String {
    let metadata = ebook.path.metadata().expect("failed to read file metadata");
    format!(
        "{}",
        modified_datetime(&metadata)
            .with_timezone(&Local)
            .format("%B %d, %Y")
    )
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
