//! Room loading logic.
//!
//! Rooms are pre-registered in the symbol table and later populated with exits
//! and overlays once all referenced objects are known.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    Location,
    idgen::{NAMESPACE_CHARACTER, NAMESPACE_ITEM, NAMESPACE_ROOM, uuid_from_token},
    npc::NpcState,
    player::Flag,
    room::{Exit, OverlayCondition, Room, RoomOverlay},
};

use super::SymbolTable;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
/// Raw room data loaded from file with tokens (names) for IDs.
pub struct RawRoom {
    id: String,
    name: String,
    base_description: String,
    location: Location,
    #[serde(default)]
    overlays: Vec<RawRoomOverlay>,
    #[serde(default)]
    visited: bool,
    #[serde(default)]
    exits: HashMap<String, RawExit>,
    #[serde(default)]
    contents: HashSet<Uuid>,
    #[serde(default)]
    npcs: HashSet<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRoomOverlay {
    condition: RawOverlayCondition,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawOverlayCondition {
    FlagComplete { flag: String },
    FlagSet { flag: String },
    FlagUnset { flag: String },
    ItemPresent { item_id: String },
    ItemAbsent { item_id: String },
    PlayerHasItem { item_id: String },
    PlayerMissingItem { item_id: String },
    NpcInState { npc_id: String, state: NpcState },
    ItemInRoom { item_id: String, room_id: String },
}

#[derive(Debug, Deserialize)]
pub struct RawExit {
    to: String,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    required_flags: HashSet<Flag>,
    #[serde(default)]
    required_items: HashSet<String>,
    #[serde(default)]
    barred_message: Option<String>,
}

impl RawRoom {
    /// Converts `RawRoom` to a `Room` object.
    /// # Errors
    /// - on failed room lookup in symbol table
    pub fn to_room(&self, symbols: &mut SymbolTable) -> Result<Room> {
        // convert raw exits to real ones in a map
        let mut exit_map = HashMap::new();
        for (dir, raw_exit) in &self.exits {
            // look destination room uuid up in symbol table using token id
            let to_uuid = *symbols
                .rooms
                .get(&raw_exit.to)
                .ok_or_else(|| anyhow!("invalid exit ({}) from ({})", raw_exit.to, self.id))?;

            // items are not loaded until after rooms, so we can't look them up in the symbol table yet
            // we'll generate their UUIDs and add them to the symbol table here as we go
            // (these items are later verified to exist by the item loader)
            let mut required_items_uuids = HashSet::<Uuid>::new();
            for required in &raw_exit.required_items {
                let item_uuid = uuid_from_token(&NAMESPACE_ITEM, required);
                symbols.items.insert(required.to_string(), item_uuid);
                required_items_uuids.insert(item_uuid);
            }

            exit_map.insert(
                dir.to_string(),
                Exit {
                    to: to_uuid,
                    hidden: raw_exit.hidden,
                    locked: raw_exit.locked,
                    required_flags: raw_exit.required_flags.clone(),
                    required_items: required_items_uuids,
                    barred_message: raw_exit.barred_message.clone(),
                },
            );
        }

        // create overlay vector from raw ones; like with exits, we must
        // reference items and NPCs that aren't loaded yet so we add them to the symbol
        // table here, and they're verified to exist later when items are loaded (
        // Room UUIDs should already be in symbol table when this is called, though.)
        let mut overlays = Vec::new();
        for raw_overlay in &self.overlays {
            let condition = convert_overlay_condition(&raw_overlay.condition, symbols)?;
            overlays.push(RoomOverlay {
                condition,
                text: raw_overlay.text.to_string(),
            });
        }

        Ok(Room {
            id: *symbols
                .rooms
                .get(&self.id)
                .ok_or_else(|| anyhow!("UUID for {} not found in symbols", self.id))?,
            symbol: self.id.to_string(),
            name: self.name.clone(),
            base_description: self.base_description.clone(),
            overlays,
            location: self.location,
            visited: self.visited,
            exits: exit_map,
            contents: self.contents.clone(),
            npcs: self.npcs.clone(),
        })
    }
}

/// Convert a `RawOverlayCondition` to an `OverlayCondition`
pub fn convert_overlay_condition(
    raw_condition: &RawOverlayCondition,
    symbols: &mut SymbolTable,
) -> Result<OverlayCondition> {
    let condition = match raw_condition {
        RawOverlayCondition::FlagComplete { flag } => OverlayCondition::FlagComplete { flag: flag.to_string() },
        RawOverlayCondition::FlagSet { flag } => OverlayCondition::FlagSet { flag: flag.to_string() },
        RawOverlayCondition::FlagUnset { flag } => OverlayCondition::FlagUnset { flag: flag.to_string() },
        RawOverlayCondition::ItemPresent { item_id } => OverlayCondition::ItemPresent {
            item_id: register_item(symbols, item_id),
        },
        RawOverlayCondition::ItemAbsent { item_id } => OverlayCondition::ItemAbsent {
            item_id: register_item(symbols, item_id),
        },
        RawOverlayCondition::PlayerHasItem { item_id } => OverlayCondition::PlayerHasItem {
            item_id: register_item(symbols, item_id),
        },
        RawOverlayCondition::PlayerMissingItem { item_id } => OverlayCondition::PlayerMissingItem {
            item_id: register_item(symbols, item_id),
        },
        RawOverlayCondition::NpcInState { npc_id, state: mood } => OverlayCondition::NpcInMood {
            npc_id: register_npc(symbols, npc_id),
            mood: mood.clone(),
        },
        RawOverlayCondition::ItemInRoom { item_id, room_id } => OverlayCondition::ItemInRoom {
            item_id: register_item(symbols, item_id),
            room_id: *symbols
                .rooms
                .get(room_id)
                .ok_or_else(|| anyhow!("OverlayCondition::ItemInRoom(_,{room_id}) - room not in symbol table"))?,
        },
    };
    Ok(condition)
}

/// Pre-registers an item symbol during room loading and returns the corresponding UUID
pub fn register_item(symbols: &mut SymbolTable, item_symbol: &str) -> Uuid {
    let uuid = uuid_from_token(&NAMESPACE_ITEM, item_symbol);
    symbols.items.insert(item_symbol.to_string(), uuid);
    uuid
}

/// Pre-registers an NPC symbol during room loading and returns the corresponding UUID
pub fn register_npc(symbols: &mut SymbolTable, npc_symbol: &str) -> Uuid {
    let uuid = uuid_from_token(&NAMESPACE_CHARACTER, npc_symbol);
    symbols.characters.insert(npc_symbol.to_string(), uuid);
    uuid
}

#[derive(Deserialize)]
/// Needed to be able to parse room vector from TOML correctly
pub struct RawRoomFile {
    rooms: Vec<RawRoom>,
}

/// Load `RawRoom` vector from file
/// # Errors
/// - if unable to read or parse the rooms.toml file
pub fn load_raw_rooms(toml_path: &Path) -> Result<Vec<RawRoom>> {
    let room_file =
        fs::read_to_string(toml_path).with_context(|| format!("reading room data from '{}'", toml_path.display()))?;
    let wrapper: RawRoomFile = toml::from_str(&room_file)?;
    info!(
        "{} raw rooms successfully loaded from '{}'",
        wrapper.rooms.len(),
        toml_path.display()
    );
    Ok(wrapper.rooms)
}

/// Build Room vector from raw rooms.
/// # Errors
/// - if symbol table lookup fails when building room instances
pub fn build_rooms(raw_rooms: &[RawRoom], symbols: &mut SymbolTable) -> Result<Vec<Room>> {
    for rr in raw_rooms {
        symbols
            .rooms
            .insert(rr.id.clone(), Uuid::new_v5(&NAMESPACE_ROOM, rr.id.as_bytes()));
    }
    let rooms: Vec<Room> = raw_rooms.iter().map(|rr| rr.to_room(symbols)).collect::<Result<_>>()?;
    info!("{} rooms built from raw_rooms", rooms.len());
    Ok(rooms)
}
