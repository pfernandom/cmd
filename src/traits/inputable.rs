use mockall::automock;

#[automock]
pub trait Inputable {
    fn get_input(self: &Self, prompt: Option<String>) -> String;

    fn select_option(self: &Self, options: &Vec<String>) -> Option<usize>;

    fn confirm(self: &Self, prompt: String) -> bool;
}