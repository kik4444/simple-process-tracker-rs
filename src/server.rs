use std::time::Duration;

use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};

use tokio::sync::RwLock;

use crate::{
    commands::Commands,
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
                if process_list.contains(&process.name) {
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
                tokio::spawn(async move { handle_user_command(conn, config, processes).await })
            }
            Err(e) => {
                // TODO log error
                eprintln!("{e}");
                continue;
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

    use Commands::*;
    let response = match command {
        Show { id, .. } => show_processes(processes, id).await,
        Add { .. } => add_new_process(processes, command).await,
        Option { .. } => todo!(),
        Change { .. } => todo!(),
        Duration { .. } => todo!(),
        Export { .. } => todo!(),
        Import { .. } => todo!(),
        Move { .. } => todo!(),
        Quit => todo!(),

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
    processes: &RwLock<Processes>,
    command: Commands,
) -> Result<String, String> {
    let Commands::Add { name, icon, duration, notes, added_date } = command else { return Err("".into()) };

    if processes.read().await.contains_process(&name) {
        return Err(format!("process {name} is already tracked"));
    }

    let duration = if let Some(duration) = duration {
        string_to_duration(&duration).map_err(|_| format!("invalid duration {duration}"))?
    } else {
        0
    };

    let added_date = if let Some(added_date) = added_date {
        chrono::NaiveDateTime::parse_from_str(&added_date, "%Y/%m/%d %H:%M:%S")
            .map_err(|e| format!("invalid date time {added_date} -> {e}"))?
    } else {
        chrono::prelude::Local::now().naive_local()
    };

    processes.write().await.0.push(Process {
        is_running: false,
        is_tracked: true,
        icon: icon.unwrap_or_default(),
        name: name.clone(),
        duration,
        notes: notes.unwrap_or_default(),
        last_seen_date: chrono::NaiveDateTime::from_timestamp_millis(0).expect("0 is in range"),
        added_date,
    });

    Ok(format!("Added {name}"))
}
