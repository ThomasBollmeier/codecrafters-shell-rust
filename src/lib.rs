use crate::arg_parse::ArgParser;
use crate::redirect::{FileOpenMode, Output, RedirectionInfo};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::process::Command;

mod arg_parse;
mod redirect;

enum ExecResult {
    Exit(i32),
    Continue,
}

const PROMPT: &str = "$ ";

pub fn repl() -> i32 {
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

    let (command, args) = ArgParser::new().parse_args(input)?;
    let (args, redirection_info) = check_for_redirections(&args);

    let mut output = redirection_info.get_output();
    output.open()?;

    let mut error_output = redirection_info.get_error_output();
    error_output.open()?;

    let result = match command.as_str() {
        "cd" => change_directory(&args),
        "echo" => {
            for arg in args {
                output.print(&format!("{arg} "));
            }
            output.println("");
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
        "pwd" => print_current_dir(&mut output),
        "type" => {
            let cmd = args
                .get(0)
                .ok_or(anyhow!("Missing command argument"))?;
            if built_in_commands.contains(cmd) {
                output.println(&format!("{cmd} is a shell builtin"));
                Ok(ExecResult::Continue)
            } else {
                find_command_in_path(cmd).map(|cmd_path| {
                    output.println(&format!("{cmd} is {cmd_path}"));
                    ExecResult::Continue
                })
            }
        }
        other => find_command_in_path(other).map(|_| {
            let out = Command::new(other)
                .args(args)
                .output();
            match out {
                Ok(out) => {
                    output.print(&format!("{}", String::from_utf8_lossy(&out.stdout)));
                    error_output.print(&format!("{}", String::from_utf8_lossy(&out.stderr)));
                }
                Err(err) => error_output.print(&format!("{}", err)),
            }
            ExecResult::Continue
        })
    };

    output.close();

    result
}

fn print_current_dir(output: &mut Box<dyn Output>) -> Result<ExecResult> {
    let current_dir = env::current_dir()?;
    output.println(&format!("{}", current_dir.display()));
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

fn check_for_redirections(args: &Vec<String>) -> (Vec<String>, RedirectionInfo) {
    let mut redirection_info = RedirectionInfo::new();
    let mut new_args = Vec::new();
    let num_args = args.len();
    if num_args == 0 {
        return (args.clone(), redirection_info);
    }

    let mut i = 0;

    while i < num_args {
        let arg = &args[i];
        if i == num_args - 1 {
            new_args.push(arg.to_string());
            i += 1;
            continue;
        }
        match arg.as_str() {
            ">" | "1>" => {
                let file_path = args[i+1].clone();
                redirection_info.redirect_stdout(file_path, FileOpenMode::Create);
                i += 2;
            }
            "2>" => {
                let file_path = args[i+1].clone();
                redirection_info.redirect_stderr(file_path, FileOpenMode::Create);
                i += 2;
            }
            ">>" | "1>>" => {
                let file_path = args[i+1].clone();
                redirection_info.redirect_stdout(file_path, FileOpenMode::Append);
                i += 2;
            }
            "2>>" => {
                let file_path = args[i+1].clone();
                redirection_info.redirect_stderr(file_path, FileOpenMode::Append);
                i += 2;
            }
            _ => {
                new_args.push(arg.clone());
                i += 1;
            },
        }
    }

    (new_args, redirection_info)
}


