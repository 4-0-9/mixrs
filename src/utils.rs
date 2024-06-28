use std::process::Command;

use anyhow::{anyhow, Result};
use pulse::{context::introspect::SinkInputInfo, volume};

#[link(name = "c")]
extern "C" {
    /// Gets the current user's ID
    pub fn getuid() -> u32;
}

const NOTIFY_SEND_REPLACE_ID: u32 = 1448531;
const NOTIFICATION_DURATION_MILLIS: u32 = 1000;
const FULL_VOLUME: u32 = 1 << 16;

pub fn volume_to_percentage(volume: volume::ChannelVolumes) -> u8 {
    let average = volume.avg().0;

    total_volume_to_percentage(average)
}

pub fn total_volume_to_percentage(volume: u32) -> u8 {
    ((volume as f32 / FULL_VOLUME as f32) * 100.0).round() as u8
}

pub fn percentage_to_total_volume(percentage: u8) -> u32 {
    ((FULL_VOLUME as f32 / 100.0) * percentage as f32).round() as u32
}

pub fn get_sink_input_name(sink_input: &SinkInputInfo) -> anyhow::Result<String> {
    let Some(name_bytes) = sink_input.proplist.get("application.name") else {
        return Err(anyhow!("Invalid sink input name"));
    };

    Ok(capitalize_string(&String::from_utf8(
        name_bytes[..name_bytes.len() - 1].to_vec(),
    )?))
}

fn capitalize_string(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn send_notification(message: &str) -> Result<()> {
    let user_id = unsafe { getuid() };

    Command::new("notify-send")
        .args(vec![
            "Mixrs",
            message,
            "-t",
            &NOTIFICATION_DURATION_MILLIS.to_string(),
            "-r",
            &NOTIFY_SEND_REPLACE_ID.to_string(),
            "-i",
            "/",
        ])
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path=/run/user/{user_id}/bus"),
        )
        .spawn()?;

    Ok(())
}
