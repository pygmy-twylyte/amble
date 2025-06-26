//! `repl::look` module
//!
//! Contains repl loop handlers for commands that involve examining items and surroundings

use std::collections::HashSet;

use crate::{
    AmbleWorld, WorldObject,
    item::ItemAbility,
    repl::{entity_not_found, find_world_object},
    style::GameStyle,
    trigger::{TriggerAction, TriggerCondition, check_triggers},
};

use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use uuid::Uuid;

/// Shows description of surroundings.
pub fn look_handler(world: &AmbleWorld) -> Result<()> {
    let room = world.player_room_ref()?;
    room.show(world)?;
    info!(
        "{} looked around {} ({})",
        world.player.name, room.name, room.id
    );
    Ok(())
}

/// Shows description of something (scoped to nearby items and npcs and inventory)
pub fn look_at_handler(world: &AmbleWorld, thing: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    // scope = local items + npcs + player inventory
    let search_scope: HashSet<Uuid> = current_room
        .contents
        .union(&current_room.npcs)
        .copied()
        .chain(world.player.inventory.iter().copied())
        .collect();
    if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, thing) {
        if let Some(item) = entity.clone().item() {
            info!(
                "{} looked at {} ({})",
                world.player.name(),
                item.name(),
                item.id()
            );
            item.show(world)?;
        }
        if let Some(npc) = entity.npc() {
            info!(
                "{} looked at {} ({})",
                world.player.name(),
                npc.name(),
                npc.id()
            );
            npc.show(world)?;
        }
    } else {
        return entity_not_found(world, thing);
    }
    Ok(())
}

/// Shows list of items held in inventory.
pub fn inv_handler(world: &AmbleWorld) -> Result<()> {
    info!("{} checked inventory.", world.player.name());
    let banner = "Inventory".item_style().underline().bold();
    println!("{banner}");
    if world.player.inventory.is_empty() {
        println!("\tYou have... nothing. Nothing at all.");
    } else {
        println!("You have {} item(s):", world.player.inventory.len());
        world
            .player
            .inventory
            .iter()
            .filter_map(|item_id| world.items.get(item_id))
            .for_each(|item| println!("\t{}", item.name.item_style()));
    }
    Ok(())
}

/// Reads item, if it can be read.
///
/// A DenyRead("reason") trigger action can be set to make reading an item conditional.
/// Ex. TriggerCondition::UseItem{...read} + TriggerCondition::HasItem(magnifying_glass) -->
/// TriggerAction::DenyRead("The print is too small for you to read unaided.")
pub fn read_handler(world: &mut AmbleWorld, pattern: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    // scope search to items in room + inventory
    let search_scope: HashSet<Uuid> = current_room
        .contents
        .union(&world.player.inventory)
        .copied()
        .collect();
    // find the item from the search pattern and collect uuid;
    // log and tell player if there's nothing there to read
    let found_item_id = if let Some(item) =
        find_world_object(&search_scope, &world.items, &world.npcs, pattern).and_then(|e| e.item())
    {
        if item.text.is_some() {
            Some(item.id())
        } else {
            println!(
                "You see nothing legible on the {}.",
                item.name().item_style()
            );
            info!(
                "{} tried to read textless item {} ({})",
                world.player.name(),
                item.name(),
                item.id()
            );
            None
        }
    } else {
        return entity_not_found(world, pattern);
    };
    // check triggers for any DenyRead action that may have fired, and show the text if not
    if let Some(item_id) = found_item_id {
        let fired = check_triggers(
            world,
            &[TriggerCondition::UseItem {
                item_id: item_id,
                ability: ItemAbility::Read,
            }],
        )?;
        let denied = fired.iter().any(|trigger| {
            trigger
                .actions
                .iter()
                .any(|action| matches!(action, TriggerAction::DenyRead(_)))
        });
        if !denied {
            let item = world
                .items
                .get(&item_id)
                .with_context(|| format!("item_id ({item_id}) not found in world items"))?;

            println!("You can read:\n");
            println!(
                "{}",
                item.text
                    .as_deref()
                    .expect("item.text already known to be Some() here")
                    .description_style()
            );
            info!(
                "{} read '{}' ({})",
                world.player.name(),
                item.name(),
                item.id()
            );
        }
    }
    Ok(())
}
