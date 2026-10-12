use std::io::{self, BufRead};
use std::process::Command;

pub(crate) fn read_stdin_line() -> Result<String, String> {
    let mut input = String::new();
    io::stdin()
        .lock()
        .read_line(&mut input)
        .map_err(|error| format!("failed to read stdin: {error}"))?;
    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }
    Ok(input)
}

pub(crate) fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd
}
