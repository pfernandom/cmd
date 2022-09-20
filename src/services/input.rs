use dialoguer::{ theme::ColorfulTheme, Select, Input, Confirm };

use crate::traits::inputable::Inputable;

pub struct InputManager {}

impl Inputable for InputManager {
    fn get_input(self: &Self, prompt: Option<String>) -> String {
        let mut note: String = Input::with_theme(&ColorfulTheme::default())
            .allow_empty(false)
            .with_prompt(prompt.unwrap_or(">".into()))
            .interact()
            .expect("Could not read the input");

        note = note.replace("\n", "");
        note
    }

    fn select_option(
        self: &Self,
        options: &Vec<String>,
        maybe_prompt: Option<String>
    ) -> Option<usize> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(match maybe_prompt {
                Some(text) => text,
                None => "Pick a command".to_string(),
            })
            .items(&options[..])
            .default(0)
            .interact_opt()
            .expect("did not get params")
    }

    fn confirm(self: &Self, prompt: String) -> bool {
        let result = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(false)
            .interact()
            .unwrap_or_default();
        result
    }
}