use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process;

use pico_args::Arguments;

#[derive(Debug)]
enum AppArgs {
    Config {},
    List {},
    Import {
        path: PathBuf,
        move_books: bool,
        dry_run: bool,
    },
    Upload {},
    Device(Device),
}

#[derive(Debug)]
enum Device {
    List {},
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = libri::config::read()?;
    match parse_args() {
        Ok(args) => match args {
            AppArgs::Config {} => {
                libri::config::run(&config);
                Ok(())
            }
            AppArgs::List {} => libri::list::run(&config),
            AppArgs::Import {
                path,
                move_books,
                dry_run,
            } => libri::import::run(&config, &path, move_books, dry_run),
            AppArgs::Upload {} => libri::upload::run(&config),
            AppArgs::Device(subcommand) => match subcommand {
                Device::List {} => libri::device::list::run(),
            },
        },
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn parse_args() -> Result<AppArgs, Box<dyn Error>> {
    let mut args = Arguments::from_env();

    match args.subcommand()?.as_deref() {
        Some("config") => {
            if args.contains(["-h", "--help"]) {
                return Err(CONFIG_HELP.into());
            }
            handle_extra_args(args.finish());
            Ok(AppArgs::Config {})
        }
        Some("list") => {
            if args.contains(["-h", "--help"]) {
                return Err(LIST_HELP.into());
            }
            handle_extra_args(args.finish());
            Ok(AppArgs::List {})
        }
        Some("import") => {
            if args.contains(["-h", "--help"]) {
                return Err(IMPORT_HELP.into());
            }
            let import = AppArgs::Import {
                path: args.free_from_os_str(parse_path)?,
                move_books: args.contains(["-m", "--move"]),
                dry_run: args.contains("--dry-run"),
            };
            handle_extra_args(args.finish());
            Ok(import)
        }
        Some("upload") => {
            if args.contains(["-h", "--help"]) {
                return Err(UPLOAD_HELP.into());
            }
            handle_extra_args(args.finish());
            Ok(AppArgs::Upload {})
        }
        Some("device") => {
            if args.contains(["-h", "--help"]) {
                return Err(DEVICE_HELP.into());
            }
            match args.subcommand()?.as_deref() {
                Some("list") => {
                    handle_extra_args(args.finish());
                    Ok(AppArgs::Device(Device::List {}))
                }
                Some(s) => {
                    Err(format!("unknown subcommand '{}'. See 'libri device --help'", s).into())
                }
                None => {
                    handle_extra_args(args.finish());
                    Err(DEVICE_HELP.into())
                }
            }
        }
        Some(s) => Err(format!("unknown subcommand '{}'. See 'libri --help'", s).into()),
        None => {
            args.contains(["-h", "--help"]);
            handle_extra_args(args.finish());
            Err(GLOBAL_HELP.into())
        }
    }
}

fn parse_path(s: &OsStr) -> Result<PathBuf, &'static str> {
    Ok(s.into())
}

fn handle_extra_args(args: Vec<OsString>) {
    if !args.is_empty() {
        eprintln!("unknown argument {:?}", args[0]);
        process::exit(1);
    }
}

const GLOBAL_HELP: &str = "\
An ebook management tool

USAGE:
  libri <SUBCOMMAND>

FLAGS:
  -h, --help            Print help information

SUBCOMMANDS:
  config                View and edit the configuration
  list                  List books in the library
  import                Import new books
  upload                Upload books to connected eReaders
  device                Manage hardware devices and their content";

const CONFIG_HELP: &str = "\
libri-config
View and edit the configuration

USAGE:
  libri config

FLAGS:
  -h, --help            Print help information";

const LIST_HELP: &str = "\
libri-list
List books in the library

USAGE:
  libri list

FLAGS:
  -h, --help            Print help information";

const IMPORT_HELP: &str = "\
libri-import
Import new books

USAGE:
  libri import PATH

FLAGS:
  -h, --help            Print help information

ARGS:
  <PATH>                Path to import directory";

const UPLOAD_HELP: &str = "\
libri-upload
Upload books to connected eReaders

USAGE:
  libri upload

FLAGS:
  -h, --help            Print help information";

const DEVICE_HELP: &str = "\
libri-device
Manage hardware devices and their content

USAGE:
  libri device <SUBCOMMAND>

FLAGS:
  -h, --help            Print help information

SUBCOMMANDS:
  list                  List connected eReaders supported by libri";
