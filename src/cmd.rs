use crate::arg_parse::CommandList;
use crate::redirect::{FileOpenMode, Output, RedirectionInfo};
use anyhow::{anyhow, Result};
use std::cmp::PartialEq;
use std::collections::HashSet;
use std::env;
use std::io::Write;
use std::process::{Child, ChildStdout, Command, Output as ProcessOutput, Stdio};

#[derive(Debug, PartialEq)]
pub enum ExecResult {
    Exit(i32),
    Continue,
}

#[derive(Debug)]
pub enum CommandOutput {
    Str(String),
    Out(ProcessOutput),
    ChildOut(ChildStdout),
}

pub fn get_builtin_commands() -> HashSet<String> {
    HashSet::from([
        "cd".to_string(),
        "echo".to_string(),
        "exit".to_string(),
        "pwd".to_string(),
        "type".to_string(),
        "history".to_string(),
    ])
}

pub fn run_commands(commands: &CommandList, history: &mut Vec<String>) -> Result<ExecResult> {
    let mut prev_output: Option<CommandOutput> = None;
    let mut exec_result = ExecResult::Continue;

    let last_idx = commands.len() - 1;

    for (idx, (command, args)) in commands.iter().enumerate() {
        let (result, output) = run_command(
            command,
            args,
            prev_output,
            last_idx > 0,
            idx == last_idx,
            history,
        )?;
        exec_result = result;
        prev_output = Some(output);
        if exec_result != ExecResult::Continue {
            break;
        }
    }

    Ok(exec_result)
}

fn run_command(
    command: &str,
    args: &Vec<String>,
    prev_output: Option<CommandOutput>,
    is_part_of_pipe: bool,
    is_last_in_pipe: bool,
    history: &mut Vec<String>,
) -> Result<(ExecResult, CommandOutput)> {
    let (args, redirection_info) = check_for_redirections(args);

    let mut output = redirection_info.get_output();
    output.open()?;

    let mut error_output = redirection_info.get_error_output();
    error_output.open()?;

    let mut out_str = String::new();
    let mut command_output_opt: Option<CommandOutput> = None;

    let piped = is_part_of_pipe && !is_last_in_pipe;

    let built_in_commands = get_builtin_commands();

    let exec_result = match command {
        "cd" => change_directory(&args),
        "echo" => {
            for arg in args {
                print_out(&mut output, &mut out_str, piped, &format!("{arg} "));
            }
            println_out(&mut output, &mut out_str, piped, "");
            Ok(ExecResult::Continue)
        }
        "exit" => {
            let code = args
                .get(0)
                .unwrap_or(&"0".to_string())
                .parse::<i32>()
                .unwrap_or(1);
            Ok(ExecResult::Exit(code))
        }
        "pwd" => print_current_dir(&mut output, &mut out_str, piped),
        "type" => {
            let cmd = args.get(0).ok_or(anyhow!("Missing command argument"))?;
            if built_in_commands.contains(cmd) {
                println_out(
                    &mut output,
                    &mut out_str,
                    piped,
                    &format!("{cmd} is a shell builtin"),
                );
                Ok(ExecResult::Continue)
            } else {
                find_command_in_path(cmd).map(|cmd_path| {
                    println_out(
                        &mut output,
                        &mut out_str,
                        piped,
                        &format!("{cmd} is {cmd_path}"),
                    );
                    ExecResult::Continue
                })
            }
        }
        "history" => run_history(args, history, &mut output, &mut out_str, piped),
        other => find_command_in_path(other).map(|_| {
            let out_result =
                run_process(other, &args, prev_output, is_part_of_pipe, is_last_in_pipe);
            match out_result {
                Ok(cmd_out) => {
                    match &cmd_out {
                        CommandOutput::Out(out) => {
                            output.print(&format!("{}", String::from_utf8_lossy(&out.stdout)));
                            error_output
                                .print(&format!("{}", String::from_utf8_lossy(&out.stderr)));
                        }
                        _ => {}
                    }
                    command_output_opt = Some(cmd_out);
                }
                Err(err) => error_output.print(&format!("{}", err)),
            }
            ExecResult::Continue
        }),
    };

    output.close();

    match exec_result {
        Ok(exec_result) => {
            let command_output = match command_output_opt {
                Some(out) => out,
                None => CommandOutput::Str(out_str),
            };
            Ok((exec_result, command_output))
        }
        Err(err) => Err(err),
    }
}

fn run_history(
    args: Vec<String>, 
    history: &mut Vec<String>, 
    output: &mut Box<dyn Output>, 
    out_str: &mut String, 
    piped: bool) -> Result<ExecResult> {
    
    let entries = if args.is_empty() {
        history
    } else {
        match args[0].as_str() {
            "-r" => {
                if args.len() < 2 {
                    return Err(anyhow!("syntax: history -r <path_to_history_file>"));
                }
                load_history(&args[1], history)?;
                return Ok(ExecResult::Continue);
            }
            _ => {
                let num = args[0].parse::<usize>()?;
                if num > history.len() {
                    history
                } else {
                    &history[history.len() - num..]
                }        
            }
        }
    };
    for (idx, input) in entries.iter().enumerate() {
        println_out(
            output,
            out_str,
            piped,
            &format!("{:>5}  {}", idx + 1, input),
        );
    }
    Ok(ExecResult::Continue)
}

fn load_history(path: &str, history: &mut Vec<String>) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if !line.trim().is_empty() {
                    history.push(line);
                }
            }
            Err(err) => return Err(anyhow!("Error reading history file: {}", err)),
        }
    }
    Ok(())
}

fn run_process(
    command: &str,
    args: &Vec<String>,
    prev_output: Option<CommandOutput>,
    is_part_of_pipe: bool,
    is_last_in_pipe: bool,
) -> Result<CommandOutput> {
    let mut cmd = Command::new(command);
    cmd.args(args);

    if !is_part_of_pipe {
        let output = cmd.output();
        return output
            .map(|out| CommandOutput::Out(out))
            .map_err(|err| err.into());
    }

    if !is_last_in_pipe {
        cmd.stdout(Stdio::piped());
    }
    cmd.stderr(Stdio::piped());

    if let Some(_) = prev_output {
        cmd.stdin(Stdio::piped());
    }

    let mut child: Child;

    if let Some(output) = prev_output {
        match output {
            CommandOutput::Str(s) => {
                child = cmd.spawn()?;
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(s.as_bytes())?;
            }
            CommandOutput::Out(out) => {
                child = cmd.spawn()?;
                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(&out.stdout)?;
            }
            CommandOutput::ChildOut(stdout) => {
                cmd.stdin(stdout);
                child = cmd.spawn()?;
            }
        }
    } else {
        child = cmd.spawn()?;
    }

    let output = if is_last_in_pipe {
        CommandOutput::Out(child.wait_with_output()?)
    } else {
        CommandOutput::ChildOut(child.stdout.unwrap())
    };

    Ok(output)
}

fn print_out(out: &mut Box<dyn Output>, out_str: &mut String, piped: bool, text: &str) {
    if !piped {
        out.print(text);
    }
    out_str.push_str(text);
}

fn println_out(out: &mut Box<dyn Output>, out_str: &mut String, piped: bool, text: &str) {
    if !piped {
        out.println(text);
    }
    out_str.push_str(text);
    if !piped {
        out_str.push('\n');
    }
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
                let file_path = args[i + 1].clone();
                redirection_info.redirect_stdout(file_path, FileOpenMode::Create);
                i += 2;
            }
            "2>" => {
                let file_path = args[i + 1].clone();
                redirection_info.redirect_stderr(file_path, FileOpenMode::Create);
                i += 2;
            }
            ">>" | "1>>" => {
                let file_path = args[i + 1].clone();
                redirection_info.redirect_stdout(file_path, FileOpenMode::Append);
                i += 2;
            }
            "2>>" => {
                let file_path = args[i + 1].clone();
                redirection_info.redirect_stderr(file_path, FileOpenMode::Append);
                i += 2;
            }
            _ => {
                new_args.push(arg.clone());
                i += 1;
            }
        }
    }

    (new_args, redirection_info)
}

fn change_directory(args: &Vec<String>) -> Result<ExecResult> {
    let dir = match args.len() {
        0 => &get_home_dir()?,
        1 => {
            if args[0] != "~".to_string() {
                &args[0]
            } else {
                &get_home_dir()?
            }
        }
        _ => return Err(anyhow!("cd allows no more then one argument")),
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
            Ok(true) => match exec_path.to_str() {
                Some(exec_path) => return Ok(exec_path.to_string()),
                None => continue,
            },
            _ => continue,
        }
    }

    Err(anyhow!("{command}: not found"))
}

fn print_current_dir(
    output: &mut Box<dyn Output>,
    out_str: &mut String,
    piped: bool,
) -> Result<ExecResult> {
    let current_dir = env::current_dir()?;
    println_out(
        output,
        out_str,
        piped,
        &format!("{}", current_dir.display()),
    );
    Ok(ExecResult::Continue)
}
