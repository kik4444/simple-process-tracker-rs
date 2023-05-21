use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::RwLock;

use crate::{
    commands::{self},
    parse_datetime, parse_range, string_to_duration,
    structures::{
        config::Config,
        legacy_process::LegacyProcesses,
        process::{Process, Processes},
    },
};

pub async fn get_processes(
    ids: Option<String>,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    let processes = &processes.read().await.0;

    let targets: Vec<&Process> = if let Some(ids) = ids {
        let range = parse_range(&ids).map_err(|e| format!("invalid range {ids} -> {e}"))?;
        processes
            .iter()
            .enumerate()
            .filter(|(id, _)| range.contains(id))
            .map(|(_, process)| process)
            .collect()
    } else {
        // We must make this into an owned Vec of Process references so this type matches with the above
        processes.iter().collect()
    };

    Ok(serde_json::to_string(&targets).expect("must serialize"))
}

pub async fn get_settings(config: &RwLock<Config>) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(&*config.read().await).expect("must serialize"))
}

pub async fn add_new_process(
    add_cmd: commands::Add,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    if processes.read().await.contains_process(&add_cmd.name) {
        return Err(format!("process {} is already tracked", add_cmd.name).into());
    }

    let duration = if let Some(duration) = add_cmd.duration {
        string_to_duration(&duration)?
    } else {
        0
    };

    let added_date = if let Some(added_date) = add_cmd.added_date {
        parse_datetime(&added_date)?
    } else {
        chrono::prelude::Local::now().naive_local()
    };

    processes.write().await.0.push(Process {
        is_running: false,
        is_tracked: true,
        icon: add_cmd.icon.unwrap_or_default(),
        name: add_cmd.name.clone(),
        duration,
        notes: add_cmd.notes.unwrap_or_default(),
        last_seen_date: chrono::NaiveDateTime::from_timestamp_millis(0).expect("0 is in range"),
        added_date,
    });

    Ok(format!("added {}", add_cmd.name))
}

pub async fn remove_processes(
    id: usize,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    let removed;

    let processes = &mut processes.write().await.0;

    if processes.is_empty() {
        return Err("no processes to remove".into());
    } else if processes.len() - 1 < id {
        return Err(format!("no process with id {id}").into());
    } else {
        removed = processes[id].name.clone();
        processes.remove(id);
    }

    Ok(format!("removed {removed}"))
}

pub async fn change_config(
    config_cmd: commands::Config,
    config: &RwLock<Config>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut config = config.write().await;

    if let Some(poll_interval) = config_cmd.poll_interval {
        config.poll_interval = poll_interval;
    }

    if let Some(duration_update_interval) = config_cmd.duration_update_interval {
        config.duration_update_interval = duration_update_interval;
    }

    if let Some(autosave_interval) = config_cmd.autosave_interval {
        config.autosave_interval = autosave_interval;
    }

    Ok("changed config".into())
}

pub async fn change_process(
    change_cmd: commands::Change,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    let processes = &mut processes.write().await.0;

    let target = processes
        .get_mut(change_cmd.id)
        .ok_or_else(|| format!("invalid ID {}", change_cmd.id))?;

    if let Some(tracking) = change_cmd.tracking {
        target.is_tracked = tracking;
    }

    if let Some(icon) = change_cmd.icon {
        target.icon = icon;
    }

    if let Some(duration) = change_cmd.duration {
        target.duration = string_to_duration(&duration)?;
    }

    if let Some(notes) = change_cmd.notes {
        target.notes = notes;
    }

    if let Some(added_date) = change_cmd.added_date {
        target.added_date = parse_datetime(&added_date)?;
    }

    Ok(format!("changed {}", target.name))
}

pub async fn change_duration(
    duration_cmd: commands::Duration,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    let processes = &mut processes.write().await.0;

    let target = processes
        .get_mut(duration_cmd.id)
        .ok_or_else(|| format!("invalid ID {}", duration_cmd.id))?;

    let (action, amount);

    target.duration = match duration_cmd.operation {
        commands::DurationCalculation::Add { seconds } => {
            action = "added";
            amount = seconds;
            target.duration.saturating_add(seconds)
        }
        commands::DurationCalculation::Subtract { seconds } => {
            action = "subtracted";
            amount = seconds;
            target.duration.saturating_sub(seconds)
        }
    };

    Ok(format!("{action} {amount} seconds for {}", target.name))
}

pub async fn import_processes(
    import_cmd: commands::Import,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&import_cmd.path)
        .map_err(|e| format!("cannot open file {} -> {e}", import_cmd.path.display()))?;

    let processes = &mut processes.write().await;

    let (mut newly_added, mut already_existed) = (vec![], vec![]);

    if !import_cmd.legacy {
        let new_processes: Processes =
            serde_json::from_reader(file).map_err(|e| format!("error parsing json -> {e}"))?;

        for new_process in new_processes.0 {
            if !processes.contains_process(&new_process.name) {
                newly_added.push(new_process.name.clone());
                processes.0.push(Process {
                    is_running: false,
                    ..new_process
                });
            } else {
                already_existed.push(new_process.name.clone());
            }
        }
    } else {
        let new_legacy_processes: LegacyProcesses =
            serde_json::from_reader(file).map_err(|e| format!("error parsing json -> {e}"))?;

        for (name, new_legacy_process) in new_legacy_processes.0 {
            if !processes.contains_process(&name) {
                newly_added.push(name.clone());

                processes.0.push(Process {
                    is_running: false,
                    is_tracked: new_legacy_process.tracking,
                    icon: new_legacy_process.icon_path,
                    name,
                    duration: new_legacy_process.duration,
                    notes: new_legacy_process.notes,
                    last_seen_date: parse_datetime(&new_legacy_process.last_seen)?,
                    added_date: parse_datetime(&new_legacy_process.date_added)?,
                })
            } else {
                already_existed.push(name.clone());
            }
        }
    }

    Ok(format!(
        "added {newly_added:?}{}",
        if already_existed.is_empty() {
            "".to_string()
        } else {
            format!(", already tracked {already_existed:?}")
        }
    ))
}

pub async fn move_process(
    move_cmd: commands::Move,
    processes: &RwLock<Processes>,
) -> Result<String, Box<dyn std::error::Error>> {
    let processes = &mut processes.write().await.0;

    if processes.is_empty() {
        return Err("no processes to move".into());
    } else if processes.len() == 1 {
        return Err("cannot move only one process".into());
    } else if processes.len() - 1 < move_cmd.id {
        return Err(format!("no process with id {}", move_cmd.id).into());
    }

    let range: Box<dyn Iterator<Item = usize>>;

    use commands::MoveDirection::*;
    match move_cmd.direction {
        Up | Top => {
            if move_cmd.id == 0 {
                return Err(format!("{} already at top", processes[move_cmd.id].name).into());
            }

            let end = move_cmd.id - 1;

            if let Top = move_cmd.direction {
                range = Box::new((0..=end).rev())
            } else {
                range = Box::new((end..=end).rev())
            }
        }
        Down | Bottom => {
            if move_cmd.id == processes.len() - 1 {
                return Err(format!("{} already at bottom", processes[move_cmd.id].name).into());
            }

            let end = processes.len() - 1;

            if let Bottom = move_cmd.direction {
                range = Box::new(0..end);
            } else {
                range = Box::new(end..=end);
            }
        }
    }

    for i in range {
        processes.swap(i, i + 1);
    }

    Ok(format!("moved {}", processes[move_cmd.id].name))
}

pub async fn set_exit_flag(
    close_server_flag: &AtomicBool,
) -> Result<String, Box<dyn std::error::Error>> {
    close_server_flag.store(true, Ordering::Relaxed);

    Ok("stopping server".into())
}
