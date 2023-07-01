use std::{collections::HashSet, cell::RefCell, rc::Rc};

use crate::{
    Deps,
    error::CmdError,
    models::{cmd_record::CmdRecord, self},
    log_debug,
    log_info,
    log_error, traits::inputable::Inputable,
};

use super::cmd_get::GetHandler;

pub struct DeleteHandler {
    deps: Rc<RefCell<Deps>>,
    get_handler: GetHandler,
}

impl DeleteHandler {
    pub fn new(deps: Rc<RefCell<Deps>>) -> Self {
        Self {
            deps:Rc::clone(&deps),
            get_handler: GetHandler::new(deps)
        }
    } 

    fn get_input(&self) -> Rc<dyn Inputable> {
        return Rc::clone(&self.deps.as_ref().borrow().input);
    }


    pub fn get_commands_list(
        &self,
        commands: &Vec<CmdRecord>,
        filter: impl FnMut(&&CmdRecord) -> bool
    ) -> Vec<models::cmd_record::CmdRecord> {
        self.get_handler.get_commands_list(commands, filter)
    }

    pub fn delete_command(&mut self) -> Result<(), CmdError> {
        let default_get_opts = vec!["Get recently used", "Get all"];
        let default_selection = self.get_input().select_option(
            &default_get_opts
                .iter()
                .map(|s| s.to_string())
                .collect(),
            Some("Pick a choice".to_string())
        );
    
        if let Some(i) = default_selection {
            let parsed = "".to_string();
            let (commands, options) = match i {
                0 => self.get_last_used(parsed.clone())?,
                _ => self.get_matches(parsed.clone())?,
            };
            log_debug!("Commands: {:?}", commands);
    
            return self.delete_selection(options, commands, parsed);
        }
        Ok(())
    }
    
    fn get_matches(&mut self, parsed: String) -> Result<(Vec<CmdRecord>, Vec<String>), CmdError> {

        let mem = &mut self.deps.as_ref().borrow_mut().controller;
    
        let mut commands = mem.get_used_commands(parsed.clone()).clone();
    
        let set: HashSet<_> = commands
            .clone()
            .drain(..)
            .map(|e| e.command)
            .collect::<HashSet<_>>(); // dedup
    
        let non_used_commands =self.get_commands_list(
            &mem.get_commands(parsed.clone()),
            |x| !set.contains(&x.command)
        );
    
        commands.extend(non_used_commands);
    
        commands.sort_by(|cmd1, cmd2| cmd2.used_times.cmp(&cmd1.used_times));
    
        let options = commands
            .iter()
            .map(|cm| cm.command.clone())
            .collect::<Vec<_>>();
    
        if options.is_empty() {
            log::warn!("No command matched the pattern");
            std::process::exit(1);
        }
    
        return Ok((commands, options));
    }
    
    fn get_last_used(
        &mut self,
        parsed: String
    ) -> Result<(Vec<CmdRecord>, Vec<String>), CmdError> {
        let mem = &mut self.deps.as_ref().borrow_mut().controller;
    
        let commands = self.get_commands_list(&mem.get_used_commands(parsed.clone()), |_x| true);
    
        println!("Used commands: {:?}", commands);
    
        let options = commands
            .iter()
            .map(|cm| cm.command.clone())
            .collect::<Vec<_>>();
    
        if options.is_empty() {
            log::warn!("No command matched the pattern");
            std::process::exit(1);
        }
    
        return Ok((commands, options));
    }
    
    fn delete_selection(
        &mut self,
        options: Vec<String>,
        commands: Vec<CmdRecord>,
        _parsed: String
    ) -> Result<(), CmdError> {
        let selection = self.get_input().select_option(&options, None);
    
        let selected_cmd_index = match selection {
            Some(ind) => { ind }
            None => {
                log_info!("No command was selected. Exiting...");
                std::process::exit(1);
            }
        };
    
        let selected_cmd = match options.get(selected_cmd_index) {
            Some(res) => res,
            None => {
                log_error!("Could not get option {} for options {:?}", selected_cmd_index, options);
                std::process::exit(1);
            }
        };
        let _parsed_cmd = String::from(selected_cmd);
    
        let selected_record = commands
            .get(selected_cmd_index)
            .unwrap_or_else(|| commands.get(selected_cmd_index).unwrap())
            .to_owned();
    
        log_debug!("Record to delete: {:?}", selected_record);
    
        self.deps.as_ref().borrow_mut().controller.delete_record(selected_record);
    
        Ok(())
    }
}

