use clap::Parser;
use simple_process_tracker_rs::{
    client, commands::Commands, process_scanner::get_running_processes, server_utils::server,
};

#[derive(Parser, Debug)]
#[command(
    about = "A simple CLI app to track how long user-selected processes have been running on your system",
    long_about = None,
    version
)]
#[command(infer_subcommands = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Launch => server::launch().await,
        Commands::Processes => show_processes().await,
        cmd => client::handle_user_command(cmd).await,
    }
}

async fn show_processes() {
    match get_running_processes().await {
        Ok(process_list) => {
            let mut sorted_process_list: Vec<&str> =
                process_list.iter().map(|name| name.as_str()).collect();

            sorted_process_list.sort_by_key(|name| name.to_lowercase());

            for process in sorted_process_list {
                println!("{process}");
            }
        }
        Err(e) => eprintln!("{e}"),
    }
}
