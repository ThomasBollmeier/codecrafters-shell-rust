pub type Command = (String, Vec<String>);
pub type CommandList = Vec<Command>;

pub struct ArgParser {
    pos: usize,
    chars: Vec<char>,
}

impl ArgParser {

    pub fn new() -> Self {
        Self {
            pos: 0,
            chars: vec![],
        }
    }

    pub fn parse_args(&mut self, input: &str) -> anyhow::Result<CommandList> {
        self.pos = 0;
        self.chars = input.chars().collect();

        if self.chars.is_empty() {
            return Err(anyhow::anyhow!("input is empty"));
        }

        let mut ret = vec![];
        let mut parts: Vec<String> = vec![];
        let mut part = String::new();

        while !self.is_done() {
            let cnt_wspace = self.skip_whitespaces();

            let ch = match self.current_char() {
                Some(ch) => ch,
                None => break,
            };

            if ch == '|' {
                self.pos += 1;
                if !part.is_empty() {
                    parts.push(part);
                    part = String::new();
                }
                if parts.is_empty() {
                    return Err(anyhow::anyhow!("pipe without command"));
                }
                ret.push(Self::get_command(parts));
                parts = vec![];
                continue;
            }

            let next_str = match ch {
                '\'' => self.scan_single_quoted_string(),
                '"' => self.scan_double_quoted_string(),
                _ => self.scan_string(),
            };

            if !part.is_empty() {
                if cnt_wspace > 0 {
                    parts.push(part);
                    part = next_str;
                } else {
                    part.push_str(&next_str);
                }
            } else {
                part = next_str;
            }
        }

        if !part.is_empty() {
            parts.push(part);
        }

        ret.push(Self::get_command(parts));

        Ok(ret)
    }

    fn get_command(parts: Vec<String>) -> Command {
        let command = parts[0].clone();
        let args = parts.into_iter().skip(1).collect::<Vec<String>>();
        (command, args)
    }

    fn is_done(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn skip_whitespaces(&mut self) -> usize {
        let mut count = 0;
        while !self.is_done() {
            if self.current_char().unwrap().is_whitespace() {
                self.pos += 1;
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    fn scan_string(&mut self) -> String {
        let mut ret = String::new();
        let mut escaped = false;

        while !self.is_done() {
            let ch = self.current_char().unwrap();
            if escaped {
                ret.push(ch);
                self.pos += 1;
                escaped = false;
                continue;
            }
            if ch.is_whitespace() || ch == '\'' || ch == '"' {
                break;
            }
            match ch {
                '\\' => {
                    escaped = true;
                }
                _ => {
                    ret.push(ch);
                }
            }
            self.pos += 1;
        }

        ret
    }

    fn scan_single_quoted_string(&mut self) -> String {
        let mut ret = String::new();
        self.pos += 1;
        while !self.is_done() {
            let ch = self.current_char().unwrap();
            self.pos += 1;
            if ch == '\'' {
                break;
            }
            ret.push(ch);
        }

        ret
    }

    fn scan_double_quoted_string(&mut self) -> String {
        let mut ret = String::new();
        self.pos += 1;
        while !self.is_done() {
            let mut ch = self.current_char().unwrap();
            self.pos += 1;
            match ch {
                '\\' => {
                    let next_ch_opt = self.current_char();
                    match next_ch_opt {
                        Some('\\') | Some('"') | Some('$') => { // escaping
                            ch = next_ch_opt.unwrap();
                            self.pos += 1;
                        }
                        _ => {},
                    }
                }
                '"' => break,
                _ => {}
            }
            ret.push(ch);
        }

        ret
    }

    fn current_char(&self) -> Option<char> {
        if self.pos >= self.chars.len() {
            return None;
        }
        Some(self.chars[self.pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let mut parser = ArgParser::new();
        let input = "echo eins   zwei drei   ";

        let commands = parser.parse_args(input).unwrap();
        assert_eq!(commands.len(), 1);
        let (command, args) = &commands[0];
        assert_eq!(command, "echo");
        assert_eq!(args, &vec!["eins", "zwei", "drei"]);
    }

    #[test]
    fn test_parse_single_quoted_args() {
        let mut parser = ArgParser::new();
        let input = "echo 'eins   zwei' drei   ";

        let commands = parser.parse_args(input).unwrap();
        assert_eq!(commands.len(), 1);
        let (command, args) = &commands[0];
        assert_eq!(command, "echo");
        assert_eq!(args, &vec!["eins   zwei", "drei"]);
    }

    #[test]
    fn test_parse_double_quoted_args() {
        let mut parser = ArgParser::new();
        let input = r#"echo "eins   'zwei' " drei   "#;

        let commands = parser.parse_args(input).unwrap();
        assert_eq!(commands.len(), 1);
        let (command, args) = &commands[0];
        assert_eq!(command, "echo");
        assert_eq!(args, &vec!["eins   'zwei' ", "drei"]);
    }

    #[test]
    fn test_parse_empty_args() {
        let mut parser = ArgParser::new();
        let input = "";

        match parser.parse_args(input) {
            Ok(_) => assert!(false, "error expected"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn test_parse_escaped_args() {
        let mut parser = ArgParser::new();
        let input = r#"echo \'\"script world\"\'"#;

        let commands = parser.parse_args(input).unwrap();
        assert_eq!(commands.len(), 1);
        let (command, args) = &commands[0];
        assert_eq!(command, "echo");
        assert_eq!(args, &vec!["'\"script", "world\"'"]);
    }
    
    #[test]
    fn test_pipe() {
        let mut parser = ArgParser::new();
        let input = "echo eins | echo zwei | echo drei";

        let commands = parser.parse_args(input).unwrap();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].0, "echo");
        assert_eq!(commands[1].0, "echo");
        assert_eq!(commands[2].0, "echo");
    }
}
