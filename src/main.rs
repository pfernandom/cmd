use cmd::{ cmd_get, cmd_add, cmd_clear };
use env_logger::Builder;
use log::LevelFilter;

use clap::Parser;
use traits::{ file_manager::FileManager, inputable::Inputable };

use crate::input::InputManager;

mod args;
mod input;
mod logging;
mod program;
mod cmd;
mod tests;
mod cmd_utils;
mod cmd_csv;
mod cmd_config_mem;

mod traits;

use args::{ Cli, Commands };
use cmd_config_mem::{ ConfigMem, build_file_manager, build_cmd_service, FileManagerImpl };

pub struct Deps {
    pub input: Box<dyn Inputable>,
    pub args: Cli,
    pub mem: ConfigMem,
}

fn create_config() -> Result<ConfigMem, String> {
    let all_file_mgr = build_file_manager("cmd.csv");
    let used_file_mgr: FileManagerImpl = build_file_manager("cmd_used.csv");
    all_file_mgr.create_cmd_file()?;
    used_file_mgr.create_cmd_file()?;
    let all_cmd_service = build_cmd_service(all_file_mgr)?;
    let used_cmd_service = build_cmd_service(used_file_mgr)?;

    Ok(ConfigMem { all: all_cmd_service, used: used_cmd_service })
}
fn main() {
    let args = Cli::parse();
    let level = match args.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    Builder::new().filter_level(level).init();

    let maybe_mem = create_config();

    match maybe_mem {
        Ok(mem) => {
            let input: InputManager = InputManager {};

            app(&(Deps { args, mem, input: Box::new(input) }))
        }
        Err(err) => {
            log_error!("Error: Could not start the app: {}", err);
        }
    }
}

pub(crate) fn app(deps: &Deps) {
    match &deps.args.command {
        Commands::Get { pattern } => {
            cmd_get::get_command(pattern, &deps);
        }
        Commands::Add { pattern, execute } => {
            cmd_add::add_command(*pattern, *execute, &deps);
        }
        Commands::Clear {} => {
            cmd_clear::clear(&deps);
        }
    }
}