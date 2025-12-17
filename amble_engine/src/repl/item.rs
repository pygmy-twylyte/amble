//! Item interaction and manipulation command handlers for the Amble game engine.
//!
//! This module contains handlers for commands that directly interact with or modify
//! the state of items in the game world. These commands allow players to use items
//! in various ways, manipulate container states, and interact with the physical
//! properties of objects.
//!
//! # Command Categories
//!
//! ## Item Usage
//! - [`use_item_on_handler`] - Use one item on another with specific interactions
//! - [`turn_on_handler`] - Activate items that can be switched on
//! - [`turn_off_handler`] - Stop / disable items that have that ability
//!
//! ## Container Management
//! - [`open_handler`] - Open closed containers to access contents
//! - [`close_handler`] - Close open containers for security or organization
//! - [`lock_handler`] - Lock containers to prevent access
//! - [`unlock_handler`] - Unlock containers using appropriate keys
//!
//! # Interaction System
//!
//! The module implements a sophisticated item interaction system where:
//! - Items can require specific capabilities for different interactions
//! - Tools must have the right abilities to perform actions on targets
//! - Interactions can trigger game events and story progression
//! - Failed interactions provide helpful feedback about requirements
//!
//! # Container States
//!
//! Containers can exist in multiple states:
//! - **Open** - Contents are accessible and visible
//! - **Closed** - Contents are hidden but can be opened
//! - **Locked** - Contents are secured and require keys to access
//!
//! # Trigger Integration
//!
//! Item interactions can trigger various game events:
//! - `TriggerCondition::UseItem` - When items are activated or used (but there is no target)
//! - `TriggerCondition::UseItemOnItem` - When specific tools are used on specific targets
//! - `TriggerCondition::ActOnItem` - When actions are performed on items (regardless of tool used)
//! - `TriggerCondition::Open` - When containers are opened
//! - `TriggerCondition::Unlock` - When locked items are unlocked
//!
//! These triggers enable rich gameplay where item interactions can advance
//! storylines, solve puzzles, unlock areas, or cause other game effects.

use std::collections::{HashMap, HashSet};

use crate::{
    AmbleWorld, View, ViewItem, WorldObject,
    command::IngestMode,
    entity_search::{self, SearchError, SearchScope},
    helpers::{plural_s, symbol_or_unknown},
    item::{ContainerState, Item, ItemAbility, ItemInteractionType, consume},
    loader::items::interaction_requirement_met,
    repl::entity_not_found,
    spinners::CoreSpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::{Result, bail};
use colored::Colorize;
use log::info;
use uuid::Uuid;

/// Touch or press an `Item`.
///
/// # Errors
/// Returns an error if the player's current room cannot be resolved or if the scoped items
/// referenced during trigger evaluation cannot be found.
pub fn touch_handler(world: &mut AmbleWorld, view: &mut View, item_str: &str) -> Result<()> {
    let room_id = world.player.location.room_id()?;
    let item_id = match entity_search::find_item_match(world, item_str, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(SearchError::NoMatchingName(_)) => {
            entity_not_found(world, view, item_str);
            return Ok(());
        },
        Err(e) => bail!(e),
    };

    let triggers_fired = check_triggers(world, view, &[TriggerCondition::Touch(item_id)])?;
    let sent_trigger_fired = triggers_contain_condition(&triggers_fired, |trig| match trig {
        TriggerCondition::Touch(triggered_item_id) => *triggered_item_id == item_id,
        _ => false,
    });
    let item = world.items.get(&item_id).expect("item_id should be valid here");
    if !sent_trigger_fired {
        info!(
            "{} touched {} ({})... and nothing happened.",
            world.player.name(),
            item.name(),
            item.symbol(),
        );
        view.push(ViewItem::ActionSuccess(
            world.spin_core(CoreSpinnerType::NoEffect, "That has no discernable effect."),
        ));
    }
    world.turn_count += 1;
    Ok(())
}

/// Ingests an item (or single portion of a multi-use item).
///
/// # Errors
/// Returns an error if the player's current room cannot be located, if scoped items cannot be
/// resolved, or if trigger evaluation encounters missing world state.
pub fn ingest_handler(world: &mut AmbleWorld, view: &mut View, item_str: &str, mode: IngestMode) -> Result<()> {
    let room_id = world.player_room_ref()?.id();
    let item_id = match entity_search::find_item_match(world, item_str, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(SearchError::NoMatchingName(_)) => {
            entity_not_found(world, view, item_str);
            return Ok(());
        },
        Err(e) => bail!(e),
    };

    // found one -- does item have the ability to be ingested this way?
    let item = world
        .items
        .get(&item_id)
        .expect("item_id already known to exist in map here");
    let (item_name, item_symbol) = (item.name().to_owned(), item.symbol().to_owned());
    let meets_mode = match mode {
        IngestMode::Eat => item.abilities.contains(&ItemAbility::Eat),
        IngestMode::Drink => item.abilities.contains(&ItemAbility::Drink),
        IngestMode::Inhale => item.abilities.contains(&ItemAbility::Inhale),
    };
    if !meets_mode {
        // no, log and notify player
        info!(
            "{} tried to {mode} an item ({}) which lacks that ability.",
            world.player.name(),
            item.symbol()
        );
        view.push(ViewItem::ActionFailure(format!(
            "Despite your best effort, you are unable to {mode} the {}.",
            item.name().item_style()
        )));
        return Ok(());
    }

    /* we now have the UUID (item_id) of an item that is available nearby,
    matches player input, and can be ingested in the specified way */

    // Check triggers for any specific reaction / feedback to this ingestion
    let sent_id = item_id;
    let sent_mode = mode;
    let fired_triggers = check_triggers(world, view, &[TriggerCondition::Ingest { item_id, mode }])?;
    let sent_trigger_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
        TriggerCondition::Ingest { item_id, mode } => *item_id == sent_id && *mode == sent_mode,
        _ => false,
    });

    // If no trigger fired, give default feedback
    if !sent_trigger_fired {
        view.push(ViewItem::ActionSuccess(format!(
            "You {mode} the {}. It doesn't seem to do much.",
            item_name.item_style()
        )));
    }

    // Consume 1 use of the item
    let ability = match mode {
        IngestMode::Eat => ItemAbility::Eat,
        IngestMode::Drink => ItemAbility::Drink,
        IngestMode::Inhale => ItemAbility::Inhale,
    };
    if let Some(uses_left) = consume(world, &item_id, ability)? {
        if uses_left == 0 {
            view.push(ViewItem::ActionSuccess(format!(
                "The {} has no more uses left.",
                item_name.item_style()
            )));
        } else {
            #[allow(clippy::cast_possible_wrap)]
            view.push(ViewItem::ActionSuccess(format!(
                "The {} has {} use{} left",
                item_name.item_style(),
                uses_left,
                plural_s(uses_left as isize)
            )));
        }
    }

    info!("{} ingested '{}' ({})", world.player.name(), item_name, item_symbol);
    world.turn_count += 1;
    Ok(())
}
/// Uses one item on another item with a specific type of interaction.
///
/// This is the core handler for complex item interactions where one item (the tool)
/// is used to perform an action on another item (the target). The interaction system
/// validates that the tool has the required capabilities and that the target can
/// accept that type of interaction.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `interaction` - The type of interaction to perform (e.g., Cut, Burn, Clean)
/// * `tool_str` - Pattern string to match the tool item in player inventory
/// * `target_str` - Pattern string to match the target item in nearby area
///
/// # Returns
///
/// Returns `Ok(())` on successful interaction attempt, regardless of whether
/// the interaction actually succeeded or failed due to game logic.
///
/// # Interaction Validation
///
/// The function performs several validation steps:
/// 1. **Tool availability** - Tool must be in player inventory
/// 2. **Target availability** - Target must be nearby (room or inventory)
/// 3. **Capability matching** - Tool must have ability required by target
/// 4. **Interaction firing** - Triggers must exist to handle the interaction
///
/// # Trigger System
///
/// Multiple triggers may fire for a single interaction:
/// - `UseItemOnItem` - Specific tool + target + interaction combination
/// - `ActOnItem` - General action on target (regardless of tool)
/// - `UseItem` - General tool usage (regardless of target)
///
/// # Consumable Items
///
/// If the tool is consumable, it will lose uses when successfully employed.
/// The function tracks remaining uses and notifies the player when items
/// are exhausted.
///
/// # Feedback System
///
/// - Success: Determined by whether appropriate triggers fire
/// - Failure: Provides specific feedback about missing requirements
/// - No effect: Generic message from the `NoEffect` spinner when no triggers handle the interaction
///
/// # Errors
/// Returns an error if the player's current room cannot be determined, if tool or target items
/// cannot be resolved from world data, or if trigger evaluation fails.
///
/// # Panics
/// Zero. The cases are checked for `None` and handled gracefully before the `expect()` calls occur
#[allow(clippy::too_many_lines)]
pub fn use_item_on_handler(
    world: &mut AmbleWorld,
    view: &mut View,
    interaction: ItemInteractionType,
    tool_str: &str,
    target_str: &str,
) -> Result<()> {
    let Some((target, tool)) = resolve_use_item_participants(world, view, tool_str, target_str)? else {
        return Ok(());
    };
    let target_name = target.name().to_string();
    let target_id = target.id();
    let tool_name = tool.name().to_string();
    let tool_id = tool.id();
    let tool_is_consumable = tool.consumable.is_some();

    // check if these items can interact in this way
    if !interaction_requirement_met(interaction, target, tool) {
        view.push(ViewItem::ActionFailure(format!(
            "You can't do that with a {}!",
            tool.name().item_style(),
        )));
        info!(
            "Player tried to {:?} {} ({}) with {} ({})",
            interaction,
            target.name(),
            target.symbol(),
            tool.name(),
            tool.symbol()
        );
        world.turn_count += 1;
        return Ok(());
    }
    // The utilized ItemAbility is needed to send a UseItem TriggerCondition. ItemAbility::Use is
    // a reasonable default but should never come up; the existence of this Interaction
    // implies a specific matching ability which has already been verified by the
    // interaction_requirement_met(...) call above.
    let used_ability = *target
        .interaction_requires
        .get(&interaction)
        .unwrap_or(&ItemAbility::Use);

    let interaction_fired = dispatch_use_item_triggers(world, view, interaction, target_id, tool_id, used_ability)?;

    // Nope, no triggered reaction to these conditions
    if !interaction_fired {
        view.push(ViewItem::ActionFailure(
            world
                .spin_core(CoreSpinnerType::NoEffect, "That appears to have had no effect.")
                .clone(),
        ));
        info!(
            "No matching trigger for {interaction:?} {target_name} ({}) with {tool_name} ({})",
            symbol_or_unknown(&world.items, target_id),
            symbol_or_unknown(&world.items, tool_id)
        );
    }
    // consume 1 use if consumable. Obviously.
    if tool_is_consumable {
        let uses_left = consume(world, &tool_id, used_ability)?;
        if let Some(uses_left) = uses_left {
            if uses_left == 0 {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} has no more uses left.",
                    tool_name.item_style()
                )));
            } else {
                #[allow(clippy::cast_possible_wrap)]
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} has {} use{} left",
                    tool_name.item_style(),
                    uses_left,
                    plural_s(uses_left as isize)
                )));
            }
        }
    }
    world.turn_count += 1;
    Ok(())
}

fn resolve_use_item_participants<'a>(
    world: &'a AmbleWorld,
    view: &mut View,
    tool_str: &str,
    target_str: &str,
) -> Result<Option<(&'a Item, &'a Item)>> {
    // find an item in inventory matching tool_str
    let tool_id = match entity_search::find_item_match(world, tool_str, SearchScope::Inventory) {
        Ok(uuid) => uuid,
        Err(SearchError::NoMatchingName(tool_pattern)) => {
            view.push(ViewItem::ActionFailure(format!(
                "There is no \"{}\" in your inventory.",
                tool_pattern.error_style()
            )));
            return Ok(None);
        },
        Err(e) => bail!(e),
    };
    // find a reachable item in the room or inventory matching target_str
    let room_id = world.player_room_ref()?.id();
    let target_id = match entity_search::find_item_match(world, target_str, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(SearchError::NoMatchingName(target_pattern)) => {
            view.push(ViewItem::ActionFailure(format!(
                "You don't see any item matching \"{}\" within reach.",
                target_pattern.error_style()
            )));
            return Ok(None);
        },
        Err(e) => bail!(e),
    };
    // return references to the items matched
    let tool = world.items.get(&tool_id).expect("tool_id already known valid");
    let target = world.items.get(&target_id).expect("target_id already known valid");
    Ok(Some((target, tool)))
}

// sends trigger event conditions appropriate to item usage, returns true if any
// of the conditions caused a trigger to fire
fn dispatch_use_item_triggers(
    world: &mut AmbleWorld,
    view: &mut View,
    interaction: ItemInteractionType,
    target_id: Uuid,
    tool_id: Uuid,
    used_ability: ItemAbility,
) -> Result<bool> {
    let fired = check_triggers(
        world,
        view,
        &[
            TriggerCondition::UseItemOnItem {
                interaction,
                target_id,
                tool_id,
            },
            TriggerCondition::ActOnItem {
                action: interaction,
                target_id,
            },
            TriggerCondition::UseItem {
                item_id: tool_id,
                ability: used_ability,
            },
        ],
    )?;

    Ok(triggers_contain_condition(&fired, |cond| match cond {
        TriggerCondition::ActOnItem {
            action,
            target_id: fired_target,
        } => *action == interaction && *fired_target == target_id,
        TriggerCondition::UseItem { ability, .. } => *ability == used_ability,
        TriggerCondition::UseItemOnItem {
            interaction: fired_interaction,
            target_id: fired_target,
            tool_id: fired_tool,
        } => *fired_interaction == interaction && *fired_target == target_id && *fired_tool == tool_id,
        _ => false,
    }))
}
/// Activates an item if it has the ability to be turned on.
///
/// This handler attempts to turn on or activate items in the current room
/// that have switch-like functionality. Items must have the `TurnOn` ability
/// to be activated, and the activation may trigger various game effects.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item_pattern` - Pattern string to match against items in current room
///
/// # Returns
///
/// Returns `Ok(())` after attempting activation, regardless of success.
///
/// # Behavior
///
/// - Searches current room for items matching the pattern
/// - Verifies the item has the `TurnOn` ability
/// - Fires `TriggerCondition::UseItem` with `ItemAbility::TurnOn`
/// - Provides appropriate feedback based on whether triggers fire
///
/// # Error Handling
///
/// - Item not found: Standard "not found" message
/// - Item cannot be turned on: Specific capability message
/// - NPC matched: Humorous rejection message
/// - No effect: Generic "nothing happens" message when no triggers fire
///
/// # Errors
/// Returns an error if the player's current room cannot be determined or if trigger execution
/// fails due to missing world data.
pub fn turn_on_handler(world: &mut AmbleWorld, view: &mut View, item_pattern: &str) -> Result<()> {
    // search player's location for an item with name matching 'pattern'
    let room_id = world.player_room_ref()?.id();
    let item_id = match entity_search::find_item_match(world, item_pattern, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(e) => match e {
            SearchError::NoMatchingName(_) => {
                entity_not_found(world, view, item_pattern);
                return Ok(());
            },
            _ => bail!(e),
        },
    };
    let item = world.items.get_mut(&item_id).expect("item ID known to be valid here");

    if item.abilities.contains(&ItemAbility::TurnOn) {
        info!("Player switched on {} ({})", item.name(), item.symbol());
        let sent_id = item.id();
        let fired_triggers = check_triggers(
            world,
            view,
            &[TriggerCondition::UseItem {
                item_id: sent_id,
                ability: ItemAbility::TurnOn,
            }],
        )?;
        let sent_trigger_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
            TriggerCondition::UseItem { item_id, ability } => *item_id == sent_id && *ability == ItemAbility::TurnOn,
            _ => false,
        });
        if !sent_trigger_fired {
            view.push(ViewItem::ActionFailure(format!(
                "{}",
                "You hear a clicking sound and then... nothing happens.".italic()
            )));
        }
    } else {
        info!(
            "Player tried to turn on unswitchable item {} ({})",
            item.name(),
            item.symbol()
        );
        view.push(ViewItem::ActionFailure(format!(
            "The {} can't be turned on.",
            item.name().item_style()
        )));
    }
    world.turn_count += 1;
    Ok(())
}

/// Disables an item if it has the ability to be turned off.
///
/// This handler attempts to turn off or disable items in the current room
/// that have switch-like functionality. Items must have the `TurnOff` ability
/// to be stopped, and that can be tied to various game world responses via triggers.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item_pattern` - Pattern string to match against items in current room
///
/// # Returns
///
/// Returns `Ok(())` after attempting to disable, regardless of success in doing so.
///
/// # Behavior
///
/// - Searches current room for items matching the pattern
/// - Verifies the item has the `TurnOff` ability
/// - Fires `TriggerCondition::UseItem` with `ItemAbility::TurnOff`
/// - Provides appropriate feedback based on whether triggers fire
///
/// # Error Handling
///
/// - Item not found: Standard "not found" message
/// - Item cannot be turned off: Specific capability message
/// - NPC matched: Humorous rejection message
/// - No effect: Generic "nothing happens" message when no triggers fire
///
/// # Errors
/// Returns an error if the player's current room cannot be determined or if trigger execution
/// fails because required world data is missing.
pub fn turn_off_handler(world: &mut AmbleWorld, view: &mut View, item_pattern: &str) -> Result<()> {
    // search player's location for an item with name matching 'pattern'
    let room_id = world.player_room_ref()?.id();
    let item_id = match entity_search::find_item_match(world, item_pattern, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(e) => match e {
            SearchError::NoMatchingName(_) => {
                entity_not_found(world, view, item_pattern);
                return Ok(());
            },
            _ => bail!(e),
        },
    };
    let item = world.items.get_mut(&item_id).expect("item ID known to be valid here");
    if item.abilities.contains(&ItemAbility::TurnOff) {
        info!("Player switched off {} ({})", item.name(), item.symbol());
        let sent_id = item.id();
        let fired_triggers = check_triggers(
            world,
            view,
            &[TriggerCondition::UseItem {
                item_id: sent_id,
                ability: ItemAbility::TurnOff,
            }],
        )?;
        let sent_trigger_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
            TriggerCondition::UseItem { item_id, ability } => *item_id == sent_id && *ability == ItemAbility::TurnOff,
            _ => false,
        });
        if !sent_trigger_fired {
            view.push(ViewItem::ActionFailure(format!(
                "{}",
                "You hear a clicking sound and then... nothing happens.".italic()
            )));
        }
    } else {
        info!(
            "Player tried to turn off unswitchable item {} ({})",
            item.name(),
            item.symbol()
        );
        view.push(ViewItem::ActionFailure(format!(
            "The {} can't be turned off.",
            item.name().item_style()
        )));
    }

    world.turn_count += 1;
    Ok(())
}
/// Opens a closed container item, making its contents accessible.
///
/// This handler attempts to open container items that are currently in a
/// closed state. Only unlocked containers can be opened; locked containers
/// must be unlocked first. Opening a container may trigger game events.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `pattern` - Pattern string to match against nearby container items
///
/// # Returns
///
/// Returns `Ok(())` after attempting to open the container.
///
/// # Container State Logic
///
/// - **Non-container**: Cannot be opened (no container state)
/// - **Locked**: Must be unlocked first before opening
/// - **Already open**: Acknowledges current state
/// - **Closed**: Successfully opens and triggers events
///
/// # Trigger Effects
///
/// Opening a container fires `TriggerCondition::Open`, which can:
/// - Reveal items inside the container
/// - Advance puzzle or story logic
/// - Trigger ambient effects or messages
/// - Cause other game state changes
///
/// # Scope
///
/// Searches both the current room and player inventory for containers,
/// allowing players to open containers they're carrying as well as
/// those in their environment.
///
/// # Errors
/// Returns an error if the player's current room cannot be resolved, if the targeted container
/// cannot be retrieved, or if trigger execution fails due to missing data.
pub fn open_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    // search player's location for an item with name matching 'pattern'
    let room_id = world.player_room_ref()?.id();
    let container_id = match entity_search::find_item_match(world, pattern, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(e) => match e {
            SearchError::NoMatchingName(_) => {
                entity_not_found(world, view, pattern);
                return Ok(());
            },
            _ => bail!(e),
        },
    };

    // either show failure reasons, or set container state to open
    let mut container_opened = false;
    if let Some(target_item) = world.get_item_mut(container_id) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} can't be opened.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked | ContainerState::TransparentLocked) => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} is locked. You'll have to unlock it first.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open | ContainerState::TransparentOpen) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already open.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Closed) => {
                // check to see if any particular interaction type is required to open
                if let Some(required_ability) = target_item.interaction_requires.get(&ItemInteractionType::Open) {
                    view.push(ViewItem::ActionFailure(format!(
                        "The {} can't be opened without using something that can {}.",
                        target_item.name().item_style(),
                        required_ability.to_string().highlight()
                    )));
                } else {
                    target_item.container_state = Some(ContainerState::Open);
                    container_opened = true;
                }
            },
            Some(ContainerState::TransparentClosed) => {
                // check to see if any particular interaction type is required to open
                if let Some(required_ability) = target_item.interaction_requires.get(&ItemInteractionType::Open) {
                    view.push(ViewItem::ActionFailure(format!(
                        "The {} can't be opened without using something that can {}.",
                        target_item.name().item_style(),
                        required_ability.to_string().highlight()
                    )));
                } else {
                    target_item.container_state = Some(ContainerState::TransparentOpen);
                    container_opened = true;
                }
            },
        }
    }

    // if a container was successfully opened: tell player, show contents, and log it
    if let Some(opened) = world.items.get(&container_id)
        && container_opened
    {
        opened.show(world, view);
        info!(
            "{} opened the {} ({})",
            world.player.name(),
            opened.name(),
            opened.symbol()
        );
        view.push_with_priority(
            ViewItem::ActionSuccess(format!("You opened the {}.\n", opened.name().item_style())),
            -100,
        );
        check_triggers(world, view, &[TriggerCondition::Open(container_id)])?;
    }

    world.turn_count += 1;
    Ok(())
}

/// Closes an open container item, hiding its contents from view.
///
/// This handler closes container items that are currently open, which can
/// be useful for organization, security, or puzzle mechanics. Closing containers
/// does not trigger game events but changes their accessibility state.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `pattern` - Pattern string to match against nearby container items
///
/// # Returns
///
/// Returns `Ok(())` after attempting to close the container.
///
/// # Container State Logic
///
/// - **Non-container**: Cannot be closed (no container state)
/// - **Already closed/locked**: Acknowledges current state
/// - **Open**: Successfully closes the container
///
/// # Behavior
///
/// Unlike opening, closing containers does not trigger game events.
/// This is primarily a state management operation that affects item
/// visibility and access but not game progression.
///
/// # Scope
///
/// Searches both current room and player inventory for containers,
/// allowing closure of both environmental and carried containers.
///
/// # Errors
/// Returns an error if the player's current room cannot be resolved or if the targeted container
/// cannot be retrieved from the world.
pub fn close_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    // search player's location for an item with name matching 'pattern'
    let room_id = world.player_room_ref()?.id();
    let container_id = match entity_search::find_item_match(world, pattern, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(e) => match e {
            SearchError::NoMatchingName(_) => {
                entity_not_found(world, view, pattern);
                return Ok(());
            },
            _ => bail!(e),
        },
    };

    if let Some(target_item) = world.get_item_mut(container_id) {
        let item_name = target_item.name().to_owned();
        let item_sym = target_item.symbol().to_owned();

        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} can't be closed.",
                    target_item.name().item_style()
                )));
            },
            Some(
                ContainerState::Closed
                | ContainerState::Locked
                | ContainerState::TransparentClosed
                | ContainerState::TransparentLocked,
            ) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already closed.",
                    item_name.item_style()
                )));
            },
            Some(ContainerState::Open) => {
                target_item.container_state = Some(ContainerState::Closed);
                view.push(ViewItem::ActionSuccess(format!(
                    "You closed the {}.\n",
                    item_name.item_style()
                )));
                info!("{} closed the {} ({})", world.player.name(), item_name, item_sym);
            },
            Some(ContainerState::TransparentOpen) => {
                target_item.container_state = Some(ContainerState::TransparentClosed);
                view.push(ViewItem::ActionSuccess(format!(
                    "You closed the {}.\n",
                    target_item.name().item_style()
                )));
                info!("{} closed the {} ({})", world.player.name(), item_name, item_sym);
            },
        }
    }
    world.turn_count += 1;
    Ok(())
}

/// Locks a container item, securing it against unauthorized access.
///
/// This handler locks container items, preventing them from being opened
/// until they are unlocked with an appropriate key. Locking is primarily
/// used for puzzle mechanics and security.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `pattern` - Pattern string to match against containers in current room
///
/// # Returns
///
/// Returns `Ok(())` after attempting to lock the container.
///
/// # Container State Logic
///
/// - **Non-container**: Cannot be locked (no lock mechanism)
/// - **Already locked**: Acknowledges current state
/// - **Open/Closed**: Successfully locks the container
///
/// # Key Requirements
///
/// Unlike unlocking, locking typically doesn't require specific keys
/// in most game implementations, though this could be extended to
/// require lock-specific tools or abilities.
///
/// # Scope
///
/// Only searches the current room for containers to lock, as locking
/// items in inventory is less commonly needed in gameplay scenarios.
///
/// # Errors
/// Returns an error if the player's current room cannot be determined or if the targeted container
/// cannot be located within the world state.
pub fn lock_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    // search player's location for an item with name matching 'pattern'
    let room_id = world.player_room_ref()?.id();
    let container_id = match entity_search::find_item_match(world, pattern, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(e) => match e {
            SearchError::NoMatchingName(_) => {
                entity_not_found(world, view, pattern);
                return Ok(());
            },
            _ => bail!(e),
        },
    };

    if let Some(target_item) = world.get_item_mut(container_id) {
        let item_name = target_item.name().to_owned();
        let item_sym = target_item.symbol().to_owned();
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} isn't something that can be locked.",
                    item_name.item_style()
                )));
            },
            Some(ContainerState::Locked | ContainerState::TransparentLocked) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already locked.",
                    item_name.item_style()
                )));
            },
            Some(ContainerState::Open | ContainerState::Closed) => {
                target_item.container_state = Some(ContainerState::Locked);
                view.push(ViewItem::ActionSuccess(format!(
                    "You locked the {}.\n",
                    item_name.item_style()
                )));
                info!("{} locked the {} ({})", world.player.name(), item_name, item_sym);
            },
            Some(ContainerState::TransparentOpen | ContainerState::TransparentClosed) => {
                target_item.container_state = Some(ContainerState::TransparentLocked);
                view.push(ViewItem::ActionSuccess(format!(
                    "You locked the {}.\n",
                    item_name.item_style()
                )));
                info!("{} locked the {} ({})", world.player.name(), item_name, item_sym);
            },
        }
    }
    world.turn_count += 1;
    Ok(())
}

/// Unlocks a locked container using an appropriate key from inventory.
///
/// This handler attempts to unlock locked containers by checking the player's
/// inventory for items with the appropriate unlocking abilities. Success
/// requires having the right key, and unlocking may trigger game events.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `pattern` - Pattern string to match against containers in current room
///
/// # Returns
///
/// Returns `Ok(())` after attempting to unlock the container.
///
/// # Key System
///
/// The function searches inventory for items with `ItemAbility::Unlock`:
/// - **Specific keys**: Target a particular container by UUID
/// - **Universal keys**: Can unlock any container (master keys)
///
/// # Container State Logic
///
/// - **Non-container**: Cannot be unlocked (no lock)
/// - **Already unlocked**: Acknowledges current state
/// - **Locked with valid key**: Successfully unlocks to closed state
/// - **Locked without key**: Denies access with helpful message
///
/// # Trigger Effects
///
/// Unlocking fires `TriggerCondition::Unlock`, which can:
/// - Advance puzzle sequences requiring specific unlocking order
/// - Reveal important story items or clues
/// - Trigger narrative events or character reactions
/// - Enable access to new areas or content
///
/// # Security Model
///
/// The key must be in the player's inventory - keys in the room or
/// in other containers cannot be used, maintaining gameplay challenge.
///
/// # Errors
/// Returns an error if the player's current room cannot be determined, if the targeted container
/// cannot be resolved from the world state, or if trigger evaluation fails.
pub fn unlock_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    // search player's location for an item with name matching 'pattern'
    let room_id = world.player_room_ref()?.id();
    let container_id = match entity_search::find_item_match(world, pattern, SearchScope::TouchableItems(room_id)) {
        Ok(uuid) => uuid,
        Err(e) => match e {
            SearchError::NoMatchingName(_) => {
                entity_not_found(world, view, pattern);
                return Ok(());
            },
            _ => bail!(e),
        },
    };

    let key_data = find_valid_key(&world.items, &world.player.inventory, container_id);

    if let Some(target_item) = world.get_item_mut(container_id) {
        let item_name = target_item.name().to_owned();
        let item_sym = target_item.symbol().to_owned();
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} doesn't have a lock.",
                    item_name.item_style()
                )));
            },
            Some(
                ContainerState::Open
                | ContainerState::TransparentOpen
                | ContainerState::Closed
                | ContainerState::TransparentClosed,
            ) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already unlocked.",
                    item_name.item_style()
                )));
            },
            Some(ContainerState::Locked | ContainerState::TransparentLocked) => {
                if let Some(key) = key_data {
                    // If it was transparent locked, make it transparent closed, otherwise regular closed
                    target_item.container_state =
                        if target_item.container_state == Some(ContainerState::TransparentLocked) {
                            Some(ContainerState::TransparentClosed)
                        } else {
                            Some(ContainerState::Closed)
                        };
                    view.push(ViewItem::ActionSuccess(format!(
                        "You unlocked the {} with the {}.\n",
                        item_name.item_style(),
                        key.name.item_style(),
                    )));
                    info!(
                        "{} unlocked '{}' ({}) using key '{}' ({})",
                        world.player.name(),
                        item_name,
                        item_sym,
                        key.name,
                        key.symbol,
                    );
                    check_triggers(world, view, &[TriggerCondition::Unlock(container_id)])?;
                } else {
                    view.push(ViewItem::ActionFailure(format!(
                        "You don't have anything that can unlock the {}.",
                        item_name.item_style()
                    )));
                }
            },
        }
        world.turn_count += 1;
    }
    Ok(())
}

// Key metadata.
#[derive(Debug, Clone, PartialEq)]
struct KeyData {
    id: Uuid,
    name: String,
    symbol: String,
    skeleton: bool,
}

/// Search a list of items for a valid key to a given lock and return `KeyData` if found.
///
/// If a specific key *and* a "skeleton" key are present, the data for the specific key is
/// returned.
fn find_valid_key(world_items: &HashMap<Uuid, Item>, maybe_keys: &HashSet<Uuid>, container: Uuid) -> Option<KeyData> {
    // find and return a specific key
    let specific = maybe_keys
        .iter()
        .filter_map(|uuid| world_items.get(uuid))
        .find(|item| item.abilities.contains(&ItemAbility::Unlock(Some(container))));
    if let Some(key) = specific {
        return Some(KeyData {
            id: key.id,
            name: key.name.clone(),
            symbol: key.symbol.clone(),
            skeleton: false,
        });
    }
    // if no specific key, find and return a skeleton key
    let skeleton = maybe_keys
        .iter()
        .filter_map(|uuid| world_items.get(uuid))
        .find(|item| item.abilities.contains(&ItemAbility::Unlock(None)));
    if let Some(key) = skeleton {
        return Some(KeyData {
            id: key.id,
            name: key.name.clone(),
            symbol: key.symbol.clone(),
            skeleton: true,
        });
    }
    // there is no key for this container on the list
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item, ItemAbility, ItemInteractionType, Movability},
        room::Room,
        trigger::{ScriptedAction, Trigger, TriggerAction, TriggerCondition},
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn build_world() -> (AmbleWorld, View, Uuid, Uuid, Uuid, Uuid) {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let room = Room {
            id: room_id,
            symbol: "r".into(),
            name: "Room".into(),
            base_description: String::new(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room_id, room);
        world.player.location = Location::Room(room_id);

        let container_id = Uuid::new_v4();
        let mut container = Item {
            id: container_id,
            symbol: "c".into(),
            name: "chest".into(),
            description: String::new(),
            location: Location::Room(room_id),
            container_state: Some(ContainerState::Locked),
            movability: Movability::Free,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        container
            .interaction_requires
            .insert(ItemInteractionType::Open, ItemAbility::Pry);
        world.rooms.get_mut(&room_id).unwrap().contents.insert(container_id);
        world.items.insert(container_id, container);

        let tool_id = Uuid::new_v4();
        let tool = Item {
            id: tool_id,
            symbol: "t".into(),
            name: "crowbar".into(),
            description: String::new(),
            location: Location::Inventory,
            movability: Movability::Free,
            container_state: None,
            contents: HashSet::new(),
            abilities: [ItemAbility::Pry].into_iter().collect(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.player.inventory.insert(tool_id);
        world.items.insert(tool_id, tool);

        let lamp_id = Uuid::new_v4();
        let lamp = Item {
            id: lamp_id,
            symbol: "l".into(),
            name: "lamp".into(),
            description: String::new(),
            location: Location::Room(room_id),
            container_state: None,
            movability: Movability::Fixed {
                reason: "lamp is glued to floor".to_string(),
            },
            contents: HashSet::new(),
            abilities: [ItemAbility::TurnOn].into_iter().collect(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.rooms.get_mut(&room_id).unwrap().contents.insert(lamp_id);
        world.items.insert(lamp_id, lamp);

        let key_id = Uuid::new_v4();
        let key = Item {
            id: key_id,
            symbol: "k".into(),
            name: "key".into(),
            description: String::new(),
            location: Location::Inventory,
            movability: Movability::Free,
            container_state: None,
            contents: HashSet::new(),
            abilities: [ItemAbility::Unlock(Some(container_id))].into_iter().collect(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.player.inventory.insert(key_id);
        world.items.insert(key_id, key);
        let view = View::new();

        (world, view, container_id, tool_id, lamp_id, key_id)
    }

    #[test]
    fn use_item_on_handler_unlocks_container() {
        let (mut world, mut view, container_id, tool_id, _, _) = build_world();
        world.triggers.push(Trigger {
            name: "open".into(),
            conditions: vec![TriggerCondition::UseItemOnItem {
                interaction: ItemInteractionType::Open,
                target_id: container_id,
                tool_id,
            }],
            actions: vec![ScriptedAction::new(TriggerAction::UnlockItem(container_id))],
            only_once: false,
            fired: false,
        });
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
        use_item_on_handler(&mut world, &mut view, ItemInteractionType::Open, "crowbar", "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn use_item_on_handler_without_ability_does_nothing() {
        let (mut world, mut view, container_id, tool_id, _, _) = build_world();
        world.items.get_mut(&tool_id).unwrap().abilities.clear();
        world.triggers.push(Trigger {
            name: "open".into(),
            conditions: vec![TriggerCondition::UseItemOnItem {
                interaction: ItemInteractionType::Open,
                target_id: container_id,
                tool_id,
            }],
            actions: vec![ScriptedAction::new(TriggerAction::UnlockItem(container_id))],
            only_once: false,
            fired: false,
        });
        use_item_on_handler(&mut world, &mut view, ItemInteractionType::Open, "crowbar", "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }

    #[test]
    fn turn_on_handler_triggers_unlock() {
        let (mut world, mut view, container_id, _, lamp_id, _) = build_world();
        world.triggers.push(Trigger {
            name: "light".into(),
            conditions: vec![TriggerCondition::UseItem {
                item_id: lamp_id,
                ability: ItemAbility::TurnOn,
            }],
            actions: vec![ScriptedAction::new(TriggerAction::UnlockItem(container_id))],
            only_once: false,
            fired: false,
        });
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
        turn_on_handler(&mut world, &mut view, "lamp").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn open_handler_opens_closed_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Closed);
        world
            .items
            .get_mut(&container_id)
            .unwrap()
            .interaction_requires
            .remove(&ItemInteractionType::Open);
        open_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn open_handler_will_not_open_container_with_special_open_requirements() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Closed);
        // note -- the "chest" test item is defined as requiring "pry" to "open"
        open_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Closed)
        );
    }

    #[test]
    fn open_handler_locked_container_stays_locked() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        open_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }

    #[test]
    fn close_handler_closes_open_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Open);
        close_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Closed)
        );
    }

    #[test]
    fn lock_handler_locks_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Closed);
        lock_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }

    #[test]
    fn unlock_handler_with_key_unlocks_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        unlock_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Closed)
        );
    }

    #[test]
    fn unlock_handler_without_key_does_not_unlock() {
        let (mut world, mut view, container_id, _, _, key_id) = build_world();
        world.player.inventory.remove(&key_id);
        unlock_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }
}
