use futures_lite::{io::BufReader, AsyncReadExt, AsyncWriteExt};
use interprocess::local_socket::tokio::LocalSocketStream;

use crate::{commands::Commands, get_socket_name, structures::process::Processes};

pub async fn send_command(command: Commands) {
    let conn = LocalSocketStream::connect(get_socket_name())
        .await
        .expect("could not connect to socket");

    let (reader, mut writer) = conn.into_split();

    let serialized = serde_json::to_string::<Commands>(&command).expect("cannot fail");
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

    if let Commands::Show { debug, .. } = command {
        let processes: Processes = serde_json::from_str(&buffer).expect("must parse");

        if debug {
            println!("{:#?}", processes.0);
        } else {
            // TODO formatted print with sixel
        }
    }
}
