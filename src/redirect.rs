use std::fmt::Debug;
use anyhow::Result;
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub struct RedirectionInfo {
    stdout: Option<String>,
    stderr: Option<String>,
}

impl RedirectionInfo {
    pub fn new() -> RedirectionInfo {
        RedirectionInfo{
            stdout: None,
            stderr: None,
        }
    }

    pub fn redirect_stdout(&mut self, file_path: String) {
        self.stdout = Some(file_path);
    }

    pub fn redirect_stderr(&mut self, file_path: String) {
        self.stderr = Some(file_path);
    }

    pub fn get_output(&self) -> Box<dyn Output> {
        let ret: Box<dyn Output> = match &self.stdout {
            Some(path) => Box::new(FileOutput::new(path.clone())),
            None => Box::new(StdOutput{})
        };

        ret
    }

    pub fn get_error_output(&self) -> Box<dyn Output> {
        let ret: Box<dyn Output> = match &self.stderr {
            Some(path) => Box::new(FileOutput::new(path.clone())),
            None => Box::new(StdErrorOutput{})
        };

        ret
    }}

pub trait Output: Debug {
    fn open(&mut self) -> Result<()>;

    fn print(&mut self, text: &str);

    fn println(&mut self, text: &str) {
        self.print(&format!("{text}\n"));
    }

    fn close(&mut self);
}

#[derive(Debug)]
struct StdOutput {}

impl Output for StdOutput {
    fn open(&mut self) -> Result<()> {
        Ok(())
    }

    fn print(&mut self, text: &str) {
        print!("{}", text);
    }

    fn close(&mut self) {}
}

#[derive(Debug)]
struct StdErrorOutput {}

impl Output for StdErrorOutput {
    fn open(&mut self) -> Result<()> {
        Ok(())
    }

    fn print(&mut self, text: &str) {
        eprint!("{}", text);
    }

    fn close(&mut self) {}
}

#[derive(Debug)]
struct FileOutput {
    file_path: String,
    file: Option<File>,
}

impl FileOutput {
    pub fn new(file_path: String) -> FileOutput {
        FileOutput{
            file_path,
            file: None,
        }
    }
}

impl Output for FileOutput {
    fn open(&mut self) -> Result<()> {
        self.file = Some(File::create(&self.file_path)?);
        Ok(())
    }

    fn print(&mut self, text: &str) {
        if let Some(ref mut file) = self.file {
            write!(file, "{text}").expect("Cannot write to file");
        }
    }

    fn close(&mut self) {
        self.file = None;
    }
}
