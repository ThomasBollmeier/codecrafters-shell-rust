use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub type TabCompletion = fn (prefix: &str) -> Option<String>;

pub fn read_line(tab_completion: TabCompletion) -> String {
    let mut buffer = String::new();
    {
        let mut stdout = stdout().into_raw_mode().unwrap();

        for key in stdin().keys().flatten() {
            match key {
                Key::Char('\n') => {
                    break;
                }
                Key::Char('\t') => {
                    if let Some(completion) = tab_completion(buffer.as_str()) {
                        write!(
                            stdout,
                            "{}{}",
                            termion::cursor::Left(buffer.len() as u16),
                            termion::clear::AfterCursor
                        ).unwrap();    buffer = completion;
                        print!("{}", buffer);
                    } else {
                        let bell = '\x07';
                        print!("{}", bell);
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
