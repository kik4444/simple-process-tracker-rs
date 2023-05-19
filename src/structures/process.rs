use std::fmt::Display;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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
