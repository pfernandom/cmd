#[cfg(test)]
mod cmd_service_test {
    use std::{ sync::Once };

    use env_logger::Builder;
    use rusqlite::Connection;
    use vfs::{ VfsPath, MemoryFS };
    use crate::traits::cmd_service::SearchFiltersBuilder;

    use crate::{
        services::{
            cmd_service_csv::build_cmd_csv_service,
            cmd_service_sql::CmdServiceSQL,
            file_manager::{ FileManagerImpl, build_file_manager },
        },
        tests::{ mocks::file_manager::MockFileManager },
        traits::{ cmd_service::{ CmdService, SearchFilters }, file_manager::FileManager },
        models::cmd_record::{ CmdRecord, CmdRecordIterable },
        error,
        log_info,
        log_debug,
    };

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            Builder::new().filter_level(log::LevelFilter::Debug).init();
        });
    }

    #[test]
    fn it_works() -> Result<(), error::CmdError> {
        initialize();
        let used_records = vec!["1,git log,0", "2,git branch,0", "3,git commit -m {},0"];
        let test_command = "git commit -m \"this is a test\"".to_string();

        let root: VfsPath = MemoryFS::new().into();
        let mut used_file_mgr = MockFileManager {
            file_name: "used_file.csv",
            root: &root,
            initial_content: &used_records,
        };

        used_file_mgr.create_cmd_file()?;

        {
            let mut cmd_service = build_cmd_csv_service(&mut used_file_mgr, false)?;

            log_debug!("Run once");
            let ran_cmd = CmdRecord {
                id: 3,
                command: test_command.clone(),
                used_times: 2,
            };
            cmd_service.update_command(ran_cmd.clone())?;

            let updated_commands = cmd_service.get_commands(SearchFilters::default());
            let cmd_map = updated_commands.iter().group_by(|x| x.command.clone());

            let get_updated = cmd_map.get(&test_command);
            assert_eq!(get_updated.is_some(), true);
            assert_eq!(get_updated.unwrap().len(), 1);
            let actual = get_updated.unwrap().get(0).unwrap().clone();
            let expected = ran_cmd.clone();
            assert_eq!(actual, expected);

            log_info!(
                "Updated: {:?}",
                updated_commands.iter().group_by(|x| x.command.clone())
            );
        }

        {
            let mut cmd_service = build_cmd_csv_service(&mut used_file_mgr, false)?;

            log_debug!("Run twice");
            cmd_service.update_command(CmdRecord {
                id: 3,
                command: test_command.clone(),
                used_times: 3,
            })?;

            let updated_commands = cmd_service.get_commands(SearchFilters::default());

            log_info!("Updated: {:?}", updated_commands);
        }

        let mut cmd_service = build_cmd_csv_service(&mut used_file_mgr, false)?;
        let updated_commands = cmd_service.get_commands(SearchFilters::default());

        assert_eq!(updated_commands.len(), used_records.len() + 1);

        Ok(())
    }

    #[test]
    fn cmd_service_sql_test1() -> Result<(), error::CmdError> {
        initialize();
        let mut cmd_service = CmdServiceSQL::build_cmd_service(
            Some(Connection::open_in_memory()?)
        )?;

        cmd_service.add_command("git log".to_string())?;
        cmd_service.add_command("git branch".to_string())?;
        cmd_service.add_command("git commit -m {}".to_string())?;

        let commands = cmd_service.get_commands(SearchFilters::default());

        log_debug!("Commands: {:?}", commands);

        assert_eq!(commands.len(), 3);

        Ok(())
    }

    #[test]
    fn cmd_service_sql_test2() -> Result<(), error::CmdError> {
        initialize();
        let mut cmd_service = CmdServiceSQL::build_cmd_service(
            Some(Connection::open_in_memory()?)
        )?;

        cmd_service.add_command("git log".to_string())?;
        cmd_service.add_command("ls -l".to_string())?;
        cmd_service.add_command("git commit -m {}".to_string())?;

        let commands = cmd_service.get_commands(
            SearchFiltersBuilder::default().command("ls".to_string()).build().unwrap()
        );

        log_debug!("Commands: {:?}", commands);

        assert_eq!(commands.len(), 1);

        Ok(())
    }

    #[test]
    fn cmd_service_sql_add() -> Result<(), error::CmdError> {
        initialize();
        let mut cmd_service = CmdServiceSQL::build_cmd_service(
            Some(Connection::open_in_memory()?)
        )?;

        cmd_service.add_command("git log".to_string())?;
        cmd_service.add_command("ls -l".to_string())?;
        cmd_service.add_command("git commit -m {}".to_string())?;

        let mut commands = cmd_service.get_commands(
            SearchFiltersBuilder::default().command("ls".to_string()).build().unwrap()
        );

        log_debug!("Commands: {:?}", commands);

        let first_usage = commands.first().unwrap().used_times;

        {
            for cmd in &mut commands {
                cmd.increase_usage();
                cmd_service.update_command(cmd.clone())?;
            }
        }

        let commands2 = cmd_service.get_commands(
            SearchFiltersBuilder::default().command("ls".to_string()).build().unwrap()
        );

        log_debug!("Commands (after update): {:?}", commands2);

        assert_eq!(commands.len(), 1);
        assert_eq!(commands2.first().unwrap().used_times, first_usage + 1);

        Ok(())
    }

    #[test]
    fn cmd_service_sql_insert() -> Result<(), error::CmdError> {
        initialize();
        let mut cmd_service = CmdServiceSQL::build_cmd_service(
            Some(Connection::open_in_memory()?)
        )?;

        for x in 1..10 {
            let cmd = CmdRecord {
                id: 1,
                command: String::from("git log"),
                used_times: x,
            };

            cmd_service.insert_command(cmd)?;
        }

        let commands = cmd_service.get_commands(
            SearchFiltersBuilder::default()
                .command("git".to_string())
                .build()
                .expect("Could not parse search params")
        );

        log_debug!("Commands: {:?}", commands);
        assert_eq!(commands.len(), 1);

        Ok(())
    }

    #[test]
    fn migrate_cvs_test() -> Result<(), error::CmdError> {
        initialize();
        let mut cmd_service = CmdServiceSQL::build_cmd_service(
            Some(Connection::open_in_memory()?)
            //None
        )?;

        {
            // let saved_file_mgr: FileManagerImpl = build_file_manager("cmd.csv");
            // let used_file_mgr: FileManagerImpl = build_file_manager("cmd_used.csv");
            let used_records = vec![
                "1,git log,0",
                "2,git branch,4",
                "3,git commit -m {},1",
                "3,git commit -m {},1",
                "3,git commit -m {},1"
            ];

            let root: VfsPath = MemoryFS::new().into();

            let saved_file_mgr = MockFileManager {
                file_name: "save_file.csv",
                root: &root,
                initial_content: &used_records,
            };

            let used_file_mgr = MockFileManager {
                file_name: "used_file.csv",
                root: &root,
                initial_content: &used_records,
            };

            saved_file_mgr.create_cmd_file()?;
            used_file_mgr.create_cmd_file()?;

            let _ = &cmd_service.migrate_cvs(saved_file_mgr, used_file_mgr)?;
        }

        let commands = cmd_service.get_commands(SearchFilters::default());

        assert_eq!(
            commands
                .iter()
                .map(|cmd| format!("{},{}", cmd.command, cmd.used_times))
                .collect::<Vec<String>>(),
            vec!["git log,0", "git branch,4", "git commit -m {},3"]
        );

        log_debug!("Commands: {:?}", commands);

        Ok(())
    }

    #[ignore]
    #[test]
    fn migrate_cvs_test_prod() -> Result<(), error::CmdError> {
        initialize();
        let mut cmd_service = CmdServiceSQL::build_cmd_service(None)?;

        {
            let saved_file_mgr: FileManagerImpl = build_file_manager("cmd.csv");
            let used_file_mgr: FileManagerImpl = build_file_manager("cmd_used.csv");

            saved_file_mgr.create_cmd_file()?;
            used_file_mgr.create_cmd_file()?;

            let _ = &cmd_service.migrate_cvs(saved_file_mgr, used_file_mgr)?;
        }

        let commands = cmd_service.get_commands(SearchFilters::default());

        log_debug!("Commands: {:?}", commands);

        Ok(())
    }
}