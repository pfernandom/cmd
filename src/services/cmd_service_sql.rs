use std::{collections::HashMap, rc::Rc, borrow::Borrow};

use rusqlite::{ Connection, params };

use crate::{
    traits::{ cmd_service::{ CmdService, SearchFilters }, file_manager::FileManager },
    error::{ CmdError, self },
    models::cmd_record::{ CmdRecord },
    log_debug,
    log_info,
    services::cmd_service_csv::build_cmd_csv_service,
};


pub struct CmdServiceSQL {
    connection: Rc<Connection>,
}

impl Clone for CmdServiceSQL{
    fn clone(&self) -> Self {
        Self {
            connection: Rc::clone(self.connection.borrow())
        }
    }
}

impl CmdServiceSQL {
    
    pub fn build_cmd_service(cntn: Option<Connection>) -> Result<CmdServiceSQL, CmdError> {
        let connection = match cntn {
            Some(cn) => { cn }
            None => {
                let mut home = home::home_dir().expect("Could not find home dir");
                home.push(".cmd");
                Connection::open(home.join("cmdb"))?
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

        Ok(CmdServiceSQL { connection: Rc::new(connection) })
    }

    pub fn migrate_cvs(
        &mut self,
        mut saved_file_mgr: impl FileManager,
        mut used_file_mgr: impl FileManager
    ) -> Result<(), CmdError> {
        let saved_cmd_service = build_cmd_csv_service(&mut saved_file_mgr, true)?;
        let used_cmd_service = build_cmd_csv_service(&mut used_file_mgr, true)?;

        let mut cache: HashMap<String, CmdRecord> = HashMap::new();

        saved_cmd_service.for_each_record(|mut cmd| {
            let c = {
                match cache.get_mut(&cmd.command.clone()) {
                    Some(cmd1) => {
                        log_debug!("In cache! {}, {}", cmd1.command, cmd.used_times);
                        cmd.used_times += cmd1.used_times;
                        log_debug!("After update! {}, {}", cmd1.command, cmd.used_times);
                        cmd
                    }
                    None => cmd,
                }
            };
            log_debug!("Saving! {}, {}", c.command, c.used_times);
            let _ = &self.insert_command(c.clone()).expect("Could not save record");
            cache.insert(c.command.clone(), c);
        });

        let mut cache: HashMap<String, CmdRecord> = HashMap::new();

        used_cmd_service.for_each_record(|mut cmd| {
            let c = {
                match cache.get_mut(&cmd.command.clone()) {
                    Some(cmd1) => {
                        cmd.used_times += cmd1.used_times;
                        cmd
                    }
                    None => cmd,
                }
            };

            log_debug!("Saving! {}, {}", c.command, c.used_times);
            let _ = &self.insert_command(c.clone()).expect("Could not save record");
            cache.insert(c.command.clone(), c);
        });

        Ok(())
    }
}

impl CmdService for CmdServiceSQL {
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
        filter: SearchFilters
    ) -> Vec<crate::models::cmd_record::CmdRecord> {
        let mut sql = Vec::new();
        sql.push(String::from("SELECT id, command, used_times FROM cmd"));

        if filter.command.is_some() || filter.used == true {
            sql.push(String::from(" WHERE "));
        }
        if let Some(name) = &filter.command {
            let s = format!("command LIKE {}", format!("'%{}%'", name));
            sql.push(s);
        }

        if filter.command.is_some() && filter.used == true {
            sql.push(String::from(" AND "));
        }

        if filter.used {
            sql.push(String::from("used_times > 0"));
        }

        log_debug!("SQL: {}", sql.join(" "));
        self.connection
            .prepare(sql.join(" ").as_str())
            .expect("Could not build the SQL statement")
            .query_map([], |row| Ok(CmdRecord::from(row.clone())))
            .expect("Could not map row")
            .map(|el| el.unwrap().clone())
            .collect()
    }

    fn get_file_name(self: &Self) -> String {
        todo!()
    }

    fn clear_commands(self: &Self) -> Result<(), crate::error::CmdError> {
        todo!()
    }

    fn insert_command(self: &mut Self, cmd: CmdRecord) -> Result<(), CmdError> {
        self.connection.execute(
            "INSERT  OR REPLACE INTO cmd (command, used_times) VALUES (?1, ?2)",
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
                        "There are {} used commands that have been used {} times",
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

    fn delete_command(self: &mut Self, command: CmdRecord) -> Result<(), CmdError> {
        self.connection.execute("DELETE FROM cmd where id = ?1", params![&command.id])?;

        Ok(())
    }
}