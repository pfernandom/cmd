use std::collections::HashSet;

use crate::{
    Deps,
    error::CmdError,
    models::cmd_record::CmdRecord,
    services::controller::Controller,
    log_debug,
    log_info,
    log_error,
};

use super::cmd_get::get_commands_list;

pub fn delete_command(deps: &mut Deps) -> Result<(), CmdError> {
    let default_get_opts = vec!["Get recently used", "Get all"];
    let default_selection = deps.input.select_option(
        &default_get_opts
            .iter()
            .map(|s| s.to_string())
            .collect(),
        Some("Pick a choice".to_string())
    );

    if let Some(i) = default_selection {
        let parsed = "".to_string();
        let (commands, options) = match i {
            0 => get_last_used(parsed.clone(), deps)?,
            _ => get_matches(parsed.clone(), deps)?,
        };
        log_debug!("Commands: {:?}", commands);

        return delete_selection(deps, options, commands, parsed);
    }
    Ok(())
}

fn get_matches(parsed: String, deps: &mut Deps) -> Result<(Vec<CmdRecord>, Vec<String>), CmdError> {
    let mem: &mut Controller = &mut deps.controller;

    let mut commands = mem.get_used_commands(parsed.clone()).clone();

    let set: HashSet<_> = commands
        .clone()
        .drain(..)
        .map(|e| e.command)
        .collect::<HashSet<_>>(); // dedup

    let non_used_commands = get_commands_list(
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
    parsed: String,
    deps: &mut Deps
) -> Result<(Vec<CmdRecord>, Vec<String>), CmdError> {
    let mem: &mut Controller = &mut deps.controller;

    let commands = get_commands_list(&mem.get_used_commands(parsed.clone()), |_x| true);

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
    deps: &mut Deps,
    options: Vec<String>,
    commands: Vec<CmdRecord>,
    _parsed: String
) -> Result<(), CmdError> {
    let selection = deps.input.select_option(&options, None);

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

    deps.controller.delete_record(selected_record);

    Ok(())
}