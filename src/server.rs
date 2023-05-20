use std::time::Duration;

use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};

use tokio::sync::RwLock;

use crate::{
    commands::{self, Commands},
    get_socket_name,
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
        eprintln!(
            "cannot start server on socket {socket_name} -> {}",
            e.to_string()
        );
        std::process::exit(1);
    }

    // Wrap global config in RwLock to allow multiple readers and one writer
    let config = RwLock::new(Config::read().unwrap_or_default());

    // Leak config so that it may live as long as the server lives and be shared across threads without an Arc
    let config = &*Box::leak(Box::new(config));

    // Do the same for the process list
    let processes = RwLock::new(Processes::read().unwrap_or_default());
    let processes = &*Box::leak(Box::new(processes));

    println!("Starting server on socket {socket_name}");

    tokio::spawn(async move { update_duration(config, processes).await });

    tokio::spawn(async move { check_running_processes(config, processes).await });

    get_user_command(config, processes).await;
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

async fn get_user_command(config: &'static RwLock<Config>, processes: &'static RwLock<Processes>) {
    let listener = LocalSocketListener::bind(get_socket_name()).expect("could not bind to socket");

    loop {
        match listener.accept().await {
            Ok(conn) => {
                tokio::spawn(async move { handle_user_command(conn, config, processes).await });
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
) {
    let (reader, mut writer) = conn.into_split();

    let mut reader = BufReader::new(reader);
    let mut buffer = String::with_capacity(256);
    _ = reader.read_to_string(&mut buffer).await;

    let command: Commands = serde_json::from_str(&buffer).expect("must not fail");

    let response = match command {
        Commands::Show(show_cmd) => show_processes(processes, show_cmd.id).await,
        Commands::Add(add_cmd) => add_new_process(add_cmd, processes).await,
        Commands::Option(config_cmd) => change_config(config_cmd, config).await,
        Commands::Change(change_cmd) => change_process(change_cmd, processes).await,
        Commands::Duration(duration_cmd) => change_duration(duration_cmd, processes).await,
        Commands::Export(_export_cmd) => todo!(),
        Commands::Import(_import_cmd) => todo!(),
        Commands::Move(_move_cmd) => todo!(),
        Commands::Quit => todo!(),

        _ => unreachable!(),
    };

    let serialized = serde_json::to_string(&response).expect("must serialize");
    _ = writer.write_all(serialized.as_bytes()).await;
}

async fn show_processes(
    processes: &RwLock<Processes>,
    id: Option<usize>,
) -> Result<String, String> {
    let processes = &processes.read().await.0;

    let target = if let Some(id) = id {
        processes.get(id).map(|p| vec![p]).unwrap_or_default()
    } else {
        // We must make this into an owned Vec of Process references so this type matches with the above
        processes.iter().collect::<Vec<&Process>>()
    };

    Ok(serde_json::to_string(&target).expect("must serialize"))
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

    Ok(format!("Added {}", add_cmd.name))
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

    Ok("Changed config".into())
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

    Ok(format!("Changed {}", target.name))
}

async fn change_duration(
    _duration_cmd: commands::Duration,
    _processes: &RwLock<Processes>,
) -> Result<String, String> {
    todo!()
}
