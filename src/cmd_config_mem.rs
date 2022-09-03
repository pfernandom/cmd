use std::fs::OpenOptions;

use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use csv::Reader;
use csv::Writer;
use mockall::automock;

use crate::cmd_csv::{ CmdRecord, read_cmd_file };
use crate::log_info;

use crate::log_debug;
use crate::log_warn;

use crate::traits::file_manager::{ FileManager };

pub struct FileManagerImpl {
    pub file_name: String,
    path: String,
    home: PathBuf,
}

pub fn build_file_manager(name: &str) -> FileManagerImpl {
    let mut home = home::home_dir().expect("Could not find home dir");
    log_warn!("Saving settings to: {}/.cmd", home.to_str().expect("could not parse home path"));

    home.push(".cmd");
    let mut dir_builder = std::fs::DirBuilder::new();
    dir_builder.recursive(true).create(&home).expect("Could not create config folder");

    let t = &home.join(name);
    let commands_path = t.to_str().expect("could not convert path to string");

    FileManagerImpl { home, path: commands_path.to_string(), file_name: name.to_string() }
}

#[automock]
impl FileManager for FileManagerImpl {
    type W = Box<dyn Write>;

    type R = Box<dyn Read>;
    fn create_cmd_file(self: &Self) -> Result<(), String> {
        let t = self.home.join(&self.file_name);
        let commands_path = t.to_str().expect("could not convert path to string");
        log_info!("Creating file: {}", &commands_path);
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&commands_path)
            .or_else(|err| Err(err.to_string()))?;
        Ok(())
    }

    fn get_cmd_writter(self: &Self, append: bool) -> Result<Writer<Self::W>, String> {
        log_info!("Opening: {}", &self.path);
        let file = OpenOptions::new()
            .write(true)
            .append(append)
            .open(&self.path)

            .or_else(|err| Err(format!("Could not open file:{}: {:?}", &self.path, err)))?;

        let w: Writer<Self::W> = csv::WriterBuilder
            ::new()
            .has_headers(false)
            .from_writer(Box::new(file));
        Ok(w)
    }

    fn get_cmd_reader(self: &Self) -> Result<Reader<Self::R>, String> {
        log_info!("Opening: {}", &self.path);
        let file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .or_else(|err| Err(format!("Could not open file:{}: {:?}", &self.path, err)))?;

        let w: Reader<Self::R> = csv::ReaderBuilder
            ::new()
            .has_headers(false)
            .from_reader(Box::new(file));
        Ok(w)
    }

    fn clear_files(self: &Self) -> Result<(), Error> {
        std::fs::remove_file(&self.path)
    }

    fn get_file_name(self: &Self) -> String {
        self.file_name.clone()
    }
}

pub struct CmdService {
    counter: AtomicUsize,
    commands: Vec<CmdRecord>,
    pub file_mgr: Box<dyn FileManager<R = Box<dyn Read>, W = Box<dyn Write>>>,
}
pub fn build_cmd_service<T>(file_mgr: T) -> Result<CmdService, String>
    where T: FileManager<R = Box<dyn Read>, W = Box<dyn Write>> + 'static
{
    let mut rdr = file_mgr.get_cmd_reader()?;
    let commands = read_cmd_file(&mut rdr);
    let counter = AtomicUsize::new(commands.len() + 1);

    Ok(CmdService { commands, counter, file_mgr: Box::new(file_mgr) })
}

impl CmdService {
    fn get_id(self: &Self) -> usize {
        let size = self.counter.fetch_add(1, Ordering::Relaxed);
        size
    }

    pub fn add_command(self: &Self, command: String) -> Result<(), String> {
        let id = self.get_id();
        let record = CmdRecord { id: id, command: String::from(&command), used_times: 1 };
        let exists = self.commands.iter().find(|x| x.command == record.command);

        if let Some(_) = exists {
            return Err("Record already exists, returning".to_string());
        }
        let mut wtr = self.file_mgr.get_cmd_writter(true)?;
        wtr.serialize(&record).or_else(|_| Err("Could not serialize"))?;
        wtr.flush().expect("clould not save csv file");
        log_debug!("Command saved");
        Ok(())
    }

    pub fn is_record_present(self: &Self, record: &CmdRecord) -> bool {
        self.commands
            .iter()
            .find(|cmd| *cmd == record)
            .is_some()
    }

    fn reset_commands(self: &Self, mut updated_commands: Vec<CmdRecord>) -> Result<(), String> {
        let mut wtr = self.file_mgr.get_cmd_writter(true)?;
        updated_commands.sort_by(|a, b| b.used_times.cmp(&a.used_times));
        for cmd in updated_commands {
            wtr.serialize(&cmd).or_else(|_| Err("Could not serialize"))?;
        }

        wtr.flush().expect("clould not save csv file");

        Ok(())
    }
}

pub struct ConfigMem {
    pub all: CmdService,
    pub used: CmdService,
}

impl ConfigMem {
    pub fn get_commands(self: &Self) -> &Vec<CmdRecord> {
        return &self.all.commands;
    }

    pub fn get_used_commands(self: &Self) -> &Vec<CmdRecord> {
        return &self.used.commands;
    }

    pub fn new_command(self: &Self, command: String) -> Result<(), String> {
        self.all.add_command(command)
    }

    pub fn add_used_command(self: &Self, mut record: CmdRecord) -> Result<(), String> {
        let record_exists = self.used.is_record_present(&record);

        record.increase_usage();

        let mut updated_commands = self.used.commands
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
                let id = self.used.get_id();
                let new_record = CmdRecord {
                    id: id,
                    command: String::from(&record.command),
                    used_times: 1,
                };
                new_record
            }
            false => record,
        };

        log_debug!("list_is_empty:{}, record_exists:{}", list_is_empty, record_exists);
        if list_is_empty {
            log_debug!("Record to add: {:?}", final_record);
            return self.used.reset_commands(vec![final_record]);
        }

        if !list_is_empty && !record_exists {
            log_debug!("Updating {:?} with {:?}", updated_commands, final_record);
            updated_commands.append(&mut vec![final_record]);
        }
        log_debug!("Reseting with {:?}", updated_commands);
        self.used.reset_commands(updated_commands)
    }
    pub fn get_all_file_path(self: &Self) -> String {
        self.all.file_mgr.get_file_name()
    }
    pub fn get_used_file_path(self: &Self) -> String {
        self.used.file_mgr.get_file_name()
    }

    pub fn clear_files(self: &Self) {
        self.all.file_mgr.clear_files().expect("Cloud not delete cmd.csv");
        self.used.file_mgr.clear_files().expect("Cloud not delete cmd_used.csv");
    }
}