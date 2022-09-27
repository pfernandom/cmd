use std::{ cell::RefCell, rc::Rc };

pub struct MockOpts<'a> {
    pub selected_record: Box<dyn (Fn(&Vec<String>) -> usize) + 'a>,
    pub captures: Captures,
}

// type OptionSelect = |&Vec<String>| -> usize;

pub fn RcMut<T>(p: T) -> MutRef<T> {
    Rc::new(RefCell::new(p))
}

impl<'a> MockOpts<'a> {
    pub fn new() -> Rc<RefCell<MockOpts<'a>>> {
        RcMut(MockOpts::default())
    }
    pub fn capture_options_for_command(self: &mut Self, options: Vec<String>) {
        self.captures.options_for_command = options;
    }
    pub fn get_selected_record(self: &Self, vec: &Vec<String>) -> usize {
        let sr = &self.selected_record;
        sr(vec)
    }
    pub fn from(selected_record: impl 'a + Fn(&Vec<String>) -> usize) -> MutRef<MockOpts<'a>> {
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

pub type MutRef<T> = Rc<RefCell<T>>;

#[derive(Default)]
pub struct Captures {
    pub options_for_command: Vec<String>,
}