use std::{ collections::HashMap, io::BufRead, convert::TryInto };

use sqlite::Connection;

use crate::{ traits::cmd_service::CmdService, error::CmdError, models::cmd_record::CmdRecord };

pub struct CmdServiceSQL {
    connection: Connection,
    commands: Vec<CmdRecord>,
}

impl CmdServiceSQL {
    pub fn build_cmd_service() -> Result<CmdServiceSQL, CmdError> {
        let mut home = home::home_dir().expect("Could not find home dir");
        home.push(".cmd");
        let connection = sqlite::open(home.join("csvdb"))?;

        // #[serde(rename = "id")]
        // pub id: usize,
        // #[serde(rename = "command")]
        // pub command: String,
        // #[serde(rename = "used_times")]
        // pub used_times: usize,

        connection.execute(
            "
        CREATE TABLE IF NOT EXISTS cmd (id INTEGER PRIMARY KEY, command TEXT, used_times INTEGER);
        "
        )?;

        Ok(CmdServiceSQL { connection: connection, commands: Vec::new() })
    }
}

impl CmdService<'_> for CmdServiceSQL {
    fn add_command(self: &mut Self, command: String) -> Result<(), crate::error::CmdError> {
        todo!()
    }

    fn update_command(
        self: &mut Self,
        command: crate::models::cmd_record::CmdRecord
    ) -> Result<(), crate::error::CmdError> {
        todo!()
    }

    fn is_record_present(self: &Self, record: &crate::models::cmd_record::CmdRecord) -> bool {
        todo!()
    }

    fn reset_commands(
        self: &mut Self,
        updated_commands: Vec<crate::models::cmd_record::CmdRecord>
    ) -> Result<(), crate::error::CmdError> {
        todo!()
    }

    fn get_commands(self: &mut Self) -> &Vec<crate::models::cmd_record::CmdRecord> {
        let mut statement = self.connection
            .prepare("SELECT * FROM users WHERE age > ?")
            // .unwrap()
            // .bind(1, 50)
            .unwrap()
            .into_cursor()
            .into_iter();

        self.commands = statement
            .filter_map(|some_row| {
                match some_row {
                    Ok(row) =>
                        Some(CmdRecord {
                            id: row.get::<i64, _>(0).try_into().unwrap(),
                            command: row.get::<String, _>(1),
                            used_times: row.get::<i64, _>(2).try_into().unwrap(),
                        }),
                    Err(_) => None,
                }
            })
            .collect::<Vec<CmdRecord>>();
        return &self.commands;
    }

    fn get_file_name(self: &Self) -> String {
        todo!()
    }

    fn clear_commands(self: &Self) -> Result<(), crate::error::CmdError> {
        todo!()
    }
}