use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::LocalSocketStream;

use crate::{
    commands::Commands,
    get_socket_name,
    structures::{config::Config, process::Processes},
};

use super::response_handler::{handle_export_command, handle_view_command};

pub async fn handle_user_command(command: Commands) {
    if let Err(e) = send_command(command).await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

async fn send_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    let conn = LocalSocketStream::connect(get_socket_name())
        .await
        .map_err(|e| format!("server may not be running -> {e}"))?;

    let (reader, mut writer) = conn.into_split();

    let serialized = serde_json::to_string(&command)?;
    writer.write_all(serialized.as_bytes()).await?;

    // We must free the writer otherwise the server cannot respond on the pipe to the client
    drop(writer);

    let mut reader = BufReader::new(reader);
    let mut buffer = String::with_capacity(256);
    reader.read_to_string(&mut buffer).await?;

    let response: Result<String, String> = serde_json::from_str(&buffer)
        .map_err(|e| format!("failed parsing server response -> {e}"))?;

    match command {
        Commands::View(_) | Commands::Export(_) => {
            let processes: Processes = serde_json::from_str(&response?)?;

            match command {
                Commands::View(view_cmd) => handle_view_command(view_cmd.debug, processes)?,
                Commands::Export(export_cmd) => handle_export_command(&export_cmd.path, processes)?,

                _ => unreachable!(),
            };
        }

        Commands::Settings => {
            let config: Config = serde_json::from_str(&response?)?;
            println!("{config:#?}");
        }

        _ => {
            let (Ok(response) | Err(response)) = response;
            println!("{response}");
        }
    };

    Ok(())
}
