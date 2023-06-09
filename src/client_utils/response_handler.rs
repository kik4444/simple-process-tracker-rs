use std::path::Path;

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, ContentArrangement,
    Table,
};

use crate::{duration_to_string, structures::process::Process, ACTIVE_ICON, PAUSED_ICON};

pub fn handle_view_command(
    debug: bool,
    processes: Vec<(usize, Process)>,
) -> Result<(), Box<dyn std::error::Error>> {
    if debug {
        println!("{:#?}", processes);
    } else {
        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header([
            Cell::new("#").set_alignment(CellAlignment::Center),
            Cell::new("Tracking").set_alignment(CellAlignment::Center),
            Cell::new("Running").set_alignment(CellAlignment::Center),
            Cell::new("Name").set_alignment(CellAlignment::Center),
            Cell::new("Duration").set_alignment(CellAlignment::Center),
            Cell::new("Notes").set_alignment(CellAlignment::Center),
            Cell::new("Last seen").set_alignment(CellAlignment::Center),
            Cell::new("Date added").set_alignment(CellAlignment::Center),
        ]);

        for (id, process) in processes {
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
                Cell::new(&process.name).set_alignment(CellAlignment::Center),
                Cell::new(duration_to_string(process.duration))
                    .set_alignment(CellAlignment::Center),
                Cell::new(&process.notes).set_alignment(CellAlignment::Center),
                Cell::new(process.last_seen_date.format("%Y/%m/%d %H:%M:%S"))
                    .set_alignment(CellAlignment::Center),
                Cell::new(process.added_date.format("%Y/%m/%d %H:%M:%S"))
                    .set_alignment(CellAlignment::Center),
            ]);
        }

        println!("{table}");
    }

    Ok(())
}

pub fn handle_export_command(
    export_path: &Path,
    processes: Vec<(usize, Process)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(export_path)
        .map_err(|e| format!("cannot open file {} -> {e}", export_path.display()))?;

    let processes: Vec<Process> = processes.into_iter().map(|(_, process)| process).collect();

    serde_json::to_writer_pretty(file, &processes)?;

    println!(
        "exported {:?} to {}",
        processes
            .iter()
            .map(|process| process.name.as_str())
            .collect::<Vec<&str>>(),
        export_path.display()
    );

    Ok(())
}
