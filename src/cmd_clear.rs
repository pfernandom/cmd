use crate::cmd_config_mem::ConfigMem;
use dialoguer::{ Confirm, theme::ColorfulTheme };

pub fn clear(mem: &ConfigMem) {
    let response = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(
            &format!(
                "Are you sure you want to delete {} and {}?",
                &mem.commands_path,
                &mem.used_commands_path
            )
        )
        .interact();

    if let Ok(i) = response {
        if i == true {
            std::fs::remove_file(&mem.commands_path).expect("Could not remove file");
            std::fs::remove_file(&mem.used_commands_path).expect("Could not remove file");
        }
    }
}