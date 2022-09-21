use std::io::{ Write, BufWriter };

use crate::{ * };
use csv::{ WriterBuilder, Reader, ReaderBuilder };
use vfs::{ VfsPath, SeekAndRead };

#[derive(Debug)]
pub struct MockFileManager<'a> {
    pub file_name: &'a str,
    pub root: &'a VfsPath,
    pub initial_content: &'a Vec<&'a str>,
}

impl<'a> FileManager for MockFileManager<'a> {
    type W = Box<dyn Write>;

    type R = Box<dyn SeekAndRead>;

    fn get_file_name(self: &Self) -> String {
        self.file_name.to_string()
    }

    fn create_cmd_file(self: &Self) -> Result<(), String> {
        let path = self.root.join(self.file_name).unwrap();
        let mut f = path.create_file().expect("Could not open file");

        let mut writer = BufWriter::new(&mut f);

        writer
            .write_all(self.initial_content.join("\n").as_bytes())
            .expect("Could not write in file");

        writer.flush().expect("Could not flush");

        Ok(())
    }

    fn get_cmd_writter(self: &mut Self, append: bool) -> Result<csv::Writer<Self::W>, String> {
        let path = self.root.join(self.file_name).unwrap();
        let f = if append { path.append_file().unwrap() } else { path.create_file().unwrap() };
        Ok(WriterBuilder::new().has_headers(false).from_writer(f))
    }

    fn get_cmd_reader(self: &Self) -> Result<Reader<Self::R>, String> {
        let path = self.root.join(self.file_name).unwrap();
        let f = path.open_file().unwrap();
        Ok(ReaderBuilder::new().has_headers(false).from_reader(f))
    }

    fn clear_files(self: &Self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl Drop for MockFileManager<'_> {
    fn drop(&mut self) {
        let path = self.root.join(self.file_name).unwrap();
        path.remove_file().expect("Could not clean up");
    }
}