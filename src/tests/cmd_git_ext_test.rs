use rusqlite::Connection;

use std::cell::RefCell;
use std::rc::Rc;
use crate::args::{ Cli, Commands };



use crate::cmd::cmd_get::GetHandler;
use crate::error::CmdError;
use crate::services::cmd_service_sql::CmdServiceSQL;
use crate::services::controller::Controller;

use crate::services::os_service::MockOSServiceImpl;
use crate::traits::cmd_service::CmdService;
use crate::traits::inputable::{ MockInputable };
use crate::{ Deps, log_info, log_debug };

use super::mocks::mock_opts::{ MutRef, MockOpts };

pub fn initialize() {
    let _ = env_logger::builder().is_test(true).filter_level(log::LevelFilter::Debug).try_init();
}
fn get_deps<'a>(
    mock_opts: MutRef<MockOpts<'static>>,
    all: Vec<&str>
) -> Result<Deps, CmdError> {
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
) -> Result<Deps, CmdError> {
    //build_file_manager("cmd_used.csv");

    let mut cmd_service_sql = CmdServiceSQL::build_cmd_service(
        Some(Connection::open_in_memory()?)
        //None
    )?;

    for cmd in all {
        cmd_service_sql.add_command(cmd.split(",").nth(1).unwrap().to_string())?;
    }

    let all_cmd_service= cmd_service_sql;

    // let all_cmd_service = build_cmd_csv_service(all_file_mgr, false)?;
    // let used_cmd_service = build_cmd_csv_service(used_file_mgr, false)?;

    let controller = Controller::<CmdServiceSQL> {
        all: all_cmd_service.clone(),
        used: all_cmd_service,
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

    Ok(Deps { args, controller, input: Rc::new(mock_input), os: Rc::new(mock_os) })
}

#[test]
fn test_git_ext() -> Result<(), CmdError> {
    initialize();
    let all_records = vec![
        "1,git log,0",
        "2,git checkout {} && git pull --rebase && git checkout {} && git merge {},0",
        "3,git commit -m {},0"
    ];

    let mock_opts = MockOpts::from(|_opts| { 1 });

    let deps = get_deps(Rc::clone(&mock_opts), all_records)?;
    let mut get_handler = GetHandler::new(Rc::new(RefCell::new(deps)));
    let result = get_handler.get_command(&None);

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

    let mock_opts = MockOpts::from(|_opts| { 2 });

    let deps = get_deps(Rc::clone(&mock_opts), all_records)?;
    let mut get_handler = GetHandler::new(Rc::new(RefCell::new(deps)));
    let result = get_handler.get_command(&None);

    let captures = &mock_opts.as_ref().take().captures;

    let captures = &captures.options_for_command;

    log_debug!("Captures: {:?}", captures);

    result
}