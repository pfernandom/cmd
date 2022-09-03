use regex::Regex;
use std::collections::HashSet;
use crate::{ * };
use crate::program::parse_programs;
use std::process::{ Command };

pub fn get_command(pattern: &Option<String>, deps: &Deps) -> bool {
    let mem: &ConfigMem = &deps.mem;
    let parsed = match pattern {
        Some(p) => p.clone(),
        None => {
            let cmd = deps.input.get_input(Some("Search for a command".to_string()));
            cmd
        }
    };

    log_debug!("{}", parsed);
    let re = Regex::new(&parsed).expect("could not parse regex");

    let mut commands = mem.get_commands().clone();

    let set: HashSet<_> = commands
        .clone()
        .drain(..)
        .collect::<HashSet<_>>(); // dedup

    let used_commands = mem
        .get_used_commands()
        .clone()
        .iter()
        .filter(|x| !set.contains(x))
        .map(|x| x.clone())
        .take(5)
        .collect::<Vec<_>>();

    commands.extend(used_commands);

    let mut options = commands
        .iter()
        .map(|cm| cm.command.clone())
        .collect::<Vec<_>>();

    log_debug!("opts:{:?}", options);

    options.retain(|option| re.is_match(option));

    if options.is_empty() {
        log::warn!("No command matched the pattern");
        return true;
    }

    let selection = deps.input.select_option(&options);

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

    let mut programs = parse_programs(&final_cmd);

    let result: Result<Vec<_>, _> = programs
        .iter_mut()
        .map(|p: &mut Command| p.spawn())
        .map(|p| p.expect("Could not start process").wait())
        .map(|x| x)
        .collect();

    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = seleted_record.clone();
            new_cmd.update_command(&final_cmd);
            new_cmd.used_times = final_count;
            match mem.add_used_command(new_cmd) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        Err(err) => {
            log_info!("Finalized with an error: {:?}", err);
            false
        }
    }

    // log_debug!("status:\n{}", result.status);
    // println!("stdout:\n{}", String::from_utf8_lossy(&result.stdout));
    // println!("stderr:\n{}", String::from_utf8_lossy(&result.stderr));

    // assert!(result.status.success());
}