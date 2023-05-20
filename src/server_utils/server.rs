use std::sync::atomic::{AtomicBool, Ordering};

use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};

use tokio::sync::RwLock;

use crate::{
    commands::Commands,
    get_socket_name,
    server_utils::background_tasks::{autosave_data, check_running_processes, update_duration},
    structures::{config::Config, process::Processes},
};

use super::{
    background_tasks::save_data,
    user_commands::{
        add_new_process, change_config, change_duration, change_process, get_processes,
        import_processes, move_process, remove_processes, set_exit_flag,
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
        Commands::Remove(remove_cmd) => remove_processes(remove_cmd.id, processes).await,
        Commands::Add(add_cmd) => add_new_process(add_cmd, processes).await,
        Commands::Option(config_cmd) => change_config(config_cmd, config).await,
        Commands::Change(change_cmd) => change_process(change_cmd, processes).await,
        Commands::Duration(duration_cmd) => change_duration(duration_cmd, processes).await,
        Commands::Export(export_cmd) => get_processes(export_cmd.ids, processes).await,
        Commands::Import(import_cmd) => import_processes(import_cmd, processes).await,
        Commands::Move(move_cmd) => move_process(move_cmd, processes).await,
        Commands::Quit => set_exit_flag(close_server_flag).await,

        _ => unreachable!(),
    }
    .map_err(|e| e.to_string());

    let serialized = serde_json::to_string(&response).expect("must serialize");

    _ = writer.write_all(serialized.as_bytes()).await;

    if close_server_flag.load(Ordering::Relaxed) {
        save_data(config, processes).await;
        std::process::exit(0)
    }
}
