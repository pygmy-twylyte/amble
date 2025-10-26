//! NPC loader for TOML content.
//!
//! Builds runtime NPC structures from serialized data, setting up dialogue, custom states,
//! movement schedules, and initial inventories.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, anyhow, bail};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    Location, WorldObject,
    idgen::{self, NAMESPACE_CHARACTER},
    npc::{MovementTiming, MovementType, Npc, NpcMovement, NpcState},
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
    pub state: NpcState,
    #[serde(default)]
    pub movement: Option<RawNpcMovement>,
}

/// Structure for loading NPC movement data from TOML files.
#[derive(Debug, Deserialize)]
pub struct RawNpcMovement {
    pub movement_type: String, // "route" or "random"
    pub rooms: Vec<String>,
    pub timing: String, // "every_N_turns" or "on_turn_N"
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(default = "default_loop_route")]
    pub loop_route: bool,
}

fn default_active() -> bool {
    true
}

fn default_loop_route() -> bool {
    true
}
impl RawNpc {
    /// Converts this `RawNpc` to a real `Npc`
    /// # Errors
    /// - on failure to resolve location or look up NPC in symbol table
    pub fn to_npc(&self, symbols: &SymbolTable) -> Result<Npc> {
        let start_room = resolve_location(&self.location, symbols)?;
        let mut processed_dialogue = HashMap::new();
        for (state_key, lines) in &self.dialogue {
            let state = NpcState::from_key(state_key);
            processed_dialogue.insert(state, lines.clone());
        }

        let movement = if let Some(ref raw_movement) = self.movement {
            Some(raw_movement.to_movement(symbols)?)
        } else {
            None
        };

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
            state: self.state.clone(),
            movement,
        })
    }
}

impl RawNpcMovement {
    /// Converts this `RawNpcMovement` to a real `NpcMovement`
    /// # Errors
    /// - on failure to resolve room symbols or parse timing
    pub fn to_movement(&self, symbols: &SymbolTable) -> Result<NpcMovement> {
        let movement_type = match self.movement_type.as_str() {
            "route" => {
                let mut room_uuids = Vec::new();
                for room_symbol in &self.rooms {
                    let room_uuid = symbols
                        .rooms
                        .get(room_symbol)
                        .ok_or_else(|| anyhow!("Room symbol '{}' not found in symbols", room_symbol))?;
                    room_uuids.push(*room_uuid);
                }
                MovementType::Route {
                    rooms: room_uuids,
                    current_idx: 0,
                    loop_route: self.loop_route,
                }
            },
            "random" => {
                let mut room_uuids = HashSet::new();
                for room_symbol in &self.rooms {
                    let room_uuid = symbols
                        .rooms
                        .get(room_symbol)
                        .ok_or_else(|| anyhow!("Room symbol '{}' not found in symbols", room_symbol))?;
                    room_uuids.insert(*room_uuid);
                }
                MovementType::RandomSet { rooms: room_uuids }
            },
            _ => bail!(
                "Invalid movement_type '{}' - must be 'route' or 'random'",
                self.movement_type
            ),
        };

        let timing = if self.timing.starts_with("every_") {
            let turns_str = self
                .timing
                .strip_prefix("every_")
                .and_then(|s| s.strip_suffix("_turns"))
                .ok_or_else(|| anyhow!("Invalid timing format '{}' - expected 'every_N_turns'", self.timing))?;
            let turns: usize = turns_str
                .parse()
                .with_context(|| format!("Failed to parse turn count from '{turns_str}'"))?;
            MovementTiming::EveryNTurns { turns }
        } else if self.timing.starts_with("on_turn_") {
            let turn_str = self
                .timing
                .strip_prefix("on_turn_")
                .ok_or_else(|| anyhow!("Invalid timing format '{}' - expected 'on_turn_N'", self.timing))?;
            let turn: usize = turn_str
                .parse()
                .with_context(|| format!("Failed to parse turn number from '{turn_str}'"))?;
            MovementTiming::OnTurn { turn }
        } else {
            bail!(
                "Invalid timing format '{}' - must start with 'every_' or 'on_turn_'",
                self.timing
            );
        };

        Ok(NpcMovement {
            movement_type,
            timing,
            active: self.active,
            last_moved_turn: 0,
            paused_until: None,
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
/// - if a pre-registered NPC (during room loading) isn't found in the NPC data file.
/// - on failure to convert any `RawNpc` to `Npc`.
pub fn build_npcs(raw_npcs: &[RawNpc], symbols: &mut SymbolTable) -> Result<Vec<Npc>> {
    // save any pre-registered NPCs and clear them from the symbol table
    let early_inserts = symbols.characters.clone();
    symbols.characters.clear();
    info!(
        "found {} NPC(s) that were pre-registered while loading rooms",
        early_inserts.len()
    );

    // add npcs from npcs.toml to character symbol table
    for rnpc in raw_npcs {
        symbols.characters.insert(
            rnpc.id.to_string(),
            idgen::uuid_from_token(&NAMESPACE_CHARACTER, &rnpc.id),
        );
    }

    // make sure each pre-registered NPC exists in loaded data and UUID is correct
    for (npc_symbol, npc_id) in &early_inserts {
        if symbols.characters.get(npc_symbol).is_none_or(|id| id != npc_id) {
            bail!("error while loading pre-registered NPC '{npc_symbol}': symbol not found or uuid mismatch");
        }
    }
    info!("existence of {} pre-registered NPC(s) verified", early_inserts.len());

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
