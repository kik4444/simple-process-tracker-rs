use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct View {
    /// The process IDs to show in 0-3,5,7 format
    pub ids: Option<String>,
    /// Debug print the processes
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Add {
    /// Name of the process to track. If unsure what the process is called, use the "processes" command to view all running processes
    /// with the names that Simple process tracker will use to track them
    pub name: String,
    /// Set the process icon
    #[arg(short, long)]
    pub icon: Option<PathBuf>,
    /// Set the process duration in HH:MM:SS
    #[arg(short, long)]
    pub duration: Option<String>,
    /// Set notes about this process
    #[arg(short, long)]
    pub notes: Option<String>,
    /// Overwrite the date this process was added in YYYY/MM/DD HH:MM:SS format
    #[arg(short, long)]
    pub added_date: Option<String>,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Remove {
    pub id: usize,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Config {
    /// How often to check if the tracked processes are still running in seconds
    #[arg(short, long, value_parser = clap::value_parser!(u64).range(10..))]
    pub poll_interval: Option<u64>,
    /// How often to update the durations for processes that are currently running in seconds
    #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..))]
    pub duration_update_interval: Option<u64>,
    /// How often to autosave in case the program quits unexpectedly in seconds
    #[arg(short, long, value_parser = clap::value_parser!(u64).range(60..))]
    pub autosave_interval: Option<u64>,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Change {
    /// The ID of the process to change
    pub id: usize,
    /// Set whether the process is currently tracked or not
    #[arg(short, long)]
    pub tracking: Option<bool>,
    /// Set the process icon
    #[arg(short, long)]
    pub icon: Option<PathBuf>,
    /// Set the process duration in HH:MM:SS
    #[arg(short, long)]
    pub duration: Option<String>,
    /// Set notes about this process
    #[arg(short, long)]
    pub notes: Option<String>,
    /// Overwrite the date this process was added in YYYY/MM/DD HH:MM:SS format
    #[arg(short, long)]
    pub added_date: Option<String>,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Duration {
    pub id: usize,
    #[command(subcommand)]
    pub operation: DurationCalculation,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Export {
    /// Where to save the exported JSON file
    pub path: PathBuf,
    /// The process IDs to export in 0-3,5,7 format
    pub ids: Option<String>,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Import {
    pub path: PathBuf,
    /// Whether the provided json is from the old Simple process tracker
    #[arg(short, long, default_value_t = false)]
    pub legacy: bool,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Move {
    pub id: usize,
    #[command(subcommand)]
    pub direction: MoveDirection,
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum DurationCalculation {
    /// Add seconds to a process's duration
    Add { seconds: u64 },

    /// Subtract seconds from a process's duration
    Subtract { seconds: u64 },
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum MoveDirection {
    Up,
    Down,
    Top,
    Bottom,
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum Commands {
    /// Launch Simple process tracker and begin tracking selected processes
    Launch,

    /// Show all processes if no IDs are given. Otherwise show the processes with the given IDs.
    /// Example: show 0-3,5,7
    View(View),

    /// List all processes running on the system with the names that Simple process tracker will use to check if they are active
    Processes,

    /// Add a process to track. Optionally add it with specific options set in advance
    Add(Add),

    /// Remove a process given its ID
    Remove(Remove),

    /// Set options for Simple process tracker
    Option(Config),

    /// Change some data about a specific process, such as its duration, notes, icon, etc.
    Change(Change),

    /// Add or subtract seconds from a process's duration
    Duration(Duration),

    /// Export all processes to the given path if no IDs are given.
    /// Otherwise export the processes with the given IDs.
    /// Example: export "./export.json" 0-3,5,7
    Export(Export),

    /// Import processes from the given JSON file. Example: import "./export.json"
    Import(Import),

    /// Update a process's ID to move it up, down, top or bottom
    Move(Move),

    /// Save and close Simple process tracker
    Quit,
}
