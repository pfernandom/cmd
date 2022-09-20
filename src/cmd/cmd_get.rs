use regex::Regex;
use std::collections::HashSet;
use crate::{ *, error::CmdError, cmd_csv::CmdRecord };

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
                return get_last_used(".*".to_string(), deps);
            } else {
                return get_matches(".*".to_string(), deps);
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

fn get_used_commands_list(
    mem: &ConfigMem,
    filter: impl FnMut(&&CmdRecord) -> bool
) -> Vec<cmd_csv::CmdRecord> {
    let mut result = mem
        .get_used_commands()
        .clone()
        .iter()
        .filter(filter)
        .map(|x| x.clone())
        .collect::<Vec<_>>();

    // result.sort_by(|a, b| a.used_times.cmp(&b.used_times));
    return result;
}

fn get_matches(parsed: String, deps: &mut Deps) -> Result<(), CmdError> {
    let mem: &mut ConfigMem = &mut deps.mem;

    log_debug!("{}", parsed);
    let re = Regex::new(&parsed).expect("could not parse regex");

    let mut commands = mem.get_commands().clone();
    println!("All commands: {:?}", commands);

    let set: HashSet<_> = commands
        .clone()
        .drain(..)
        .map(|e| e.command)
        .collect::<HashSet<_>>(); // dedup

    let used_commands = get_used_commands_list(mem, |x| !set.contains(&x.command));

    commands.extend(used_commands);

    let mut options = commands
        .iter()
        .map(|cm| cm.command.clone())
        .collect::<Vec<_>>();

    log_debug!("opts:{:?}", options);

    options.retain(|option| re.is_match(option));

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

    let seleted_record = mem
        .get_used_commands()
        .get(selected_cmd_index)
        .unwrap_or_else(|| commands.get(selected_cmd_index).unwrap());

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
        false => (parsed_cmd, seleted_record.used_times),
    };

    log_debug!("Executing '{}'!", &final_cmd);

    let result = deps.os.execute_command(&final_cmd);

    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = seleted_record.clone();
            new_cmd.update_command(&final_cmd);
            new_cmd.used_times = final_count;
            match mem.add_used_command(new_cmd) {
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
    let mem: &mut ConfigMem = &mut deps.mem;

    log_debug!("{}", parsed);
    let re = Regex::new(&parsed).expect("could not parse regex");

    let commands = get_used_commands_list(mem, |_x| true);

    println!("Used commands: {:?}", commands);

    let mut options = commands
        .iter()
        .map(|cm| cm.command.clone())
        .collect::<Vec<_>>();

    log_debug!("opts:{:?}", options);

    options.retain(|option| re.is_match(option));

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

    let seleted_record = mem
        .get_used_commands()
        .get(selected_cmd_index)
        .unwrap_or_else(|| commands.get(selected_cmd_index).unwrap());

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
        false => (parsed_cmd, seleted_record.used_times),
    };

    log_debug!("Executing '{}'!", &final_cmd);

    let result = deps.os.execute_command(&final_cmd);

    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = seleted_record.clone();
            new_cmd.update_command(&final_cmd);
            new_cmd.used_times = final_count;
            match mem.add_used_command(new_cmd) {
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