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
    npc::{Npc, NpcMood},
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
    pub dialogue: HashMap<NpcMood, Vec<String>>,
    pub mood: NpcMood,
}
impl RawNpc {
    /// Converts this `RawNpc` to a real `Npc`.
    pub fn to_npc(&self, symbols: &SymbolTable) -> Result<Npc> {
        let start_room = resolve_location(&self.location, symbols)?;
        Ok(Npc {
            id: *symbols
                .characters
                .get(&self.id)
                .ok_or_else(|| anyhow!("UUID for ({}) not found in character symbols", self.id))?,
            name: self.name.to_string(),
            description: self.description.to_string(),
            location: start_room,
            inventory: HashSet::new(),
            dialogue: self.dialogue.clone(),
            mood: self.mood,
        })
    }
}

/// Loads raw NPCs from file.
pub fn load_raw_npcs(toml_path: &Path) -> Result<Vec<RawNpc>> {
    let file_contents = fs::read_to_string(toml_path)
        .with_context(|| format!("reading NPC data from '{}'", toml_path.display()))?;
    let wrapper: RawNpcFile = toml::from_str(&file_contents)
        .with_context(|| "parsing NPC data from file contents".to_string())?;
    info!(
        "{} raw NPCs loaded from {:?}",
        wrapper.npcs.len(),
        toml_path
    );
    Ok(wrapper.npcs)
}

/// Builds full NPCs from a vector of `RawNpcs`
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
            }
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
