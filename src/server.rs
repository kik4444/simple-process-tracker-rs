use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};

use tokio::sync::RwLock;

use crate::{
    commands::{self, Commands},
    get_config_dir, get_socket_name, parse_range,
    process_scanner::get_running_processes,
    string_to_duration,
    structures::{
        config::Config,
        process::{Process, Processes},
    },
};

pub async fn launch() {
    let socket_name = get_socket_name();

    if let Err(e) = LocalSocketListener::bind(socket_name) {
        eprintln!("cannot start server on socket {socket_name} -> {e}");
        std::process::exit(1);
    }

    // Wrap global config in RwLock to allow multiple readers and one writer
    let config = RwLock::new(Config::read().unwrap_or_default());

    // Leak config so that it may live as long as the server lives and be shared across threads without an Arc
    let config = &*Box::leak(Box::new(config));

    // Do the same for the process list
    let processes = RwLock::new(Processes::read().unwrap_or_default());
    let processes = &*Box::leak(Box::new(processes));

    let close_server_flag = &*Box::leak(Box::new(AtomicBool::new(false)));

    println!("Starting server on socket {socket_name}");

    tokio::spawn(async move { update_duration(config, processes).await });

    tokio::spawn(async move { check_running_processes(config, processes).await });

    // Autosave in a blocking thread because it involves disk IO
    tokio::task::spawn_blocking(move || async move { autosave_data(config, processes).await });

    get_user_command(config, processes, close_server_flag).await;
}

async fn save_data(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    let config_dir = get_config_dir().expect("cannot find config dir");

    let config_path = config_dir.join("config.json");
    let config_lock = config_path.with_extension("lock");

    let processes_path = config_dir.join("processes.json");
    let processes_lock = processes_path.with_extension("lock");

    let config = &*config.read().await;
    let processes = &processes.read().await.0;

    let mut builder = std::fs::OpenOptions::new();
    builder.create(true).write(true).truncate(true);

    // We use lock files to prevent a conflict in case this function is called twice simultaneously:
    // once in the autosave thread and once in the handle_user_command thread during server close
    if config_lock.exists() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    _ = builder.open(&config_lock);
    let config_file = builder.open(config_path).expect("must open config path");
    serde_json::to_writer_pretty(config_file, &config).expect("must write");
    _ = std::fs::remove_file(config_lock);

    if processes_lock.exists() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    _ = builder.open(&processes_lock);
    let processes_file = builder
        .open(processes_path)
        .expect("must open processes path");
    serde_json::to_writer_pretty(processes_file, &processes).expect("must write");
    _ = std::fs::remove_file(processes_lock);
}

async fn autosave_data(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    loop {
        let sleep_seconds = config.read().await.autosave_interval;

        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;

        save_data(config, processes).await;
    }
}

async fn update_duration(config: &RwLock<Config>, processes: &RwLock<Processes>) {
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

async fn check_running_processes(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    loop {
        let sleep_seconds = config.read().await.poll_interval;

        tokio::time::sleep(Duration::from_secs(sleep_seconds)).await;

        if let Ok(process_list) = get_running_processes().await {
            for process in processes.write().await.0.iter_mut() {
                if process_list.contains(&process.name) && process.is_tracked {
                    process.is_running = true;
                    process.last_seen_date = chrono::prelude::Local::now().naive_local();
                } else {
                    process.is_running = false;
                }
            }
        } else {
            // TODO log error maybe?
        }
    }
}

async fn get_user_command(
    config: &'static RwLock<Config>,
    processes: &'static RwLock<Processes>,
    close_server_flag: &'static AtomicBool,
) {
    let listener = LocalSocketListener::bind(get_socket_name()).expect("could not bind to socket");

    loop {
        match listener.accept().await {
            Ok(conn) => {
                tokio::spawn(async move {
                    handle_user_command(conn, config, processes, close_server_flag).await
                });
            }
            Err(e) => {
                // TODO log error
                eprintln!("{e}");
            }
        };
    }
}

async fn handle_user_command(
    conn: LocalSocketStream,
    config: &RwLock<Config>,
    processes: &RwLock<Processes>,
    close_server_flag: &AtomicBool,
) {
    let (reader, mut writer) = conn.into_split();

    let mut reader = BufReader::new(reader);
    let mut buffer = String::with_capacity(256);
    _ = reader.read_to_string(&mut buffer).await;

    let command: Commands = serde_json::from_str(&buffer).expect("must not fail");

    let response = match command {
        Commands::Show(show_cmd) => get_processes(show_cmd.ids, processes).await,
        Commands::Add(add_cmd) => add_new_process(add_cmd, processes).await,
        Commands::Option(config_cmd) => change_config(config_cmd, config).await,
        Commands::Change(change_cmd) => change_process(change_cmd, processes).await,
        Commands::Duration(duration_cmd) => change_duration(duration_cmd, processes).await,
        Commands::Export(export_cmd) => get_processes(export_cmd.ids, processes).await,
        Commands::Import(_import_cmd) => todo!(),
        Commands::Move(_move_cmd) => todo!(),
        Commands::Quit => set_exit_flag(close_server_flag).await,

        _ => unreachable!(),
    };

    let serialized = serde_json::to_string(&response).expect("must serialize");
    _ = writer.write_all(serialized.as_bytes()).await;

    if close_server_flag.load(Ordering::Relaxed) {
        save_data(config, processes).await;
        std::process::exit(0)
    }
}

async fn get_processes(
    ids: Option<String>,
    processes: &RwLock<Processes>,
) -> Result<String, String> {
    let processes = &processes.read().await.0;

    let targets: Vec<&Process> = if let Some(ids) = ids {
        let range = parse_range(&ids).map_err(|e| e.to_string())?;
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

async fn add_new_process(
    add_cmd: commands::Add,
    processes: &RwLock<Processes>,
) -> Result<String, String> {
    if processes.read().await.contains_process(&add_cmd.name) {
        return Err(format!("process {} is already tracked", add_cmd.name));
    }

    let duration = if let Some(duration) = add_cmd.duration {
        string_to_duration(&duration).map_err(|_| format!("invalid duration {duration}"))?
    } else {
        0
    };

    let added_date = if let Some(added_date) = add_cmd.added_date {
        chrono::NaiveDateTime::parse_from_str(&added_date, "%Y/%m/%d %H:%M:%S")
            .map_err(|e| format!("invalid date time {added_date} -> {e}"))?
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

async fn change_config(
    config_cmd: commands::Config,
    config: &RwLock<Config>,
) -> Result<String, String> {
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

async fn change_process(
    change_cmd: commands::Change,
    processes: &RwLock<Processes>,
) -> Result<String, String> {
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
        target.duration =
            string_to_duration(&duration).map_err(|_| format!("invalid duration {duration}"))?;
    }

    if let Some(notes) = change_cmd.notes {
        target.notes = notes;
    }

    if let Some(added_date) = change_cmd.added_date {
        target.added_date = chrono::NaiveDateTime::parse_from_str(&added_date, "%Y/%m/%d %H:%M:%S")
            .map_err(|e| format!("invalid date time {added_date} -> {e}"))?;
    }

    Ok(format!("changed {}", target.name))
}

async fn change_duration(
    duration_cmd: commands::Duration,
    processes: &RwLock<Processes>,
) -> Result<String, String> {
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
async fn set_exit_flag(close_server_flag: &AtomicBool) -> Result<String, String> {
    close_server_flag.store(true, Ordering::Relaxed);

    Ok("stopping server".into())
}
