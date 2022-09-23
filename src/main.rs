use cmd::{ cmd_get, cmd_add, cmd_clear };
use env_logger::Builder;
use log::LevelFilter;

use clap::Parser;
use services::{
    controller::Controller,
    file_manager::{ build_file_manager, FileManagerImpl },
    os_service::OSServiceImpl,
    cmd_service_sql::CmdServiceSQL,
};
use traits::{ file_manager::FileManager, inputable::Inputable, os_service::OSService };

use crate::services::input::InputManager;

mod args;
mod logging;
mod program;
mod cmd;
pub mod services;
mod tests;
mod cmd_csv;
mod traits;
mod error;
mod models;
use args::{ Cli, Commands };

pub struct Deps<'a> {
    pub input: Box<dyn Inputable>,
    pub args: Cli,
    pub mem: Controller<'a>,
    pub os: Box<dyn OSService>,
}

fn create_config<'a>(
    all_file_mgr: &'a mut FileManagerImpl,
    used_file_mgr: &'a mut FileManagerImpl
) -> Result<Controller<'a>, String> {
    all_file_mgr.create_cmd_file()?;
    used_file_mgr.create_cmd_file()?;
    // let all_cmd_service = build_cmd_csv_service(all_file_mgr)?;
    // let used_cmd_service = build_cmd_csv_service(used_file_mgr)?;

    let all_cmd_service = CmdServiceSQL::build_cmd_service(None).unwrap();
    let used_cmd_service = CmdServiceSQL::build_cmd_service(None).unwrap();

    Ok(Controller { all: Box::new(all_cmd_service), used: Box::new(used_cmd_service) })
}

fn main() {
    let args = Cli::parse();
    let level = match args.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    Builder::new().filter_level(level).init();

    let mut all_file_mgr = build_file_manager("cmd.csv");
    let mut used_file_mgr: FileManagerImpl = build_file_manager("cmd_used.csv");

    let maybe_mem = create_config(&mut all_file_mgr, &mut used_file_mgr);

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
    let mut args = deps.args.clone();
    let command = &mut args.command;

    let cmd: Commands = match command {
        Some(c) => c.clone(),
        None => Commands::Get { pattern: args.get_command },
    };

    match cmd {
        Commands::Get { pattern } => {
            match cmd_get::get_command(&pattern, deps) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
        Commands::Add { pattern, execute } => {
            match cmd_add::add_command(pattern, execute, deps) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
        Commands::Clear {} => {
            cmd_clear::clear(&deps);
        }
        Commands::Debug {} => {
            let ctrl = &deps.mem;
            ctrl.debug();
        }
    }
}