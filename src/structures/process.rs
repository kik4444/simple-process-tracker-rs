use std::{collections::HashMap, path::PathBuf};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::get_config_dir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Process {
    pub is_running: bool,
    pub is_tracked: bool,
    pub icon: PathBuf,
    pub name: String,
    pub duration: u64,
    pub notes: String,
    pub last_seen_date: NaiveDateTime,
    pub added_date: NaiveDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Processes(pub Vec<Process>);

impl Processes {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let processes_path = get_config_dir()
            .expect("cannot find config dir")
            .join("processes.json");

        let reader = std::fs::OpenOptions::new()
            .read(true)
            .open(processes_path)?;

        // We panic in some errors because if the json has become damaged or duplicates have somehow been added to it,
        // it's better to stop and let the user fix this instead of potentially overwriting
        // existing process entries
        let processes: Processes = serde_json::from_reader(reader)
            .unwrap_or_else(|e| panic!("error reading existing processes.json -> {e}"));

        let mut process_names: HashMap<&str, usize> = HashMap::new();
        for process in processes.0.iter() {
            *process_names.entry(&process.name).or_insert(0) += 1;
        }

        for count in process_names.values() {
            if *count > 1 {
                panic!("processes.json contains duplicates!");
            }
        }

        Ok(processes)
    }

    pub fn contains_process(&self, name: &str) -> bool {
        for process in self.0.iter() {
            if process.name == name {
                return true;
            }
        }
        false
    }
}
