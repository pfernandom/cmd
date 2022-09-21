use std::io::Read;
use std::io::Write;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use crate::cmd_csv::{ read_cmd_file };

use crate::error::CmdError;
use crate::log_debug;

use crate::models::cmd_record::CmdRecord;
use crate::traits::cmd_service::CmdService;
use crate::traits::file_manager::{ FileManager };

#[derive(Debug)]
pub struct CmdServiceCSV<'a, T, V> {
    counter: AtomicUsize,
    commands: Vec<CmdRecord>,
    pub file_mgr: &'a mut dyn FileManager<R = T, W = V>,
}
pub fn build_cmd_csv_service<'a, T, V>(
    file_mgr: &'a mut impl FileManager<R = T, W = V>
)
    -> Result<CmdServiceCSV<T, V>, String>
    where T: Read, V: Write
{
    let mut rdr = file_mgr.get_cmd_reader()?;
    let commands = read_cmd_file(&mut rdr);
    let counter = AtomicUsize::new(commands.len() + 1);

    Ok(CmdServiceCSV { commands, counter, file_mgr })
}

impl<'a, T, V> CmdServiceCSV<'a, T, V> {
    fn get_id(self: &Self) -> usize {
        let size = self.counter.fetch_add(1, Ordering::Relaxed);
        size
    }
}

impl<'a, T, V> CmdService<'a> for CmdServiceCSV<'a, T, V> where T: Read, V: Write {
    fn add_command(self: &mut Self, command: String) -> Result<(), CmdError> {
        let id = self.get_id();
        let record = CmdRecord { id: id, command: String::from(&command), used_times: 1 };
        let exists = self.commands.iter().find(|x| x.command == record.command);

        if let Some(_) = exists {
            return Err(CmdError::DuplicateCmdError);
        }
        let mut wtr = self.file_mgr.get_cmd_writter(true)?;
        wtr.serialize(&record).or_else(|err| Err(CmdError::from(err)))?;
        wtr.flush().expect("clould not save csv file");
        log_debug!("Command saved");
        Ok(())
    }

    fn is_record_present(self: &Self, record: &CmdRecord) -> bool {
        self.commands
            .iter()
            .find(|cmd| *cmd == record)
            .is_some()
    }

    fn reset_commands(
        self: &mut Self,
        mut updated_commands: Vec<CmdRecord>
    ) -> Result<(), CmdError> {
        let mut wtr = self.file_mgr.get_cmd_writter(false)?;
        updated_commands.sort_by(|a, b| b.used_times.cmp(&a.used_times));
        for cmd in &updated_commands {
            wtr.serialize(&cmd)?;
        }

        self.commands = updated_commands;
        Ok(wtr.flush()?)
    }

    fn get_commands(self: &mut Self) -> &Vec<CmdRecord> {
        return &self.commands;
    }

    fn update_command(self: &mut Self, record: CmdRecord) -> Result<(), CmdError> {
        let record_exists = self.is_record_present(&record);
        let mut updated_commands = self
            .get_commands()
            .iter()
            .map(|cmd| {
                if cmd == &record {
                    return record.clone();
                }
                cmd.clone()
            })
            .collect::<Vec<_>>();

        let list_is_empty = updated_commands.is_empty();
        let final_record = match record_exists || list_is_empty {
            true => {
                log_debug!("Record does not exist, updating with record");
                let id = self.get_id();
                let new_record = CmdRecord {
                    id: id,
                    command: String::from(&record.command),
                    used_times: 1,
                };
                new_record
            }
            false => record,
        };

        if list_is_empty {
            log_debug!("Record to add: {:?}", final_record);
            return self.reset_commands(vec![final_record]);
        }

        if !list_is_empty && !record_exists {
            log_debug!("Updating {:?} with {:?}", updated_commands, final_record);
            updated_commands.append(&mut vec![final_record]);
        }
        self.reset_commands(updated_commands)
    }

    fn get_file_name(self: &Self) -> String {
        self.file_mgr.get_file_name()
    }

    fn clear_commands(self: &Self) -> Result<(), CmdError> {
        self.file_mgr.clear_files()?;
        Ok(())
    }
}