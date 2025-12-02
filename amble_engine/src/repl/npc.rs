//! NPC interaction command handlers for the Amble game engine.
//!
//! This module contains handlers for commands that involve interactions with
//! non-player characters (NPCs) in the game world. NPCs are autonomous entities
//! that can hold items, engage in dialogue, and respond to player actions based
//! on their current state and mood.
//!
//! # Command Categories
//!
//! ## Communication
//! - [`talk_to_handler`] - Initiate dialogue with NPCs
//!
//! ## Item Exchange
//! - [`give_to_npc_handler`] - Give items from inventory to NPCs
//!
//! # NPC Behavior System
//!
//! NPCs have sophisticated behavior including:
//! - **Mood-based responses** - Different dialogue based on NPC state
//! - **Conditional item acceptance** - NPCs may accept or refuse items
//! - **State-dependent interactions** - Behavior changes with NPC state
//! - **Trigger-driven responses** - Custom responses to specific situations
//!
//! # Dialogue System
//!
//! NPC dialogue operates through multiple mechanisms:
//! - **Trigger-based dialogue** - Specific responses to conversation attempts
//! - **Mood-based responses** - Random dialogue selected based on NPC state
//! - **Fallback responses** - Default dialogue when no specific triggers fire
//!
//! # Item Transfer System
//!
//! Item transfers to NPCs are controlled by the trigger system:
//! - Transfers only succeed if specific triggers accept them
//! - NPCs can refuse items with custom messages
//! - Successful transfers update both player and NPC inventories
//! - Failed transfers provide appropriate feedback to the player
//!
//! # Trigger Integration
//!
//! NPC interactions can trigger various game events:
//! - `TriggerCondition::TalkToNpc` - When initiating conversation
//! - `TriggerCondition::GiveToNpc` - When attempting item transfers
//! - `TriggerAction::NpcSays` - For scripted dialogue responses
//! - `TriggerAction::NpcRefuseItem` - For item refusal with custom messages

use std::collections::HashMap;

use crate::{
    AmbleWorld, ItemHolder, Location, View, ViewItem, WorldObject,
    helpers::symbol_or_unknown,
    npc::Npc,
    repl::{entity_not_found, find_world_object},
    spinners::CoreSpinnerType,
    style::GameStyle,
    trigger::{TriggerAction, TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::{Context, Result};
use colored::Colorize;
use log::{info, warn};
use uuid::Uuid;

/// Finds an NPC in the specified location by partial name matching.
///
/// This utility function searches for NPCs in a given location using
/// case-insensitive partial string matching against NPC names.
///
/// # Parameters
///
/// * `location` - The location to search for NPCs
/// * `world_npcs` - Collection of all NPCs in the world
/// * `query` - Partial name string to match against NPC names
///
/// # Returns
///
/// Returns `Some(&Npc)` if a matching NPC is found, `None` otherwise.
///
/// # Behavior
///
/// - Uses case-insensitive matching for user convenience
/// - Returns the first NPC whose name contains the query string
/// - Only searches NPCs actually present in the specified location
fn select_npc<'a>(location: &Location, world_npcs: &'a HashMap<Uuid, Npc>, query: &str) -> Option<&'a Npc> {
    let npcs_in_room = world_npcs
        .values()
        .filter(|npc| npc.location() == location)
        .collect::<Vec<_>>();
    let query = query.to_lowercase();
    npcs_in_room
        .into_iter()
        .find(|&npc| npc.name().to_lowercase().contains(&query))
}
/// Initiates dialogue with an NPC in the current room.
///
/// This handler manages conversation attempts with NPCs, supporting both
/// trigger-based specific dialogue and fallback mood-based responses.
/// The dialogue system prioritizes custom trigger responses over generic
/// mood-based dialogue.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing NPCs
/// * `view` - Mutable reference to the player's view for dialogue display
/// * `npc_name` - Partial name string to identify the target NPC
///
/// # Returns
///
/// Returns `Ok(())` on successful dialogue attempt, or an error if
/// trigger processing fails.
///
/// # Dialogue Priority
///
/// 1. **Trigger-based dialogue** - Checked first for specific responses
/// 2. **Mood-based dialogue** - Fallback using NPC's current emotional state
/// 3. **Default responses** - Generic dialogue if no specific responses exist
///
/// # Behavior
///
/// - Searches current room for NPCs matching the name pattern
/// - Fires `TriggerCondition::TalkToNpc` to check for specific responses
/// - If no triggers fire, uses NPC's mood to select random dialogue
/// - All dialogue is displayed with proper NPC speech formatting
/// - Conversation attempts are logged for debugging and narrative tracking
///
/// # Errors
/// Returns an error if trigger evaluation fails or if the player's current location cannot be resolved.
pub fn talk_to_handler(world: &mut AmbleWorld, view: &mut View, npc_name: &str) -> Result<()> {
    // find one that matches npc_name in present room
    let sent_id = if let Some(npc) = select_npc(world.player.location(), &world.npcs, npc_name) {
        npc.id()
    } else {
        entity_not_found(world, view, npc_name);
        return Ok(());
    };

    // set a movement pause for 4 turns so NPC doesn't run off mid-interaction
    if let Some(npc) = world.npcs.get_mut(&sent_id) {
        npc.pause_movement(world.turn_count, 4);
    }

    // check for any condition-specific dialogue
    let fired_triggers = check_triggers(world, view, &[TriggerCondition::TalkToNpc(sent_id)])?;
    let dialogue_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
        TriggerCondition::TalkToNpc(npc_id) => sent_id == *npc_id,
        _ => false,
    });

    // if no dialogue was triggered, fire random response according to Npc's mood
    if !dialogue_fired {
        if let Some(npc) = world.npcs.get(&sent_id) {
            if let Some(ignore_spinner) = world
                .spinners
                .get(&crate::spinners::SpinnerType::Core(CoreSpinnerType::NpcIgnore))
            {
                let dialogue = npc.random_dialogue(ignore_spinner);
                view.push(ViewItem::NpcSpeech {
                    speaker: npc.name.clone(),
                    quote: dialogue.clone(),
                });
                info!("NPC \"{}\" ({}) said \"{}\"", npc.name(), npc.symbol(), dialogue);
            }
        }
    }
    world.turn_count += 1;
    Ok(())
}

/// Attempts to give an item from player inventory to an NPC.
///
/// This handler manages item transfers from the player to NPCs through a
/// trigger-based system. Items are only successfully transferred if specific
/// triggers exist to handle the exchange, allowing for complex NPC behavior
/// and story-driven item acceptance logic.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item` - Pattern string to match against inventory items
/// * `npc` - Pattern string to match against NPCs in current room
///
/// # Returns
///
/// Returns `Ok(())` on successful transfer attempt, regardless of whether
/// the NPC actually accepts the item.
///
/// # Transfer Logic
///
/// 1. **Target validation** - Finds and validates both item and NPC
/// 2. **Portability check** - Ensures item can be transferred
/// 3. **Trigger evaluation** - Checks if NPC will accept the item
/// 4. **Transfer execution** - Updates world state if accepted
/// 5. **Refusal handling** - Provides feedback if NPC refuses
///
/// # Trigger System
///
/// The transfer is controlled by triggers:
/// - `TriggerCondition::GiveToNpc` - Evaluated for each transfer attempt
/// - `TriggerAction::NpcRefuseItem` - Can provide custom refusal messages
/// - No matching triggers = automatic refusal with generic message
///
/// # World State Updates
///
/// On successful transfer:
/// - Item location updated to NPC
/// - Item removed from player inventory
/// - Item added to NPC inventory
/// - `TriggerCondition::Drop` fired for item placement effects
///
/// # Errors
///
/// - NPC not found in current room
/// - Item not found in player inventory
/// - Item is not portable (cannot be transferred)
/// - World state corruption (UUID lookup failures)
pub fn give_to_npc_handler(world: &mut AmbleWorld, view: &mut View, item: &str, npc: &str) -> Result<()> {
    // find the target npc in the current room and collect metadata
    let current_room = world.player_room_ref()?;
    let (npc_id, npc_name) = if let Some(entity) = find_world_object(&current_room.npcs, &world.items, &world.npcs, npc)
    {
        if let Some(npc) = entity.npc() {
            (npc.id(), npc.name.to_string())
        } else {
            view.push(ViewItem::Error(format!(
                "{} matches an item. Did you mean 'put {} in {}'?",
                npc.error_style(),
                item.italic(),
                npc.italic()
            )));
            return Ok(());
        }
    } else {
        entity_not_found(world, view, npc);
        return Ok(());
    };

    // set a movement pause for 4 turns so NPC doesn't run off mid-interaction
    if let Some(npc) = world.npcs.get_mut(&npc_id) {
        npc.pause_movement(world.turn_count, 4);
    }

    // find the target item in inventory, ensure it's portable, collect metadata
    let (item_id, item_name) =
        if let Some(entity) = find_world_object(&world.player.inventory, &world.items, &world.npcs, item) {
            if let Some(item) = entity.item() {
                if !item.portable {
                    info!("player tried to move fixed item {} ({})", item.name(), item.symbol());
                    view.push(ViewItem::ActionFailure(format!(
                        "Sorry, the {} isn't portable.",
                        item.name().error_style()
                    )));
                    return Ok(());
                }
                (item.id(), item.name().to_string())
            } else {
                warn!("non-Item entity matching '{item}' found in inventory");
                view.push(ViewItem::Error(format!(
                    "{} matched an entity that shouldn't exist in inventory. Let's pretend this never happened.",
                    item.error_style()
                )));
                return Ok(());
            }
        } else {
            entity_not_found(world, view, item);
            return Ok(());
        };

    let fired_triggers = check_triggers(world, view, &[TriggerCondition::GiveToNpc { item_id, npc_id }])?;
    let fired = fired_triggers.iter().any(|&trigger| {
        trigger
            .conditions
            .iter()
            .any(|cond| matches!(cond, TriggerCondition::GiveToNpc { .. }))
    });

    let refused = fired_triggers.iter().any(|t| {
        t.actions
            .iter()
            .any(|a| matches!(&a.action, TriggerAction::NpcRefuseItem { .. }))
    });

    // the trigger fired -- proceed with item transfer if it wasn't a refusal
    if fired && !refused {
        // The item may be despawned by a fired trigger -- so we skip
        // the location transfer below if the item is "nowhere" (otherwise the despawned
        // item winds up in the NPCs inventory anyway.)
        if world
            .items
            .get(&item_id)
            .with_context(|| format!("looking up item {item_id}"))?
            .location
            .is_not_nowhere()
        {
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
        }
        check_triggers(world, view, &[TriggerCondition::Drop(item_id)])?;

        // report and log success
        view.push(ViewItem::ActionSuccess(format!(
            "You gave the {} to {}.",
            item_name.item_style(),
            npc_name.npc_style()
        )));
        info!(
            "{} gave {} ({}) to {} ({})",
            world.player.name(),
            item_name,
            symbol_or_unknown(&world.items, item_id),
            npc_name,
            symbol_or_unknown(&world.npcs, npc_id),
        );
    // trigger didn't fire, so NPC refuses the item by default; a specific refusal reason
    // can be defined for particular items by setting an `NpcRefuseItem` trigger action.
    } else {
        if !fired {
            view.push(ViewItem::ActionFailure(format!(
                "{} has no use for {}, and won't hold it for you.",
                npc_name.npc_style(),
                item_name.item_style()
            )));
        }
        info!(
            "{npc_name} ({}) refused a gift of {item_name} ({})",
            symbol_or_unknown(&world.npcs, npc_id),
            symbol_or_unknown(&world.items, item_id)
        );
    }
    world.turn_count += 1;
    Ok(())
}
