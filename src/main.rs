mod instructions;
pub mod mixer;
pub mod pulseaudio;
pub mod utils;

use anyhow::Result;
use mixer::Mixer;
use pulseaudio::PulseInstruction;
use std::{process::Command, sync::mpsc::channel};

const NOTIFY_SEND_REPLACE_ID: u32 = 1448531;

#[tokio::main]
async fn main() {
    let mainloop = pulse::mainloop::standard::Mainloop::new().expect("Error getting main loop");

    let (pulse_ix_tx, pulse_ix_rx) = channel::<PulseInstruction>();

    let mut mixer = Mixer::new(mainloop, pulse_ix_tx);

    mixer.run(pulse_ix_rx);
}

pub fn send_notification(message: &str) -> Result<()> {
    Command::new("notify-send")
        .args(vec![
            "Mixrs",
            message,
            "-r",
            &NOTIFY_SEND_REPLACE_ID.to_string(),
        ])
        .env("DBUS_SESSION_BUS_ADDRESS", "unix:path=/run/user/1000/bus")
        .spawn()?;

    Ok(())
}

pub fn playerctl_toggle(target: &str) -> Result<()> {
    let get_players = Command::new("playerctl").arg("-l").output()?;
    let get_players_output = String::from_utf8(get_players.stdout)?;
    let players: Vec<&str> = get_players_output.split("\n").collect();

    match players.iter().find(|p| p.to_lowercase().contains(&target.to_lowercase())) {
        Some(player) => {
            Command::new("playerctl").args(vec!["-p", player, "play-pause"]).spawn()?;
        },
        None => {},
    };
    Ok(())
}
