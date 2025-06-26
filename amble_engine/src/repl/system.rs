//! `repl::system` module
//!
//! Contains repl loop handlers for commands that are for system utilities.

use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::spinners::default_spinners;
use crate::style::GameStyle;
use crate::{AmbleWorld, WorldObject, repl::ReplControl, spinners::SpinnerType};

use anyhow::{Context, Result};
use log::info;

/// Quit the game.
pub fn quit_handler(world: &AmbleWorld) -> Result<ReplControl> {
    info!(
        "{} quit with a score of {}",
        world.player.name(),
        world.player.score
    );
    info!("ending achievements:");
    world
        .player
        .achievements
        .iter()
        .for_each(|i| info!("* {i}"));
    info!("ending inventory:");
    world
        .player
        .inventory
        .iter()
        .filter_map(|uuid| world.items.get(uuid))
        .for_each(|i| info!("- {} ({})", i.name(), i.id()));

    println!("{}", world.spin_spinner(SpinnerType::QuitMsg, "Goodbye."));
    Ok(ReplControl::Quit)
}

/// Show available commands.
pub fn help_handler() -> Result<()> {
    println!(
        r"
Available commands:
  look
  look at <item>
  go/move <direction>
  inventory/inv
  take <item>
  drop <item>
  put <item> in <container>
  take <item> from <container>
  open <container>
  close <container>
  lock <container>
  unlock <container>
  read <item>
  turn <item> on
  talk to <npc>
  give <item> to <npc>
  help
  quit
"
    );
    Ok(())
}

/// load game from a file
pub fn load_handler(world: &mut AmbleWorld, gamefile: &str) -> Result<()> {
    let load_path = PathBuf::from("saved_games").join(format!("amble-{gamefile}.ron"));
    if let Ok(world_ron) = fs::read_to_string(load_path.as_path()) {
        if let Ok(new_world) = ron::from_str::<AmbleWorld>(&world_ron) {
            *world = new_world;
            world.spinners = default_spinners();
            println!(
                "Saved game {} loaded successfully. Sally forth.",
                gamefile.underline().green()
            );
            info!("Player reloaded AmbleWorld from file ({load_path:?})")
        } else {
            println!(
                "Unable to parse the {} save. World structure may have changed since it was created.",
                gamefile.error_style()
            );
        }
    } else {
        println!(
            "Unable to find {} save file. Load aborted.",
            gamefile.error_style()
        );
    }
    Ok(())
}

/// save game to a file
pub fn save_handler(world: &AmbleWorld, gamefile: &str) -> Result<()> {
    // serialize the current AmbleWorld state to RON format
    let world_ron = ron::ser::to_string(world)
        .with_context(|| "error converting AmbleWorld to 'ron' format".to_string())?;

    // create save dir if doesn't exist
    fs::create_dir_all("saved_games")
        .with_context(|| "error creating saved_games folder".to_string())?;

    // create save file
    let save_path = PathBuf::from("saved_games").join(format!("amble-{gamefile}.ron"));
    let mut save_file = fs::File::create(save_path.as_path())
        .with_context(|| format!("creating file {save_path:?}"))?;

    // write world to file
    save_file
        .write_all(world_ron.as_bytes())
        .with_context(|| "failed to write AmbleWorld to .ron file".to_string())?;

    // disco!
    println!("Game saved as {}", gamefile.underline());
    info!("Player saved game to \"{}\"", gamefile);
    Ok(())
}
