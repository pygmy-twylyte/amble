//! Helpers Module
//!
//! This module contains helper / simplifier functions that don't clearly belong in another module.

use std::collections::HashMap;

use crate::{Item, Npc, Room};
use uuid::Uuid;

/// Returns the TOML symbol for a given room's uuid.
pub fn room_symbol_from_id(rooms: &HashMap<Uuid, Room>, room_id: Uuid) -> Option<&str> {
    rooms.get(&room_id).map(|room| room.symbol.as_str())
}

/// Returns the TOML symbol for a given item's uuid.
pub fn item_symbol_from_id(items: &HashMap<Uuid, Item>, item_id: Uuid) -> Option<&str> {
    items.get(&item_id).map(|item| item.symbol.as_str())
}

/// Returns the TOML symbol for a given character's uuid.
pub fn npc_symbol_from_id(npcs: &HashMap<Uuid, Npc>, npc_id: Uuid) -> Option<&str> {
    npcs.get(&npc_id).map(|npc| npc.symbol.as_str())
}
