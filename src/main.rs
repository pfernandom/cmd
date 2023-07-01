use std::{cell::RefCell, rc::Rc};

use clap_complete::{ Generator, generate_to };
use cmd::{cmd_get::GetHandler, cmd_add::AddHandler, cmd_clear::ClearHandler, cmd_delete::DeleteHandler};
use env_logger::Builder;
use log::LevelFilter;
extern crate derive_builder;
#[macro_use]
extern crate lazy_static;

use clap::{ Parser, CommandFactory, Command };
use services::{
    controller::{ Controller },
    file_manager::{ FileManagerBuilder },
    os_service::OSServiceImpl,
    cmd_service_sql::CmdServiceSQL,
    // cmd_extension_git::CmdExtensionGit,
};
use traits::{
    file_manager::FileManager,
    inputable::Inputable,
    os_service::OSService,
    // cmd_extension::CmdExtension,
};

use crate::services::input::InputManager;

mod args;
mod logging;
mod program;
mod cmd;
pub mod services;
#[cfg(test)]
mod tests;
mod cmd_csv;
mod traits;
mod error;
mod models;
use args::{ Cli, Commands };

pub struct Deps {
    pub input: Rc<dyn Inputable>,
    pub args: Cli,
    pub controller: Controller<CmdServiceSQL>,
    pub os: Rc<dyn OSService>,
}

impl <'a> Deps {
    fn new(args: Cli) -> Self {
        let input: InputManager = InputManager {};
        let os_service = OSServiceImpl {};

        let all_file_mgr = FileManagerBuilder::new("cmd.csv".to_string()).build();
        let used_file_mgr = FileManagerBuilder::new("cmd_used.csv".to_string()).build();

        all_file_mgr.create_cmd_file().expect("Cannot create config file");
        used_file_mgr.create_cmd_file().expect("Cannot create config file");

        let all_cmd_service = CmdServiceSQL::build_cmd_service(None).unwrap();


        Self {
            args,
            controller: Controller { all: all_cmd_service.clone(), used: all_cmd_service },
            input: Rc::new(input),
            os: Rc::new(os_service),
        }
    }
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

    app(Deps::new(args))
}

pub(crate) fn app(deps: Deps) {
    let mut args = deps.args.clone();

    let command = &mut args.command;

    let cmd: Commands = match command {
        Some(c) => c.clone(),
        None => Commands::Get { pattern: args.get_command },
    };

    let deps_ref = Rc::new(RefCell::new(deps));

    let mut get_handler = GetHandler::new(Rc::clone(&deps_ref));
    let mut add_handler = AddHandler::new(Rc::clone(&deps_ref));
    let clear_handler = ClearHandler::new(Rc::clone(&deps_ref));
    let mut delete_handler = DeleteHandler::new(Rc::clone(&deps_ref));

    match cmd {
        Commands::Get { pattern } => {
            match get_handler.get_command(&pattern) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
        Commands::Add { pattern, execute } => {
            match add_handler.add_command(pattern, execute) {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
        Commands::Clear {} => {
            clear_handler.clear();
        }
        Commands::Debug { pattern: _ } => {
            let ctrl = &deps_ref.as_ref().borrow().controller;
            ctrl.debug();
        }
        Commands::Delete {} => {
            match delete_handler.delete_command() {
                Ok(_) => { log_info!("Completed successfully.") }
                Err(err) => {
                    log_error!("Error: {}", err.to_string());
                }
            }
        }
    }
}