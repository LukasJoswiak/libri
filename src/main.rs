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
    Import(Import),
    List(List),
}

/// View and edit the configuration
#[derive(Parser)]
struct Config {}

/// Import new books
#[derive(Parser)]
struct Import {
    /// Path to import dir
    path: String,
}

/// List books in library
#[derive(Parser)]
struct List {}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    let config = libri::config::read();

    match opts.subcmd {
        SubCommand::Config(_c) => libri::config::run(&config),
        SubCommand::Import(i) => libri::import::run(&config, &Path::new(&i.path))?,
        SubCommand::List(_l) => libri::list::run(&config)?,
    }

    Ok(())
}
