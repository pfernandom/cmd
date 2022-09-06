#[cfg(test)]
mod tests {
    use csv::{ ReaderBuilder, WriterBuilder };
    use env_logger::Builder;
    // use assert_cmd::prelude::*; // Add methods on commands

    use std::io::{ BufWriter, BufReader, Cursor };
    use std::sync::Once;
    use crate::args::{ Cli, Commands };
    use crate::cmd::cmd_add::add_command;
    use crate::cmd::cmd_get::get_command;
    use crate::error::CmdError;
    use crate::services::cmd_service::build_cmd_service;
    use crate::services::controller::ConfigMem;
    use crate::services::file_manager::MockFileManagerImpl;
    use crate::services::os_service::MockOSServiceImpl;
    use crate::traits::inputable::{ MockInputable };
    use crate::{ FileManager, Deps, log_info, log_debug };

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            Builder::new().filter_level(log::LevelFilter::Debug).init();
        });
    }

    struct MockOpts<'a> {
        all_records: Vec<&'a str>,
        used_records: Vec<&'a str>,
        selected_record: Option<usize>,
    }

    fn get_cursor(records: &Vec<&str>) -> Cursor<Vec<u8>> {
        let v = records.join("\n");
        let content = v.as_bytes();
        let buf1: Vec<u8> = Vec::from(content);
        Cursor::new(buf1)
    }

    fn get_file_manager(records: Vec<&str>) -> MockFileManagerImpl {
        let mut mock = MockFileManagerImpl::new();

        let buf_read = BufReader::new(get_cursor(&records));
        let buf_write = BufWriter::new(get_cursor(&records));

        let reader: csv::Reader<Box<dyn std::io::Read>> = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Box::new(buf_read));
        let writer: csv::Writer<Box<dyn std::io::Write>> = WriterBuilder::new()
            .has_headers(false)
            .from_writer(Box::new(buf_write));

        mock.expect_create_cmd_file().returning(|| Ok(()));
        mock.expect_get_cmd_reader().return_once_st(|| Ok(reader));
        mock.expect_get_cmd_writter().return_once_st(|_| Ok(writer));
        mock
    }

    fn get_deps(mock_opts: MockOpts) -> Result<Deps, CmdError> {
        let all_file_mgr = get_file_manager(mock_opts.all_records); // = build_file_manager("cmd.csv");
        let used_file_mgr = get_file_manager(mock_opts.used_records);

        //build_file_manager("cmd_used.csv");
        all_file_mgr.create_cmd_file()?;
        used_file_mgr.create_cmd_file()?;
        let all_cmd_service = build_cmd_service(all_file_mgr)?;
        let used_cmd_service = build_cmd_service(used_file_mgr)?;

        let mem = ConfigMem { all: Box::new(all_cmd_service), used: Box::new(used_cmd_service) };

        let args: Cli = Cli {
            command: Commands::Add { pattern: false, execute: false },
            verbose: true,
        };

        let mut mock_input = MockInputable::new();
        mock_input.expect_get_input().returning(|_| "git".to_string());

        let result = mock_opts.selected_record.clone();

        mock_input.expect_select_option().returning_st(move |opts| {
            log_debug!("Select an option:");
            for o in opts {
                log_debug!("- {}", o);
            }
            result
        });

        let mut mock_os = MockOSServiceImpl::new();
        mock_os.expect_execute_command().returning_st(|arg| {
            log_info!("Running command {}", arg);
            Ok(true)
        });

        Ok(Deps { args, mem, input: Box::new(mock_input), os: Box::new(mock_os) })
    }

    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        initialize();
        let mock_opts = MockOpts {
            all_records: vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"],
            used_records: vec!["1,git log,2"],
            selected_record: None,
        };

        let mut deps = get_deps(mock_opts)?;
        crate::app(&mut deps);

        Ok(())
    }

    #[test]
    fn add_command_test() -> Result<(), CmdError> {
        initialize();
        let mock_opts = MockOpts {
            all_records: vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"],
            used_records: vec!["1,git log,2"],
            selected_record: None,
        };
        let deps = get_deps(mock_opts)?;
        add_command(false, true, &deps)
    }

    #[test]
    fn get_command_test() -> Result<(), CmdError> {
        initialize();
        let mock_opts = MockOpts {
            all_records: vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"],
            used_records: vec!["1,git log,2"],
            selected_record: Some(1),
        };
        let mut deps = get_deps(mock_opts)?;
        get_command(&None, &mut deps)
    }

    #[test]
    fn get_command_test_pattern() -> Result<(), CmdError> {
        initialize();
        let mock_opts = MockOpts {
            all_records: vec!["1,git log,0", "2,git branch,0", "3,git commit -m \"{}\",0"],
            used_records: vec!["1,git log,2"],
            selected_record: Some(2),
        };
        let mut deps = get_deps(mock_opts)?;
        get_command(&None, &mut deps)?;

        log_debug!("{:?}", deps.mem.get_commands());
        log_debug!("{:?}", deps.mem.get_used_commands());

        Ok(())
    }
}