use std::{ io::{ Error, Read, Write } };

use csv::{ Reader, Writer };

// #[automock(type W=BufWriter<Writer<Box<dyn Write>>>; type R=BufReader<Reader<Box<dyn Read>>>;)]
pub trait FileManager: std::fmt::Debug {
    type W: Write;
    type R: Read;

    fn get_file_name(self: &Self) -> String;

    fn create_cmd_file(self: &Self) -> Result<(), String>;

    fn get_cmd_writter(self: &mut Self, append: bool) -> Result<Writer<Self::W>, String>;

    fn get_cmd_reader(self: &Self) -> Result<Reader<Self::R>, String>;

    fn clear_files(self: &Self) -> Result<(), Error>;
}