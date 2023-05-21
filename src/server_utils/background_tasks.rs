use std::time::Duration;

use tokio::sync::RwLock;

use crate::{
    get_config_dir,
    process_scanner::get_running_processes,
    structures::{config::Config, process::Processes},
};

pub async fn save_data(
    config: &RwLock<Config>,
    processes: &RwLock<Processes>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir().ok_or_else(|| "cannot find config dir".to_string())?;

    let config_path = config_dir.join("config.json");
    let config_lock = config_path.with_extension("lock");

    let processes_path = config_dir.join("processes.json");
    let processes_lock = processes_path.with_extension("lock");

    let mut builder = std::fs::OpenOptions::new();
    builder.create(true).write(true).truncate(true);

    // We use lock files to prevent a conflict in case this function is called twice simultaneously:
    // once in the autosave thread and once in the handle_user_command thread during server close
    if config_lock.exists() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    _ = builder.open(&config_lock);
    let config_file = builder
        .open(config_path)
        .map_err(|e| format!("cannot open config path -> {e}"))?;
    serde_json::to_writer_pretty(config_file, &*config.read().await)?;
    _ = std::fs::remove_file(config_lock);

    if processes_lock.exists() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    _ = builder.open(&processes_lock);
    let processes_file = builder
        .open(processes_path)
        .map_err(|e| format!("cannot open processes path -> {e}"))?;
    serde_json::to_writer_pretty(processes_file, &processes.read().await.0)?;
    _ = std::fs::remove_file(processes_lock);

    Ok(())
}

pub async fn autosave_data(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    loop {
        let sleep_seconds = config.read().await.autosave_interval;

        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;

        if let Err(e) = save_data(config, processes).await {
            eprintln!("{e}");
        };
    }
}

pub async fn update_duration(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    loop {
        let sleep_seconds = config.read().await.duration_update_interval;

        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;

        for process in processes.write().await.0.iter_mut() {
            if process.is_running && process.is_tracked {
                process.duration += sleep_seconds;
            }
        }
    }
}

pub async fn check_running_processes(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    loop {
        // In case a process was running when the server closed, its is_running would be saved as true in the json file.
        // As a result, when the server starts it might incorrectly update that process's duration before checking
        // if it was running. Therefore, we check which processes are running first before sleeping
        match get_running_processes().await {
            Ok(process_list) => {
                for process in processes.write().await.0.iter_mut() {
                    if process_list.contains(&process.name) && process.is_tracked {
                        process.is_running = true;
                        process.last_seen_date = chrono::prelude::Local::now().naive_local();
                    } else {
                        process.is_running = false;
                    }
                }
            }
            Err(e) => eprintln!("{e}"),
        }

        let sleep_seconds = config.read().await.poll_interval;

        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;
    }
}
