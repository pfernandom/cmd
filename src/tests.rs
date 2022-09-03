#[cfg(test)]
mod tests {
    // #[macro_export]
    // macro_rules! format_vec {
    //     (
    //         $pattern:ident,
    //         $($arg:tt)*
    //     ) => {
    //      format!($($arg)*)
    //     };
    // }

    // fn parse_str(args: &Arguments) {
    //     if let Some(s) = args.as_str() {
    //         println!("{}", s)
    //     } else {
    //         println!("{}", &args.to_string());
    //     }
    // }

    use csv::{ Reader, Writer };
    use env_logger::Builder;
    // use assert_cmd::prelude::*; // Add methods on commands

    use std::io::{ BufWriter, BufReader, Cursor };

    use crate::args::{ Cli, Commands };
    use crate::cmd_config_mem::{ ConfigMem, build_cmd_service };
    use crate::cmd_config_mem::MockFileManagerImpl;
    use crate::traits::inputable::{ MockInputable };
    use crate::{ FileManager, Deps };

    fn get_file_manager() -> MockFileManagerImpl {
        let mut mock = MockFileManagerImpl::new();

        let buf1: Vec<u8> = Vec::new();
        let buf2: Vec<u8> = Vec::new();

        let mockfile1 = Cursor::new(buf1);
        let mockfile2 = Cursor::new(buf2);

        let buf_read = BufReader::new(mockfile1);
        let buf_write = BufWriter::new(mockfile2);

        // let b = Box::new(bytes);

        let reader: csv::Reader<Box<dyn std::io::Read>> = Reader::from_reader(Box::new(buf_read));
        let writer: csv::Writer<Box<dyn std::io::Write>> = Writer::from_writer(Box::new(buf_write));

        mock.expect_create_cmd_file().returning(|| Ok(()));
        mock.expect_get_cmd_reader().return_once_st(|| Ok(reader));
        mock.expect_get_cmd_writter().return_once_st(|_| Ok(writer));
        mock
    }

    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        Builder::new().filter_level(log::LevelFilter::Debug).init();

        let all_file_mgr = get_file_manager(); // = build_file_manager("cmd.csv");
        let used_file_mgr = get_file_manager();

        //build_file_manager("cmd_used.csv");
        all_file_mgr.create_cmd_file()?;
        used_file_mgr.create_cmd_file()?;
        let all_cmd_service = build_cmd_service(all_file_mgr)?;
        let used_cmd_service = build_cmd_service(used_file_mgr)?;

        let mem = ConfigMem { all: all_cmd_service, used: used_cmd_service };

        let args: Cli = Cli {
            command: Commands::Add { pattern: false, execute: false },
            verbose: true,
        };

        let mut mock_input = MockInputable::new();
        mock_input.expect_get_input().returning(|_| "git".to_string());
        let deps = Deps { args, mem, input: Box::new(mock_input) };
        crate::app(&deps);

        // let mock1 = deps.mem.all.file_mgr;
        // assert_eq!("true", mock_input.get_input(Some("hello".to_string())));

        // cmd.arg("-v").arg("get");

        // cmd.assert().failure().stderr(predicate::str::contains("could not read file"));

        Ok(())
    }
}