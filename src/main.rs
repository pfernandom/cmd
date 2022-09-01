use env_logger::Builder;
use log::LevelFilter;

use clap::Parser;

mod args;
mod logging;
mod program;
mod cmd_get;
mod cmd_add;
mod cmd_clear;
mod tests;
mod cmd_utils;
mod cmd_csv;
mod cmd_config_mem;

use args::{ Cli, Commands };
use cmd_config_mem::ConfigMem;

fn main() {
    let args = Cli::parse();
    let mem = ConfigMem::config();

    let level = match args.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    Builder::new().filter_level(level).init();

    match args.command {
        Commands::Get { pattern } => {
            cmd_get::get_command(pattern, &mem);
        }
        Commands::Add { pattern, execute } => {
            cmd_add::add_command(pattern, execute, &mem);
        }
        Commands::Clear {} => {
            cmd_clear::clear(&mem);
        }
    }

    // Continued program logic goes here...
}