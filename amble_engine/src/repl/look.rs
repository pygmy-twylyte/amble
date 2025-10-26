//! Observation and examination command handlers for the Amble game engine.
//!
//! This module provides handlers for commands that allow players to examine
//! their environment, items, and inventory without modifying world state.
//! These commands are essential for player understanding and navigation.
//!
//! # Command Categories
//!
//! ## Environmental Observation
//! - [`look_handler`] - Examine current surroundings in detail
//! - [`look_at_handler`] - Examine specific items, NPCs, or objects
//!
//! ## Inventory Management
//! - [`inv_handler`] - Display current inventory contents
//!
//! ## Text Interaction
//! - [`read_handler`] - Read text on items (books, signs, documents)
//!
//! # Scope Management
//!
//! The module implements intelligent scoping for examination commands:
//! - Current room contents (items and NPCs)
//! - Player inventory items
//! - Items within reach (including container contents)
//!
//! # Conditional Access
//!
//! Some examination commands may be conditional:
//! - Reading may require special tools (magnifying glass, light source)
//! - Examination triggers may provide different descriptions based on game state
//! - Certain items may only be readable under specific conditions
//!
//! # Trigger Integration
//!
//! Observation commands can trigger game events:
//! - Looking around may reveal hidden details or trigger story events
//! - Reading specific items may advance plot or provide crucial information
//! - Examination may unlock new areas or interactions

use std::collections::HashSet;

use crate::{
    AmbleWorld, View, ViewItem, WorldObject,
    item::ItemAbility,
    repl::{entity_not_found, find_world_object},
    style::GameStyle,
    trigger::{TriggerAction, TriggerCondition, check_triggers},
    view::{ContentLine, ViewMode},
    world::{nearby_reachable_items, nearby_visible_items},
};

use anyhow::{Context, Result};
use log::info;
use uuid::Uuid;

/// Shows description of surroundings.
///
/// # Errors
/// Returns an error if the player's current room cannot be resolved.
pub fn look_handler(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let room = world.player_room_ref()?;
    room.show(
        world,
        view,
        if view.mode == ViewMode::Brief {
            Some(ViewMode::Verbose)
        } else {
            None
        },
    )?;

    info!(
        "{} looked around {} ({})",
        world.player.name(),
        room.name(),
        room.symbol()
    );
    // Though "look" (at surroundings) doesn't generate an event, we still want ambient
    // and other non-event-driven triggers to fire -- so we check triggers with an empty
    // list of TriggerConditions.
    let _fired = check_triggers(world, view, &[]);
    world.turn_count += 1;
    Ok(())
}

/// Shows description of something (scoped to nearby items and npcs and inventory)
///
/// # Errors
/// Returns an error if the player's current room or the scoped items cannot be resolved.
pub fn look_at_handler(world: &mut AmbleWorld, view: &mut View, thing: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    // scope = local items + npcs + player inventory (including items visible in transparent containers)
    let items_visible = nearby_visible_items(world, current_room.id())?;
    let search_scope: HashSet<Uuid> = items_visible
        .union(&current_room.npcs)
        .copied()
        .chain(world.player.inventory.iter().copied())
        .collect();
    if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, thing) {
        if let Some(item) = entity.item() {
            info!("{} looked at {} ({})", world.player.name(), item.name(), item.symbol());
            item.show(world, view);
            let _fired = check_triggers(world, view, &[TriggerCondition::LookAt(item.id())]);
        } else if let Some(npc) = entity.npc() {
            info!("{} looked at {} ({})", world.player.name(), npc.name(), npc.symbol());
            npc.show(world, view);
            let _fired = check_triggers(world, view, &[]);
        }
    } else {
        entity_not_found(world, view, thing);
        return Ok(());
    }
    world.turn_count += 1;
    Ok(())
}

/// Shows list of items held in inventory.
///
/// # Errors
/// This handler never produces an error and always returns `Ok(())`.
pub fn inv_handler(world: &AmbleWorld, view: &mut View) -> Result<()> {
    info!("{} checked inventory.", world.player.name());
    view.push(ViewItem::Inventory(
        world
            .player
            .inventory
            .iter()
            .filter_map(|item_id| world.items.get(item_id))
            .map(|item| ContentLine {
                item_name: item.name.clone(),
                restricted: false,
            })
            .collect(),
    ));
    Ok(())
}

/// Reads item, if it can be read.
///
/// A DenyRead("reason") trigger action can be set to make reading an item conditional.
/// Ex. `TriggerCondition::UseItem{...read`} + `TriggerCondition::MissingItem(magnifying_glass)` -->
/// `TriggerAction::DenyRead("The` text is too small for you to read unaided.")
///
/// # Errors
/// Returns an error if the current room cannot be determined, if scoping nearby items fails,
/// or if trigger evaluation encounters missing world entities.
pub fn read_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    // scope search to items in room + inventory
    let items_in_reach = nearby_reachable_items(world, current_room.id())?;
    let search_scope: HashSet<Uuid> = items_in_reach.union(&world.player.inventory).copied().collect();
    // find the item from the search pattern and collect uuid;
    // log and tell player if there's nothing there to read
    let found_item_id = if let Some(item) =
        find_world_object(&search_scope, &world.items, &world.npcs, pattern).and_then(super::WorldEntity::item)
    {
        if item.text.is_some() {
            Some(item.id())
        } else {
            view.push(ViewItem::ActionFailure(format!(
                "You see nothing special about the {}, and nothing legible on it.",
                item.name().item_style()
            )));
            info!(
                "{} tried to read textless item {} ({})",
                world.player.name(),
                item.name(),
                item.symbol()
            );
            None
        }
    } else {
        entity_not_found(world, view, pattern);
        return Ok(());
    };
    // check triggers for any DenyRead action that may have fired, and show the text if not
    if let Some(item_id) = found_item_id {
        let fired = check_triggers(
            world,
            view,
            &[TriggerCondition::UseItem {
                item_id,
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

            view.push(ViewItem::ItemText(
                item.text.clone().unwrap_or_else(|| "(Nothing legible.)".to_string()),
            ));
            info!("{} read '{}' ({})", world.player.name(), item.name(), item.symbol());
        }
    }
    world.turn_count += 1;
    Ok(())
}
