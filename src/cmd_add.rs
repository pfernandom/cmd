use crate::cmd_utils::{ get_input };
use crate::cmd_config_mem::ConfigMem;
use crate::{ log_error, log_info, log_debug };
use crate::program::parse_program;

pub fn add_command(pattern: bool, execute: bool, mem: &ConfigMem) {
    let note = get_input(Some("Write your command"));
    print!("{}", note);

    if pattern {
    }

    if execute {
        let mut program = parse_program(&note);

        let cmd_result = program.spawn().map(|mut p| { p.wait().or_else(|_| Err(false)) });

        match cmd_result {
            Ok(_) => {
                if let Err(err) = mem.add_command(note) {
                    log_debug!("Cannot add command: {:?}", err);
                    return ();
                }

                log_info!("SUCCESS: Command added");
            }
            Err(err) => {
                log_error!("\nERROR: Failed to execute command: {:?}", err);
            }
        }
    } else {
        if
            let Err(err) = mem.add_command(note).map(|_| {
                log_info!("SUCCESS: Command added");
            })
        {
            log_error!("\nERROR: Failed to execute command: {:?}", err);
        }
    }
}