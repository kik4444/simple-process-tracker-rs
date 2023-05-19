use std::fmt::Display;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::get_config_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    is_running: bool,
    is_tracked: bool,
    icon: String,
    name: String,
    duration: u64,
    notes: String,
    last_seen_date: NaiveDateTime,
    added_date: NaiveDateTime,
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {} | {} | {} | {}",
            self.is_running,
            self.is_tracked,
            self.icon,
            self.name,
            self.duration,
            self.notes,
            self.last_seen_date.format("%Y/%m/%d %H:%M:%S"),
            self.added_date.format("%Y/%m/%d %H:%M:%S")
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Processes(Vec<Process>);

impl Processes {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let processes_path = get_config_dir()
            .expect("cannot find config dir")
            .join("processes.json");

        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(processes_path)?;

        Ok(serde_json::from_reader(reader)?)
    }
}
