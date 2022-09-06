use cmd::{ cmd_get, cmd_add, cmd_clear };
use env_logger::Builder;
use log::LevelFilter;

use clap::Parser;
use services::{
    cmd_service::build_cmd_service,
    controller::ConfigMem,
    file_manager::{ build_file_manager, FileManagerImpl },
    os_service::OSServiceImpl,
};
use traits::{ file_manager::FileManager, inputable::Inputable, os_service::OSService };

use crate::input::InputManager;

mod args;
mod input;
mod logging;
mod program;
mod cmd;
mod services;
mod tests;
mod cmd_utils;
mod cmd_csv;
mod traits;
mod error;
use args::{ Cli, Commands };

pub struct Deps {
    pub input: Box<dyn Inputable>,
    pub args: Cli,
    pub mem: ConfigMem,
    pub os: Box<dyn OSService>,
}

fn create_config() -> Result<ConfigMem, String> {
    let all_file_mgr = build_file_manager("cmd.csv");
    let used_file_mgr: FileManagerImpl = build_file_manager("cmd_used.csv");
    all_file_mgr.create_cmd_file()?;
    used_file_mgr.create_cmd_file()?;
    let all_cmd_service = build_cmd_service(all_file_mgr)?;
    let used_cmd_service = build_cmd_service(used_file_mgr)?;

    Ok(ConfigMem { all: Box::new(all_cmd_service), used: Box::new(used_cmd_service) })
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
            let os_service = OSServiceImpl {};

            app(&mut (Deps { args, mem, input: Box::new(input), os: Box::new(os_service) }))
        }
        Err(err) => {
            log_error!("Error: Could not start the app: {}", err);
        }
    }
}

pub(crate) fn app(deps: &mut Deps) {
    let command = &mut deps.args.clone().command;
    match command {
        Commands::Get { pattern } => {
            match cmd_get::get_command(&pattern, deps) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
        Commands::Add { pattern, execute } => {
            match cmd_add::add_command(*pattern, *execute, &deps) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
        Commands::Clear {} => {
            cmd_clear::clear(&deps);
        }
    }
}