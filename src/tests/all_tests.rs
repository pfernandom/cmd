#[cfg(test)]
mod tests {
    use env_logger::Builder;
    use vfs::{ VfsPath, MemoryFS };
    // use assert_cmd::prelude::*; // Add methods on commands
    use std::collections::{ HashMap };
    use std::io::{ BufWriter, Cursor, Write, Read };
    use std::sync::Once;
    use crate::args::{ Cli, Commands };
    use crate::cmd::cmd_add::add_command;
    use crate::cmd::cmd_get::get_command;
    use crate::cmd_csv::{ read_cmd_file };
    use crate::models::cmd_record::{ CmdRecord, CmdRecordIterable };
    use crate::error::CmdError;
    use crate::services::cmd_service_csv::build_cmd_csv_service;
    use crate::services::controller::Controller;
    use crate::services::file_manager::{ FileManagerImpl, build_file_manager };
    use crate::services::os_service::MockOSServiceImpl;
    use crate::tests::mocks::file_manager::MockFileManager;
    use crate::traits::inputable::{ MockInputable };
    use crate::{ FileManager, Deps, log_info, log_debug };

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            Builder::new().filter_level(log::LevelFilter::Debug).init();
        });
    }

    struct MockOpts<'a> {
        selected_record: Box<dyn (FnMut(&Vec<String>) -> usize) + 'a>,
    }

    fn get_cursor(records: &mut Vec<&str>) -> Cursor<Vec<u8>> {
        let v = records.join("\n");
        let content = v.as_bytes();
        let buf1: Vec<u8> = Vec::from(content);
        Cursor::new(buf1)
    }

    fn get_deps<'a>(
        mut mock_opts: MockOpts<'static>,
        all_file_mgr: &'a mut MockFileManager,
        used_file_mgr: &'a mut MockFileManager,
        create_files: bool
    ) -> Result<Deps<'a>, CmdError> {
        //build_file_manager("cmd_used.csv");

        if create_files {
            all_file_mgr.create_cmd_file()?;
            used_file_mgr.create_cmd_file()?;
        }
        let all_cmd_service = build_cmd_csv_service(all_file_mgr)?;
        let used_cmd_service = build_cmd_csv_service(used_file_mgr)?;

        let mem = Controller { all: Box::new(all_cmd_service), used: Box::new(used_cmd_service) };

        let args: Cli = Cli {
            get_command: Some("".to_string()),
            command: Some(Commands::Add { pattern: false, execute: false }),
            verbose: true,
        };

        let mut mock_input = MockInputable::new();
        mock_input.expect_get_input().returning(|_| "git".to_string());

        mock_input.expect_select_option().returning_st(move |opts, _maybe_prompt| {
            log_debug!("Select an option:");
            for o in opts {
                log_debug!("- {}", o);
            }
            let select_record = &mut mock_opts.selected_record;

            let result = select_record(opts);
            Some(std::cmp::min(result, opts.len() - 1))
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
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];
        let used_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mock_opts = MockOpts {
            selected_record: Box::new(|_opts| { 0 }),
        };
        let root: VfsPath = MemoryFS::new().into();
        let mut all_file_mgr = MockFileManager {
            file_name: "all_file.csv",
            root: &root,
            initial_content: &all_records,
        };
        // = build_file_manager("cmd.csv");
        let mut used_file_mgr = MockFileManager {
            file_name: "used_file.csv",
            root: &root,
            initial_content: &used_records,
        };

        let mut deps = get_deps(mock_opts, &mut all_file_mgr, &mut used_file_mgr, true)?;
        crate::app(&mut deps);

        Ok(())
    }

    #[test]
    fn add_command_test() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];
        let used_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mock_opts = MockOpts {
            selected_record: Box::new(|_opts| { 0 }),
        };
        let root: VfsPath = MemoryFS::new().into();
        let mut all_file_mgr = MockFileManager {
            file_name: "all_file.csv",
            root: &root,
            initial_content: &all_records,
        };
        // = build_file_manager("cmd.csv");
        let mut used_file_mgr = MockFileManager {
            file_name: "used_file.csv",
            root: &root,
            initial_content: &used_records,
        };

        all_file_mgr.create_cmd_file()?;
        used_file_mgr.create_cmd_file()?;

        let mut deps = get_deps(mock_opts, &mut all_file_mgr, &mut used_file_mgr, true)?;
        let result = add_command(false, true, &mut deps);

        result
    }

    #[test]
    fn get_command_test() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];
        let used_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mock_opts = MockOpts {
            selected_record: Box::new(|_opts| { 0 }),
        };
        let root: VfsPath = MemoryFS::new().into();
        let mut all_file_mgr = MockFileManager {
            file_name: "all_file.csv",
            root: &root,
            initial_content: &all_records,
        };
        // = build_file_manager("cmd.csv");
        let mut used_file_mgr = MockFileManager {
            file_name: "used_file.csv",
            root: &root,
            initial_content: &used_records,
        };

        let mut deps = get_deps(mock_opts, &mut all_file_mgr, &mut used_file_mgr, true)?;
        let result = get_command(&None, &mut deps);

        result
    }

    #[test]
    fn get_command_test_many_times() -> Result<(), CmdError> {
        initialize();

        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];
        let used_records = vec!["1,git log,2", "2,git branch,0"];

        let root: VfsPath = MemoryFS::new().into();
        let mut all_file_mgr = MockFileManager {
            file_name: "all_file.csv",
            root: &root,
            initial_content: &all_records,
        };
        // = build_file_manager("cmd.csv");
        let mut used_file_mgr = MockFileManager {
            file_name: "used_file.csv",
            root: &root,
            initial_content: &used_records,
        };

        all_file_mgr.create_cmd_file()?;
        used_file_mgr.create_cmd_file()?;
        let mut results = Vec::<Vec<CmdRecord>>::new();
        let uses: usize = 5;
        for _i in 0..uses {
            let mock_opts = MockOpts {
                selected_record: Box::new(|opts| {
                    opts.binary_search(&"git commit -m {}".to_string()).unwrap()
                }),
            };
            let mut deps = get_deps(mock_opts, &mut all_file_mgr, &mut used_file_mgr, false)?;
            let _result = get_command(&None, &mut deps);

            results.push(deps.mem.get_used_commands("".to_string()).clone());
        }

        let test_cmd = &results
            .last()
            .unwrap()
            .iter()
            .group_by(|x| x.command.clone());

        let end_cmd = test_cmd.get("git commit -m git").unwrap();

        assert_eq!(end_cmd.len(), 1);
        assert_eq!(end_cmd.iter().sum_count(), uses);

        Ok(())
    }

    #[test]
    fn get_command_test_pattern() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];
        let used_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mock_opts = MockOpts {
            selected_record: Box::new(|_x| { 0 }),
        };
        let root: VfsPath = MemoryFS::new().into();
        let mut all_file_mgr = MockFileManager {
            file_name: "all_file.csv",
            root: &root,
            initial_content: &all_records,
        };
        // = build_file_manager("cmd.csv");
        let mut used_file_mgr = MockFileManager {
            file_name: "used_file.csv",
            root: &root,
            initial_content: &used_records,
        };

        let mut deps = get_deps(mock_opts, &mut all_file_mgr, &mut used_file_mgr, true)?;
        get_command(&None, &mut deps)?;

        log_debug!("{:?}", deps.mem.get_commands("".to_string()));
        log_debug!("{:?}", deps.mem.get_used_commands("".to_string()));

        Ok(())
    }

    #[test]
    fn test_cursor() -> Result<(), CmdError> {
        let mut examples = vec!["test1"];
        let mut c = get_cursor(&mut examples);
        let mut out1 = Vec::new();
        c.read_to_end(&mut out1).expect("Could not read");

        let mut w = || {
            let mut buf_write = BufWriter::new(&mut c);

            match buf_write.write_fmt(format_args!("Hello: {:.*}", 2, 1.234567)) {
                Ok(data) => println!("data:{:?}", data),
                Err(_) => {}
            }

            match buf_write.flush() {
                Ok(res) => { println!("FlusJ:{:?}", res) }
                Err(_) => (),
            }
        };

        w();

        let mut out = Vec::new();
        c.set_position(0);
        c.read_to_end(&mut out).unwrap();

        println!("Result: {:?}", std::str::from_utf8(out.as_slice()));
        Ok(())
    }

    #[test]
    fn clean_used() -> Result<(), CmdError> {
        let saved_file_mgr: FileManagerImpl = build_file_manager("cmd.csv");
        let mut used_file_mgr: FileManagerImpl = build_file_manager("cmd_used.csv");

        let mut reader = used_file_mgr.get_cmd_reader()?;
        let commands = read_cmd_file(&mut reader);

        let mut saved_reader = saved_file_mgr.get_cmd_reader()?;
        let saved_commands = read_cmd_file(&mut saved_reader);

        let map: HashMap<String, CmdRecord> = HashMap::new();

        let gouped = commands.iter().fold(map, |mut acc, item| {
            match acc.get(&item.command) {
                Some(record) => {
                    let mut new_record = record.clone();

                    new_record.used_times += item.used_times;

                    acc.insert(item.command.clone(), new_record);
                }
                None => {
                    acc.insert(item.command.clone(), item.clone());
                }
            }
            acc
        });

        println!("{gouped:?}, {saved_commands:?}");

        let mut writer = used_file_mgr.get_cmd_writter(false)?;

        for x in gouped.values() {
            print!("Write {x:?}");
            writer.serialize(x)?;
        }

        writer.flush()?;

        Ok(())
    }
}