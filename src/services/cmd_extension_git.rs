use std::{ env, str::{ from_utf8 }, ops::Range };

use crate::{
    traits::cmd_extension::CmdExtension,
    log_info,
    log_debug,
    models::cmd_record::CmdRecord,
    Deps,
    error::CmdError,
};
use git2::{ Repository, BranchType };
use regex::bytes::Regex;

pub struct CmdExtensionGit {}

impl CmdExtension for CmdExtensionGit {
    fn handle_if_match(
        self,
        pattern: &Option<CmdRecord>,
        deps: &mut crate::Deps
    ) -> Option<Result<(), crate::error::CmdError>> {
        if let Some(cmd_record) = pattern {
            let cmd = cmd_record.command.clone();
            match cmd.as_str() {
                "git checkout {}" => {
                    if let Some(value) = fill_with_branch(deps, cmd, cmd_record) {
                        return value;
                    }
                    return None;
                }
                "git checkout {} && git pull --rebase && git checkout {}" => {
                    if let Some(value) = fill_with_branch(deps, cmd, cmd_record) {
                        return value;
                    }
                    return None;
                }
                "git checkout {} && git pull --rebase && git checkout {} && git merge {}" => {
                    if let Some(value) = fill_with_branch(deps, cmd, cmd_record) {
                        return value;
                    }
                    return None;
                }
                _ => {}
            }
        }
        None
    }
}

fn fill_with_branch(
    deps: &mut crate::Deps,
    cmd: String,
    cmd_record: &CmdRecord
) -> Option<Option<Result<(), crate::error::CmdError>>> {
    let repo = match
        Repository::open(env::current_dir().expect("Cannot get the current directory"))
    {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
    let _branches = get_branches(&repo).expect("cannot retrieve branches");
    // let selected_branch = deps.input
    //     .select_option(&branches, Some("Select a branch to checkout".to_string()))
    //     .unwrap();
    // let note = branches.get(selected_branch).unwrap();
    //let final_cmd = cmd.replacen("{}", &note, 1);
    let parsed_cmd = String::from(&cmd);
    let (final_cmd, _count) = fill_placeholders(&cmd, deps, parsed_cmd, &mut cmd_record.clone());

    log_debug!("Executing '{}'!", &final_cmd);
    // let result = deps.os.execute_command(&final_cmd);
    let result: Result<bool, CmdError> = Ok(true);
    let _used_commands = deps.controller.get_used_commands(cmd.clone());
    match result {
        Ok(_) => {
            log_info!("Finalized successfully");
            let mut new_cmd = cmd_record.to_owned().clone();
            // new_cmd.update_command(&final_cmd);
            let alias = if final_cmd.eq(&cmd_record.command) { None } else { Some(final_cmd) };
            new_cmd.used_times = cmd_record.used_times + 1;
            let used_command = &mut deps.controller.add_used_command(new_cmd.to_owned(), alias);
            if let Ok(_cmd) = used_command {
                return Some(Some(Ok(())));
            }
        }
        Err(err) => {
            log_info!("Finalized with an error: {:?}", err);
            return Some(None);
        }
    }
    None
}

fn get_branches(repo: &Repository) -> Result<Vec<String>, crate::error::CmdError> {
    let branches = repo.branches(Some(BranchType::Local))?;

    let all = branches
        .map(|branch| branch.expect("Open branch"))
        .map(|b| b.0)
        .map(|b| {
            match b.name() {
                Ok(name) =>
                    match name {
                        Some(name) => name.to_string(),
                        None => String::new(),
                    }
                Err(_) => String::new(),
            }
        })
        .collect::<Vec<String>>();

    log_info!("branches: {:?}", all);

    Ok(all)
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

            let repo = match
                Repository::open(env::current_dir().expect("Cannot get the current directory"))
            {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            };
            let branches = get_branches(&repo).expect("cannot retrieve branches");

            let rest = selected_cmd.matches("{}");
            let count = rest.count();

            println!("Fill {} params...", count);

            // let re = Regex::new(
            //     r"git checkout (?P<b1>\{\}).+git checkout(?P<b2>\{\}).+git merge (?P<b3>\{\})"
            // ).expect("Could not build regex");

            let re = Regex::new(r"git (checkout|merge) (\{\})").expect("Could not build regex");

            let names = re
                .captures_iter(selected_cmd.as_bytes())
                .map(|c| { c.get(1).map_or(&b""[..], |m| m.as_bytes()) })
                .map(|c| from_utf8(c).unwrap().to_string())
                .collect::<Vec<String>>();

            log_info!("Names: {:?}", names);

            let subcommands = selected_cmd.split("&&").clone();
            let windows: &Vec<String> = &subcommands
                .map(|subcmd| {
                    let indeces: &Vec<usize> = &subcmd
                        .match_indices("{}")
                        .map(|i| i.0)
                        //.map(|i| (0..i.0).map(|x| if x == i.0 { "v" } else { "-" }).collect::<String>())
                        .collect();
                    log_info!("find: {:?}", indeces);

                    let mut indeces2 = vec![0];
                    indeces2.extend(indeces);

                    let windows: String = indeces2
                        .windows(2)
                        .map(|w| String::from(subcmd[Range { start: w[0], end: w[1] }].trim()))
                        .collect::<Vec<String>>()
                        .join("");
                    windows
                })
                .collect();

            log_info!("slices: {:?}", windows);

            for i in 0..count {
                let selected_branch = deps.input
                    .select_option(
                        &branches,
                        Some(format!("Select a branch for {} ({})", windows[i], i).to_string())
                    )
                    .unwrap();
                let note = branches.get(selected_branch).unwrap();
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