use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process;

use pico_args::Arguments;

#[derive(Debug)]
enum AppArgs {
    Global {
        config_dir: Option<PathBuf>,
        remaining_args: Vec<OsString>,
    },
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
    run(Arguments::from_env(), None)
}

fn run(args: Arguments, config_dir: Option<&Path>) -> Result<(), Box<dyn Error>> {
    match parse_args(args) {
        Ok(args) => match args {
            AppArgs::Global {
                config_dir,
                remaining_args,
            } => {
                let config_dir = match &config_dir {
                    Some(dir) => Some(dir.as_path()),
                    None => None,
                };
                run(Arguments::from_vec(remaining_args), config_dir)
            }
            AppArgs::Config {} => {
                libri::config::run(&libri::config::read(config_dir)?);
                Ok(())
            }
            AppArgs::List {} => libri::list::run(&libri::config::read(config_dir)?),
            AppArgs::Import {
                path,
                move_books,
                dry_run,
            } => libri::import::run(
                &libri::config::read(config_dir)?,
                &path,
                move_books,
                dry_run,
            ),
            AppArgs::Upload {} => libri::upload::run(&libri::config::read(config_dir)?),
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

fn parse_args(mut args: Arguments) -> Result<AppArgs, Box<dyn Error>> {
    match args.subcommand()?.as_deref() {
        Some("config") => {
            if args.contains(["-h", "--help"]) {
                println!("{}", CONFIG_HELP);
                process::exit(0);
            }
            handle_extra_args(args.finish());
            Ok(AppArgs::Config {})
        }
        Some("list") => {
            if args.contains(["-h", "--help"]) {
                println!("{}", LIST_HELP);
                process::exit(0);
            }
            handle_extra_args(args.finish());
            Ok(AppArgs::List {})
        }
        Some("import") => {
            if args.contains(["-h", "--help"]) {
                println!("{}", IMPORT_HELP);
                process::exit(0);
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
                println!("{}", UPLOAD_HELP);
                process::exit(0);
            }
            handle_extra_args(args.finish());
            Ok(AppArgs::Upload {})
        }
        Some("device") => {
            if args.contains(["-h", "--help"]) {
                println!("{}", DEVICE_HELP);
                process::exit(0);
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
                    println!("{}", DEVICE_HELP);
                    process::exit(0);
                }
            }
        }
        Some(s) => Err(format!("unknown subcommand '{}'. See 'libri --help'", s).into()),
        None => {
            if args.contains(["-h", "--help"]) {
                println!("{}", GLOBAL_HELP);
                process::exit(0);
            }
            let config_dir: Option<PathBuf> =
                args.opt_value_from_os_str("--config-dir", parse_path)?;
            if config_dir.is_some() {
                return Ok(AppArgs::Global {
                    config_dir,
                    remaining_args: args.finish(),
                });
            }
            handle_extra_args(args.finish());
            println!("{}", GLOBAL_HELP);
            process::exit(1);
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
  libri [OPTIONS] <SUBCOMMAND>

FLAGS:
  -h, --help            Print help information

OPTIONS:
  --config-dir PATH     Path to an alternate configuration directory

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
