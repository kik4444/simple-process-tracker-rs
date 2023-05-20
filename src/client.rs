use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::LocalSocketStream;

use crate::{
    commands::Commands, duration_to_string, get_socket_name, structures::process::Processes,
    ACTIVE_ICON, PAUSED_ICON,
};

pub async fn handle_user_command(command: Commands) {
    if let Err(e) = send_command(command).await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

pub async fn send_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    let conn = LocalSocketStream::connect(get_socket_name())
        .await
        .map_err(|e| format!("failed connecting to the socket -> {e}"))?;

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

            if let Commands::View(view_cmd) = command {
                if view_cmd.debug {
                    println!("{:#?}", processes.0);
                } else {
                    println!(
                        "# | Tracking | Icon | Name | Duration | Notes | Last seen | Date added"
                    );
                    for (id, process) in processes.0.iter().enumerate() {
                        let tracking_icon = if process.is_tracked {
                            ACTIVE_ICON
                        } else {
                            PAUSED_ICON
                        };

                        // let icon_image = get_sixel(&process.icon);

                        println!(
                            "{} | {} | {} | {} | {} | {} | {} | {}",
                            id,
                            tracking_icon,
                            "TODO",
                            process.name,
                            duration_to_string(process.duration),
                            process.notes,
                            process.last_seen_date.format("%Y/%m/%d %H:%M:%S"),
                            process.added_date.format("%Y/%m/%d %H:%M:%S")
                        );
                    }
                }
            } else if let Commands::Export(export_cmd) = command {
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&export_cmd.path)
                    .map_err(|e| {
                        format!("cannot open file {} -> {e}", export_cmd.path.display())
                    })?;

                serde_json::to_writer_pretty(file, &processes)?;

                println!(
                    "exported {:?} to {}",
                    processes
                        .0
                        .iter()
                        .map(|process| process.name.as_str())
                        .collect::<Vec<&str>>(),
                    export_cmd.path.display()
                )
            }
        }
        _ => {
            let (Ok(response) | Err(response)) = response;
            println!("{response}");
        }
    };

    Ok(())
}
