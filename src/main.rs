use clap::Parser;
use std::error::Error;
use std::path::Path;

/// Libri is a command line tool to organize your ebooks
#[derive(Parser)]
#[clap(version = "0.1.0")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Config(Config),
    Device(Device),
    Import(Import),
    List(List),
    Upload(Upload),
}

/// View and edit the configuration
#[derive(Parser)]
struct Config {}

/// Import new books
#[derive(Parser)]
struct Import {
    /// Copy books into the library (default)
    #[clap(short, long)]
    copy: bool,

    /// Move books into the library (overrides --copy)
    #[clap(name = "move", short, long)]
    move_books: bool,

    /// List books that would be imported from the given path without actually modifying the file
    /// system.
    #[clap(long)]
    dry_run: bool,

    /// Path to import directory
    path: String,
}

/// List books in library
#[derive(Parser)]
struct List {}

/// Upload books to connected eReaders
#[derive(Parser)]
struct Upload {
    // TODO: Add flags to filter books and devices
}

/// Manage hardware devices and their content
#[derive(Parser)]
struct Device {
    #[clap(subcommand)]
    subcmd: DeviceSubCommand,
}

#[derive(Parser)]
enum DeviceSubCommand {
    #[clap(name = "list")]
    DeviceList(DeviceList),
}

/// List connected eReaders supported by libri
#[derive(Parser)]
struct DeviceList {}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let config = libri::config::read();

    match opts.subcmd {
        SubCommand::Config(_c) => libri::config::run(&config),
        SubCommand::Device(d) => match d.subcmd {
            DeviceSubCommand::DeviceList(_l) => libri::device::list::run()?,
        },
        SubCommand::Import(i) => {
            libri::import::run(&config, &Path::new(&i.path), i.move_books, i.dry_run)?
        }
        SubCommand::List(_l) => libri::list::run(&config)?,
        SubCommand::Upload(_u) => libri::upload::run(&config)?,
    }

    Ok(())
}
