use std::process::Command;

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
    return output;
}