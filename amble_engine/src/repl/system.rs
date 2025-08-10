//! `repl::system` module
//!
//! Contains repl loop handlers for commands that are for system utilities.

use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::goal::GoalStatus;
use crate::style::GameStyle;

use crate::{AMBLE_VERSION, Goal, View, ViewItem};
use crate::{AmbleWorld, WorldObject, repl::ReplControl};

use anyhow::{Context, Result};
use log::{info, warn};

/// Quit the game.
pub fn quit_handler(world: &AmbleWorld, view: &mut View) -> Result<ReplControl> {
    info!("{} quit with a score of {}", world.player.name(), world.player.score);
    info!("ending flags:");
    world.player.flags.iter().for_each(|i| info!("* {i}"));
    info!("ending inventory:");
    world
        .player
        .inventory
        .iter()
        .filter_map(|uuid| world.items.get(uuid))
        .for_each(|i| info!("- {} ({})", i.name(), i.id()));

    let percent = (world.player.score as f32 / world.max_score as f32) * 100.0;

    let (rank, eval) = match percent {
        p if p == 100.0 => (
            "Quantum Overachiever",
            "You saw the multiverse, understood it, then filed a bug report.",
        ),
        p if p >= 90.0 => (
            "Senior Field Operative",
            "A nearly flawless run. Someone give this candidate a promotion.",
        ),
        p if p >= 75.0 => (
            "Licensed Reality Bender",
            "Impressive grasp of nonlinear environments and cake-based paradoxes.",
        ),
        p if p >= 60.0 => (
            "Rogue Intern, Level II",
            "You got the job done, and only melted one small pocket universe.",
        ),
        p if p >= 45.0 => (
            "Unpaid Research Assistant",
            "Solid effort. Some concepts may have slipped through dimensional cracks.",
        ),
        p if p >= 30.0 => (
            "Junior Sandwich Technician",
            "Good instincts, questionable execution. Especially with condiments.",
        ),
        p if p >= 15.0 => (
            "Volunteer Tour Guide",
            "You wandered. You looked at stuff. It was something.",
        ),
        p if p >= 5.0 => (
            "Mailbox Stuffing Trainee",
            "You opened a box, tripped on a rug, and called it a day.",
        ),
        p if p >= 1.0 => (
            "Accidental Hire",
            "We're not sure how you got in. Please return your lanyard.",
        ),
        _ => ("Amnesiac Test Subject", "Did you… play? Were you even awake?"),
    };

    let visited = world.rooms.values().filter(|r| r.visited).count();

    view.push(ViewItem::QuitSummary {
        rank: rank.to_string(),
        notes: eval.to_string(),
        score: world.player.score,
        max_score: world.max_score,
        visited,
        max_visited: world.rooms.len(),
    });

    Ok(ReplControl::Quit)
}

/// Show available commands.
pub fn help_handler(view: &mut View) {
    view.push(ViewItem::Help);
}

/// Show current game game goals / status.
pub fn goals_handler(world: &AmbleWorld, view: &mut View) {
    filtered_goals(world, GoalStatus::Active).iter().for_each(|goal| {
        view.push(ViewItem::ActiveGoal {
            name: goal.name.clone(),
            description: goal.description.clone(),
        })
    });

    filtered_goals(world, GoalStatus::Complete).iter().for_each(|goal| {
        view.push(ViewItem::CompleteGoal {
            name: goal.name.clone(),
            description: goal.description.clone(),
        })
    });
}

/// Returns a list of game `Goals`, filtered by status
pub fn filtered_goals(world: &AmbleWorld, status: GoalStatus) -> Vec<&Goal> {
    world.goals.iter().filter(|goal| goal.status(world) == status).collect()
}

/// Loads a saved game.
///
/// # Errors
/// - on save file not found or RON parsing error.
pub fn load_handler(world: &mut AmbleWorld, gamefile: &str) {
    let load_path = PathBuf::from("saved_games").join(format!("amble-{gamefile}.ron"));
    if let Ok(world_ron) = fs::read_to_string(load_path.as_path()) {
        if let Ok(new_world) = ron::from_str::<AmbleWorld>(&world_ron) {
            if new_world.version != AMBLE_VERSION {
                warn!(
                    "player loaded '{gamefile}' (v{}), current version is v{AMBLE_VERSION}",
                    new_world.version
                );
                println!(
                    "{}: '{gamefile}' version is v{} -- does not match current game (v{AMBLE_VERSION}).",
                    "WARNING".bold().yellow(),
                    new_world.version.error_style(),
                );
            }
            *world = new_world;
            println!(
                "Saved game {} loaded successfully. Sally forth.",
                gamefile.underline().green()
            );
            info!("Player reloaded AmbleWorld from file '{}'", load_path.display());
        } else {
            println!(
                "Unable to load the {} save file. The Amble engine may have changed since it was created.",
                gamefile.error_style()
            );
            warn!("player attempted to load '{gamefile}': failed to parse, likely version conflict");
        }
    } else {
        println!("Unable to find {} save file. Load aborted.", gamefile.error_style());
    }
}

/// save game to a file
pub fn save_handler(world: &AmbleWorld, gamefile: &str) -> Result<()> {
    // serialize the current AmbleWorld state to RON format
    let world_ron =
        ron::ser::to_string(world).with_context(|| "error converting AmbleWorld to 'ron' format".to_string())?;

    // create save dir if doesn't exist
    fs::create_dir_all("saved_games").with_context(|| "error creating saved_games folder".to_string())?;

    // create save file
    let save_path = PathBuf::from("saved_games").join(format!("amble-{gamefile}.ron"));
    let mut save_file =
        fs::File::create(save_path.as_path()).with_context(|| format!("creating file '{}'", save_path.display()))?;

    // write world to file
    save_file
        .write_all(world_ron.as_bytes())
        .with_context(|| "failed to write AmbleWorld to .ron file".to_string())?;

    // disco!
    println!("Game saved as {}", gamefile.underline());
    info!("Player saved game to \"{gamefile}\"");
    Ok(())
}
