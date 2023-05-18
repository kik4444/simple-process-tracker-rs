use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum Commands {
    /// Launch Simple process tracker and begin tracking selected processes
    Launch,

    /// Show all processes if no arg is given. If an arg is given, show the process with that ID
    Show { id: Option<usize> },

    /// List all processes running on the system with the names that Simple process tracker will use to check if they are active
    Processes,

    /// Add a process to track. Optionally add it with specific options set in advance
    Add {
        /// Name of the process to track. If unsure what the process is called, use the "processes" command to view all running processes
        /// with the names that Simple process tracker will use to track them
        name: String,
        /// Set the process icon
        #[arg(short, long)]
        icon: Option<PathBuf>,
        /// Set the process duration in HH:MM:SS
        #[arg(short, long)]
        duration: Option<String>,
        /// Set notes about this process
        #[arg(short, long)]
        notes: Option<String>,
        /// Overwrite the date this process was added in YYYY/MM/DD HH:MM:SS format
        #[arg(short, long)]
        added_date: Option<NaiveDateTime>,
    },

    /// Set options for Simple process tracker
    Option {
        /// How often to check if the tracked processes are still running in seconds
        #[arg(short, long)]
        poll_interval: Option<u64>,
        /// How often to update the durations for processes that are currently running in seconds
        #[arg(short, long)]
        duration_update_interval: Option<u64>,
        /// How often to autosave in case the program quits unexpectedly in seconds
        #[arg(short, long)]
        autosave_interval: Option<u64>,
    },

    /// Change some data about a specific process, such as its duration, notes, icon, etc.
    Change {
        /// The ID of the process to change
        id: usize,
        /// Set whether the process is currently tracked or not
        #[arg(short, long)]
        tracking: Option<bool>,
        /// Set the process icon
        #[arg(short, long)]
        icon: Option<PathBuf>,
        /// Set the process duration in HH:MM:SS
        #[arg(short, long)]
        duration: Option<String>,
        /// Set notes about this process
        #[arg(short, long)]
        notes: Option<String>,
        /// Overwrite the date this process was added in YYYY/MM/DD HH:MM:SS format
        #[arg(short, long)]
        added_date: Option<NaiveDateTime>,
    },

    /// Add or subtract seconds from a process's duration
    Duration {
        #[command(subcommand)]
        command: DurationCalculation,
    },

    /// Export all processes to the given path if no IDs are given.
    /// Otherwise export the processes with the given IDs.
    /// Example: export "./export.json" 0-3,5,7
    Export {
        /// Where to save the exported JSON file
        path: PathBuf,
        /// The process IDs to export in 0-3,5,7 format
        ids: Option<String>,
    },

    /// Import processes from the given JSON file. Example: import "./export.json"
    Import { path: PathBuf },

    /// Update a process's ID to move it up, down, top or bottom
    Move {
        #[command(subcommand)]
        command: MoveDirection,
    },

    /// Save and close Simple process tracker
    Quit,
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum DurationCalculation {
    /// Add seconds to a process's duration
    Add { seconds: u64, id: usize },

    /// Subtract seconds from a process's duration
    Subtract { seconds: u64, id: usize },
}

#[derive(Debug, Subcommand, Serialize, Deserialize)]
pub enum MoveDirection {
    Up { id: usize },
    Down { id: usize },
    Top { id: usize },
    Bottom { id: usize },
}
