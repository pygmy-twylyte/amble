pub mod goals;
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
use anyhow::{Context, Result, anyhow};
use items::{build_items, place_items};
use log::{error, info};
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
    if table.contains_key("Inventory") {
        Ok(Location::Inventory)
    } else if let Some(room_token) = table.get("Room") {
        symbols.rooms.get(room_token).map_or_else(
            || {
                error!("room token '{room_token}' not found in symbol table");
                Err(anyhow!("room token '{}' not found in symbol table", room_token))
            },
            |uuid| Ok(Location::Room(*uuid)),
        )
    } else if let Some(chest_token) = table.get("Chest") {
        symbols.items.get(chest_token).map_or_else(
            || {
                error!("container item token '{chest_token}' not found in symbol table");
                Err(anyhow!("chest_token '{}' not found in symbol table", chest_token))
            },
            |uuid| Ok(Location::Item(*uuid)),
        )
    } else if let Some(npc_token) = table.get("Npc") {
        symbols.characters.get(npc_token).map_or_else(
            || {
                error!("Npc item token '{npc_token}' not found in symbol table");
                Err(anyhow!("npc token '{}' not found in symbol table", npc_token))
            },
            |uuid| Ok(Location::Npc(*uuid)),
        )
    } else {
        Ok(Location::Nowhere)
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
    let rooms = build_rooms(&raw_rooms, &mut symbols).context("while building rooms from raw_rooms")?;
    for rm in rooms {
        world.rooms.insert(rm.id(), rm);
    }
    world.max_score += world.rooms.len();
    info!("{} rooms added to AmbleWorld", world.rooms.len());

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

    /* Load NPCs */
    let raw_npcs = load_raw_npcs(npc_toml_path).context("while loading raw npcs from file")?;
    let npcs = build_npcs(&raw_npcs, &mut symbols).context("while building npcs from raw npcs")?;
    for npc in npcs {
        world.npcs.insert(npc.id(), npc);
    }
    info!("{} NPCs added to AmbleWorld", world.npcs.len());
    () = place_npcs(&mut world)?;

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
