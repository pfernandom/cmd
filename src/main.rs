use std::{ rc::Rc, cell::RefCell, io };

use clap_complete::{ Shell, Generator, generate, generate_to };
use cmd::{ cmd_get, cmd_add, cmd_clear, cmd_delete::{ self, delete_command } };
use env_logger::Builder;
use log::LevelFilter;
extern crate derive_builder;

use clap::{ Parser, CommandFactory, Command };
use services::{
    controller::{ Controller, RefCmdService },
    file_manager::{ build_file_manager, FileManagerImpl },
    os_service::OSServiceImpl,
    cmd_service_sql::CmdServiceSQL,
    cmd_extension_git::CmdExtensionGit,
};
use traits::{
    file_manager::FileManager,
    inputable::Inputable,
    os_service::OSService,
    cmd_extension::CmdExtension,
};

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
    pub controller: Controller<'a>,
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

    let all_cmd_service: RefCmdService<'_> = Rc::new(
        RefCell::new(CmdServiceSQL::build_cmd_service(None).unwrap())
    );

    Ok(Controller { all: Rc::clone(&all_cmd_service), used: Rc::clone(&all_cmd_service) })
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    match generate_to(gen, cmd, cmd.get_name().to_string(), "./") {
        Ok(msg) => { println!("Autocomplete generated succesfully {:?}", msg) }
        Err(err) => { println!("Could not generate autocomplete: {}", err.to_string()) }
    }
}
fn main() {
    let args = Cli::parse();

    if let Some(generator) = args.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {:?}...", generator);
        print_completions(generator, &mut cmd);
        return;
    }

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

            app(
                &mut (Deps {
                    args,
                    controller: mem,
                    input: Box::new(input),
                    os: Box::new(os_service),
                })
            )
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
        Commands::Debug { pattern } => {
            let ctrl = &deps.controller;
            ctrl.debug();
        }
        Commands::Delete {} => {
            match delete_command(deps) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
    }
}