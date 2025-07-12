//! `repl::movement` module
//!
//! Contains repl loop handlers for commands that change player location

use std::collections::HashSet;

use crate::{
    AmbleWorld, Location, WorldObject,
    idgen::{NAMESPACE_ROOM, uuid_from_token},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
};

use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use log::{info, warn};

/// Move the player to a neighboring location, if all exit conditions are met.
pub fn move_to_handler(world: &mut AmbleWorld, input_dir: &str) -> Result<()> {
    let player_name = world.player.name.clone();
    let travel_message = world.spin_spinner(SpinnerType::Movement, "You head that way...");
    let leaving_id = world.player.location.unwrap_room();
    // let found_key;
    let destination_exit = {
        let current_room = world.player_room_ref()?;
        let destination = current_room
            .exits
            .keys()
            .find(|dir| dir.to_lowercase().contains(input_dir));
        if let Some(exit_key) = destination {
            current_room.exits.get(exit_key)
        } else {
            println!("Which way is {}?", input_dir.error_style());
            return Ok(());
        }
    };

    if let Some(destination_exit) = destination_exit {
        if destination_exit.locked {
            println!(
                "You can't go that way ({}) -- it's locked.",
                input_dir.exit_locked_style()
            );
            info!("{} tried to enter locked exit.", world.player.name());
            return Ok(());
        }

        // check for unmet actions (recorded as achievements) for this exit
        let unmet_actions: HashSet<_> = destination_exit
            .required_actions
            .difference(&world.player.achievements)
            .collect();

        let unmet_items: HashSet<_> = destination_exit
            .required_items
            .difference(&world.player.inventory)
            .collect();

        if unmet_actions.is_empty() && unmet_items.is_empty() {
            let destination_id = destination_exit.to;
            world.player.location = Location::Room(destination_id);
            let new_room = world
                .rooms
                .get(&destination_id)
                .ok_or_else(|| anyhow!("invalid move destination ({})", destination_id))?;
            info!(
                "{} moved to {} ({})",
                player_name,
                new_room.name(),
                new_room.id()
            );
            println!("{travel_message}\n");
            if new_room.visited {
                println!("{}", new_room.name().room_style().underline());
                new_room.show_exits(world)?;
                new_room.show_npcs(world);
            } else {
                world.player.score = world.player.score.saturating_add(1);
                new_room.show(world)?;
            }
            if let Some(new_room) = world.rooms.get_mut(&destination_id) {
                new_room.visited = true;
            }
            check_triggers(
                world,
                &[
                    TriggerCondition::Leave(leaving_id),
                    TriggerCondition::Enter(destination_id),
                ],
            )?;
        } else {
            if let Some(msg) = &destination_exit.barred_message {
                println!("{}", msg.italic());
            } else {
                println!(
                    "{}",
                    "You can't go that way because... \"reasons\"".italic()
                );
            }
            let dest_name = world
                .rooms
                .get(&destination_exit.to)
                .with_context(|| format!("accessing room {}", destination_exit.to))?
                .name();
            info!(
                "{} denied access to {dest_name}: missing items ({:?}) or achievements ({:?})",
                world.player.name(),
                unmet_items,
                unmet_actions,
            );
        }
    } else {
        println!("Which way is {}? You stay put.\n", input_dir.error_style());
    }
    Ok(())
}

/// Instantly transport player elsewhere, if you know the id from the TOML file.
/// This is for development purposes only.
pub fn teleport_handler(world: &mut AmbleWorld, room_toml_id: &str) {
    let room_uuid = uuid_from_token(&NAMESPACE_ROOM, room_toml_id);
    if let Some(room) = world.rooms.get(&room_uuid) {
        world.player.location = Location::Room(room_uuid);
        warn!(
            "DEV only command used: Teleported player to {} ({})",
            room.name(),
            room.id()
        );
        println!("You teleported...");
        let _ = room.show(world);
    } else {
        println!("Teleport failed. Lookup of '{room_toml_id}' failed.");
    }
}
