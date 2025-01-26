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

    pub fn parse_args(&mut self, input: &str) -> anyhow::Result<(String, Vec<String>)> {
        self.pos = 0;
        self.chars = input.chars().collect();

        if self.chars.is_empty() {
            return Err(anyhow::anyhow!("input is empty"));
        }

        let mut parts: Vec<String> = vec![];

        while !self.is_done() {
            self.skip_whitespaces();
            let ch = match self.current_char() {
                Some(ch) => ch,
                None => break,
            };
            let part = match ch {
                '\'' => self.scan_single_quoted_string(),
                '"' => self.scan_double_quoted_string(),
                _ => self.scan_string(),
            };
            parts.push(part);
        }

        let command = parts[0].clone();
        let args = parts.into_iter().skip(1).collect::<Vec<String>>();

        Ok((command, args))
    }

    fn is_done(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn skip_whitespaces(&mut self) {
        while !self.is_done() {
            if self.current_char().unwrap().is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
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
                    self.pos += 1;
                    escaped = true;
                    continue;
                }
                _ => {
                    ret.push(ch);
                    self.pos += 1;
                }
            }
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
                match self.current_char() {
                    Some('\'') => { // two consecutive single quotes are ignored
                        self.pos += 1;
                        continue;
                    }
                    _ => {
                        break;
                    },
                }
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
                '"' => match self.current_char() {
                    Some('"') => { // two consecutive double quotes are ignored
                        self.pos += 1;
                        continue;
                    }
                    _ => {
                        break;
                    },
                },
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

        let (command, args) = parser.parse_args(input).unwrap();

        assert_eq!(command, "echo");
        assert_eq!(args, vec!["eins", "zwei", "drei"]);
    }

    #[test]
    fn test_parse_single_quoted_args() {
        let mut parser = ArgParser::new();
        let input = "echo 'eins   zwei' drei   ";

        let (command, args) = parser.parse_args(input).unwrap();

        assert_eq!(command, "echo");
        assert_eq!(args, vec!["eins   zwei", "drei"]);
    }

    #[test]
    fn test_parse_double_quoted_args() {
        let mut parser = ArgParser::new();
        let input = r#"echo "eins   'zwei' " drei   "#;

        let (command, args) = parser.parse_args(input).unwrap();

        assert_eq!(command, "echo");
        assert_eq!(args, vec!["eins   'zwei' ", "drei"]);
    }

    #[test]
    fn test_parse_empty_args() {
        let mut parser = ArgParser::new();
        let input = "";

        match parser.parse_args(input) {
            Ok((command, _)) => {
                assert!(false, "error expected, but got {}", command);
            }
            Err(_) => assert!(true),
        }
    }
}
