use std::collections::HashSet;
use crate::{ *, error::CmdError, models::cmd_record::CmdRecord };

pub fn get_command(pattern: &Option<String>, deps: &mut Deps) -> Result<(), CmdError> {
    if deps.args.get_command.is_none() && deps.args.command.is_none() {
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
            return return_selection(deps, options, commands, parsed);
        }
    }

    let parsed = match pattern {
        Some(p) => p.clone(),
        None => {
            let cmd = deps.input.get_input(Some("Search for a command".to_string()));
            cmd
        }
    };

    let (commands, options) = get_matches(parsed.clone(), deps)?;
    return_selection(deps, options, commands, parsed)
}

pub fn get_commands_list(
    commands: &Vec<CmdRecord>,
    filter: impl FnMut(&&CmdRecord) -> bool
) -> Vec<models::cmd_record::CmdRecord> {
    let mut result = commands
        .clone()
        .iter()
        .filter(filter)
        .map(|x| x.clone())
        .collect::<Vec<_>>();

    result.sort_by(|a, b| a.used_times.cmp(&b.used_times));
    return result;
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

fn return_selection(
    deps: &mut Deps,
    options: Vec<String>,
    commands: Vec<CmdRecord>,
    parsed: String
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
    let parsed_cmd = String::from(selected_cmd);

    let selected_record = &mut deps.controller
        .get_used_commands(parsed.clone())
        .get(selected_cmd_index)
        .unwrap_or_else(|| commands.get(selected_cmd_index).unwrap())
        .to_owned();

    let git = CmdExtensionGit {};

    if git.handle_if_match(&Some(selected_record.clone()), deps).is_some() {
        return Ok(());
    }

    let (final_cmd, final_count) = fill_placeholders(
        selected_cmd,
        deps,
        parsed_cmd,
        selected_record
    );

    log_debug!("Executing '{}'!", &final_cmd);

    let result = deps.os.execute_command(&final_cmd);

    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = selected_record.to_owned().clone();
            // new_cmd.update_command(&final_cmd);
            let alias = if final_cmd.eq(&selected_record.command) { None } else { Some(final_cmd) };
            new_cmd.used_times = final_count;
            let used_command = &mut deps.controller.add_used_command(new_cmd.to_owned(), alias);
            match used_command {
                Ok(_) => Ok(()),
                Err(err) => Err(err.to_owned().into()),
            }
        }
        Err(err) => {
            log_info!("Finalized with an error: {:?}", err);
            Err(err.into())
        }
    }
}

pub fn fill_placeholders(
    selected_cmd: &String,
    deps: &mut Deps,
    mut parsed_cmd: String,
    selected_record: &mut CmdRecord
) -> (String, usize) {
    let (final_cmd, final_count) = match selected_cmd.contains("{}") {
        true => {
            log_debug!("Fill placeholders");

            let rest = selected_cmd.matches("{}");
            let count = rest.count();

            println!("Fill {} params...", count);

            for i in 0..count {
                let note = deps.input.get_input(Some(format!("Set param No.{}:", i + 1)));
                parsed_cmd = parsed_cmd.replacen("{}", &note, 1);
                // print!("{esc}c", esc = 27 as char);
                // print!("\r");
                // print!("{}", 8u8 as char);
            }

            (parsed_cmd, 0)
        }
        false => (parsed_cmd, selected_record.used_times),
    };
    (final_cmd, final_count)
}