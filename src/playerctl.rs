use std::process::Command;

use anyhow::{anyhow, Result};
use simsearch::{SearchOptions, SimSearch};

fn get_playerctl_player(target: &str) -> Result<String> {
    let get_players = Command::new("playerctl").arg("-l").output()?;
    let get_players_output = String::from_utf8(get_players.stdout)?;
    let players: Vec<&str> = get_players_output.split("\n").filter(|s| !s.is_empty()).collect();

    let search_options = SearchOptions::new().threshold(0.8);
    let mut engine: SimSearch<u32> = SimSearch::new_with(search_options);

    for (i, p) in players.iter().enumerate() {
        engine.insert(i as u32, p);
    }

    let results: Vec<u32> = engine.search(target);

    let Some(player_index) = results.get(0) else {
        return Err(anyhow!("Error getting player '{target}'"));
    };

    Ok(players.iter().nth(*player_index as usize).unwrap().to_string())
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
