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
            if i == 0 {
                return get_last_used("".to_string(), deps);
            } else {
                return get_matches("".to_string(), deps);
            }
        }
    }

    let parsed = match pattern {
        Some(p) => p.clone(),
        None => {
            let cmd = deps.input.get_input(Some("Search for a command".to_string()));
            cmd
        }
    };

    return get_matches(parsed, deps);
}

fn get_commands_list(
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

fn get_matches(parsed: String, deps: &mut Deps) -> Result<(), CmdError> {
    let mem: &mut Controller = &mut deps.mem;

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
        return Ok(());
    }

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
    let mut parsed_cmd = String::from(selected_cmd);

    let selected_record = &mem
        .get_used_commands(parsed.clone())
        .get(selected_cmd_index)
        .unwrap_or_else(|| commands.get(selected_cmd_index).unwrap())
        .to_owned();

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

    log_debug!("Executing '{}'!", &final_cmd);

    let result = deps.os.execute_command(&final_cmd);

    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = selected_record.to_owned().clone();
            new_cmd.update_command(&final_cmd);
            new_cmd.used_times = final_count;
            match mem.add_used_command(new_cmd.to_owned()) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.into()),
            }
        }
        Err(err) => {
            log_info!("Finalized with an error: {:?}", err);
            Err(err.into())
        }
    }
}

fn get_last_used(parsed: String, deps: &mut Deps) -> Result<(), CmdError> {
    let mem: &mut Controller = &mut deps.mem;

    let commands = get_commands_list(&mem.get_used_commands(parsed.clone()), |_x| true);

    println!("Used commands: {:?}", commands);

    let options = commands
        .iter()
        .map(|cm| cm.command.clone())
        .collect::<Vec<_>>();

    if options.is_empty() {
        log::warn!("No command matched the pattern");
        return Ok(());
    }

    let selection = deps.input.select_option(&options, None);

    let selected_cmd_index = match selection {
        Some(ind) => { ind }
        None => {
            log_info!("No command was selected. Exiting...");
            std::process::exit(1);
        }
    };

    let selected_cmd = options.get(selected_cmd_index).unwrap();
    let mut parsed_cmd = String::from(selected_cmd);

    let selected_record = &mem
        .get_used_commands(parsed.clone())
        .get(selected_cmd_index)
        .unwrap_or_else(|| commands.get(selected_cmd_index).unwrap())
        .to_owned();

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

    log_debug!("Executing '{}'!", &final_cmd);

    let result = deps.os.execute_command(&final_cmd);

    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = selected_record.to_owned().clone();
            new_cmd.update_command(&final_cmd);
            new_cmd.used_times = final_count;
            match mem.add_used_command(new_cmd.to_owned()) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.into()),
            }
        }
        Err(err) => {
            log_info!("Finalized with an error: {:?}", err);
            Err(err.into())
        }
    }
}