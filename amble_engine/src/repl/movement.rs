//! `repl::movement` module
//!
//! Contains repl loop handlers for commands that change player location

use std::collections::HashSet;

use crate::{
    AmbleWorld, Location, View, ViewItem, WorldObject,
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
    view::ViewMode,
};

use anyhow::{Context, Result, anyhow};
use log::info;

/// Move the player to a neighboring location, if all exit conditions are met.
pub fn move_to_handler(world: &mut AmbleWorld, view: &mut View, input_dir: &str) -> Result<()> {
    let player_name = world.player.name.clone();
    let travel_message = world.spin_spinner(SpinnerType::Movement, "You head that way...");
    let leaving_id = world.player.location.unwrap_room();

    let destination_exit = {
        let current_room = world.player_room_ref()?;
        // find a direction (e.g. "up") in current room that matches user input
        let direction = current_room
            .exits
            .keys()
            .find(|dir| dir.to_lowercase().contains(input_dir));
        // if valid direction found, return the associated Exit
        if let Some(exit_key) = direction {
            current_room.exits.get(exit_key)
        } else {
            // no valid direction matched -- report and return
            view.push(ViewItem::Error(format!(
                "{}? {}",
                input_dir.error_style(),
                world.spin_spinner(SpinnerType::DestinationUnknown, "Which way is that?")
            )));
            return Ok(());
        }
    };

    if let Some(destination_exit) = destination_exit {
        if destination_exit.locked {
            view.push(ViewItem::ActionFailure(format!(
                "You can't go that way ({}) -- it's locked.",
                input_dir.exit_locked_style()
            )));
            info!("{} tried to enter locked exit.", world.player.name());
            return Ok(());
        }

        // check for unmet actions (recorded as flags) for this exit
        let unmet_flags: HashSet<_> = destination_exit
            .required_flags
            .difference(&world.player.flags)
            .collect();

        let unmet_items: HashSet<_> = destination_exit
            .required_items
            .difference(&world.player.inventory)
            .collect();

        if unmet_flags.is_empty() && unmet_items.is_empty() {
            let destination_id = destination_exit.to;
            world.player.location = Location::Room(destination_id);
            let new_room = world
                .rooms
                .get(&destination_id)
                .ok_or_else(|| anyhow!("invalid move destination ({})", destination_id))?;
            info!("{} moved to {} ({})", player_name, new_room.name(), new_room.id());
            view.push(ViewItem::TransitionMessage(travel_message));
            if new_room.visited {
                new_room.show(world, view, None)?;
            } else {
                world.player.score = world.player.score.saturating_add(1);
                new_room.show(world, view, Some(ViewMode::Verbose))?;
            }
            if let Some(new_room) = world.rooms.get_mut(&destination_id) {
                new_room.visited = true;
            }
            check_triggers(
                world,
                view,
                &[
                    TriggerCondition::Leave(leaving_id),
                    TriggerCondition::Enter(destination_id),
                ],
            )?;
        } else {
            // the Exit is barred due to a missing item or flag
            if let Some(msg) = &destination_exit.barred_message {
                view.push(ViewItem::ActionFailure((*msg).denied_style().to_string()));
            } else {
                view.push(ViewItem::ActionFailure(format!(
                    "{}",
                    "You can't go that way because... \"reasons\"".denied_style()
                )));
            }
            let dest_name = world
                .rooms
                .get(&destination_exit.to)
                .with_context(|| format!("accessing room {}", destination_exit.to))?
                .name();
            info!(
                "{} denied access to {dest_name}: missing items ({:?}) or flags ({:?})",
                world.player.name(),
                unmet_items,
                unmet_flags,
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Flag;
    use crate::room::{Exit, Room};
    use crate::world::{AmbleWorld, Location};
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn build_test_world() -> (AmbleWorld, Uuid, Uuid, View) {
        let view = View::new();
        let mut world = AmbleWorld::new_empty();
        let start = Uuid::new_v4();
        let dest = Uuid::new_v4();
        let mut start_room = Room {
            id: start,
            symbol: "start".into(),
            name: "Start".into(),
            base_description: String::new(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: true,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        start_room.exits.insert("north".into(), Exit::new(dest));
        let dest_room = Room {
            id: dest,
            symbol: "dest".into(),
            name: "Dest".into(),
            base_description: String::new(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(start, start_room);
        world.rooms.insert(dest, dest_room);
        world.player.location = Location::Room(start);
        (world, start, dest, view)
    }

    #[test]
    fn move_to_hidden_exit_allowed() {
        let (mut world, start, dest, mut view) = build_test_world();
        {
            world
                .rooms
                .get_mut(&start)
                .unwrap()
                .exits
                .get_mut("north")
                .unwrap()
                .hidden = true;
        }
        let initial = world.player.score;
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == dest));
        assert_eq!(world.player.score, initial + 1);
        assert!(world.rooms.get(&dest).unwrap().visited);
    }

    #[test]
    fn move_to_locked_exit_blocked() {
        let (mut world, start, dest, mut view) = build_test_world();
        world
            .rooms
            .get_mut(&start)
            .unwrap()
            .exits
            .get_mut("north")
            .unwrap()
            .locked = true;
        let initial = world.player.score;
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == start));
        assert_eq!(world.player.score, initial);
        assert!(!world.rooms.get(&dest).unwrap().visited);
    }

    #[test]
    fn move_requires_item() {
        let (mut world, start, dest, mut view) = build_test_world();
        let item_id = Uuid::new_v4();
        world
            .rooms
            .get_mut(&start)
            .unwrap()
            .exits
            .get_mut("north")
            .unwrap()
            .required_items
            .insert(item_id);

        let initial = world.player.score;
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == start));
        assert_eq!(world.player.score, initial);
        assert!(!world.rooms.get(&dest).unwrap().visited);

        world.player.inventory.insert(item_id);
        let initial = world.player.score;
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == dest));
        assert_eq!(world.player.score, initial + 1);
        assert!(world.rooms.get(&dest).unwrap().visited);
    }

    #[test]
    fn move_requires_flag() {
        let (mut world, start, dest, mut view) = build_test_world();
        let flag = Flag::simple("alpha");
        world
            .rooms
            .get_mut(&start)
            .unwrap()
            .exits
            .get_mut("north")
            .unwrap()
            .required_flags
            .insert(flag.clone());

        let initial = world.player.score;
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == start));
        assert_eq!(world.player.score, initial);
        assert!(!world.rooms.get(&dest).unwrap().visited);

        world.player.flags.insert(flag);
        let initial = world.player.score;
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == dest));
        assert_eq!(world.player.score, initial + 1);
        assert!(world.rooms.get(&dest).unwrap().visited);
    }
}
