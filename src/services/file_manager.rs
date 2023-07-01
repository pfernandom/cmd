use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

use std::io::Error;
use std::path::PathBuf;
use csv::Reader;
use csv::Writer;
use mockall::automock;
use crate::log_debug;
use crate::traits;
use crate::traits::file_manager::{ FileManager };

#[derive(Debug)]
pub struct FileManagerImpl {
    pub file_name: String,
    path: String,
    home: PathBuf,
}


pub struct FileManagerBuilder {
    file_name: String,
    path: String,
    home: PathBuf,
}

impl FileManagerBuilder {
    pub fn new(name: String) -> Self {
        let mut home = home::home_dir().expect("Could not find home dir");
        home.push(".cmd");
        Self{
            file_name: name.clone(),
            path: home.join(name).to_str().expect("could not convert path to string").to_string(),
            home: home ,
        }
    }
    pub fn on_dir(mut self, dir: String) -> Self {
        self.home =  Path::new(dir.as_str()).to_path_buf();
        self
    }

    pub fn build(self) -> impl FileManager {
        FileManagerImpl{file_name:self.file_name, path:self.path, home:self.home}
    }
}

pub trait Dependable: traits::file_manager::FileManager {}

impl Dependable for FileManagerImpl {}

pub fn build_file_manager<'a>(name: &'a str) -> FileManagerImpl {
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
    type W = File;

    type R = File;
    fn create_cmd_file(self: &Self) -> Result<(), String> {
        let mut dir_builder = std::fs::DirBuilder::new();
        dir_builder.recursive(true).create(&self.home).expect("Could not create config folder");


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

    fn get_cmd_writer(self: &mut Self, append: bool) -> Result<Writer<Self::W>, String> {
        log_debug!("Opening: {}", &self.path);
        let file = OpenOptions::new()
            .write(true)
            .append(append)
            .open(&self.path)

            .or_else(|err| Err(format!("Could not open file:{}: {:?}", &self.path, err)))?;

        let w: Writer<Self::W> = csv::WriterBuilder::new().has_headers(false).from_writer(file);
        Ok(w)
    }

    fn get_cmd_reader(self: &Self) -> Result<Reader<Self::R>, String> {
        log_debug!("Opening: {}", &self.path);
        let file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .or_else(|err| Err(format!("Could not open file:{}: {:?}", &self.path, err)))?;

        let w: Reader<Self::R> = csv::ReaderBuilder::new().has_headers(false).from_reader(file);
        Ok(w)
    }

    fn clear_files(self: &Self) -> Result<(), Error> {
        std::fs::remove_file(&self.path)
    }

    fn get_file_name(self: &Self) -> String {
        self.file_name.clone()
    }

    fn get_home_dir(&self) -> &PathBuf {
        return &self.home;
    }
}

#[cfg(test)]
mod tests {
    use crate::{services::file_manager::FileManagerBuilder, traits::file_manager::FileManager};


    #[test]
    fn test_create_file() {
        let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Debug).try_init();

        let file_manager = FileManagerBuilder::new("test.csv".to_string()).on_dir("./tmp".to_string()).build();
        assert_eq!(file_manager.get_home_dir().to_str().unwrap(), "./tmp");

        let result = file_manager.create_cmd_file();
        assert!(result.is_ok());

    }
}