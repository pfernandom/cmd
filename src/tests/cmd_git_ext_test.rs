#[cfg(test)]
mod tests {
    use env_logger::Builder;
    use rusqlite::Connection;
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
    use crate::{ FileManager, Deps, log_info, log_debug, log_error };

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            Builder::new().filter_level(log::LevelFilter::Debug).init();
        });
    }

    struct MockOpts<'a> {
        selected_record: Box<dyn (Fn(&Vec<String>) -> usize) + 'a>,
        captures: Captures,
    }

    // type OptionSelect = |&Vec<String>| -> usize;

    fn RcMut<T>(p: T) -> MutRef<T> {
        Rc::new(RefCell::new(p))
    }

    impl<'a> MockOpts<'a> {
        fn new() -> Rc<RefCell<MockOpts<'a>>> {
            RcMut(MockOpts::default())
        }
        fn capture_options_for_command(self: &mut Self, options: Vec<String>) {
            self.captures.options_for_command = options;
        }
        fn get_selected_record(self: &Self, vec: &Vec<String>) -> usize {
            let sr = &self.selected_record;
            sr(vec)
        }
        fn from(selected_record: impl 'a + Fn(&Vec<String>) -> usize) -> MutRef<MockOpts<'a>> {
            RcMut(MockOpts {
                selected_record: Box::new(selected_record),
                ..MockOpts::default()
            })
        }
    }

    impl Default for MockOpts<'_> {
        fn default() -> Self {
            Self {
                selected_record: Box::new(|_opts| { 0 }),
                captures: Captures::default(),
            }
        }
    }

    type MutRef<T> = Rc<RefCell<T>>;

    #[derive(Default)]
    struct Captures {
        options_for_command: Vec<String>,
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
    fn test_git_ext() -> Result<(), CmdError> {
        initialize();
        let all_records = vec![
            "1,git log,0",
            "2,git checkout {} && git pull --rebase && git checkout {} && git merge {},0",
            "3,git commit -m {},0"
        ];

        let mock_opts = MockOpts::from(|opts| { 1 });

        let mut deps = get_deps(Rc::clone(&mock_opts), all_records)?;
        let result = get_command(&None, &mut deps);

        let captures = &mock_opts.as_ref().take().captures;

        let captures = &captures.options_for_command;

        log_debug!("Captures: {:?}", captures);

        result
    }

    #[test]
    fn test_git_ext_2() -> Result<(), CmdError> {
        initialize();
        let all_records = vec![
            "1,git log,0",
            "2,git checkout {} && git pull --rebase && git checkout {} && git merge {},0",
            "3,git checkout {} && git pull --rebase && git checkout {},0"
        ];

        let mock_opts = MockOpts::from(|opts| { 2 });

        let mut deps = get_deps(Rc::clone(&mock_opts), all_records)?;
        let result = get_command(&None, &mut deps);

        let captures = &mock_opts.as_ref().take().captures;

        let captures = &captures.options_for_command;

        log_debug!("Captures: {:?}", captures);

        result
    }
}