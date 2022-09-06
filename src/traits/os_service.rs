use crate::error::CmdError;

pub trait OSService {
    fn execute_command(self: &Self, command: &str) -> Result<bool, CmdError>;
}