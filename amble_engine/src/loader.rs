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
pub mod spinners;
pub mod triggers;

use crate::loader::goals::{build_goals, load_raw_goals};
use crate::loader::items::load_raw_items;
use crate::loader::player::load_player;
use crate::loader::rooms::load_raw_rooms;
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
use std::path::Path;
use triggers::build_triggers;
use uuid::Uuid;

#[derive(Default, Debug)]
/// Lookup table to find the uuid for a given token
pub struct SymbolTable {
    rooms: HashMap<String, Uuid>,
    items: HashMap<String, Uuid>,
    characters: HashMap<String, Uuid>,
}

/// Converts the TOML table representation of location into a proper Location
/// # Errors
/// - if room or container token cannot be found in the symbol table
pub fn resolve_location(table: &HashMap<String, String>, symbols: &SymbolTable) -> Result<Location> {
    match table.keys().next().map(|key| key.as_str()) {
        Some("Inventory") => Ok(Location::Inventory),
        Some("Room") => room_resolver(table, symbols),
        Some("Chest") => chest_resolver(table, symbols),
        Some("Npc") => npc_resolver(table, symbols),
        Some("Nowhere") => Ok(Location::Nowhere),
        Some(_) => Err(anyhow!("Invalid location type found in TOML table [{table:?}]")),
        None => Err(anyhow!("No location type found in TOML table [{table:?}]")),
    }
}

/// Passed a TOML table location containing a Room entry, this returns the corresponding Location::Room(room_uuid).
fn room_resolver(table: &HashMap<String, String>, symbols: &SymbolTable) -> Result<Location> {
    if let Some(room_uuid) = symbols.rooms.get(
        table
            .get("Room")
            .context("room_resolver called without a 'Room' in location table")?,
    ) {
        Ok(Location::Room(*room_uuid))
    } else {
        bail!("room_resolver: Room symbol from TOML location table [{table:?}] not found in symbol table")
    }
}

/// Passed a TOML table location containing a Chest entry, this returns the corresponding Location::Item(item_uuid).
fn chest_resolver(table: &HashMap<String, String>, symbols: &SymbolTable) -> Result<Location> {
    if let Some(item_uuid) = symbols.items.get(
        table
            .get("Chest")
            .context("chest_resolver called without a 'Chest' in location table")?,
    ) {
        Ok(Location::Item(*item_uuid))
    } else {
        bail!("chest_resolver: Item symbol from TOML location table [{table:?}] not found in symbol table")
    }
}

/// Passed a TOML table location containing a Npc entry, this returns the corresponding Location::Npc(npc_uuid).
fn npc_resolver(table: &HashMap<String, String>, symbols: &SymbolTable) -> Result<Location> {
    if let Some(npc_uuid) = symbols.characters.get(
        table
            .get("Npc")
            .context("npc_resolver called without a 'Npc' in location table")?,
    ) {
        Ok(Location::Npc(*npc_uuid))
    } else {
        bail!("npc_resolver: Npc symbol from TOML table [{table:?}] not found in symbol table")
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

    let mut world = AmbleWorld::new_empty();
    let mut symbols = SymbolTable::default();

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
