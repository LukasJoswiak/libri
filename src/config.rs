use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_library")]
    pub library: PathBuf,
}

fn default_library() -> PathBuf {
    // TODO: Make platform specific
    let home = std::env::var("HOME").unwrap();
    PathBuf::from(format!("{}/Documents/books/", home))
}

fn config_path() -> PathBuf {
    // TODO: Make platform specific
    let home = std::env::var("HOME").unwrap();
    PathBuf::from(format!("{}/.config/libri/config.toml", home))
}

/// Reads the configuration from disk and returns it as a struct.
pub fn read() -> Config {
    // For now, always look in ~/.config/libri/config.toml. Should migrate to platform specific
    // paths (https://github.com/dirs-dev/directories-rs).
    let config_path = config_path();

    let contents = match fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => String::new(),
            other_error => {
                panic!("problem opening the configuration file: {:?}", other_error)
            }
        },
    };

    toml::from_str(&contents).unwrap_or_else(|error| {
        panic!("failed to parse the configuration file: {}", error);
    })
}

pub fn run(config: &Config) {
    println!("{:?}", config);
}
