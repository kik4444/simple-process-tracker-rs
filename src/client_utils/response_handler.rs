use std::path::Path;

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement,
    Table,
};

use crate::{duration_to_string, structures::process::Processes, ACTIVE_ICON, PAUSED_ICON};

pub fn handle_view_command(
    debug: bool,
    processes: Processes,
) -> Result<(), Box<dyn std::error::Error>> {
    if debug {
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

    Ok(())
}

pub fn handle_export_command(
    export_path: &Path,
    processes: Processes,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(export_path)
        .map_err(|e| format!("cannot open file {} -> {e}", export_path.display()))?;

    serde_json::to_writer_pretty(file, &processes)?;

    println!(
        "exported {:?} to {}",
        processes
            .0
            .iter()
            .map(|process| process.name.as_str())
            .collect::<Vec<&str>>(),
        export_path.display()
    );

    Ok(())
}
