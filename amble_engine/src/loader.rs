//! Loader utilities for building an `AmbleWorld` from serialized data.
//!
//! World content is loaded from the compiled `WorldDef` (RON), while legacy
//! TOML files still provide player and scoring configuration.

pub mod goals;
pub mod help;
pub mod items;
pub mod npcs;
pub mod player;
pub mod rooms;
pub mod scoring;
pub mod spinners;
pub mod triggers;
pub mod worlddef;

use crate::loader::player::load_player;
use crate::loader::scoring::load_scoring;
use crate::loader::worlddef::{build_world_from_def, load_worlddef};

use crate::Id;
use crate::data_paths::data_path;
use crate::trigger::TriggerAction;
use crate::{AmbleWorld, Location, WorldObject};
use amble_data::WorldDef;
use anyhow::{Context, Result, anyhow, bail};
use items::place_items;
use log::info;
use npcs::place_npcs;
use player::build_player;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Lookup table to find the uuid for a given token
#[derive(Default, Debug)]
pub struct SymbolTable {
    pub(crate) rooms: HashMap<String, Id>,
    pub(crate) items: HashMap<String, Id>,
    pub(crate) characters: HashMap<String, Id>,
}

/// Resolve a token from a TOML location table against the symbol cache.
fn map_resolver<S, H>(table: &HashMap<String, String, S>, map: &HashMap<String, Id, H>, key: &str) -> Result<Id>
where
    S: BuildHasher,
    H: BuildHasher,
{
    let key_lc = key.to_lowercase();
    if let Some(uuid) = map.get(
        table
            .get(key)
            .with_context(|| format!("{key_lc}_resolver called without a '{key}' in location table"))?,
    ) {
        Ok(uuid.clone())
    } else {
        bail!("{key_lc}_resolver: {key} symbol from TOML location table [{table:?}] not found in symbol table")
    }
}

/// Converts the TOML table representation of location into a proper Location
/// # Errors
/// - if room or container token cannot be found in the symbol table
pub fn resolve_location<S: BuildHasher>(table: &HashMap<String, String, S>, symbols: &SymbolTable) -> Result<Location> {
    match table.keys().next().map(std::string::String::as_str) {
        Some("Inventory") => Ok(Location::Inventory),
        Some("Room") => map_resolver(table, &symbols.rooms, "Room").map(Location::Room),
        Some("Chest") => map_resolver(table, &symbols.items, "Chest").map(Location::Item),
        Some("Npc") => map_resolver(table, &symbols.characters, "Npc").map(Location::Npc),
        Some("Nowhere") => Ok(Location::Nowhere),
        Some(_) => Err(anyhow!("Invalid location type found in TOML table [{table:?}]")),
        None => Err(anyhow!("No location type found in TOML table [{table:?}]")),
    }
}
/// Loads the `AmbleWorld` from TOML files
/// # Errors
/// Multiple errors can be returned from file IO and symbol table lookups
pub fn load_world() -> Result<AmbleWorld> {
    let world_ron_path = data_path("world.ron");
    let player_toml_path = data_path("player.toml");
    let scoring_toml_path = data_path("scoring.toml");

    let worlddef = load_worlddef(&world_ron_path).context("while loading worlddef from file")?;
    validate_worlddef(&worlddef)?;
    let mut world = build_world_from_def(&worlddef).context("while building world from worlddef")?;
    info!("{} spinners added to AmbleWorld", world.spinners.len());
    info!("{} rooms added to AmbleWorld", world.rooms.len());
    info!("{} NPCs added to AmbleWorld", world.npcs.len());
    info!("{} items added to AmbleWorld", world.items.len());
    info!("{} triggers added to AmbleWorld", world.triggers.len());
    info!("{} goals added to AmbleWorld", world.goals.len());

    world.scoring = load_scoring(&scoring_toml_path);
    info!("Scoring configuration loaded with {} ranks", world.scoring.ranks.len());

    let raw_player = load_player(&player_toml_path).context("while loading player from file")?;
    let start_room_token = raw_player
        .location
        .get("Room")
        .ok_or_else(|| anyhow!("failed extraction of start room token id"))?;
    let mut symbols = symbols_from_worlddef(&worlddef);
    world.player = build_player(&raw_player, &mut symbols).context("while building player from raw player")?;
    world.player_path.push(
        world
            .player
            .location
            .room_id()
            .context("player start location is not a room")?,
    );
    info!(
        "player \"{}\" added to AmbleWorld at {}",
        world.player.name(),
        start_room_token
    );

    place_npcs(&mut world)?;
    place_items(&mut world)?;

    for trigger in &world.triggers {
        for action in &trigger.actions {
            if let TriggerAction::AwardPoints { amount, .. } = &action.action
                && *amount > 0
            {
                world.max_score = world.max_score.saturating_add_signed(*amount);
            }
        }
    }

    Ok(world)
}

fn symbols_from_worlddef(def: &WorldDef) -> SymbolTable {
    let mut symbols = SymbolTable::default();
    for room in &def.rooms {
        symbols.rooms.insert(room.id.clone(), room.id.clone());
    }
    for item in &def.items {
        symbols.items.insert(item.id.clone(), item.id.clone());
    }
    for npc in &def.npcs {
        symbols.characters.insert(npc.id.clone(), npc.id.clone());
    }
    symbols
}

fn validate_worlddef(def: &WorldDef) -> Result<()> {
    let errors = amble_data::validate_world(def);
    if errors.is_empty() {
        return Ok(());
    }
    let details = errors
        .into_iter()
        .map(|err| format!("- {err}"))
        .collect::<Vec<_>>()
        .join("\n");
    bail!("worlddef validation failed:\n{details}");
}
