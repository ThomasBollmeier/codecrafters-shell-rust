#[allow(unused_imports)]
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
    if input.starts_with("exit") {
        return Ok(ExecResult::Exit(0));
    }

    if input.starts_with("echo ") {
        print!("{}", &input[5..]);
        return Ok(ExecResult::Continue);
    }

    Err(anyhow!(format!("{}: command not found", input.trim())))
}
