use std::error::Error;
use std::path::PathBuf;

use configparser::ini::Ini;

#[derive(Debug)]
pub struct Config {
    pub library: PathBuf,
}

fn default_library() -> String {
    // TODO: Make platform specific
    let home = std::env::var("HOME").unwrap();
    format!("{}/Documents/books/", home)
}

fn config_path() -> PathBuf {
    // TODO: Make platform specific
    let home = std::env::var("HOME").unwrap();
    PathBuf::from(format!("{}/.config/libri/config.ini", home))
}

/// Reads the configuration from disk and returns it as a struct.
pub fn read() -> Result<Config, Box<dyn Error>> {
    // For now, always look in ~/.config/libri/config.ini. Should migrate to platform specific
    // paths (https://github.com/dirs-dev/directories-rs).
    let mut config = Ini::new();
    config.load(config_path())?;
    let library = config.get("default", "library");

    Ok(Config {
        library: PathBuf::from(library.unwrap_or_else(default_library)),
    })
}

pub fn run(config: &Config) {
    println!("{:?}", config);
}
