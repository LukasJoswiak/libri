//! # Libri
//!
//! eBook management tool.

mod common;
mod format;
mod xml;

pub mod config;
pub mod device;
pub mod import;
pub mod list;
pub mod upload;

use std::path::{Path, PathBuf};

/// Represents an eBook.
#[derive(Debug)]
pub struct Ebook {
    identifier: String,
    title: String,
    author: String,
    path: PathBuf,
}

impl Ebook {
    fn new(identifier: String, title: String, author: String, path: &Path) -> Ebook {
        Ebook {
            identifier,
            title,
            author,
            path: path.to_path_buf(),
        }
    }
}
