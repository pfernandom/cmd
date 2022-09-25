use std::cell::RefCell;
use std::rc::Rc;

use crate::models::cmd_record::{ CmdRecord, CmdRecordIterable };
use crate::error::CmdError;
use crate::traits::cmd_service::{ CmdService, SearchFilters, SearchFiltersBuilder };

pub type RefCmdService<'a> = Rc<RefCell<dyn CmdService<'a>>>;
pub struct Controller<'a> {
    pub all: RefCmdService<'a>,
    pub used: RefCmdService<'a>,
}

impl<'a> Controller<'a> {
    pub fn get_commands(self: &mut Self, pattern: String) -> Vec<CmdRecord> {
        return self.all
            .borrow_mut()
            .get_commands(SearchFiltersBuilder::default().command(pattern).build().unwrap());
    }

    pub fn get_used_commands(self: &mut Self, pattern: String) -> Vec<CmdRecord> {
        let mut builder = SearchFiltersBuilder::default();
        if pattern.len() > 0 {
            builder = builder.command(pattern);
        }

        return self.used.borrow_mut().get_commands(builder.used(true).build().unwrap());
    }

    pub fn new_command(self: &mut Self, command: String) -> Result<(), CmdError> {
        self.all.borrow_mut().add_command(command)
    }

    pub fn add_used_command(
        self: &mut Self,
        mut record: CmdRecord,
        alias: Option<String>
    ) -> Result<(), CmdError> {
        let sum = self.used
            .borrow_mut()
            .get_commands(SearchFilters::default())
            .iter()
            .filter(|cmd| cmd.command == record.command)
            .sum_count();

        record.used_times = sum;

        record.increase_usage();

        let _ = self.used.borrow_mut().update_command(record.clone());
        if let Some(a) = alias {
            record.update_command(a.as_str());
            let _ = self.used.borrow_mut().insert_command(record);
        }

        Ok(())
    }

    pub fn get_all_file_path(self: &Self) -> String {
        self.all.borrow().get_file_name()
    }
    pub fn get_used_file_path(self: &Self) -> String {
        self.used.borrow().get_file_name()
    }

    pub fn clear_files(self: &Self) {
        self.all.borrow().clear_commands().expect("Cloud not delete cmd.csv");
        self.used.borrow().clear_commands().expect("Cloud not delete cmd_used.csv");
    }

    pub fn debug(self: &Self) {
        self.all.borrow().debug();
    }
}