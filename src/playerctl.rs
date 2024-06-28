use std::process::Command;

use anyhow::{anyhow, Result};

fn get_playerctl_player(target: &str) -> Result<String> {
    let get_players = Command::new("playerctl").arg("-l").output()?;
    let get_players_output = String::from_utf8(get_players.stdout)?;
    let players: Vec<&str> = get_players_output.split("\n").collect();

    let Some(player) = players
        .iter()
        .find(|p| p.to_lowercase().contains(&target.to_lowercase()))
    else {
        return Err(anyhow!("Error getting player '{target}'"));
    };

    Ok(player.to_string())
}

pub fn playerctl_play_pause(target: &str) -> Result<()> {
    let player = get_playerctl_player(target)?;

    Command::new("playerctl")
        .args(vec!["-p", &player, "play-pause"])
        .spawn()?;

    Ok(())
}

pub fn playerctl_next(target: &str) -> Result<()> {
    let player = get_playerctl_player(target)?;

    Command::new("playerctl")
        .args(vec!["-p", &player, "next"])
        .spawn()?;

    Ok(())
}

pub fn playerctl_previous(target: &str) -> Result<()> {
    let player = get_playerctl_player(target)?;

    Command::new("playerctl")
        .args(vec!["-p", &player, "previous"])
        .spawn()?;

    Ok(())
}
