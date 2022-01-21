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
    title: String,
    author: String,
    path: PathBuf,
}

impl Ebook {
    fn new(title: String, author: String, path: &Path) -> Ebook {
        Ebook {
            title,
            author,
            path: path.to_path_buf(),
        }
    }
}
