use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::process::Command;
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
        "cd".to_string(),
        "echo".to_string(),
        "exit".to_string(),
        "pwd".to_string(),
        "type".to_string(),
    ]);

    let (command, args) = parse_input(input)?;

    match command.as_str() {
        "cd" => change_directory(&args),
        "echo" => {
            print!("{}", &input[5..]);
            Ok(ExecResult::Continue)
        }
        "exit" => {
            let code = args
                .get(0)
                .unwrap_or(&"0".to_string())
                .parse::<i32>()
                .unwrap_or(1);
            Ok(ExecResult::Exit(code))
        },
        "pwd" => print_current_dir(),
        "type" => {
            let cmd = args
                .get(0)
                .ok_or(anyhow!("Missing command argument"))?;
            if built_in_commands.contains(cmd) {
                println!("{cmd} is a shell builtin");
                Ok(ExecResult::Continue)
            } else {
                find_command_in_path(cmd).map(|cmd_path| {
                    println!("{cmd} is {cmd_path}");
                    ExecResult::Continue
                })
            }
        }
        other => find_command_in_path(other).map(|_| {
            let output = Command::new(other)
                .args(args)
                .output();
            match output {
                Ok(output) =>
                    print!("{}", String::from_utf8_lossy(&output.stdout)),
                Err(err) => eprint!("{}", err),
            }
            ExecResult::Continue
        })
    }
}

fn print_current_dir() -> Result<ExecResult> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display());
    Ok(ExecResult::Continue)
}

fn change_directory(args: &Vec<String>) -> Result<ExecResult> {
    let dir = match args.len() {
        0 => &get_home_dir()?,
        1 => if args[0] != "~".to_string() {
            &args[0]
        } else {
            &get_home_dir()?
        },
        _ => return Err(anyhow!("cd allows no more then one argument"))
    };

    match env::set_current_dir(dir) {
        Ok(_) => Ok(ExecResult::Continue),
        Err(_) => Err(anyhow!("cd: {}: No such file or directory", &args[0])),
    }
}

fn get_home_dir() -> Result<String> {
    env::var("HOME").map_err(|_| anyhow!("$HOME is not set"))
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

fn parse_input(input: &str) -> Result<(String, Vec<String>)> {
    let mut parts = input.trim().split_whitespace();
    let command = parts.next().ok_or(anyhow!("input is empty"))?.to_string();
    let args = parts
        .map(|s| s.to_string())
        .collect();
    Ok((command, args))
}
