use std::{ ffi::OsString, io::BufRead };
use std::io;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::fs::OpenOptions;
use std::io::BufReader;
use dialoguer::{ theme::ColorfulTheme, Select };

use clap::{ Args, Parser, Subcommand };

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[clap(name = "git")]
#[clap(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Clones repos
    #[clap(arg_required_else_help = true)]
    Clone {
        /// The remote to clone
        #[clap(value_parser)]
        remote: String,
    },
    /// pushes things
    #[clap(arg_required_else_help = true)]
    Push {
        /// The remote to target
        #[clap(value_parser)]
        remote: String,
    },
    /// adds things
    Add {
        // /// Stuff to add
        #[clap(name = "pattern", long, short, parse(from_flag))]
        pattern: bool,

        #[clap(name = "execute", long, short, parse(from_flag))]
        execute: bool,
    },
    Get {
        #[clap(value_parser)]
        pattern: String,
    },
    Stash(Stash),
    #[clap(external_subcommand)] External(Vec<OsString>),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Stash {
    #[clap(subcommand)]
    command: Option<StashCommands>,

    #[clap(flatten)]
    push: StashPush,
}

#[derive(Debug, Subcommand)]
enum StashCommands {
    Push(StashPush),
    Pop {
        #[clap(value_parser)]
        stash: Option<String>,
    },
    Apply {
        #[clap(value_parser)]
        stash: Option<String>,
    },
}

#[derive(Debug, Args)]
struct StashPush {
    #[clap(short, long, value_parser)]
    message: Option<String>,
}

fn parse_program(program: &str) -> Command {
    let n = program.replace("\n", "");
    let parts = n.split(" ");
    let vec = parts.collect::<Vec<&str>>();
    let mut iter = vec.iter();

    let cmd = match iter.next() {
        Some(inner_cmd) => inner_cmd,
        None => "echo",
    };

    let mut output = Command::new(cmd);
    for arg in iter {
        print!("Arg:({})", arg);
        output.arg(arg);
    }
    print!("Program: {:?}", output.get_program());
    return output;
}

fn main() {
    let mut home = home::home_dir().expect("Could not find home dir");
    println!("Saving settings to: {}/.cmd", home.to_str().expect("could not parse home path"));

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

    match args.command {
        Commands::Get { pattern } => {
            println!("Getting {}", pattern);

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

            println!("Options: {:?}", options);

            options.retain(|option| re.is_match(&option));

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Pick a command")
                .default(0)
                .items(&options[..])
                .interact()
                .expect("did not get params");

            let selected_cmd = options.get(selection).unwrap();

            if selected_cmd.contains("{}") {
                println!("Fill placeholders");
            }

            println!("Executing '{}'!", selected_cmd);

            let mut program = parse_program(selected_cmd);
            let result = program.output().expect("failed to execute process");

            println!("status: {}", result.status);
            println!("stdout:\n{}", String::from_utf8_lossy(&result.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&result.stderr));

            assert!(result.status.success());
        }
        Commands::Clone { remote } => {
            println!("Cloning {}", remote);
        }
        Commands::Push { remote } => {
            println!("Pushing to {}", remote);
        }
        Commands::Add { pattern, execute } => {
            println!("Adding");
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
                }
                Err(_) => { print!("None") }
            }
        }
        Commands::Stash(stash) => {
            let stash_cmd = stash.command.unwrap_or(StashCommands::Push(stash.push));
            match stash_cmd {
                StashCommands::Push(push) => {
                    println!("Pushing {:?}", push);
                }
                StashCommands::Pop { stash } => {
                    println!("Popping {:?}", stash);
                }
                StashCommands::Apply { stash } => {
                    println!("Applying {:?}", stash);
                }
            }
        }
        Commands::External(args) => {
            println!("Calling out to {:?} with {:?}", &args[0], &args[1..]);
        }
    }

    // Continued program logic goes here...
}