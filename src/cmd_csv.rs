use std::fs::OpenOptions;
use serde::{ Serialize, Deserialize };
use crate::log_debug;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Hash, Eq, Clone)]
pub struct CmdRecord {
    #[serde(rename = "id")]
    pub id: usize,
    #[serde(rename = "command")]
    pub command: String,
    #[serde(rename = "used_times")]
    pub used_times: usize,
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

pub fn read_cmd_file(commands_path: &str) -> Vec<CmdRecord> {
    log_debug!("Reading {}", commands_path);
    let file = OpenOptions::new().read(true).open(commands_path).expect("Could not open file");

    // let reader = BufReader::new(file);

    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(file);

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