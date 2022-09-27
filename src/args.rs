use clap::{ Parser, Subcommand, ValueHint, value_parser };
use clap_complete::Shell;

// cargo run -- --generate=zsh
// sudo cp _cmd /usr/local/share/zsh/site-functions/
// autoload -Uz compinit
// compinit
// ./target/debug/examples/value_hints_derive --<TAB>

/// A commands manager tool
#[derive(Debug, Parser, Clone, Default)] // requires `derive` feature
#[clap(name = "cmd")]
#[clap(about = "A command line manager", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,

    #[clap(value_parser, id = "subcommand")]
    pub get_command: Option<String>,

    #[clap(name = "verbose", long, short, parse(from_flag))]
    pub verbose: bool,

    #[clap(name = "dry-run", long, short, parse(from_flag))]
    pub dry_run: bool,

    #[clap(long = "generate", value_enum)]
    pub generator: Option<Shell>,
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
        #[clap(value_parser, value_hint = ValueHint::CommandName)]
        pattern: Option<String>,
    },
    Clear {},

    Delete {},

    Debug {
        #[clap(value_parser, value_hint = ValueHint::CommandName)]
        pattern: Option<Shell>,
    },
    // Stash(Stash),
    // #[clap(external_subcommand)] External(Vec<OsString>),
}