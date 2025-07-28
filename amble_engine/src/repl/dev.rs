//! repl/dev.rs
//!
//! Handlers for DEV_MODE commands

use log::{info, warn};

use crate::{
    AmbleWorld, Location, WorldObject,
    idgen::{NAMESPACE_ITEM, NAMESPACE_ROOM, uuid_from_token},
    player::Flag,
    style::GameStyle,
    trigger::{self, spawn_item_in_inventory},
};

/// Spawn an item in inventory, removing it from elsewhere if necessary (does not duplicate).
/// DEV_MODE only.
pub fn dev_spawn_item_handler(world: &mut AmbleWorld, symbol: &str) {
    let item_id = uuid_from_token(&NAMESPACE_ITEM, symbol);
    if world.items.contains_key(&item_id) {
        spawn_item_in_inventory(world, &item_id).expect("should not err; item_id already known to be valid");
        info!("player used DEV_MODE SpawnItem({symbol}) [uuid:{item_id}]");
        println!("Item '{symbol}' moved to inventory.");
    } else {
        println!("No item matching '{}' found in AmbleWorld data.", symbol.error_style());
    }
}

/// Instantly transport player elsewhere, if you know the id from the TOML file.
/// This is for development purposes only.
pub fn dev_teleport_handler(world: &mut AmbleWorld, room_toml_id: &str) {
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

/// Add a sequence type flag
pub fn dev_start_seq_handler(world: &mut AmbleWorld, seq_name: &str, end: &str) {
    let limit = if end.to_lowercase() == "none" {
        None
    } else {
        end.parse::<u8>().ok()
    };
    let seq = Flag::sequence(seq_name, limit);
    println!("Sequence flag '{}' started with step limit {limit:?}.", seq.value());
    warn!("DEV_MODE command StartSeq used: '{}' set, limit {limit:?}", seq.value());
    trigger::add_flag(world, &seq);
}

/// Set a simple flag.
pub fn dev_set_flag_handler(world: &mut AmbleWorld, flag_name: &str) {
    let flag = Flag::simple(flag_name);
    println!("Simple flag '{}' set.", flag.value());
    warn!("DEV_MODE command SetFlag used: '{}' set.", flag.value());
    trigger::add_flag(world, &flag);
}

/// Advance a sequence flag.
pub fn dev_advance_seq_handler(world: &mut AmbleWorld, seq_name: &str) {
    world.player.advance_flag(seq_name);
    let target = Flag::simple(seq_name);
    if let Some(flag) = world.player.flags.get(&target) {
        println!("Sequence '{}' advanced to [{}].", flag.name(), flag.value());
        warn!(
            "DEV_MODE AdvanceSeq used: '{}' advanced to [{}].",
            flag.name(),
            flag.value()
        )
    }
}

/// Reset a sequence flag.
pub fn dev_reset_seq_handler(world: &mut AmbleWorld, seq_name: &str) {
    world.player.reset_flag(seq_name);
    let target = Flag::simple(seq_name);
    if let Some(flag) = world.player.flags.get(&target) {
        println!("Sequence '{}' reset to [{}].", flag.name(), flag.value());
        warn!("DEV_MODE ResetSeq used: '{}' reset to [{}].", flag.name(), flag.value())
    }
}
