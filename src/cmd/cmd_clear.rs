use std::{cell::RefCell, rc::Rc};

use crate::{ Deps, traits::inputable::Inputable };

pub struct ClearHandler {
    deps: Rc<RefCell<Deps>>
}

impl ClearHandler {
    pub fn new(deps: Rc<RefCell<Deps>>) -> Self {
        Self {
            deps:deps
        }
    }

    fn get_input(&self) -> Rc<dyn Inputable> {
        return Rc::clone(&self.deps.as_ref().borrow().input);
    }


    pub fn clear(&self) {
        let controller = &self.deps.as_ref().borrow_mut().controller;
        let response = self.get_input().confirm(
            format!(
                "Are you sure you want to delete {} and {}?",
                controller.get_all_file_path(),
                controller.get_used_file_path()
            )
        );
    
        if response == true {
            controller.clear_files();
        }
    }
}