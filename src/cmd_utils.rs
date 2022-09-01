use dialoguer::Input;
use dialoguer::{ theme::ColorfulTheme };

pub fn get_input(prompt: Option<&str>) -> String {
    let mut note: String = Input::with_theme(&ColorfulTheme::default())
        .allow_empty(false)
        .with_prompt(prompt.unwrap_or(">"))
        .interact()
        .expect("Could not read the input");

    note = note.replace("\n", "");
    note
}