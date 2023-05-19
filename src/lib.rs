use std::path::PathBuf;

use interprocess::local_socket::NameTypeSupport;

pub mod commands;
pub mod errors;
pub mod process_scanner;
pub mod server;
pub mod structures;

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("simple-process-tracker"))
}

pub fn get_socket_name() -> &'static str {
    use NameTypeSupport::*;

    match NameTypeSupport::query() {
        OnlyPaths => "/tmp/simple-process-tracker.sock",
        OnlyNamespaced | Both => "@simple-process-tracker.sock",
    }
}
