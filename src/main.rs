use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use anyhow::{anyhow, Result};

fn main() {
    std::process::exit(repl());
}

enum ExecResult {
    Exit(i32),
    Continue,
}

const PROMPT: &str = "$ ";

fn repl() -> i32 {
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        match handle_input(&input) {
            Ok(exec_result   ) => match exec_result {
                ExecResult::Exit(code) => return code,
                ExecResult::Continue => continue,
            },
            Err(msg)        => eprintln!("{}", msg),
        }
    }
}

fn handle_input(input: &str) -> Result<ExecResult> {
    let built_in_commands = HashSet::from([
        "exit",
        "echo",
        "type",
    ]);

    if input.starts_with("exit") {
        return Ok(ExecResult::Exit(0));
    }

    if input.starts_with("echo ") {
        print!("{}", &input[5..]);
        return Ok(ExecResult::Continue);
    }

    if input.starts_with("type ") {
        let command = input[5..].trim().to_string();
        return if built_in_commands.contains(&command.as_str()) {
            println!("{command} is a shell builtin");
            Ok(ExecResult::Continue)
        } else {
            find_command_in_path(&command).map(|cmd_path| {
                println!("{command} is {cmd_path}");
                ExecResult::Continue
            })
        }
    }

    Err(anyhow!("{}: command not found", input.trim()))
}

fn find_command_in_path(command: &str) -> Result<String> {
    let path_var = env::var("PATH")?;
    for path in env::split_paths(&path_var) {
        let mut exec_path = path;
        exec_path.push(command);
        match exec_path.try_exists() {
            Ok(true) => {
                match exec_path.to_str() {
                    Some(exec_path) => return Ok(exec_path.to_string()),
                    None => continue,
                }
            },
            _ => continue,
        }
    }

    Err(anyhow!("{command}: not found"))
}
