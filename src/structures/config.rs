use serde::{Deserialize, Serialize};

use crate::get_config_file;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    poll_interval: u64,
    duration_update_interval: u64,
    autosave_interval: u64,
}

impl Config {
    pub fn read() -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = get_config_file().expect("cannot find config dir");

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
