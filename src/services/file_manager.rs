use std::fs::OpenOptions;

use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use csv::Reader;
use csv::Writer;
use mockall::automock;
use crate::log_debug;
use crate::traits::file_manager::{ FileManager };

#[derive(Debug)]
pub struct FileManagerImpl {
    pub file_name: String,
    path: String,
    home: PathBuf,
}

pub fn build_file_manager(name: &str) -> FileManagerImpl {
    let mut home = home::home_dir().expect("Could not find home dir");
    log_debug!("Saving settings to: {}/.cmd", home.to_str().expect("could not parse home path"));

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
        log_debug!("Creating file: {}", &commands_path);
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&commands_path)
            .or_else(|err| Err(err.to_string()))?;
        Ok(())
    }

    fn get_cmd_writter(self: &Self, append: bool) -> Result<Writer<Self::W>, String> {
        log_debug!("Opening: {}", &self.path);
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
        log_debug!("Opening: {}", &self.path);
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