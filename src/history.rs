use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::ops::Index;
use anyhow::{anyhow, Result};

pub struct History {
    saved_entries: Vec<String>,
    unsaved_entries: Vec<String>,
}

impl History {
    pub fn new() -> Self {
        Self {
            saved_entries: vec![],
            unsaved_entries: vec![],
        }
    }

    pub fn add_entry(&mut self, entry: String) {
        self.unsaved_entries.push(entry);
    }

    pub fn load(&mut self, path: &str) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        self.unsaved_entries.push(line);
                    }
                }
                Err(err) => return Err(anyhow!("Error reading history file: {}", err)),
            }
        }
        Ok(())
    }

    pub fn save(&mut self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        self.saved_entries.append(&mut self.unsaved_entries);
        self.unsaved_entries.clear();
        for entry in &self.saved_entries {
            writeln!(file, "{}", entry)?;
        }
        Ok(())
    }

    pub fn append(&mut self, path: &str) -> Result<()> {
        let mut file = File::options().append(true).open(path)?;
        for entry in &self.unsaved_entries {
            writeln!(file, "{}", entry)?;
        }
        self.saved_entries.append(&mut self.unsaved_entries);
        self.unsaved_entries.clear();
        Ok(())
    }

    pub fn get_all_entries(&self) -> Vec<String> {
        self.saved_entries.clone().into_iter().chain(self.unsaved_entries.clone()).collect()
    }

    pub fn get_latest_entries(&self, n: usize) -> Vec<String> {
        let num_recent = self.unsaved_entries.len();

        if n <= num_recent {
            return self.unsaved_entries[num_recent - n..].to_vec();
        }

        let remaining = n - num_recent;
        let num_saved = self.saved_entries.len();

        if remaining <= num_saved {
            return self.saved_entries[num_saved - remaining..].to_vec()
                .into_iter()
                .chain(self.unsaved_entries.clone())
                .collect();
        }

        self.saved_entries.clone()
            .into_iter()
            .chain(self.unsaved_entries.clone())
            .collect()
    }

    pub fn size(&self) -> usize {
        self.saved_entries.len() + self.unsaved_entries.len()
    }
}

impl Index<usize> for History {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.saved_entries.len() {
            &self.saved_entries[index]
        } else {
            &self.unsaved_entries[index - self.saved_entries.len()]
        }
    }
}