use crate::cmd_csv::{ CmdRecord, CmdRecordIterable };
use crate::error::CmdError;
use crate::traits::cmd_service::CmdService;

pub struct ConfigMem<'a> {
    pub all: Box<dyn CmdService<'a> + 'a>,
    pub used: Box<dyn CmdService<'a> + 'a>,
}

impl<'a> ConfigMem<'a> {
    pub fn get_commands(self: &Self) -> &Vec<CmdRecord> {
        return &self.all.get_commands();
    }

    pub fn get_used_commands(self: &Self) -> &Vec<CmdRecord> {
        return &self.used.get_commands();
    }

    pub fn new_command(self: &mut Self, command: String) -> Result<(), CmdError> {
        self.all.add_command(command)
    }

    pub fn add_used_command(self: &mut Self, mut record: CmdRecord) -> Result<(), CmdError> {
        let sum = self.used
            .get_commands()
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
}