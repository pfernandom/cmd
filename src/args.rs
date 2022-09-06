use clap::{ Parser, Subcommand };

/// A fictional versioning CLI
#[derive(Debug, Parser, Clone)] // requires `derive` feature
#[clap(name = "cmd")]
#[clap(about = "A command line manager", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[clap(name = "verbose", long, short, parse(from_flag))]
    pub verbose: bool,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    Add {
        // /// Stuff to add
        #[clap(name = "pattern", long, short, parse(from_flag))]
        pattern: bool,

        #[clap(name = "execute", long, short, parse(from_flag))]
        execute: bool,
    },
    Get {
        #[clap(value_parser)]
        pattern: Option<String>,
    },
    Clear {},
    // Stash(Stash),
    // #[clap(external_subcommand)] External(Vec<OsString>),
}