use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub type TabCompletion = fn (prefix: &str) -> Vec<String>;

pub fn read_line(prompt: &str, tab_completion: TabCompletion) -> String {
    let mut buffer = String::new();
    {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let mut commands = vec![];

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
                                let bell = '\x07';
                                print!("{}", bell);
                            }
                        }
                    } else {
                        let offset = buffer.len() + prompt.len();
                        write!(
                            stdout,
                            "{}{}",
                            termion::cursor::Left(offset as u16),
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
                _ => continue,
            }
            stdout.flush().unwrap();
        }
    }
    println!();

    buffer
}
