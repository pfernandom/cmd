use dialoguer::{ theme::ColorfulTheme, Select };
use env_logger::Builder;
use log::LevelFilter;
use std::default::Default;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;

use clap::Parser;

mod args;
mod logging;
mod program;
mod tests;

use args::{ Cli, Commands };
use program::parse_program;

fn main() {
    let mut home = home::home_dir().expect("Could not find home dir");
    log_warn!("Saving settings to: {}/.cmd", home.to_str().expect("could not parse home path"));

    let args = Cli::parse();

    home.push(".cmd");
    let mut dir_builder = std::fs::DirBuilder::new();
    dir_builder.recursive(true).create(&home).expect("Could not create config folder");

    let t = home.join("cmds.txt");
    let commands_path = t.to_str().expect("could not convert path to string");

    OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(commands_path)
        .expect("could not create or open config file");

    let level = match args.verbose {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    Builder::new().filter_level(level).init();

    match args.command {
        Commands::Get { pattern } => {
            use regex::Regex;
            let re = Regex::new(&pattern).expect("could not parse regex");

            let file = File::open(commands_path).expect("Could not read file");
            let reader = BufReader::new(file);

            let mut options = reader
                .lines()
                .enumerate()
                .map(|x| x.1)
                .map(|x| x.expect("could not read line"))
                .collect::<Vec<_>>();

            options.retain(|option| re.is_match(&option));

            if options.is_empty() {
                log::warn!("No command matched the pattern");
                return;
            }

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Pick a command")
                .default(0)
                .items(&options[..])
                .interact()
                .expect("did not get params");

            let selected_cmd = options.get(selection).unwrap();
            let mut parsed_cmd = String::from(selected_cmd);

            if selected_cmd.contains("{}") {
                log_debug!("Fill placeholders");

                let rest = selected_cmd.matches("{}");
                let count = rest.count();

                println!("Fill {} params...", count);

                for _ in 0..count {
                    let mut note = String::new();

                    log_info!("Set param:");
                    std::io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut note).expect("could not read input");
                    note = note.replace("\n", "");
                    parsed_cmd = parsed_cmd.replacen("{}", &note, 1);
                }

                log_info!("Parsed cmd: {}", parsed_cmd);

                // selected_cmd.matches("{}");
            }

            log_debug!("Executing '{}'!", selected_cmd);

            let mut program = parse_program(selected_cmd);
            let result = program.output().expect("failed to execute process");

            log_debug!("status:\n{}", result.status);
            println!("stdout:\n{}", String::from_utf8_lossy(&result.stdout));
            println!("stderr:\n{}", String::from_utf8_lossy(&result.stderr));

            assert!(result.status.success());
        }
        Commands::Add { pattern, execute } => {
            log::info!("Write your command");
            let mut note = String::new();

            std::io::stdout().flush().unwrap();
            match io::stdin().read_line(&mut note) {
                Ok(_) => {
                    print!("{}", note);

                    if !pattern || execute {
                        let mut program = parse_program(&note);
                        let result = program.output().expect("failed to execute process");

                        println!("status: {}", result.status);
                        println!("stdout:\n{}", String::from_utf8_lossy(&result.stdout));
                        println!("stderr: {}", String::from_utf8_lossy(&result.stderr));

                        assert!(result.status.success());
                    }

                    let mut file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(commands_path)
                        .expect(&format!("could not open file to write: {}", commands_path));

                    file.write_all(note.as_bytes()).expect("Could not write");
                    log::info!("SUCCESS: Command added");
                }
                Err(_) => { print!("None") }
            }
        }
    }

    // Continued program logic goes here...
}