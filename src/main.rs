#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;

use std::borrow::Cow;
use std::fs::{File};
use std::io::{ErrorKind as IoErrorKind, Seek, SeekFrom};
use std::path::{Path};
use std::process::{ExitCode};

use anyhow::{Result as AnyResult, Context as _};
use rfd::MessageLevel;
use time::Time;

use crate::config::{AppConfig};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,

        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("ASCH Error")
                .set_description(&format!("Failed to apply config updates: {:#}", e))
                .set_level(rfd::MessageLevel::Error)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();

            ExitCode::FAILURE
        },
    }
}

fn run() -> AnyResult<()> {
    let config_path = Path::new("cfg/asch.yml");

    let config_raw = match std::fs::read(config_path) {
        Ok(r) => Cow::Owned(r),

        Err(e) if e.kind() == IoErrorKind::NotFound => {
            eprintln!("Configuration file not found at <{}>. A default configuration will be created.", config_path.display());

            let config_raw = include_bytes!("default.yaml");
            std::fs::write(config_path, config_raw)
                .with_context(|| format!("Failed to write default configuration file to <{}>", config_path.display()))?;

            Cow::Borrowed(config_raw.as_slice())
        },

        Err(e) => return Err(e).context(format!("Failed to load configuration from <{}>", config_path.display())),
    };

    let config: AppConfig = serde_yaml::from_slice(&config_raw)
        .with_context(|| format!("Failed to parse configuration from <{}>", config_path.display()))?;

    let results = [
        config.mark_traffic.then(mark_traffic),
        config.time.map(update_time),
    ];

    let level = if results.iter().flat_map(|r| r).any(Result::is_err) {
        MessageLevel::Error
    } else {
        MessageLevel::Info
    };

    let message = results.into_iter()
        .flat_map(|r| r)
        .map(|r| r.unwrap_or_else(|e| Cow::Owned(format!("{}", e))))
        .filter(|r| !r.is_empty())
        .fold(String::new(), |mut s, r| {
            if !s.is_empty() {
                s.push_str("\n\n");
            }

            s.push_str(&r);
            s
        });

    let message = if message.is_empty() {
        Cow::Borrowed("There are no configured actions to perform.")
    } else {
        Cow::Owned(message)
    };

    rfd::MessageDialog::new()
        .set_title("ASCH Finished")
        .set_description(&message)
        .set_level(level)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();

    Ok(())
}

fn mark_traffic() -> AnyResult<Cow<'static, str>> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .open("cfg/entry_list.ini")
        .context("Failed to open entry list to mark traffic slots")?;

    let mut ini = ini::Ini::read_from(&mut file)
        .context("Failed to parse entry list to mark traffic slots")?;

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
            .context("Failed to seek to start of entry list to write updated entry list")?;

        ini.write_to(&mut file)
            .context("Failed to write updated entry list")?;

        let length = file.stream_position().context("Failed to get updated entry list length")?;
        file.set_len(length).context("Failed to update entry list length")?;

        Ok(Cow::Owned(format!("{} car(s) were marked as traffic-capable.", num_marked)))
    } else {
        Ok(Cow::Borrowed("No cars found that were eligible to be marked as traffic. Entry list was not modified."))
    }
}

fn update_time(time: Time) -> AnyResult<Cow<'static, str>> {
    let path = Path::new("cfg/server_cfg.ini");

    let mut file = File::options()
        .read(true)
        .write(true)
        .open(path)
        .context("Failed to open server config to update time")?;

    let mut ini = ini::Ini::read_from(&mut file)
        .context("Failed to parse server config to update time")?;

    let old_angle = ini.get_from_mut(Some("SERVER"), "SUN_ANGLE")
        .context("Server config does not have a configured sun angle.")?
        .parse::<f64>()
        .context("Failed to parse sun angle from server config")?;
    let new_angle = sun_angle_from_time(time);

    let difference = (new_angle - old_angle + 180.0).rem_euclid(360.0) - 180.0;

    if difference.abs() < 0.05 {
        return Ok(Cow::Borrowed("Configured time of day matches the configured time. Server config was not modified."));
    }

    ini.set_to(Some("SERVER"), String::from("SUN_ANGLE"), format!("{:.3}", new_angle));

    file.seek(SeekFrom::Start(0))
        .context("Failed to seek to start of server config to write updated server config")?;

    ini.write_to(&mut file)
        .context("Failed to write updated server config")?;

    let length = file.stream_position().context("Failed to get updated server config length")?;
    file.set_len(length).context("Failed to update server config list length")?;

    Ok(Cow::Borrowed("Time of day was updated in server config."))
}

fn sun_angle_from_time(time: Time) -> f64 {
    let t = (time - time::macros::time!(13:00))
        .as_seconds_f64()
        .rem_euclid(24.0 * 60.0 * 60.0);

    let a = t * (360.0 / (24.0 * 3600.0));
    (a + 180.0) % 360.0 - 180.0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sun_angle_from_time() {
        // NOTE: This doesn't match what CM produces, but it's mathematically consistent
        assert_eq!(sun_angle_from_time(time::macros::time!(00:00)),  165.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(01:00)), -180.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(02:00)), -165.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(03:00)), -150.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(04:00)), -135.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(05:00)), -120.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(06:00)), -105.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(07:00)), -90.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(08:00)), -75.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(09:00)), -60.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(10:00)), -45.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(11:00)), -30.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(12:00)), -15.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(13:00)), 0.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(14:00)), 15.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(15:00)), 30.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(16:00)), 45.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(17:00)), 60.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(18:00)), 75.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(19:00)), 90.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(20:00)), 105.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(21:00)), 120.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(22:00)), 135.0);
        assert_eq!(sun_angle_from_time(time::macros::time!(23:00)), 150.0);
    }
}
