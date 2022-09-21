#[cfg(test)]
mod cmd_service_test {
    use std::sync::Once;

    use env_logger::Builder;
    use vfs::{ VfsPath, MemoryFS };

    use crate::{
        services::cmd_service_csv::build_cmd_csv_service,
        tests::{ mocks::file_manager::MockFileManager },
        traits::{ cmd_service::CmdService, file_manager::FileManager },
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
            let mut cmd_service = build_cmd_csv_service(&mut used_file_mgr)?;

            log_debug!("Run once");
            let ran_cmd = CmdRecord {
                id: 3,
                command: test_command.clone(),
                used_times: 2,
            };
            cmd_service.update_command(ran_cmd.clone())?;

            let updated_commands = cmd_service.get_commands();
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
            let mut cmd_service = build_cmd_csv_service(&mut used_file_mgr)?;

            log_debug!("Run twice");
            cmd_service.update_command(CmdRecord {
                id: 3,
                command: test_command.clone(),
                used_times: 3,
            })?;

            let updated_commands = cmd_service.get_commands();

            log_info!("Updated: {:?}", updated_commands);
        }

        let mut cmd_service = build_cmd_csv_service(&mut used_file_mgr)?;
        let updated_commands = cmd_service.get_commands();

        assert_eq!(updated_commands.len(), used_records.len() + 1);

        Ok(())
    }
}