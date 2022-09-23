use crate::models::cmd_record::{ CmdRecord, CmdRecordIterable };
use crate::error::CmdError;
use crate::traits::cmd_service::CmdService;

pub struct Controller<'a> {
    pub all: Box<dyn CmdService<'a> + 'a>,
    pub used: Box<dyn CmdService<'a> + 'a>,
}

impl<'a> Controller<'a> {
    pub fn get_commands(self: &mut Self, pattern: String) -> Vec<CmdRecord> {
        return self.all.get_commands(Some(pattern));
    }

    pub fn get_used_commands(self: &mut Self, pattern: String) -> Vec<CmdRecord> {
        return self.used.get_commands(Some(pattern));
    }

    pub fn new_command(self: &mut Self, command: String) -> Result<(), CmdError> {
        self.all.add_command(command)
    }

    pub fn add_used_command(self: &mut Self, mut record: CmdRecord) -> Result<(), CmdError> {
        let sum = self.used
            .get_commands(None)
            .iter()
            .filter(|cmd| cmd.command == record.command)
            .sum_count();

        record.used_times = sum;

        record.increase_usage();

        self.used.update_command(record)
    }

    pub fn get_all_file_path(self: &Self) -> String {
        self.all.get_file_name()
    }
    pub fn get_used_file_path(self: &Self) -> String {
        self.used.get_file_name()
    }

    pub fn clear_files(self: &Self) {
        self.all.clear_commands().expect("Cloud not delete cmd.csv");
        self.used.clear_commands().expect("Cloud not delete cmd_used.csv");
    }

    pub fn debug(self: &Self) {
        self.all.debug();
    }
}