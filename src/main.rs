use clap::Parser;
use simple_process_tracker_rs::Commands;

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

    // https://github.com/kotauskas/interprocess/blob/main/src/local_socket/tokio/listener.rs
    // https://github.com/kotauskas/interprocess/blob/main/src/local_socket/tokio/stream/mod.rs

    match args.command {
        Commands::Launch => println!("Start everything"),
        Commands::Processes => println!("List processes"),
        cmd => println!("{:#?}", cmd),
    }
}
