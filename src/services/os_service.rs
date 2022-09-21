pub struct OSServiceImpl {}

use std::{ fs::OpenOptions, io::{ BufWriter, Write } };

use mockall::automock;

use crate::{
    traits::os_service::OSService,
    program::{ parse_programs },
    error::CmdError,
    log_debug,
};

#[automock]
impl OSService for OSServiceImpl {
    fn execute_command(self: &Self, command: &str) -> Result<bool, CmdError> {
        let programs = parse_programs(&command);

        if programs.is_empty() {
            return Err(CmdError::BaseError("No command to execute".to_string()));
        }

        // println!("print -s \"{}\"", command);
        // let extra = parse_program(&format!("setopt INC_APPEND_HISTORY"));
        // programs.append(&mut vec![extra]);

        let mut res = None;
        for mut p in programs {
            let mut s = p.spawn()?;
            let w = s.wait()?;

            res = Some(w);
        }

        let r = res.unwrap();

        let mut home = home::home_dir().expect("Could not find home dir");
        log_debug!(
            "Saving settings to: {}/.cmd",
            home.to_str().expect("could not parse home path")
        );

        home.push(".zsh_history");

        let f = OpenOptions::new().append(true).open(home)?;
        let mut w = BufWriter::new(f);
        w.write_all(command.as_bytes())?;
        w.flush()?;

        Ok(r.success())
    }
}