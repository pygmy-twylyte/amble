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
    entity_search::{self, SearchError, SearchScope},
    health::{LifeState, LivingEntity},
    helpers::symbol_or_unknown,
    item::Movability,
    npc::Npc,
    repl::entity_not_found,
    spinners::CoreSpinnerType,
    style::GameStyle,
    trigger::{Trigger, TriggerAction, TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::{Context, Result, bail};

use crate::Id;
use log::info;

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
fn select_npc<'a>(location: &Location, world_npcs: &'a HashMap<Id, Npc>, query: &str) -> Option<&'a Npc> {
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
        // disallow talking to the dead
        if matches!(npc.life_state(), LifeState::Dead) {
            info!("talking to dead NPC {} disallowed", npc.symbol());
            view.push(ViewItem::ActionFailure(format!(
                "Sorry - {} is dead and there is no Ouija board in sight.",
                npc.name().npc_style()
            )));
            return Ok(());
        }
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
    let fired_triggers = check_triggers(world, view, &[TriggerCondition::TalkToNpc(sent_id.clone())])?;
    let dialogue_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
        TriggerCondition::TalkToNpc(npc_id) => sent_id == npc_id.as_str(),
        _ => false,
    });

    // if no dialogue was triggered, fire random response according to Npc's mood
    if !dialogue_fired
        && let Some(npc) = world.npcs.get(&sent_id)
        && let Some(ignore_spinner) = world
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
/// - Item is fixed (cannot be transferred)
/// - World state corruption (UUID lookup failures)
pub fn give_to_npc_handler(
    world: &mut AmbleWorld,
    view: &mut View,
    item_pattern: &str,
    npc_pattern: &str,
) -> Result<()> {
    // find the target npc in the current room and collect metadata
    let room_id = world.player_room_ref()?.id();
    let npc_id = match entity_search::find_npc_match(world, npc_pattern, SearchScope::TouchableNpcs(room_id)) {
        Ok(id) => {
            if let Some(npc) = world.npcs.get(&id)
                && npc.life_state() == LifeState::Dead
            {
                info!("gift to dead npc {} disallowed", npc.symbol());
                view.push(ViewItem::ActionFailure(format!(
                    "Sorry -- {} is dead, and cannot accept your gift.",
                    npc.name().npc_style()
                )));
                return Ok(());
            }
            id
        },
        Err(SearchError::NoMatchingName(input)) => {
            entity_not_found(world, view, input.as_str());
            return Ok(());
        },
        Err(e) => bail!(e),
    };

    // set a movement pause for 4 turns so NPC doesn't run off mid-interaction
    if let Some(npc) = world.npcs.get_mut(&npc_id) {
        npc.pause_movement(world.turn_count, 4);
    }

    // find the target item in inventory, ensure it isn't fixed
    let item_id = match entity_search::find_item_match(world, item_pattern, SearchScope::Inventory) {
        Ok(id) => {
            let item = world.items.get(&id).expect("validated in find_item_match()");
            if let Movability::Fixed { reason } = &item.movability {
                info!("player tried to move fixed item {} ({})", item.name(), item.symbol());
                view.push(ViewItem::ActionFailure(format!(
                    "Sorry, the {} isn't transferrable. {reason}",
                    item.name().error_style()
                )));
                return Ok(());
            }
            id
        },
        Err(SearchError::NoMatchingName(input)) => {
            entity_not_found(world, view, input.as_str());
            return Ok(());
        },
        Err(e) => bail!(e),
    };

    let fired_triggers = check_triggers(
        world,
        view,
        &[TriggerCondition::GiveToNpc {
            item_id: item_id.clone(),
            npc_id: npc_id.clone(),
        }],
    )?;
    let gift_response = check_fired_and_refused(&fired_triggers);

    // the trigger fired -- proceed with item transfer if it wasn't a refusal
    if gift_response.trigger_fired && !gift_response.npc_refused {
        transfer_if_not_despawned(world, npc_id.clone(), item_id.clone())?;
        check_triggers(world, view, &[TriggerCondition::Drop(item_id.clone())])?;
        show_npc_acceptance(world, view, npc_id.clone(), item_id.clone())?;
    } else if !gift_response.trigger_fired {
        // NPCs refuse gift items by default (triggers must be set to accept the gift)
        // a generic refusal message is given but responses to specific items can be set in triggers
        show_npc_refusal(world, view, npc_id.clone(), item_id.clone())?;
    }
    world.turn_count += 1;
    Ok(())
}

/// Displays NPC item refusal to the player and logs it.
fn show_npc_refusal(world: &AmbleWorld, view: &mut View, npc_id: Id, item_id: Id) -> Result<()> {
    let npc = world
        .npcs
        .get(&npc_id)
        .with_context(|| format!("missing npc UUID: {npc_id}"))?;
    let item = world
        .items
        .get(&item_id)
        .with_context(|| format!("missing item UUID: {item_id}"))?;

    view.push(ViewItem::ActionFailure(format!(
        "{} has no use for {}, and won't hold it for you.",
        npc.name(),
        item.name()
    )));

    info!(
        "{} ({}) refused a gift of {} ({})",
        npc.name(),
        symbol_or_unknown(&world.npcs, npc_id),
        item.name(),
        symbol_or_unknown(&world.items, item_id)
    );
    Ok(())
}

/// Displays NPC item acceptance and logs it.
///
/// # Errors
/// - on failed item or NPC retrieval by UUID
fn show_npc_acceptance(world: &AmbleWorld, view: &mut View, npc_id: Id, item_id: Id) -> Result<()> {
    let npc = world
        .npcs
        .get(&npc_id)
        .with_context(|| format!("missing npc UUID: {npc_id}"))?;
    let item = world
        .items
        .get(&item_id)
        .with_context(|| format!("missing item UUID: {item_id}"))?;

    view.push(ViewItem::ActionSuccess(format!(
        "{} accepted the {}.",
        npc.name(),
        item.name()
    )));

    info!(
        "'{}' ({}) accepted '{}' ({}) from '{}'",
        npc.name(),
        symbol_or_unknown(&world.npcs, npc_id),
        item.name(),
        symbol_or_unknown(&world.items, item_id),
        world.player.name()
    );
    Ok(())
}

/// Struct to encapsulate whether there was any response to a gift to an NPC, and whether it was a refusal.
struct GiftResponse {
    /// True if any triggers fired in response to the attempted gift to NPC
    trigger_fired: bool,
    /// True if the NPC refused an attempted gift
    npc_refused: bool,
}

/// Checks whether any triggers fired in response to a gift and whether the NPC refused it.
fn check_fired_and_refused(all_fired: &Vec<&Trigger>) -> GiftResponse {
    let trigger_fired = all_fired.iter().any(|&trigger| {
        trigger
            .conditions
            .iter()
            .any(|cond| matches!(cond, TriggerCondition::GiveToNpc { .. }))
    });

    let npc_refused = all_fired.iter().any(|t| {
        t.actions
            .iter()
            .any(|a| matches!(&a.action, TriggerAction::NpcRefuseItem { .. }))
    });

    GiftResponse {
        trigger_fired,
        npc_refused,
    }
}

/// Transfers an item from player to NPC if it hasn't been despawned
fn transfer_if_not_despawned(world: &mut AmbleWorld, npc_id: Id, item_id: Id) -> Result<()> {
    // The item may have been despawned by a fired trigger -- so we skip
    // the location transfer below if the item is found to be `Nowhere`
    if world
        .items
        .get(&item_id)
        .with_context(|| format!("looking up item {item_id}"))?
        .location
        .is_not_nowhere()
    {
        // set new location in NPC on world item
        world
            .get_item_mut(item_id.clone())
            .with_context(|| format!("looking up item {item_id}"))?
            .set_location_npc(npc_id.clone());

        // add to npc inventory
        world
            .npcs
            .get_mut(&npc_id)
            .with_context(|| format!("looking up NPC {npc_id}"))?
            .add_item(item_id.clone());

        // remove from player inventory
        world.player.remove_item(item_id);
    }
    Ok(())
}
