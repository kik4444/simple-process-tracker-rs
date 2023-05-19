use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    poll_interval: u64,
    duration_update_interval: u64,
    autosave_interval: u64,
}
