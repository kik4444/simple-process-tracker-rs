use serde::{Deserialize, Serialize};

use crate::get_config_dir;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub poll_interval: u64,
    pub duration_update_interval: u64,
    pub autosave_interval: u64,
}

impl Config {
    pub fn read() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = get_config_dir()
            .expect("cannot find config dir")
            .join("config.json");

        let reader = std::fs::OpenOptions::new().read(true).open(config_path)?;

        Ok(serde_json::from_reader(reader)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_interval: 15,
            duration_update_interval: 10,
            autosave_interval: 300,
        }
    }
}
