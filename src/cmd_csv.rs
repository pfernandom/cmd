use std::{ io::Read };
use csv::Reader;
use serde::{ Serialize, Deserialize };
use std::hash::Hash;

#[derive(Debug, Deserialize, Serialize, Eq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CmdRecord {
    #[serde(rename = "id")]
    pub id: usize,
    #[serde(rename = "command")]
    pub command: String,
    #[serde(rename = "used_times")]
    pub used_times: usize,
}

impl Hash for CmdRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.command.hash(state);
        self.used_times.hash(state);
    }
}

impl PartialEq for CmdRecord {
    fn eq(&self, other: &Self) -> bool {
        self.command == other.command
    }
}

impl CmdRecord {
    pub fn update_command(mut self: &mut Self, command: &str) {
        self.command = command.to_string();
    }

    pub fn increase_usage(mut self: &mut Self) {
        self.used_times += 1;
    }
}

pub fn read_cmd_file<T: Read>(rdr: &mut Reader<T>) -> Vec<CmdRecord> {
    let options = rdr
        .deserialize()
        .enumerate()
        .map(|x| x.1)
        .filter(|x| x.is_ok())
        .map(|x| {
            let rec: CmdRecord = x.unwrap();
            rec
        })
        .collect::<Vec<_>>();

    options
}