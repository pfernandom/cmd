use std::fs::OpenOptions;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use crate::cmd_csv::{ CmdRecord, read_cmd_file };
use crate::log_debug;
use crate::log_warn;

pub struct ConfigMem {
    counter: AtomicUsize,
    used_counter: AtomicUsize,
    commands: Vec<CmdRecord>,
    pub used_commands: Vec<CmdRecord>,
    pub commands_path: String,
    pub used_commands_path: String,
}

impl ConfigMem {
    pub fn config() -> ConfigMem {
        let mut home = home::home_dir().expect("Could not find home dir");
        log_warn!("Saving settings to: {}/.cmd", home.to_str().expect("could not parse home path"));

        home.push(".cmd");
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true).create(&home).expect("Could not create config folder");

        let t = home.join("cmds.csv");
        let commands_path = t.to_str().expect("could not convert path to string");

        let t2 = home.join("cmds_used.csv");
        let used_commands_path = t2.to_str().expect("could not convert path to string");

        {
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&commands_path)
                .expect("could not create or open config file");
        }

        {
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&used_commands_path)
                .expect("could not create or open config file");
        }

        ConfigMem::build(commands_path, used_commands_path)
    }

    pub fn build(commands_path: &str, used_commands_path: &str) -> ConfigMem {
        let commands = ConfigMem::init_csv_commands(commands_path);
        let used_commands = ConfigMem::init_csv_commands(used_commands_path);
        let counter = ConfigMem::init_counter(&commands);
        let used_counter = ConfigMem::init_counter(&used_commands);
        return ConfigMem {
            commands,
            used_commands,
            counter,
            used_counter,
            commands_path: String::from(commands_path),
            used_commands_path: String::from(used_commands_path),
        };
    }

    fn init_csv_commands(commands_path: &str) -> Vec<CmdRecord> {
        let csv_contents = read_cmd_file(commands_path);
        csv_contents
    }

    fn init_counter(csv_contents: &Vec<CmdRecord>) -> AtomicUsize {
        let counter = AtomicUsize::new(csv_contents.len() + 1);
        counter
    }

    fn get_id(self: &Self) -> usize {
        // static COUNTER: AtomicUsize = AtomicUsize::new(1);

        let size = self.counter.fetch_add(1, Ordering::Relaxed);
        size
    }

    fn get_used_cmd_id(self: &Self) -> usize {
        // static COUNTER: AtomicUsize = AtomicUsize::new(1);

        let size = self.used_counter.fetch_add(1, Ordering::Relaxed);
        size
    }

    pub fn get_commands(self: &Self) -> Vec<CmdRecord> {
        return self.commands.clone();
    }

    pub fn add_command(self: &Self, command: String) -> Result<(), &'static str> {
        let id = self.get_id();
        let record = CmdRecord { id: id, command: String::from(&command), used_times: 1 };
        let exists = self.commands.iter().find(|x| x.command == record.command);

        if let Some(_) = exists {
            return Err("Record already exists, returning");
        }
        self._add_command(&self.commands_path, record)
    }

    pub fn add_used_command(self: &Self, mut record: CmdRecord) -> Result<(), &'static str> {
        let record_exists = self.used_commands
            .iter()
            .find(|cmd| **cmd == record)
            .is_some();

        record.increase_usage();

        let mut updated_commands = self.used_commands
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
                let id = self.get_used_cmd_id();
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
            return self.reset_used_commands(vec![final_record]);
        }

        if !list_is_empty && !record_exists {
            log_debug!("Updating {:?} with {:?}", updated_commands, final_record);
            updated_commands.append(&mut vec![final_record]);
        }
        log_debug!("Reseting with {:?}", updated_commands);
        self.reset_used_commands(updated_commands)
    }

    fn reset_used_commands(
        self: &Self,
        mut updated_commands: Vec<CmdRecord>
    ) -> Result<(), &'static str> {
        let file = OpenOptions::new()
            .write(true)
            .open(&self.used_commands_path)
            .or_else(|_| Err("Could not open file"))?;

        let mut wtr = csv::WriterBuilder::new().has_headers(false).from_writer(file);
        updated_commands.sort_by(|a, b| b.used_times.cmp(&a.used_times));
        for cmd in updated_commands {
            wtr.serialize(&cmd).or_else(|_| Err("Could not serialize"))?;
        }

        wtr.flush().expect("clould not save csv file");

        Ok(())
    }

    fn _add_command(
        self: &Self,
        commands_path: &String,
        record: CmdRecord
    ) -> Result<(), &'static str> {
        let file = OpenOptions::new()
            .append(true)
            .open(commands_path)
            .or_else(|_| Err("Could not open file"))?;

        let mut wtr = csv::WriterBuilder::new().has_headers(false).from_writer(file);

        wtr.serialize(&record).or_else(|_| Err("Could not serialize"))?;
        wtr.flush().expect("clould not save csv file");
        log_debug!("Command saved to {}", commands_path);
        Ok(())
    }
}

impl Clone for ConfigMem {
    fn clone(&self) -> Self {
        Self {
            counter: AtomicUsize::new(self.counter.fetch_max(0, Ordering::SeqCst)),
            used_counter: AtomicUsize::new(self.used_counter.fetch_max(0, Ordering::SeqCst)),
            commands: self.commands.clone(),
            used_commands: self.used_commands.clone(),
            commands_path: self.commands_path.clone(),
            used_commands_path: self.used_commands_path.clone(),
        }
    }
}