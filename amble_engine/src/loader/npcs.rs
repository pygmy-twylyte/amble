//! NPC loader submodule

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
    Location, WorldObject,
    idgen::{self, NAMESPACE_CHARACTER},
    npc::{Npc, NpcState},
    world::AmbleWorld,
};

use super::{SymbolTable, resolve_location};

/// Needed to deserialize a vector of NPCs correctly.
#[derive(Debug, Deserialize)]
pub struct RawNpcFile {
    pub npcs: Vec<RawNpc>,
}

/// Structure used for first-stage loading of NPC data from TOML files.
#[derive(Debug, Deserialize)]
pub struct RawNpc {
    pub id: String,
    pub name: String,
    pub description: String,
    pub location: HashMap<String, String>,
    #[serde(default)]
    pub inventory: HashSet<String>,
    pub dialogue: HashMap<String, Vec<String>>,
    pub state: String,
}
impl RawNpc {
    /// Converts this `RawNpc` to a real `Npc`
    /// # Errors
    /// - on failure to resolve location or look up NPC in symbol table
    pub fn to_npc(&self, symbols: &SymbolTable) -> Result<Npc> {
        let start_room = resolve_location(&self.location, symbols)?;
        let mut processed_dialogue = HashMap::new();
        for (state_key, lines) in &self.dialogue {
            let state = NpcState::from_key(&state_key);
            processed_dialogue.insert(state, lines.clone());
        }

        Ok(Npc {
            id: *symbols
                .characters
                .get(&self.id)
                .ok_or_else(|| anyhow!("UUID for ({}) not found in character symbols", self.id))?,
            symbol: self.id.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            location: start_room,
            inventory: HashSet::new(),
            dialogue: processed_dialogue,
            state: NpcState::from_key(&self.state),
        })
    }
}

/// Loads raw NPCs from file.
/// # Errors
/// - on failed access to NPC TOML file or error when parsing file
pub fn load_raw_npcs(toml_path: &Path) -> Result<Vec<RawNpc>> {
    let file_contents =
        fs::read_to_string(toml_path).with_context(|| format!("reading NPC data from '{}'", toml_path.display()))?;
    let wrapper: RawNpcFile =
        toml::from_str(&file_contents).with_context(|| "parsing NPC data from file contents".to_string())?;
    info!("{} raw NPCs loaded from '{}'", wrapper.npcs.len(), toml_path.display(),);
    Ok(wrapper.npcs)
}

/// Builds full NPCs from a vector of `RawNpcs`
/// # Errors
/// - on failure to convert any `RawNpc` to `Npc`.
pub fn build_npcs(raw_npcs: &[RawNpc], symbols: &mut SymbolTable) -> Result<Vec<Npc>> {
    // add npcs to character symbol table - follows pattern of others, but necessary?
    for rnpc in raw_npcs {
        symbols.characters.insert(
            rnpc.id.to_string(),
            idgen::uuid_from_token(&NAMESPACE_CHARACTER, &rnpc.id),
        );
    }
    // build them
    let npcs: Vec<Npc> = raw_npcs
        .iter()
        .map(|rnpc| rnpc.to_npc(symbols))
        .collect::<Result<_, _>>()?;
    info!("{} NPCs successfully built from raw npcs", npcs.len());
    Ok(npcs)
}

/// Place NPCs in their respective starting areas on the map.
/// # Errors
/// - on invalid placement location
pub fn place_npcs(world: &mut AmbleWorld) -> Result<()> {
    // create job list of placements (NPC id , room id) and count unspawned for logging
    let mut placements: Vec<(Uuid, Uuid)> = Vec::new();
    let mut unspawned = 0;
    for npc in world.npcs.values() {
        match npc.location {
            Location::Room(uuid) => placements.push((npc.id, uuid)),
            Location::Nowhere => unspawned += 1,
            _ => {
                return Err(anyhow!(
                    "NPC {} ({}) bad location ({:?}) - must be Room or Nowhere",
                    npc.name(),
                    npc.id(),
                    npc.location()
                ));
            },
        }
    }
    // add each NPC's UUID to their start room
    for (npc_id, room_id) in &placements {
        let room = world
            .rooms
            .get_mut(room_id)
            .with_context(|| format!("looking up {room_id} to place {npc_id}"))?;
        room.npcs.insert(*npc_id);
    }
    info!("{} NPCs placed into their starting rooms", placements.len());
    info!("{unspawned} NPCs remain unspawned (Location::Nowhere)");
    Ok(())
}
