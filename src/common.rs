use std::error::Error;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub fn find_books(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut ebook_paths: Vec<PathBuf> = Vec::new();
    visit_dirs(path, &mut |entry| {
        if let Some(extension) = entry.path().extension() {
            if extension == "epub" {
                ebook_paths.push(entry.path());
            }
        }
    })?;
    Ok(ebook_paths)
}

/// Copies the file at `from` to `to`.
///
/// If `to` already exists, it will be overwritten.
///
/// # Examples
///
/// ```
/// use std::io;
/// use libri::common;
/// use std::path::{Path, PathBuf};
///
/// # fn main() -> io::Result<()> {
/// let from = Path::new("./foo/bar.txt");
/// common::copy(&from, &PathBuf::from("baz.txt").as_path())?;
/// # Ok(())
/// # }
/// ```
pub fn copy(from: &Path, to: &Path) -> io::Result<()> {
    fs::copy(from, to)?;
    Ok(())
}

/// Returns a modified path containing only file-system safe characters.
pub fn sanitize(path: &str) -> String {
    // TODO: Improve the implementation by modifying the path in-place
    let path = path.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(path.len());
    for i in 0..path.len() {
        // TODO: Sanitize all problem characters
        if path[i] == b':' {
            out.push(b'_');
        } else {
            out.push(path[i]);
        }
    }
    String::from_utf8(out).expect("invalid path")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::common;

    #[test]
    fn sanitize_clean_path() {
        let mut path = PathBuf::from("/foo/bar.txt");
        common::sanitize(&mut path);
        assert_eq!(path, PathBuf::from("/foo/bar.txt"));
    }

    #[test]
    fn sanitize_dirty_path() {
        let mut path = PathBuf::from("/foo:bar.txt");
        common::sanitize(&mut path);
        assert_eq!(path, PathBuf::from("/foo_bar.txt"));
    }
}
