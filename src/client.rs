use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement,
    Table,
};
use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::LocalSocketStream;

use crate::{
    commands::Commands,
    duration_to_string, get_socket_name,
    structures::{config::Config, process::Processes},
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

            if let Commands::View(view_cmd) = command {
                if view_cmd.debug {
                    println!("{:#?}", processes.0);
                } else {
                    let mut table = Table::new();

                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_content_arrangement(ContentArrangement::Dynamic);

                    table.set_header([
                        "#",
                        "Tracking",
                        "Running",
                        "Name",
                        "Duration",
                        "Notes",
                        "Last seen",
                        "Date added",
                    ]);

                    for (id, process) in processes.0.iter().enumerate() {
                        let tracking_icon = if process.is_tracked {
                            ACTIVE_ICON
                        } else {
                            PAUSED_ICON
                        };

                        let running_icon = if process.is_running {
                            ACTIVE_ICON
                        } else {
                            PAUSED_ICON
                        };

                        table.add_row([
                            Cell::new(id),
                            Cell::new(tracking_icon).set_alignment(CellAlignment::Center),
                            Cell::new(running_icon).set_alignment(CellAlignment::Center),
                            Cell::new(&process.name),
                            Cell::new(duration_to_string(process.duration)),
                            Cell::new(&process.notes),
                            Cell::new(process.last_seen_date.format("%Y/%m/%d %H:%M:%S")),
                            Cell::new(process.added_date.format("%Y/%m/%d %H:%M:%S")),
                        ]);
                    }

                    println!("{table}");
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
