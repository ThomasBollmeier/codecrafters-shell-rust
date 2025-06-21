use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use crate::history::History;

pub type TabCompletion = fn (prefix: &str) -> Vec<String>;

pub fn read_line(prompt: &str, tab_completion: TabCompletion, history: &History) -> String {
    let mut buffer = String::new();
    {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let mut commands = vec![];
        let mut history_idx = history.size();

        for key in stdin().keys().flatten() {
            match key {
                Key::Char('\n') => {
                    break;
                }
                Key::Char('\t') => {
                    if commands.is_empty() {
                        commands = tab_completion(buffer.as_str());
                        match commands.len() {
                            0 => {
                                let bell = '\x07';
                                print!("{}", bell);
                            }
                            1 => {
                                write!(
                                    stdout,
                                    "{}{}",
                                    termion::cursor::Left(buffer.len() as u16),
                                    termion::clear::AfterCursor
                                ).unwrap();
                                buffer = commands[0].to_string();
                                buffer.push(' ');
                                print!("{}", buffer);
                            }
                            _ => {
                                let common_prefix = find_common_prefix(&commands);
                                if common_prefix.len() > buffer.len() {
                                    for c in common_prefix.chars().skip(buffer.len()) {
                                        buffer.push(c);
                                        print!("{}", c);
                                    }
                                    commands.clear();
                                } else {
                                    let bell = '\x07';
                                    print!("{}", bell);
                                }
                            }
                        }
                    } else {
                        show_matching_commands(&mut stdout, prompt, &buffer, &commands);
                        commands.clear();
                    }
                },
                Key::Char(c) => {
                    buffer.push(c);
                    print!("{}", c);
                }
                Key::Backspace => {
                    buffer.pop();
                    write!(
                        stdout,
                        "{}{}",
                        termion::cursor::Left(1),
                        termion::clear::AfterCursor
                    ).unwrap();
                }
                Key::Up => {
                    if history_idx > 0 {
                        history_idx -= 1;
                        goto_begin_of_line(&mut stdout, prompt, &buffer);
                        buffer = history[history_idx].clone();
                        print!("{}{}{}", termion::clear::AfterCursor, prompt, buffer);
                    }
                }
                Key::Down => {
                    if history_idx < history.size() - 1 {
                        history_idx += 1;
                        goto_begin_of_line(&mut stdout, prompt, &buffer);
                        buffer = history[history_idx].clone();
                        print!("{}{}{}", termion::clear::AfterCursor, prompt, buffer);
                    } else if history_idx == history.size() - 1 {
                        history_idx += 1; // Move past the last entry
                        goto_begin_of_line(&mut stdout, prompt, &buffer);
                        buffer.clear();
                        print!("{}{}", termion::clear::AfterCursor, prompt);
                    }
                }
                _ => continue,
            }
            stdout.flush().unwrap();
        }
    }
    println!();

    buffer
}

fn goto_begin_of_line(
    stdout: &mut RawTerminal<Stdout>,
    prompt: &str,
    buffer: &str) {
    let offset = prompt.len() + buffer.len();
    write!(stdout, "{}", termion::cursor::Left(offset as u16)).unwrap();
}

fn show_matching_commands(
    stdout: &mut RawTerminal<Stdout>,
    prompt: &str,
    buffer: &str,
    commands: &Vec<String>) {
    goto_begin_of_line(stdout, prompt, buffer);
    write!(
        stdout,
        "{}",
        termion::cursor::Down(1),
    ).unwrap();
    let commands_str = commands.join("  ");
    write!(stdout,
           "{}{}{}{}{}",
           commands_str,
           termion::cursor::Down(1),
           termion::cursor::Left(commands_str.len() as u16),
           prompt,
           buffer,
    ).unwrap();
}

fn find_common_prefix(strings: &Vec<String>) -> String {
    let mut prefix: Vec<char> = vec![];
    let mut first = true;

    for s in strings {
        if first {
            prefix = s.chars().collect();
            first = false;
            continue;
        }

        for (i, c) in s.chars().enumerate() {
            if i >= prefix.len() || c != prefix[i] {
                prefix.truncate(i);
                break;
            }
        }
    }

    prefix.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_common_prefix() {
        let commands = vec![
            "xyz_foo".to_string(),
            "xyz_foo_bar".to_string(),
            "xyz_foo_bar_baz".to_string(),
        ];

        assert_eq!(find_common_prefix(&commands), String::from("xyz_foo"));
    }

}