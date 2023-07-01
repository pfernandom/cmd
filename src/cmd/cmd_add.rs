use std::{rc::Rc, cell::RefCell};

use crate::{ error::CmdError, Deps, traits::{inputable::Inputable, os_service::OSService} };

pub struct AddHandler {
    deps: Rc<RefCell<Deps>>
}

impl AddHandler {
    pub fn new(deps: Rc<RefCell<Deps>>) -> Self {
        Self {
            deps:deps
        }
    } 

    fn get_os(&self) -> Rc<dyn OSService> {
        return Rc::clone(&self.deps.as_ref().borrow().os)
    }

    fn get_input(&self) -> Rc<dyn Inputable> {
        return Rc::clone(&self.deps.as_ref().borrow().input);
    }

    pub fn add_command(&mut self, _pattern: bool, execute: bool) -> Result<(), CmdError> {
        // let deps = &mut self.deps.as_ref().borrow_mut();
        let note = self.get_input().get_input(Some("Write your command".into()));
        print!("{}", note);
        let os  = self.get_os();

       

        let controller = &mut self.deps.as_ref().borrow_mut().controller;
        
        match execute {
            true => {
                os.execute_command(&note)?;
                controller.new_command(note)
            }
            false => { controller.new_command(note) }
        }
    }
}
