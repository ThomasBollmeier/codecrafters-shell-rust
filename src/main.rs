#[allow(unused_imports)]
use std::io::{self, Write};
use anyhow::{anyhow, Result};

fn main() {
    std::process::exit(repl());
}

enum ExecResult {
    Exit(i32)
}

fn repl() -> i32 {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        match handle_input(&input) {
            Ok(exec_result   ) => return match exec_result {
                ExecResult::Exit(code) => code,
            },
            Err(msg)        => eprintln!("{}", msg),
        }
    }
}

fn handle_input(input: &str) -> Result<ExecResult> {
    if input.starts_with("exit") {
        return Ok(ExecResult::Exit(0));
    }

    Err(anyhow!(format!("{}: command not found", input.trim())))
}
