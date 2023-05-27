# Simple process tracker
A simple CLI app to track how long user-selected processes have been running on your system. Works both as a server that runs in the background and a client that communicates with it via a local OS pipe.

# Example usage
## List commands
```console
simple-process-tracker-cli --help
A simple CLI app to track how long user-selected processes have been running on your system

Usage: simple-process-tracker-cli [COMMAND]

Commands:
  launch     Launch Simple process tracker and begin tracking selected processes
  processes  List all processes running on the system with the names that Simple process tracker will use to check if they are active
  add        Add a process to track. Optionally add it with specific options set in advance
  view       Show all processes if no IDs are given. Otherwise show the processes with the given IDs. Example: show 0-3,5,7
  change     Change some data about a specific process, such as its duration, notes, icon, etc
  duration   Add or subtract seconds from a process's duration
  remove     Remove a process given its ID
  move       Update a process's ID to move it up, down, top or bottom
  export     Export all processes to the given path if no IDs are given. Otherwise export the processes with the given IDs. Example: export "./export.json" 0-3,5,7
  import     Import processes from the given JSON file. Example: import "./export.json"
  option     Set options for Simple process tracker
  settings   Show Simple process tracker's configuration
  quit       Save and close Simple process tracker
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
## Launch server
You must launch Simple process tracker as a server first to be able to interact with it.
```console
simple-process-tracker-cli launch
```
This will start the server in the current terminal. It is recommended to start the server in the background so as not to accidentally stop it. For example `simple-process-tracker-cli launch & disown` for Bash or `simple-process-tracker-cli launch &|` for Zsh.
## Show currently running processes
This will show you the names of the currently running processes on the system. When adding a process to track, you will need to check its name with this command
```console
simple-process-tracker-cli processes
strawberry
mpv
firefox
...
```
## Add a process to track
Now that the server is running in the background you can use any of the other client commands to communicate with it. You can start tracking processes like this.
```console
simple-process-tracker-cli add mpv
simple-process-tracker-cli add strawberry
```
## View the current status of tracked processes
```console
simple-process-tracker-cli view
╭───┬──────────┬─────────┬────────────┬────────────┬─────────────────────────┬─────────────────────┬─────────────────────╮
│ # ┆ Tracking ┆ Running ┆    Name    ┆  Duration  ┆          Notes          ┆      Last seen      ┆      Date added     │
╞═══╪══════════╪═════════╪════════════╪════════════╪═════════════════════════╪═════════════════════╪═════════════════════╡
│ 0 ┆    ✅    ┆    ❌   ┆     mpv    ┆ 1409:32:00 ┆     MPV video player    ┆ 2023/05/27 16:43:20 ┆ 2023/05/27 16:42:03 │
├╌╌╌┼╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 1 ┆    ✅    ┆    ✅   ┆ strawberry ┆  724:25:34 ┆ Strawberry music player ┆ 2023/05/27 16:43:35 ┆ 2023/05/27 16:42:50 │
╰───┴──────────┴─────────┴────────────┴────────────┴─────────────────────────┴─────────────────────┴─────────────────────╯
```
## Other commands
Write `simple-process-tracker-cli [COMMAND] --help` to get more info about the other commands
## Shortening commands
Simple process tracker will infer the name of the command without writing all of it. For example `simple-process-tracker-cli l` -> `simple-process-tracker-cli launch`, `simple-process-tracker-cli d` -> `simple-process-tracker-cli duration` and so on. Also, invoking `simple-process-tracker-cli` without an argument will default to the `view` command.

# Building
Install Rust for your operating system in order to compile this program. More info at https://rustup.rs/. Then run
```bash
git clone https://github.com/kik4444/simple-process-tracker-rs
cd simple-process-tracker-rs
CARGO_TARGET_DIR=target RUSTFLAGS="-C target-cpu=native" cargo build --release
```
The compiled executable will now be in `./target/release/simple-process-tracker-cli` Place it anywhere you wish
