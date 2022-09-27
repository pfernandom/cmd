use crate::{ error::CmdError, Deps };

pub fn add_command(pattern: bool, execute: bool, deps: &mut Deps) -> Result<(), CmdError> {
    let mem = &mut deps.controller;
    let note = deps.input.get_input(Some("Write your command".into()));
    print!("{}", note);

    if pattern {
    }

    match execute {
        true => {
            deps.os.execute_command(&note)?;
            mem.new_command(note)
        }
        false => { mem.new_command(note) }
    }
}