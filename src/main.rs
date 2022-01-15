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
}

/// View and edit the configuration
#[derive(Parser)]
struct Config {}

/// Import new ebooks
#[derive(Parser)]
struct Import {
    /// Path to import dir
    path: String,
}

/// List books in library
#[derive(Parser)]
struct List {}

/// Manage hardware devices and their content
#[derive(Parser)]
struct Device {}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let config = libri::config::read();

    match opts.subcmd {
        SubCommand::Config(_c) => libri::config::run(&config),
        SubCommand::Device(_d) => libri::device::run()?,
        SubCommand::Import(i) => libri::import::run(&config, &Path::new(&i.path))?,
        SubCommand::List(_l) => libri::list::run(&config)?,
    }

    Ok(())
}
