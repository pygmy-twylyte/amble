//! Observation and examination command handlers for the Amble game engine.
//!
//! This module provides handlers for commands that allow players to examine
//! their environment, items, and inventory without modifying world state.
//!
//! # Commands
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
//! The player's overall "field of view" is limited by the `SearchScope` supplied,
//! and generally is limited to some subset of the `Items` and `Npcs` present at
//! the current location (inventory included). This can be further be limited in
//! a more granular way through `ItemVisibility` settings.
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
//! - Actions can alter the text / details on read/examine-enabled items.

use crate::{
    AmbleWorld, View, ViewItem, WorldObject,
    entity_search::{EntityId, SearchError, SearchScope, find_entity_match, find_item_match},
    item::ItemAbility,
    repl::entity_not_found,
    room::RoomScenery,
    style::GameStyle,
    trigger::{TriggerAction, TriggerCondition, check_triggers},
    view::{ContentLine, ViewMode},
};

use anyhow::{Context, Result, bail};
use log::info;

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
    let room_id = world.player_room_ref()?.id();
    let entity_id = match find_entity_match(world, thing, SearchScope::AllVisible(room_id)) {
        Ok(id) => id,
        Err(SearchError::NoMatchingName(input)) => {
            let room = world.player_room_ref()?;
            let lc_input = input.to_lowercase();
            if let Some(entry) = room
                .scenery
                .iter()
                .find(|scenery| scenery.name.to_lowercase().contains(&lc_input))
            {
                let desc = entry
                    .desc
                    .clone()
                    .or_else(|| {
                        room.scenery_default
                            .clone()
                            .map(|text| text.replace("{thing}", &markup_scenery_item(&entry.name)))
                    })
                    .unwrap_or_else(|| create_default_scenery_description(world, entry));
                view.push(ViewItem::ItemDescription {
                    name: entry.name.clone(),
                    description: desc,
                });
                info!("player looked at scenery item \"{}\"", entry.name);
                world.turn_count += 1;
                return Ok(());
            }
            entity_not_found(world, view, input.as_str());
            return Ok(());
        },
        Err(e) => bail!(e),
    };

    match entity_id {
        EntityId::Item(item_id) => {
            let item = world.items.get(&item_id).expect("known OK from find_entity_match");
            info!("{} looked at {} ({})", world.player.name(), item.name(), item.symbol());
            item.show(world, view);
            let _fired = check_triggers(world, view, &[TriggerCondition::LookAt(item.id())]);
        },
        EntityId::Npc(npc_id) => {
            let npc = world.npcs.get(&npc_id).expect("known OK from find_entity_match");
            info!("{} looked at {} ({})", world.player.name(), npc.name(), npc.symbol());
            npc.show(world, view);
            let _fired = check_triggers(world, view, &[]);
        },
    }

    world.turn_count += 1;
    Ok(())
}

/// Lowercases and adds markup to an item name for display in default descriptions.
fn markup_scenery_item(item_name: &str) -> String {
    format!("[[item]][[i]]{}[[/i]][[/item]]", item_name.to_lowercase())
}

/// Uses the `NothingSpecial` core spinner to build a default description for a scenery `Item`.
fn create_default_scenery_description(world: &AmbleWorld, entry: &RoomScenery) -> String {
    let default_desc = world.spin_core(
        crate::spinners::CoreSpinnerType::NothingSpecial,
        "Just a plain {thing}, you see nothing curious about it.",
    );
    let thing_substitute = markup_scenery_item(&entry.name);
    default_desc.replace("{thing}", &thing_substitute)
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
    let room_id = world.player_room_ref()?.id();
    let item_id = match find_item_match(world, pattern, SearchScope::VisibleItems(room_id)) {
        Ok(uuid) => uuid,
        Err(SearchError::NoMatchingName(input)) => {
            entity_not_found(world, view, input.as_str());
            return Ok(());
        },
        Err(e) => bail!(e),
    };
    let item = world.items.get(&item_id).expect("item_id known to be valid here");

    if item.text.is_none() || !item.abilities.contains(&ItemAbility::Read) {
        view.push(ViewItem::ActionFailure(format!(
            "You see nothing special about the {}, and nothing legible on it.",
            item.name().item_style()
        )));
        info!(
            "{} examined a detail-less item {} ({})",
            world.player.name(),
            item.name(),
            item.symbol()
        );
        return Ok(());
    }

    // check triggers for any DenyRead action that may have fired, and show the text if not
    let fired = check_triggers(
        world,
        view,
        &[TriggerCondition::UseItem {
            item_id: item.id(),
            ability: ItemAbility::Read,
        }],
    )?;

    let denied = fired.iter().any(|trigger| {
        trigger
            .actions
            .iter()
            .any(|action| matches!(&action.action, TriggerAction::DenyRead(_)))
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

    world.turn_count += 1;
    Ok(())
}
