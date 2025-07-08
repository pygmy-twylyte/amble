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
    idgen::{NAMESPACE_ITEM, NAMESPACE_ROOM, uuid_from_token},
    room::{Exit, Room},
};

use super::SymbolTable;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
/// Raw room data loaded from file with tokens (names) for IDs.
pub struct RawRoom {
    id: String,
    name: String,
    description: String,
    location: Location,
    #[serde(default)]
    visited: bool,
    #[serde(default)]
    exits: HashMap<String, RawExit>,
    #[serde(default)]
    contents: HashSet<Uuid>,
    #[serde(default)]
    npcs: HashSet<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RawExit {
    to: String,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    required_actions: HashSet<String>,
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
        let mut exit_map = HashMap::new();
        for (dir, raw_exit) in &self.exits {
            // look destination room uuid up in symbol table using token id
            let to_uuid = *symbols
                .rooms
                .get(&raw_exit.to)
                .ok_or_else(|| anyhow!("invalid exit ({}) from ({})", raw_exit.to, self.id))?;

            // items are not loaded until after rooms, so we can't look them up in the symbol table yet
            // we'll generate their UUIDs and add them to the symbol table here as we go
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
                    required_actions: raw_exit.required_actions.clone(),
                    required_items: required_items_uuids,
                    barred_message: raw_exit.barred_message.clone(),
                },
            );
        }

        Ok(Room {
            id: *symbols
                .rooms
                .get(&self.id)
                .ok_or_else(|| anyhow!("UUID for {} not found in symbols", self.id))?,
            name: self.name.clone(),
            description: self.description.clone(),
            location: self.location.clone(),
            visited: self.visited,
            exits: exit_map,
            contents: self.contents.clone(),
            npcs: self.npcs.clone(),
        })
    }
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
    let room_file = fs::read_to_string(toml_path)
        .with_context(|| format!("reading room data from '{}'", toml_path.display()))?;
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
        symbols.rooms.insert(
            rr.id.clone(),
            Uuid::new_v5(&NAMESPACE_ROOM, rr.id.as_bytes()),
        );
    }
    let rooms: Vec<Room> = raw_rooms
        .iter()
        .map(|rr| rr.to_room(symbols))
        .collect::<Result<_>>()?;
    info!("{} rooms built from raw_rooms", rooms.len());
    Ok(rooms)
}
