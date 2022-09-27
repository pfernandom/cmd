use derive_builder::Builder;

use crate::{ models::cmd_record::CmdRecord, error::CmdError };

#[derive(Default, Builder)]
#[builder(setter(into), pattern = "owned", default)]
pub struct SearchFilters {
    pub id: Option<usize>,
    pub command: Option<String>,
    pub used: bool,
}

pub trait CmdService<'a> {
    fn insert_command(self: &mut Self, command: CmdRecord) -> Result<(), CmdError>;
    fn add_command(self: &mut Self, command: String) -> Result<(), CmdError>;
    fn update_command(self: &mut Self, command: CmdRecord) -> Result<(), CmdError>;
    fn is_record_present(self: &Self, record: &CmdRecord) -> bool;
    fn reset_commands(self: &mut Self, updated_commands: Vec<CmdRecord>) -> Result<(), CmdError>;
    fn get_commands(self: &mut Self, filter: SearchFilters) -> Vec<CmdRecord>;
    fn get_file_name(self: &Self) -> String;
    fn clear_commands(self: &Self) -> Result<(), CmdError>;
    fn debug(self: &Self);
    fn delete_command(self: &mut Self, command: CmdRecord) -> Result<(), CmdError>;
}