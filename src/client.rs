use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::LocalSocketStream;

use crate::{
    commands::Commands, duration_to_string, get_socket_name, structures::process::Processes,
    ACTIVE_ICON, PAUSED_ICON,
};

pub async fn send_command(command: Commands) {
    let Ok(conn) = LocalSocketStream::connect(get_socket_name()).await
    else {
        eprintln!("server is not running");
        std::process::exit(1);
    };

    let (reader, mut writer) = conn.into_split();

    let serialized = serde_json::to_string(&command).expect("cannot fail");
    writer
        .write_all(serialized.as_bytes())
        .await
        .expect("failed writing to server");

    // We must free the writer otherwise the server cannot respond on the pipe to the client
    drop(writer);

    let mut reader = BufReader::new(reader);
    let mut buffer = String::with_capacity(256);
    reader
        .read_to_string(&mut buffer)
        .await
        .expect("failed getting response");

    let response: Result<String, String> = serde_json::from_str(&buffer).expect("must parse");

    if let Commands::Show { debug, .. } = command {
        let processes: Processes =
            serde_json::from_str(&response.expect("always Ok")).expect("must parse");

        if debug {
            println!("{:#?}", processes.0);
        } else {
            println!("# | Tracking | Icon | Name | Duration | Notes | Last seen | Date added");
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
    } else {
        let (Ok(response) | Err(response)) = response;
        println!("{}", response);
    }
}
