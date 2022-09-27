#[cfg(test)]
mod tests {
    use env_logger::Builder;
    use rusqlite::Connection;
    use std::borrow::Borrow;
    use std::cell::RefCell;
    use std::collections::{ HashMap, HashSet };
    use std::io::{ BufWriter, Cursor, Write, Read };
    use std::rc::Rc;
    use std::sync::Once;
    use crate::args::{ Cli, Commands };
    use crate::cmd::cmd_add::add_command;
    use crate::cmd::cmd_get::get_command;
    use crate::cmd_csv::{ read_cmd_file };
    use crate::models::cmd_record::{ CmdRecord, CmdRecordIterable };
    use crate::error::CmdError;
    use crate::services::cmd_service_sql::CmdServiceSQL;
    use crate::services::controller::Controller;
    use crate::services::file_manager::{ FileManagerImpl, build_file_manager };
    use crate::services::os_service::MockOSServiceImpl;
    use crate::traits::cmd_service::CmdService;
    use crate::traits::inputable::{ MockInputable };
    use crate::tests::mocks::mock_opts::*;
    use crate::{ FileManager, Deps, log_info, log_debug, log_error };

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            Builder::new().filter_level(log::LevelFilter::Debug).init();
        });
    }

    fn get_cursor(records: &mut Vec<&str>) -> Cursor<Vec<u8>> {
        let v = records.join("\n");
        let content = v.as_bytes();
        let buf1: Vec<u8> = Vec::from(content);
        Cursor::new(buf1)
    }

    fn get_deps<'a>(
        mock_opts: MutRef<MockOpts<'static>>,
        all: Vec<&str>
    ) -> Result<Deps<'a>, CmdError> {
        let args: Cli = Cli {
            get_command: Some("".to_string()),
            command: Some(Commands::Add { pattern: false, execute: false }),
            verbose: true,
            dry_run: false,
            generator: None,
        };
        get_deps_2(mock_opts, args, all)
    }

    fn get_deps_2<'a>(
        mock_opts: MutRef<MockOpts<'static>>,
        args: Cli,
        all: Vec<&str>
    ) -> Result<Deps<'a>, CmdError> {
        //build_file_manager("cmd_used.csv");

        let mut cmd_service_sql = CmdServiceSQL::build_cmd_service(
            Some(Connection::open_in_memory()?)
            //None
        )?;

        for cmd in all {
            cmd_service_sql.add_command(cmd.split(",").nth(1).unwrap().to_string())?;
        }

        let all_cmd_service: Rc<RefCell<dyn CmdService<'_>>> = Rc::new(
            RefCell::new(cmd_service_sql)
        );

        // let all_cmd_service = build_cmd_csv_service(all_file_mgr, false)?;
        // let used_cmd_service = build_cmd_csv_service(used_file_mgr, false)?;

        let controller = Controller {
            all: Rc::clone(&all_cmd_service),
            used: Rc::clone(&all_cmd_service),
        };

        let mut mock_input = MockInputable::new();
        mock_input.expect_get_input().returning(|_| "git".to_string());

        mock_input.expect_select_option().returning_st(move |opts, _maybe_prompt| {
            let prompt = match _maybe_prompt {
                Some(prompt) => prompt,
                None => String::from("Select an option:"),
            };
            log_debug!("{}", prompt);

            for o in opts {
                log_debug!("- {}", o);
            }

            // let mo: &RefCell<MockOpts> = mock_opts.borrow();
            // let mo: &MockOpts = &mut mo.borrow();

            let x = mock_opts.as_ref();

            let result = x.borrow_mut().get_selected_record(opts);
            x.borrow_mut().capture_options_for_command(opts.clone());

            // let _ = mock_opts.capture_options_for_command(*opts);

            Some(std::cmp::min(result, opts.len() - 1))
        });

        let mut mock_os = MockOSServiceImpl::new();
        mock_os.expect_execute_command().returning_st(|arg| {
            log_info!("Running command {}", arg);
            Ok(true)
        });

        Ok(Deps { args, controller, input: Box::new(mock_input), os: Box::new(mock_os) })
    }

    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mut mock_opts = MockOpts::new();

        let mut deps = get_deps(mock_opts, all_records)?;
        crate::app(&mut deps);

        Ok(())
    }

    #[test]
    fn add_command_test() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mut mock_opts = MockOpts::new();

        let mut deps = get_deps(mock_opts, all_records)?;
        let result = add_command(false, true, &mut deps);

        result
    }

    #[test]
    fn get_command_test() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mock_opts = MockOpts::new();

        let mut deps = get_deps(Rc::clone(&mock_opts), all_records)?;
        let result = get_command(&None, &mut deps);

        let captures = &mock_opts.as_ref().take().captures;

        let captures = &captures.options_for_command;

        log_debug!("Captures: {:?}", captures);

        result
    }

    #[test]
    fn get_command_test_2() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,2", "3,git commit -m {},0"];

        let mock_opts = MockOpts::from(|opts| {
            match opts.len() {
                2 => 1,
                _ => 0,
            }
        });

        let args = Cli {
            command: None,
            get_command: None,
            verbose: true,
            dry_run: false,
            generator: None,
        };

        let mut deps = get_deps_2(Rc::clone(&mock_opts), args, all_records)?;
        let result = get_command(&None, &mut deps);

        let captures = &mock_opts.as_ref().take().captures;

        let captures = &captures.options_for_command;

        log_debug!("Captures: {:?}", captures);

        result
    }

    #[test]
    fn get_command_test_many_times() -> Result<(), CmdError> {
        initialize();

        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mut results = Vec::<Vec<CmdRecord>>::new();
        let uses: usize = 5;

        let mock_opts = MockOpts::from(|opts: &Vec<String>| {
            match opts.binary_search(&"git commit -m {}".to_string()) {
                Ok(index) => index,
                Err(err) => {
                    log_debug!("{}", format!("Option is not preset: {:?}", opts).as_str());
                    log_error!("{}", err);
                    0
                }
            }
        });

        let mut deps = get_deps(Rc::clone(&mock_opts), all_records.clone())?;

        for _i in 0..uses {
            let _result = get_command(&None, &mut deps);

            results.push(deps.controller.get_used_commands("".to_string()).clone());

            log_debug!("All results: {:?}", &deps.controller.get_commands("".to_string()).clone());
        }

        let test_cmd = &results
            .last()
            .unwrap()
            .iter()
            .group_by(|x| x.command.clone());

        log_debug!("Results: {:?}", test_cmd);
        let end_cmd = test_cmd.get("git commit -m git").unwrap();

        assert_eq!(end_cmd.len(), 1);
        assert_eq!(end_cmd.iter().sum_count(), uses);

        let mocks = mock_opts.take();

        let mut captures = mocks.captures.options_for_command;
        log_debug!("Captures: {:?}", captures);
        let mut h = captures
            .iter()
            .map(|x| x.clone())
            .collect::<HashSet<String>>()
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<String>>();

        captures.sort();
        h.sort();
        assert_eq!(captures, h);

        Ok(())
    }

    #[test]
    fn get_command_test_many_times_2() -> Result<(), CmdError> {
        initialize();

        let all_records = vec!["1,git log,2", "2,git branch,0", "3,git commit -m {},0"];

        let mut results = Vec::<Vec<CmdRecord>>::new();
        let uses: usize = 5;

        let mock_opts = MockOpts::from(|opts: &Vec<String>| {
            match opts.len() {
                2 => 1,
                _ =>
                    match opts.binary_search(&"git commit -m {}".to_string()) {
                        Ok(index) => index,
                        Err(err) => {
                            log_debug!("{}", format!("Option is not preset: {:?}", opts).as_str());
                            log_error!("{}", err);
                            0
                        }
                    }
            }
        });

        let args = Cli::default();

        let mut deps = get_deps_2(Rc::clone(&mock_opts), args, all_records.clone())?;

        for _i in 0..uses {
            let _result = get_command(&None, &mut deps);

            results.push(deps.controller.get_used_commands("".to_string()).clone());

            log_debug!("All results: {:?}", &deps.controller.get_commands("".to_string()).clone());
        }

        let test_cmd = &results
            .last()
            .unwrap()
            .iter()
            .group_by(|x| x.command.clone());

        log_debug!("Results: {:?}", test_cmd);
        let end_cmd = test_cmd.get("git commit -m git").unwrap();

        assert_eq!(end_cmd.len(), 1);
        assert_eq!(end_cmd.iter().sum_count(), uses);

        let mocks = mock_opts.take();

        let mut captures = mocks.captures.options_for_command;
        log_debug!("Captures: {:?}", captures);
        let mut h = captures
            .iter()
            .map(|x| x.clone())
            .collect::<HashSet<String>>()
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<String>>();

        captures.sort();
        h.sort();
        assert_eq!(captures, h);

        Ok(())
    }

    #[test]
    fn get_command_test_pattern() -> Result<(), CmdError> {
        initialize();
        let all_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];

        let mut mock_opts = MockOpts::new();

        let mut deps = get_deps(mock_opts, all_records)?;
        get_command(&None, &mut deps)?;

        log_debug!("{:?}", deps.controller.get_commands("".to_string()));
        log_debug!("{:?}", deps.controller.get_used_commands("".to_string()));

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

        let grouped = commands.iter().fold(map, |mut acc, item| {
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

        println!("{grouped:?}, {saved_commands:?}");

        let mut writer = used_file_mgr.get_cmd_writer(false)?;

        for x in grouped.values() {
            print!("Write {x:?}");
            writer.serialize(x)?;
        }

        writer.flush()?;

        Ok(())
    }
}