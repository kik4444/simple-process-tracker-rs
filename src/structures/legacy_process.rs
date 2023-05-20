use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyProcess {
    // pub categories: String, // ignored
    pub date_added: String,
    pub duration: u64,
    pub icon_path: PathBuf,
    pub last_seen: String,
    pub notes: String,
    pub tracking: bool,
}

#[derive(Debug, Deserialize)]
pub struct LegacyProcesses(pub HashMap<String, LegacyProcess>);
