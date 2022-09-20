pub struct OSServiceImpl {}

use mockall::automock;

use crate::{
    traits::os_service::OSService,
    program::{ parse_programs, parse_program },
    error::CmdError,
};

#[automock]
impl OSService for OSServiceImpl {
    fn execute_command(self: &Self, command: &str) -> Result<bool, CmdError> {
        let mut programs = parse_programs(&command);

        if programs.is_empty() {
            return Err(CmdError::BaseError("No command to execute".to_string()));
        }

        println!("print -s {}", command);
        let extra = parse_program(&format!("add_history {}", command));
        programs.append(&mut vec![extra]);

        let mut res = None;
        for mut p in programs {
            res = Some(p.spawn()?.wait()?);
        }

        let r = res.unwrap();

        Ok(r.success())
    }
}