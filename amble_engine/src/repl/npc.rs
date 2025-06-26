//! `repl::inventory` module
//!
//! Contains repl loop handlers for commands that affect player inventory

use std::collections::HashMap;

use crate::{
    AmbleWorld, ItemHolder, Location, WorldObject,
    npc::Npc,
    repl::{entity_not_found, find_world_object},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
};

use anyhow::{Context, Result};
use colored::Colorize;
use log::{info, warn};
use uuid::Uuid;

/// Selects an NPC in given location by first partial name match.
fn select_npc<'a>(
    location: &Location,
    world_npcs: &'a HashMap<Uuid, Npc>,
    query: &str,
) -> Option<&'a Npc> {
    let npcs_in_room = world_npcs
        .values()
        .filter(|npc| npc.location() == location)
        .collect::<Vec<_>>();
    let query = query.to_lowercase();
    npcs_in_room
        .into_iter()
        .find(|&npc| npc.name().to_lowercase().contains(&query))
}
/// Handles TalkTo(npc) commands
pub fn talk_to_handler(world: &mut AmbleWorld, npc_name: &str) -> Result<()> {
    // find one that matches npc_name in present room
    if let Some(npc) = select_npc(world.player.location(), &world.npcs, npc_name) {
        // success -> call random dialogue
        let stem = format!("{}", npc.name().npc_style());
        let dialogue = npc.random_dialogue(world.spinners.get(&SpinnerType::NpcIgnore).unwrap());
        println!("{stem}: {dialogue}");
        info!(
            "{} talked to NPC \"{}\" ({})",
            world.player.name(),
            npc.name(),
            npc.id()
        );
    } else {
        return entity_not_found(world, npc_name);
    }
    Ok(())
}

/// Gives an inventory item to an NPC
pub fn give_to_npc_handler(world: &mut AmbleWorld, item: &str, npc: &str) -> Result<()> {
    // find the target npc in the current room and collect metadata
    let current_room = world.player_room_ref()?;
    let (npc_id, npc_name) = if let Some(entity) =
        find_world_object(&current_room.npcs, &world.items, &world.npcs, npc)
    {
        if let Some(npc) = entity.npc() {
            (npc.id(), npc.name.to_string())
        } else {
            println!(
                "{} matches an item. Did you mean 'put {} in {}'?",
                npc.error_style(),
                item.italic(),
                npc.italic()
            );
            return Ok(());
        }
    } else {
        return entity_not_found(world, npc);
    };

    // find the target item in inventory, ensure it's portable, collect metadata
    let (item_id, item_name) = if let Some(entity) =
        find_world_object(&world.player.inventory, &world.items, &world.npcs, item)
    {
        if let Some(item) = entity.item() {
            if !item.portable {
                info!(
                    "player tried to move fixed item {} ({})",
                    item.name(),
                    item.id()
                );
                println!("Sorry, the {} isn't portable.", item.name().error_style());
                return Ok(());
            }
            (item.id(), item.name().to_string())
        } else {
            warn!("non-Item entity matching '{item}' found in inventory");
            println!(
                "{} matched an entity that shouldn't exist in inventory. Let's pretend this never happened.",
                item.error_style()
            );
            return Ok(());
        }
    } else {
        return entity_not_found(world, item);
    };

    // set new location in NPC on world item
    world
        .get_item_mut(item_id)
        .with_context(|| format!("looking up item {item_id}"))?
        .set_location_npc(npc_id);

    // add to npc inventory
    world
        .npcs
        .get_mut(&npc_id)
        .with_context(|| format!("looking up NPC {npc_id}"))?
        .add_item(item_id);

    // remove from player inventory
    world.player.remove_item(item_id);

    // report and log success
    println!(
        "You gave the {} to {}.\n",
        item_name.item_style(),
        npc_name.npc_style()
    );
    info!(
        "{} gave {} ({}) to {} ({})",
        world.player.name(),
        item_name,
        item_id,
        npc_name,
        npc_id
    );
    // check appropriate triggers
    check_triggers(
        world,
        &[
            TriggerCondition::Drop(item_id),
            TriggerCondition::GiveToNpc { item_id, npc_id },
        ],
    )?;
    Ok(())
}
