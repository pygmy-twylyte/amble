//! Player movement and navigation command handlers for the Amble game engine.
//!
//! This module handles all commands that change the player's location within
//! the game world. Movement is a core mechanic that enables exploration,
//! progression, and access to different areas of the game.
//!
//! # Movement System
//!
//! Player movement operates through a sophisticated exit system where:
//! - Rooms define exits in specific directions (north, south, up, down, etc.)
//! - Exits can have requirements (items, flags, keys)
//! - Exits can be locked, hidden, or conditional
//! - Movement attempts trigger validation and game events
//!
//! # Exit Requirements
//!
//! Exits may require players to have:
//! - **Required Items** - Specific tools, keys, or objects
//! - **Required Flags** - Story progression markers or achievements
//! - **Unlocked State** - Some exits start locked and must be opened
//!
//! # Trigger System Integration
//!
//! Movement triggers various game events:
//! - `TriggerCondition::Leave` - Fired when leaving a room
//! - `TriggerCondition::Enter` - Fired when entering a new room
//!
//! These triggers enable:
//! - Story events when entering/leaving specific areas
//! - Environmental changes based on player movement
//! - Character interactions triggered by location changes
//! - Dynamic world updates as player explores
//!
//! # Scoring and Discovery
//!
//! - Players gain points for visiting new rooms (first time only)
//! - Room visit status is tracked for scoring and trigger logic
//! - New rooms display verbose descriptions automatically
//! - Previously visited rooms show brief descriptions unless requested otherwise

use std::collections::HashSet;

use crate::{
    AmbleWorld, View, ViewItem, WorldObject,
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
    view::ViewMode,
};

use anyhow::{Context, Result, anyhow};
use log::info;

/// Attempts to move the player in the specified direction.
///
/// This is the main movement handler that validates and executes player movement
/// between rooms. It performs comprehensive validation of exit conditions and
/// requirements before allowing movement, and handles all the associated game
/// state updates and trigger effects.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing rooms and player state
/// * `view` - Mutable reference to the player's view for feedback messages and room display
/// * `input_dir` - Direction string from player input (e.g., "north", "n", "up")
///
/// # Returns
///
/// Returns `Ok(())` on successful movement attempt, or an error if world state
/// is corrupted (invalid room references).
///
/// # Movement Process
///
/// 1. **Direction Matching** - Finds exits matching the input direction
/// 2. **Lock Validation** - Ensures the exit is not locked
/// 3. **Requirement Checking** - Validates required items and flags
/// 4. **Movement Execution** - Updates player location and triggers events
/// 5. **Room Display** - Shows new location with appropriate detail level
///
/// # Exit Requirements
///
/// Movement may be blocked if the player lacks:
/// - Required flags (story progression markers)
/// - Required items (keys, tools, passes)
/// - Proper exit state (unlocked, revealed)
///
/// # Scoring System
///
/// - First visit to any room awards 1 point
/// - Subsequent visits to the same room award no points
/// - Room visit status is permanently tracked
///
/// # Display Behavior
///
/// - **First visit**: Full verbose description shown automatically
/// - **Return visit**: Brief description shown (unless in verbose mode)
/// - **Travel message**: Randomized flavor text for immersion
///
/// # Error Conditions
///
/// - **Invalid direction**: No exit matches the input direction
/// - **Locked exit**: Exit exists but is currently locked
/// - **Missing requirements**: Player lacks required items or flags
/// - **Invalid destination**: Exit points to non-existent room (returns error)
/// Handles the "back" command to return to a previous room.
///
/// This function attempts to move the player back to their most recently visited room
/// using the location history maintained in the player's state. If no history exists,
/// the player receives an appropriate error message.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing rooms and player state
/// * `view` - Mutable reference to the player's view for feedback messages and room display
///
/// # Returns
///
/// Returns `Ok(())` on successful attempt (regardless of whether movement occurred),
/// or an error if world state is corrupted.
///
/// # Behavior
///
/// - If location history exists, moves player to most recent room
/// - Removes the returned-to room from history (prevents ping-ponging)
/// - Shows room description appropriate to visit status
/// - Triggers Leave/Enter events for the room transition
/// - If no history exists, shows appropriate error message
///
/// # No Scoring
///
/// Moving back to previously visited rooms does not award points, as this is
/// considered backtracking rather than exploration of new areas.
pub fn go_back_handler(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let leaving_id = world.player.location.unwrap_room();

    if let Some(previous_room_id) = world.player.go_back() {
        let travel_message = world.spin_spinner(SpinnerType::Movement, "You retrace your steps...");

        let previous_room = world
            .rooms
            .get(&previous_room_id)
            .ok_or_else(|| anyhow!("invalid previous room in history ({})", previous_room_id))?;

        info!(
            "{} went back to {} ({})",
            world.player.name(),
            previous_room.name(),
            previous_room.symbol()
        );

        view.push(ViewItem::TransitionMessage(travel_message));
        previous_room.show(world, view, None)?;

        check_triggers(
            world,
            view,
            &[
                TriggerCondition::Leave(leaving_id),
                TriggerCondition::Enter(previous_room_id),
            ],
        )?;
    } else {
        view.push(ViewItem::ActionFailure(
            "You haven't been anywhere else yet.".to_string(),
        ));
    }

    Ok(())
}

pub fn move_to_handler(world: &mut AmbleWorld, view: &mut View, input_dir: &str) -> Result<()> {
    let player_name = world.player.name.clone();
    let travel_message = world.spin_spinner(SpinnerType::Movement, "You head that way...");
    let leaving_id = world.player.location.unwrap_room();

    // match "input_dir" to an Exit
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

        // check for missing items or flags required to use this Exit
        let unmet_flags: HashSet<_> = destination_exit
            .required_flags
            .difference(&world.player.flags)
            .collect();

        let unmet_items: HashSet<_> = destination_exit
            .required_items
            .difference(&world.player.inventory)
            .collect();

        if unmet_flags.is_empty() && unmet_items.is_empty() {
            // we've met all of the requirements to move now
            // update player's location using the new history-tracking method
            let destination_id = destination_exit.to;
            world.player.move_to_room(destination_id);
            let new_room = world
                .rooms
                .get(&destination_id)
                .ok_or_else(|| anyhow!("invalid move destination ({})", destination_id))?;

            // log and push display items for the new location
            info!("{} moved to {} ({})", player_name, new_room.name(), new_room.symbol());
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
            let (dest_name, dest_sym) = world
                .rooms
                .get(&destination_exit.to)
                .map(|rm| (rm.name(), rm.symbol()))
                .with_context(|| format!("accessing room {}", destination_exit.to))?;

            info!(
                "{} denied access to {dest_name} ({dest_sym}): missing items ({:?}) or flags ({:?})",
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

    #[test]
    fn go_back_with_no_history_fails() {
        let (mut world, _start, _dest, mut view) = build_test_world();

        assert!(go_back_handler(&mut world, &mut view).is_ok());
        // Should still be in start room since no history
        assert!(matches!(world.player.location, Location::Room(id) if id == _start));
    }

    #[test]
    fn go_back_with_history_works() {
        let (mut world, start, dest, mut view) = build_test_world();

        // Move to destination first to create history
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == dest));
        assert_eq!(world.player.location_history.len(), 1);
        assert_eq!(world.player.location_history[0], start);

        // Now go back
        assert!(go_back_handler(&mut world, &mut view).is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == start));
        assert_eq!(world.player.location_history.len(), 0);
    }

    #[test]
    fn location_history_maintains_max_size() {
        let (mut world, start, dest, _view) = build_test_world();

        // Create additional rooms for testing history limit
        let room3 = Uuid::new_v4();
        let room4 = Uuid::new_v4();
        let room5 = Uuid::new_v4();
        let room6 = Uuid::new_v4();
        let room7 = Uuid::new_v4();

        for &room_id in &[room3, room4, room5, room6, room7] {
            let room = Room {
                id: room_id,
                symbol: format!("room_{}", room_id.to_string()[0..8].to_string()),
                name: format!("Room {}", room_id.to_string()[0..8].to_string()),
                base_description: String::new(),
                overlays: vec![],
                location: Location::Nowhere,
                visited: false,
                exits: HashMap::new(),
                contents: HashSet::new(),
                npcs: HashSet::new(),
            };
            world.rooms.insert(room_id, room);
        }

        // Simulate moving through 6 rooms (should only keep last 5 in history)
        world.player.move_to_room(dest);
        world.player.move_to_room(room3);
        world.player.move_to_room(room4);
        world.player.move_to_room(room5);
        world.player.move_to_room(room6);
        world.player.move_to_room(room7);

        // History should be limited to 5 items
        assert_eq!(world.player.location_history.len(), 5);
        assert!(!world.player.location_history.contains(&start)); // start should be dropped
        assert!(world.player.location_history.contains(&dest));
    }

    #[test]
    fn go_back_multiple_times() {
        let (mut world, start, dest, mut view) = build_test_world();

        // Add a third room for more complex history
        let room3 = Uuid::new_v4();
        let room3_obj = Room {
            id: room3,
            symbol: "room3".into(),
            name: "Room3".into(),
            base_description: String::new(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world
            .rooms
            .get_mut(&dest)
            .unwrap()
            .exits
            .insert("east".into(), Exit::new(room3));
        world.rooms.insert(room3, room3_obj);

        // Move start -> dest -> room3
        assert!(move_to_handler(&mut world, &mut view, "north").is_ok());
        assert!(move_to_handler(&mut world, &mut view, "east").is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == room3));
        assert_eq!(world.player.location_history, vec![start, dest]);

        // Go back to dest
        assert!(go_back_handler(&mut world, &mut view).is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == dest));
        assert_eq!(world.player.location_history, vec![start]);

        // Go back to start
        assert!(go_back_handler(&mut world, &mut view).is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == start));
        assert_eq!(world.player.location_history.len(), 0);

        // Try to go back again - should fail gracefully
        assert!(go_back_handler(&mut world, &mut view).is_ok());
        assert!(matches!(world.player.location, Location::Room(id) if id == start));
    }
}
