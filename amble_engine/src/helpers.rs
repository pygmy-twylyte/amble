//! Helpers Module
//!
//! This module contains helper / simplifier functions that don't clearly
//! belong in another module. Prefer adding generally useful, low‑level
//! utilities here to avoid duplication across the codebase.

use std::collections::HashMap;

use crate::world::WorldObject;
use crate::{Item, Npc, Room};
use uuid::Uuid;

/// Generic: Returns the TOML symbol for a given object's uuid.
pub fn symbol_from_id<T: WorldObject>(map: &HashMap<Uuid, T>, id: Uuid) -> Option<&str> {
    map.get(&id).map(super::world::WorldObject::symbol)
}

/// Generic: Returns the display name for a given object's uuid.
pub fn name_from_id<T: WorldObject>(map: &HashMap<Uuid, T>, id: Uuid) -> Option<&str> {
    map.get(&id).map(super::world::WorldObject::name)
}

/// Convenience: Returns the symbol or a standard fallback string.
pub fn symbol_or_unknown<T: WorldObject>(map: &HashMap<Uuid, T>, id: Uuid) -> String {
    symbol_from_id(map, id).unwrap_or("<not_found>").to_string()
}

/// Pluralization helper for simple English "s" suffix rules.
pub fn plural_s(count: isize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// Pluralize a word with a simple "s" suffix based on count.
pub fn pluralize(word: &str, count: isize) -> String {
    format!("{}{}", word, plural_s(count))
}

/// Returns the TOML symbol for a given room's uuid.
pub fn room_symbol_from_id(rooms: &HashMap<Uuid, Room>, room_id: Uuid) -> Option<&str> {
    symbol_from_id(rooms, room_id)
}

/// Returns the TOML symbol for a given item's uuid.
pub fn item_symbol_from_id(items: &HashMap<Uuid, Item>, item_id: Uuid) -> Option<&str> {
    symbol_from_id(items, item_id)
}

/// Returns the TOML symbol for a given character's uuid.
pub fn npc_symbol_from_id(npcs: &HashMap<Uuid, Npc>, npc_id: Uuid) -> Option<&str> {
    symbol_from_id(npcs, npc_id)
}
