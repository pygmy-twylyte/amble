//! Helpers Module
//!
//! This module contains helper / simplifier functions that don't clearly belong in another module.

use std::collections::HashMap;

use crate::{Item, Npc, Room};
use uuid::Uuid;

/// Returns the TOML symbol for a given room's uuid.
pub fn room_symbol_from_id(rooms: &HashMap<Uuid, Room>, room_id: Uuid) -> String {
    rooms
        .get(&room_id)
        .map_or_else(|| "<not_found>".to_string(), |room| room.symbol.clone())
}

/// Returns the TOML symbol for a given item's uuid.
pub fn item_symbol_from_id(items: &HashMap<Uuid, Item>, item_id: Uuid) -> String {
    items
        .get(&item_id)
        .map_or_else(|| "<not_found>".to_string(), |item| item.symbol.clone())
}

/// Returns the TOML symbol for a given character's uuid.
pub fn npc_symbol_from_id(npcs: &HashMap<Uuid, Npc>, npc_id: Uuid) -> String {
    npcs.get(&npc_id)
        .map_or_else(|| "<not_found>".to_string(), |npc| npc.symbol.clone())
}
