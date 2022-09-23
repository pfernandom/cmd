use rusqlite::Connection;

use crate::{
    traits::{ cmd_service::CmdService, file_manager::FileManager },
    error::{ CmdError, self },
    models::cmd_record::{ CmdRecord, CmdRecordIterable },
    cmd_csv::read_cmd_file,
    log_debug,
    log_info,
};

pub struct CmdServiceSQL {
    connection: Connection,
}

impl CmdServiceSQL {
    pub fn build_cmd_service(cntn: Option<Connection>) -> Result<CmdServiceSQL, CmdError> {
        let connection = match cntn {
            Some(cn) => { cn }
            None => {
                let mut home = home::home_dir().expect("Could not find home dir");
                home.push(".cmd");
                Connection::open(home.join("csvdb"))?
            }
        };

        // #[serde(rename = "id")]
        // pub id: usize,
        // #[serde(rename = "command")]
        // pub command: String,
        // #[serde(rename = "used_times")]
        // pub used_times: usize,

        connection.execute(
            "
        CREATE TABLE IF NOT EXISTS cmd (id INTEGER PRIMARY KEY, command TEXT UNIQUE, used_times INTEGER);
        CREATE INDEX IF NOT EXISTS commands_ind ON cmd (command);
        ",
            []
        )?;

        Ok(CmdServiceSQL { connection: connection })
    }

    pub fn migrate_cvs(
        &mut self,
        saved_file_mgr: impl FileManager,
        used_file_mgr: impl FileManager
    ) -> Result<(), CmdError> {
        let mut reader = used_file_mgr.get_cmd_reader()?;
        let used_commands = read_cmd_file(&mut reader);

        let mut saved_reader = saved_file_mgr.get_cmd_reader()?;
        let saved_commands = read_cmd_file(&mut saved_reader);

        let mut list = Vec::new();
        list.extend(used_commands);
        list.extend(saved_commands);

        log_debug!("Before: {:?}", list);

        let result = list.iter().group_and_sum_count(|cmd| cmd.command.clone());

        log_debug!("SET: {:?}", &result);

        for r in result {
            self.insert_command(r.clone())?;
        }

        Ok(())
    }
}

impl CmdService<'_> for CmdServiceSQL {
    fn add_command(self: &mut Self, command: String) -> Result<(), error::CmdError> {
        self.connection.execute("INSERT INTO cmd (command, used_times) VALUES (?1, ?2)", (
            &command,
            0,
        ))?;

        Ok(())
    }

    fn update_command(
        self: &mut Self,
        command: crate::models::cmd_record::CmdRecord
    ) -> Result<(), crate::error::CmdError> {
        self.connection
            .execute("UPDATE cmd SET command=?1, used_times=?2 wHERE id = ?3", (
                &command.command,
                &command.used_times,
                &command.id,
            ))
            .map_err(|err| CmdError::SQLError(format!("Could not update: {}", err.to_string())))?;

        Ok(())
    }

    fn is_record_present(self: &Self, record: &crate::models::cmd_record::CmdRecord) -> bool {
        match
            self.connection.query_row(
                "SELECT id, command, used_times FROM cmd WHERE command = ?1)",
                (&record.command,),
                |_row| Ok(())
            )
        {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn reset_commands(
        self: &mut Self,
        _updated_commands: Vec<crate::models::cmd_record::CmdRecord>
    ) -> Result<(), crate::error::CmdError> {
        log_debug!("No need for SQL");
        Ok(())
    }

    fn get_commands(
        self: &mut Self,
        filter: Option<String>
    ) -> Vec<crate::models::cmd_record::CmdRecord> {
        match filter {
            Some(command) => {
                self.connection
                    .prepare(
                        format!(
                            "SELECT id, command, used_times FROM cmd WHERE command LIKE {}",
                            format!("'%{}%'", command)
                        ).as_str()
                    )
                    .unwrap()
                    .query_map([], |row| Ok(CmdRecord::from(row.clone())))
                    .unwrap()
                    .map(|el| el.unwrap().clone())
                    .collect()
            }
            None => {
                self.connection
                    .prepare("SELECT id, command, used_times FROM cmd")
                    .unwrap()
                    .query_map([], |row| Ok(CmdRecord::from(row.clone())))
                    .unwrap()
                    .map(|el| el.unwrap().clone())
                    .collect()
            }
        }
    }

    fn get_file_name(self: &Self) -> String {
        todo!()
    }

    fn clear_commands(self: &Self) -> Result<(), crate::error::CmdError> {
        todo!()
    }

    fn insert_command(self: &mut Self, cmd: CmdRecord) -> Result<(), CmdError> {
        self.connection.execute(
            "INSERT INTO cmd (command, used_times) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            (&cmd.command, &cmd.used_times)
        )?;

        Ok(())
    }

    fn debug(self: &Self) {
        let vec: Vec<String> = self.connection
            .prepare(
                "SELECT COUNT(*), used_times FROM cmd GROUP BY used_times ORDER BY used_times DESC"
            )
            .unwrap()
            .query_map([], |row|
                Ok(
                    format!(
                        "There are {} used commands that have beed used {} times",
                        row.get::<usize, usize>(0).unwrap(),
                        row.get::<usize, usize>(1).unwrap()
                    )
                )
            )
            .unwrap()
            .map(|e| e.unwrap())
            .collect();
        log_info!("Stored commands:");
        for data in &vec {
            log_info!("- {}", data);
        }
    }
}