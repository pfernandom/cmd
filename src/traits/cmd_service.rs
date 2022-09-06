use crate::{ cmd_csv::CmdRecord, error::CmdError };

pub trait CmdService {
    fn add_command(self: &Self, command: String) -> Result<(), CmdError>;
    fn update_command(self: &mut Self, command: CmdRecord) -> Result<(), CmdError>;
    fn is_record_present(self: &Self, record: &CmdRecord) -> bool;
    fn reset_commands(self: &mut Self, updated_commands: Vec<CmdRecord>) -> Result<(), CmdError>;
    fn get_commands(self: &Self) -> &Vec<CmdRecord>;
    fn get_file_name(self: &Self) -> String;
    fn clear_commands(self: &Self) -> Result<(), CmdError>;
}