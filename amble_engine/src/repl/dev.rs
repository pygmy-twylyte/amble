//! Development mode command handlers for the Amble game engine.
//!
//! This module provides handlers for special development commands that are only
//! available when the game is running in `DEV_MODE`. These commands allow developers
//! and content creators to manipulate game state for testing, debugging, and
//! content creation purposes.
//!
//! # Security Note
//!
//! All functions in this module bypass normal game restrictions and can modify
//! world state in ways that would not be possible through regular gameplay.
//! They should only be available in development builds or when explicitly
//! enabled by developers.
//!
//! # Available Commands
//!
//! ## Item Manipulation
//! - [`dev_spawn_item_handler`] - Instantly add any item to player inventory
//!
//! ## Player Movement
//! - [`dev_teleport_handler`] - Instantly transport player to any room
//!
//! ## Flag Management
//! - [`dev_start_seq_handler`] - Create new sequence flags with custom limits
//! - [`dev_set_flag_handler`] - Add simple boolean flags to player
//! - [`dev_advance_seq_handler`] - Advance sequence flags by one step
//! - [`dev_reset_seq_handler`] - Reset sequence flags back to step 0
//!
//! # Logging
//!
//! All dev commands use `warn`-level logging to create an audit trail of
//! development actions, making it easy to track what changes were made
//! during testing sessions.

use log::{info, warn};

use crate::{
    AmbleWorld, Location, View, ViewItem, WorldObject,
    idgen::{NAMESPACE_ITEM, NAMESPACE_ROOM, uuid_from_token},
    player::Flag,
    style::GameStyle,
    trigger::{self, spawn_item_in_inventory},
};

/// Spawns an item directly into the player's inventory (DEV_MODE only).
///
/// This development command instantly adds any item to the player's inventory
/// by its symbol name. If the item already exists elsewhere in the world, it
/// will be moved rather than duplicated to maintain world consistency.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `symbol` - The item's symbol identifier from the game data files
///
/// # Behavior
///
/// - Converts the symbol to the item's UUID using the item namespace
/// - If the item exists, moves it to inventory (removing from current location)
/// - Displays success message with item name
/// - If item doesn't exist, displays error message
/// - All actions are logged at `warn` level for audit trail
///
/// # Security
///
/// This function bypasses all normal game restrictions including:
/// - Item portability checks
/// - Container access restrictions
/// - Inventory space limitations
/// - Story progression requirements
pub fn dev_spawn_item_handler(world: &mut AmbleWorld, view: &mut View, symbol: &str) {
    let item_id = uuid_from_token(&NAMESPACE_ITEM, symbol);
    if world.items.contains_key(&item_id) {
        spawn_item_in_inventory(world, &item_id).expect("should not err; item_id already known to be valid");
        info!("player used DEV_MODE SpawnItem({symbol})");
        view.push(ViewItem::ActionSuccess(format!("Item '{symbol}' moved to inventory.")));
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "No item matching '{}' found in AmbleWorld data.",
            symbol.error_style()
        )));
    }
}

/// Instantly teleports the player to any room by its TOML identifier (DEV_MODE only).
///
/// This development command bypasses all normal movement restrictions and
/// immediately transports the player to the specified room. This is useful
/// for testing different areas, debugging room connections, or quickly
/// navigating during content development.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing rooms and player
/// * `view` - Mutable reference to the player's view for feedback and room display
/// * `room_toml_id` - The room's string identifier from the TOML configuration files
///
/// # Behavior
///
/// - Converts the TOML ID to the room's UUID using the room namespace
/// - If the room exists, immediately updates player location
/// - Displays the new room with full description (as if entering for first time)
/// - If room doesn't exist, displays error message
/// - All teleportation is logged at `warn` level for audit trail
///
/// # Security
///
/// This function bypasses all normal movement restrictions including:
/// - Locked exits and doors
/// - Required items or keys
/// - Story progression flags
/// - Movement-blocking conditions
pub fn dev_teleport_handler(world: &mut AmbleWorld, view: &mut View, room_toml_id: &str) {
    let room_uuid = uuid_from_token(&NAMESPACE_ROOM, room_toml_id);
    if let Some(room) = world.rooms.get(&room_uuid) {
        world.player.location = Location::Room(room_uuid);
        warn!(
            "DEV only command used: Teleported player to {} ({})",
            room.name(),
            room.id()
        );
        view.push(ViewItem::ActionSuccess("You teleported...".to_string()));
        let _ = room.show(world, view, None);
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "Teleport failed: Lookup of '{room_toml_id}' failed."
        )));
    }
}

/// Creates a new sequence flag with optional step limit (DEV_MODE only).
///
/// This development command creates a sequence flag that can track multi-step
/// progress through game events. Sequence flags start at step 0 and can be
/// advanced, reset, or checked by trigger conditions.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `seq_name` - Name identifier for the new sequence flag
/// * `end` - Maximum number of steps ("none" for unlimited, or numeric string)
///
/// # Behavior
///
/// - Parses the end parameter: "none" creates unlimited sequence, numbers set limit
/// - Creates new sequence flag with specified name and limit
/// - Adds flag to player's flag collection (replacing any existing flag with same name)
/// - Displays success message with flag details
/// - All flag creation is logged at `warn` level for audit trail
///
/// # Examples
///
/// - `start_seq puzzle_steps 5` - Creates sequence limited to 5 steps
/// - `start_seq story_progress none` - Creates unlimited sequence
pub fn dev_start_seq_handler(world: &mut AmbleWorld, view: &mut View, seq_name: &str, end: &str) {
    let limit = if end.to_lowercase() == "none" {
        None
    } else {
        end.parse::<u8>().ok()
    };
    let seq = Flag::sequence(seq_name, limit, world.turn_count);
    view.push(ViewItem::ActionSuccess(format!(
        "Sequence flag '{}' started with step limit {limit:?}.",
        seq.value()
    )));
    warn!("DEV_MODE command StartSeq used: '{}' set, limit {limit:?}", seq.value());
    trigger::add_flag(world, view, &seq);
}

/// Sets a simple boolean flag on the player (DEV_MODE only).
///
/// This development command creates or sets a simple flag that represents
/// a boolean condition or completed state. Simple flags are either present
/// (true) or absent (false) and are commonly used for unlock conditions,
/// story progression markers, or achievement tracking.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `flag_name` - Name identifier for the simple flag to set
///
/// # Behavior
///
/// - Creates a simple (non-sequence) flag with the specified name
/// - Adds flag to player's flag collection (replacing any existing flag with same name)
/// - Displays success message confirming flag was set
/// - All flag setting is logged at `warn` level for audit trail
///
/// # Use Cases
///
/// - Unlocking areas or content that requires specific flags
/// - Testing trigger conditions that depend on flag states
/// - Simulating completed story events or achievements
pub fn dev_set_flag_handler(world: &mut AmbleWorld, view: &mut View, flag_name: &str) {
    let flag = Flag::simple(flag_name, world.turn_count);
    view.push(ViewItem::ActionSuccess(format!("Simple flag '{}' set.", flag.value())));
    warn!("DEV_MODE command SetFlag used: '{}' set.", flag.value());
    trigger::add_flag(world, view, &flag);
}

/// Advances a sequence flag to its next step (`DEV_MODE` only).
///
/// This development command increments a sequence flag by one step, simulating
/// progress through a multi-step process. This is useful for testing triggers
/// that depend on specific sequence flag values or for advancing past points
/// in development.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `seq_name` - Name of the sequence flag to advance
///
/// # Behavior
///
/// - Advances the named sequence flag by one step (if it exists)
/// - Respects the flag's step limit (won't advance beyond maximum)
/// - Displays current step value after advancement
/// - If flag doesn't exist, displays error message with helpful suggestion
/// - All advancement is logged at `warn` level for audit trail
///
/// # Use Cases
///
/// - Testing multi-step puzzle logic
/// - Simulating story progression
/// - Debugging sequence-dependent triggers
/// - Fast-forwarding through lengthy sequences during testing
pub fn dev_advance_seq_handler(world: &mut AmbleWorld, view: &mut View, seq_name: &str) {
    let target = Flag::simple(seq_name, world.turn_count);

    // Check if the flag exists before trying to advance it
    if world.player.flags.contains(&target) {
        world.player.advance_flag(seq_name);
        if let Some(flag) = world.player.flags.get(&target) {
            view.push(ViewItem::ActionSuccess(format!(
                "Sequence '{}' advanced to [{}].",
                flag.name(),
                flag.value()
            )));
            warn!(
                "DEV_MODE AdvanceSeq used: '{}' advanced to [{}].",
                flag.name(),
                flag.value()
            );
        }
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "No sequence flag '{}' found. Use :init-seq to create it first.",
            seq_name.error_style()
        )));
        warn!("DEV_MODE AdvanceSeq failed: sequence flag '{}' not found", seq_name);
    }
}

/// Resets a sequence flag back to step 0 (DEV_MODE only).
///
/// This development command resets a sequence flag to its initial state,
/// effectively undoing any progress made on that sequence. This is useful
/// for testing the beginning of multi-step processes or resetting puzzle
/// states during development.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `seq_name` - Name of the sequence flag to reset
///
/// # Behavior
///
/// - Resets the named sequence flag to step 0 (if it exists)
/// - Preserves the flag's step limit setting
/// - Displays current step value after reset (should be 0)
/// - If flag doesn't exist, displays error message with helpful suggestion
/// - All resets are logged at `warn` level for audit trail
///
/// # Use Cases
///
/// - Testing puzzle logic from the beginning
/// - Resetting story sequences for repeated testing
/// - Debugging sequence initialization
/// - Clearing progress to test different approaches
pub fn dev_reset_seq_handler(world: &mut AmbleWorld, view: &mut View, seq_name: &str) {
    let target = Flag::simple(seq_name, world.turn_count);

    // Check if the flag exists before trying to reset it
    if world.player.flags.contains(&target) {
        world.player.reset_flag(seq_name);
        if let Some(flag) = world.player.flags.get(&target) {
            view.push(ViewItem::ActionSuccess(format!(
                "Sequence '{}' reset to [{}].",
                flag.name(),
                flag.value()
            )));
            warn!("DEV_MODE ResetSeq used: '{}' reset to [{}].", flag.name(), flag.value());
        }
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "No sequence flag '{}' found. Use :init-seq to create it first.",
            seq_name.error_style()
        )));
        warn!("DEV_MODE ResetSeq failed: sequence flag '{}' not found", seq_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AmbleWorld, View, ViewItem,
        player::{Flag, Player},
    };

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::default();
        world.player = Player::default();
        world
    }

    #[test]
    fn dev_advance_seq_handler_shows_error_for_nonexistent_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        dev_advance_seq_handler(&mut world, &mut view, "nonexistent_flag");

        // Should have an error message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionFailure(msg) = &view.items[0] {
            // Strip ANSI color codes for comparison
            let clean_msg = msg.replace("\u{1b}[38;2;230;30;30m", "").replace("\u{1b}[0m", "");
            assert!(clean_msg.contains("No sequence flag 'nonexistent_flag' found"));
            assert!(clean_msg.contains("Use :init-seq to create it first"));
        } else {
            panic!("Expected ActionFailure, got {:?}", view.items[0]);
        }
    }

    #[test]
    fn dev_advance_seq_handler_works_for_existing_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        // First create a sequence flag
        let flag = Flag::sequence("test_seq", Some(3), world.turn_count);
        world.player.flags.insert(flag);

        dev_advance_seq_handler(&mut world, &mut view, "test_seq");

        // Should have a success message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionSuccess(msg) = &view.items[0] {
            assert!(msg.contains("Sequence 'test_seq' advanced to [test_seq#1]"));
        } else {
            panic!("Expected ActionSuccess, got {:?}", view.items[0]);
        }
    }

    #[test]
    fn dev_reset_seq_handler_shows_error_for_nonexistent_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        dev_reset_seq_handler(&mut world, &mut view, "nonexistent_flag");

        // Should have an error message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionFailure(msg) = &view.items[0] {
            // Strip ANSI color codes for comparison
            let clean_msg = msg.replace("\u{1b}[38;2;230;30;30m", "").replace("\u{1b}[0m", "");
            assert!(clean_msg.contains("No sequence flag 'nonexistent_flag' found"));
            assert!(clean_msg.contains("Use :init-seq to create it first"));
        } else {
            panic!("Expected ActionFailure, got {:?}", view.items[0]);
        }
    }

    #[test]
    fn dev_reset_seq_handler_works_for_existing_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        // First create and advance a sequence flag
        let mut flag = Flag::sequence("test_seq", Some(3), world.turn_count);
        if let Flag::Sequence { step, .. } = &mut flag {
            *step = 2; // Set to step 2
        }
        world.player.flags.insert(flag);

        dev_reset_seq_handler(&mut world, &mut view, "test_seq");

        // Should have a success message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionSuccess(msg) = &view.items[0] {
            assert!(msg.contains("Sequence 'test_seq' reset to [test_seq#0]"));
        } else {
            panic!("Expected ActionSuccess, got {:?}", view.items[0]);
        }
    }
}
