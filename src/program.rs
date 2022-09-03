use std::process::Command;

use crate::log_debug;

pub(crate) fn parse_program(program: &str) -> Command {
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
        output.arg(arg);
    }
    log_debug!("Program: {:?}", output);
    return output;
}

pub fn parse_programs(program: &str) -> Vec<Command> {
    program
        .split("&&")
        .map(|cmd| cmd.trim())
        .map(|cmd| parse_program(cmd))
        .collect::<Vec<_>>()
}