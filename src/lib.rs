use crate::arg_parse::ArgParser;
use crate::cmd::ExecResult;
use crate::read_line::read_line;
use anyhow::Result;
use std::collections::HashSet;
use std::env;
use std::fs::{read_dir, DirEntry};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

mod arg_parse;
mod cmd;
mod read_line;
mod redirect;

const PROMPT: &str = "$ ";

pub fn repl() -> i32 {
    let mut history: Vec<String> = vec![];
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        // Wait for user input
        let input = read_line(PROMPT, command_completion, &history);
        history.push(input.clone());

        match handle_input(&input, &mut history) {
            Ok(exec_result) => match exec_result {
                ExecResult::Exit(code) => return code,
                ExecResult::Continue => continue,
            },
            Err(msg) => eprintln!("{}", msg),
        }
    }
}

fn handle_input(input: &str, history: &mut Vec<String>) -> Result<ExecResult> {
    let commands = ArgParser::new().parse_args(input)?;
    cmd::run_commands(&commands, history)
}

fn get_executables() -> HashSet<String> {
    let mut ret = HashSet::new();

    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            for exec in get_executables_in_path(&path) {
                ret.insert(exec);
            }
        }
    } else {
        return ret;
    }

    ret
}

fn get_executables_in_path(path: &PathBuf) -> HashSet<String> {
    let mut ret = HashSet::new();

    if let Ok(entries) = read_dir(&path) {
        for entry in entries {
            if entry.is_err() {
                continue;
            }
            let entry = entry.unwrap();
            if is_executable(&entry) {
                let file_name = entry.file_name().into_string().unwrap();
                ret.insert(file_name);
            }
        }
    }

    ret
}

fn is_executable(entry: &DirEntry) -> bool {
    match entry.metadata() {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            permissions.mode() & 0o100 == 0o100
        }
        Err(_) => false,
    }
}

fn command_completion(prefix: &str) -> Vec<String> {
    let mut matched_commands = vec![];

    for cmd in cmd::get_builtin_commands().union(&get_executables()) {
        if cmd.starts_with(prefix) {
            matched_commands.push(cmd.clone());
        }
    }

    matched_commands.sort();
    matched_commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_input_pipe() {
        let input = "tail -f README.md | head -n 5";
        let result = handle_input(input, &mut vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_input_out_redir() {
        let input = "ls -l  >> /dev/null";
        let result = handle_input(input, &mut vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_input_error_redir() {
        let input = "ls -l nonexistent 2>> /dev/null";
        let result = handle_input(input, &mut vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_input_builtin_w_pipe() {
        let input = "echo pineapple-grape | wc";
        let result = handle_input(input, &mut vec![]);
        assert!(result.is_ok());
    }
}
