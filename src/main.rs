#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{File};
use std::io::{Seek, SeekFrom};
use anyhow::{Result as AnyResult, Context as _};

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("Mark Failed")
                .set_description(&format!("Failed to mark traffic slots: {:#}", e))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();

            std::process::exit(1);
        },
    }
}

fn run() -> AnyResult<()> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .open("cfg/entry_list.ini")
        .context("Failed to open entry list")?;

    let mut ini = ini::Ini::read_from(&mut file)
        .context("Failed to parse entry list")?;

    let mut num_marked = 0;

    for (section, keys) in ini.iter_mut() {
        if let Some(section) = section {
            if section.starts_with("CAR_") {
                if keys.get("AI").is_some() {
                    continue;
                }

                let mut mark = None;

                if let Some(model) = keys.get("MODEL") {
                    let model_lower = model.to_ascii_lowercase();
                    if model_lower.contains("traffic") {
                        mark = Some("fixed");
                    }
                }

                if let Some(team) = keys.get("TEAM") {
                    let team_lower = team.to_ascii_lowercase();

                    if team_lower.contains("traffic") {
                        mark = Some("fixed");
                    }

                    if team_lower.contains("reserve") {
                        mark = Some("auto");
                    }
                }

                if let Some(mark) = mark {
                    keys.insert("AI", mark);
                    num_marked += 1;
                }
            }
        }
    }

    if num_marked > 0 {
        file.seek(SeekFrom::Start(0))
            .context("Failed to seek to start of entry list")?;

        ini.write_to(&mut file)
            .context("Failed to write updated entry list")?;

        let length = file.stream_position().context("Failed to get updated entry list length")?;
        file.set_len(length).context("Failed to update entry list length")?;

        rfd::MessageDialog::new()
            .set_title("Traffic Marked")
            .set_description(&format!("{} car(s) were marked as traffic-capable.", num_marked))
            .set_level(rfd::MessageLevel::Info)
            .show();
    } else {
        rfd::MessageDialog::new()
            .set_title("No Changes")
            .set_description("No cars found that were eligible to be marked as traffic. Entry list was not modified.")
            .set_level(rfd::MessageLevel::Warning)
            .show();
    }

    Ok(())
}