//! Helpers Module
//!
//! This module contains helper / simplifier functions that don't clearly
//! belong in another module. Prefer adding generally useful, low‑level
//! utilities here to avoid duplication across the codebase.

use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::world::WorldObject;
use crate::{Id, ItemId, NpcId, RoomId};
use crate::{Item, Npc};

/// Generic: Returns the symbol for a given object's id.
pub fn symbol_from_id<T: WorldObject, S: BuildHasher>(map: &HashMap<Id, T, S>, id: impl AsRef<str>) -> Option<&str> {
    map.get(id.as_ref()).map(super::world::WorldObject::symbol)
}

/// Generic: Returns the display name for a given object's id.
pub fn name_from_id<T: WorldObject, S: BuildHasher>(map: &HashMap<Id, T, S>, id: impl AsRef<str>) -> Option<&str> {
    map.get(id.as_ref()).map(super::world::WorldObject::name)
}

/// Convenience: Returns the symbol or a standard fallback string.
pub fn symbol_or_unknown<T: WorldObject, S: BuildHasher>(map: &HashMap<Id, T, S>, id: impl AsRef<str>) -> String {
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

/// Returns the symbol for a given room's id.
pub fn room_symbol_from_id<S: BuildHasher>(room_id: &RoomId) -> String {
    room_id.to_string()
}

/// Returns the symbol for a given item's id.
pub fn item_symbol_from_id<S: BuildHasher>(items: &HashMap<ItemId, Item, S>, item_id: impl AsRef<str>) -> Option<&str> {
    symbol_from_id(items, item_id)
}

/// Returns the symbol for a given character's id.
pub fn npc_symbol_from_id<S: BuildHasher>(npcs: &HashMap<NpcId, Npc, S>, npc_id: impl AsRef<str>) -> Option<&str> {
    symbol_from_id(npcs, npc_id)
}
