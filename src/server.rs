use tokio::sync::RwLock;

use crate::structures::{config::Config, process::Processes};

pub async fn launch() {
    // TODO if server already running exit 1

    // Wrap global config in RwLock to allow multiple readers and one writer
    let config = RwLock::new(Config::read().unwrap_or_default());

    // Leak config so that it may live as long as the server lives and be shared across threads without an Arc
    let config = &*Box::leak(Box::new(config));

    // Do the same for the process list
    let processes = RwLock::new(Processes::read().unwrap_or_default());
    let processes = &*Box::leak(Box::new(processes));

    tokio::spawn(async move { update_duration(config, processes).await });

    tokio::spawn(async move { check_running_processes(config, processes).await });

    tokio::spawn(async move { get_user_command(config, processes).await });
}

async fn update_duration(config: &RwLock<Config>, processes: &RwLock<Processes>) {
    todo!()
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
