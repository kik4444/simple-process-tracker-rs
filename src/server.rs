use std::time::Duration;

use interprocess::local_socket::tokio::LocalSocketListener;
use tokio::sync::RwLock;

use crate::{
    get_socket_name,
    structures::{config::Config, process::Processes},
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

    tokio::spawn(async move { get_user_command(config, processes).await });
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
    todo!()
}

async fn get_user_command(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    todo!()
}

async fn handle_user_command(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    todo!()
}
