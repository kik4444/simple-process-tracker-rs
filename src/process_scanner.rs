use std::collections::HashSet;

use snafu::ResultExt;

use crate::errors::process_error::*;

#[cfg(target_os = "linux")]
pub async fn get_running_processes() -> Result<HashSet<String>, ProcessError> {
    let mut pids = tokio::fs::read_dir("/proc").await.context(ProcSnafu)?;

    let mut process_list = HashSet::new();

    while let Ok(Some(dir)) = pids.next_entry().await {
        if let Ok(process_short_name) = tokio::fs::read_to_string(dir.path().join("comm")).await {
            process_list.insert(process_short_name.trim().to_owned());
        }
    }

    Ok(process_list)
}

#[cfg(target_os = "macos")]
pub async fn get_running_processes() -> Result<HashSet<String>, ProcessError> {
    compile_error!("macos not yet implemented");
    todo!()
}

#[cfg(target_os = "windows")]
pub async fn get_running_processes() -> Result<HashSet<String>, ProcessError> {
    compile_error!("windows not yet implemented");
    todo!()
}
