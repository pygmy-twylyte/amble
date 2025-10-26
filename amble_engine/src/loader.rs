//! Loader utilities for building an `AmbleWorld` from TOML files.
//!
//! Each submodule handles parsing and conversion of one data type such as rooms
//! or items. The main [`load_world`] function ties them all together.

pub mod goals;
pub mod help;
pub mod items;
pub mod npcs;
pub mod player;
pub mod rooms;
pub mod scoring;
pub mod spinners;
pub mod triggers;

use crate::loader::goals::{build_goals, load_raw_goals};
use crate::loader::items::load_raw_items;
use crate::loader::player::load_player;
use crate::loader::rooms::load_raw_rooms;
use crate::loader::scoring::load_scoring;
use crate::loader::spinners::load_spinners;
use crate::loader::triggers::load_raw_triggers;

use crate::trigger::TriggerAction;
use crate::{AmbleWorld, Location, WorldObject};
use anyhow::{Context, Result, anyhow, bail};
use items::{build_items, place_items};
use log::info;
use npcs::{build_npcs, load_raw_npcs, place_npcs};
use player::build_player;
use rooms::build_rooms;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::path::Path;
use triggers::build_triggers;
use uuid::Uuid;

/// Lookup table to find the uuid for a given token
#[derive(Default, Debug)]
pub struct SymbolTable {
    pub(crate) rooms: HashMap<String, Uuid>,
    pub(crate) items: HashMap<String, Uuid>,
    pub(crate) characters: HashMap<String, Uuid>,
}

/// Resolve a token from a TOML location table against the symbol cache.
fn map_resolver<S, H>(table: &HashMap<String, String, S>, map: &HashMap<String, Uuid, H>, key: &str) -> Result<Uuid>
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
        Ok(*uuid)
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
    let item_toml_path = Path::new("amble_engine/data/items.toml");
    let room_toml_path = Path::new("amble_engine/data/rooms.toml");
    let player_toml_path = Path::new("amble_engine/data/player.toml");
    let npc_toml_path = Path::new("amble_engine/data/npcs.toml");
    let trigger_toml_path = Path::new("amble_engine/data/triggers.toml");
    let spinners_toml_path = Path::new("amble_engine/data/spinners.toml");
    let goal_toml_path = Path::new("amble_engine/data/goals.toml");
    let scoring_toml_path = Path::new("amble_engine/data/scoring.toml");

    let mut world = AmbleWorld::new_empty();
    let mut symbols = SymbolTable::default();

    /* Load Scoring Configuration */
    world.scoring = load_scoring(scoring_toml_path);
    info!("Scoring configuration loaded with {} ranks", world.scoring.ranks.len());

    /* Load Spinners */
    world.spinners = load_spinners(spinners_toml_path).context("while loading spinners from file")?;
    info!("{} spinners added to AmbleWorld", world.spinners.len());

    /* Load Empty Rooms */
    let raw_rooms = load_raw_rooms(room_toml_path).context("while loading rooms from file")?;
    let rooms = build_rooms(&raw_rooms, &mut symbols).context("whi)le building rooms from raw_rooms")?;
    for rm in rooms {
        world.rooms.insert(rm.id(), rm);
    }
    world.max_score += world.rooms.len();
    info!("{} rooms added to AmbleWorld", world.rooms.len());

    /* Load NPCs */
    let raw_npcs = load_raw_npcs(npc_toml_path).context("while loading raw npcs from file")?;
    let npcs = build_npcs(&raw_npcs, &mut symbols).context("while building npcs from raw npcs")?;
    for npc in npcs {
        world.npcs.insert(npc.id(), npc);
    }
    info!("{} NPCs added to AmbleWorld", world.npcs.len());
    () = place_npcs(&mut world)?;

    /* Load Player */
    let raw_player = load_player(player_toml_path).context("while loading player from file")?;
    let start_room_token = raw_player
        .location
        .get("Room")
        .ok_or_else(|| anyhow!("failed extraction of start room token id"))?;
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

    /* Load Items */
    let raw_items = load_raw_items(item_toml_path).context("while loading raw items from file")?;
    let items = build_items(&raw_items, &mut symbols).context("while building items from raw items")?;
    for item in items {
        world.items.insert(item.id(), item);
    }
    info!("{} items added to AmbleWorld", world.items.len());
    () = place_items(&mut world)?;

    /* Load Triggers */
    let raw_triggers = load_raw_triggers(trigger_toml_path).context("when loading triggers from file")?;
    world.triggers = build_triggers(&raw_triggers, &symbols)?;
    info!("{} triggers added to AmbleWorld", world.triggers.len());

    for trigger in &world.triggers {
        for action in &trigger.actions {
            if let TriggerAction::AwardPoints(amount) = action
                && *amount > 0
            {
                world.max_score = world.max_score.saturating_add_signed(*amount);
            }
        }
    }

    /* Load Goals */
    let raw_goals = load_raw_goals(goal_toml_path).context("when loading goals from file")?;
    world.goals = build_goals(&raw_goals, &symbols)?;
    info!("{} goals added to AmbleWorld", world.goals.len());

    Ok(world)
}
