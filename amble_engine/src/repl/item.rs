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
//! - `TriggerCondition::UseItem` - When items are activated or used
//! - `TriggerCondition::UseItemOnItem` - When tools are used on targets
//! - `TriggerCondition::ActOnItem` - When actions are performed on items
//! - `TriggerCondition::Open` - When containers are opened
//! - `TriggerCondition::Unlock` - When locked items are unlocked
//!
//! These triggers enable rich gameplay where item interactions can advance
//! storylines, solve puzzles, unlock areas, or cause other game effects.

use std::collections::HashSet;

use crate::{
    AmbleWorld, View, ViewItem, WorldObject,
    helpers::item_symbol_from_id,
    item::{ContainerState, ItemAbility, ItemInteractionType, consume},
    loader::items::interaction_requirement_met,
    repl::{entity_not_found, find_world_object},
    spinners::CoreSpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::Result;
use colored::Colorize;
use log::{info, warn};
use uuid::Uuid;

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
/// - No effect: Generic message when no triggers handle the interaction
pub fn use_item_on_handler(
    world: &mut AmbleWorld,
    view: &mut View,
    interaction: ItemInteractionType,
    tool_str: &str,
    target_str: &str,
) -> Result<()> {
    // make sure we can find valid matches for tool and target items and notify player if not
    let items_nearby = &world.player_room_ref()?.contents;
    let target_scope: HashSet<_> = items_nearby.union(&world.player.inventory).collect();
    let maybe_target =
        find_world_object(target_scope, &world.items, &world.npcs, target_str).and_then(super::WorldEntity::item);
    let maybe_tool = find_world_object(&world.player.inventory, &world.items, &world.npcs, tool_str)
        .and_then(super::WorldEntity::item);
    if maybe_target.is_none() {
        view.push(ViewItem::ActionFailure(format!(
            "You don't see any {} nearby.",
            target_str.error_style()
        )));
        return Ok(());
    }
    if maybe_tool.is_none() {
        view.push(ViewItem::ActionFailure(format!(
            "You don't have any {} in inventory.",
            tool_str.error_style()
        )));
        return Ok(());
    }
    // unwrap OK here because we just checked for None above
    let target = maybe_target.unwrap();
    let tool = maybe_tool.unwrap();
    let target_name = target.name().to_string();
    let target_id = target.id();
    let tool_name = tool.name().to_string();
    let tool_id = tool.id();
    let tool_is_consumable = tool.consumable.is_some();

    // Can you even do this to the target? ("burn water with lighter" -> you can't burn water!)
    // This may have unwanted consequences when items don't have a specifice reaction defined but it's still
    // a reasonable command to try -- e.g. "clean flamethrower with towel" -- reasonable to do but won't have a defined requirement for cleaning
    // message would be "You can't clean the flamethrower!"... problematic.
    // if !target.interaction_requires.contains_key(&interaction) {
    //     view.push(ViewItem::ActionFailure(format!(
    //         "You can't {interaction} the {target_name}!"
    //     )));
    // }

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
        return Ok(());
    }
    // do the interaction as appropriate
    let sent_interaction = interaction;
    let sent_target_id = target.id();
    let sent_tool_id = tool.id();

    // This is needed for the UseItem TriggerCondition. ItemAbility::Use is a reasonable default but
    // should never come up, since the presence of this interaction is already verified by the
    // interaction_requirement_met(...) call above.
    let used_ability = *target
        .interaction_requires
        .get(&interaction)
        .unwrap_or(&ItemAbility::Use);

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
                target_id: target.id(),
            },
            TriggerCondition::UseItem {
                item_id: tool_id,
                ability: used_ability,
            },
        ],
    )?;
    // check to see if the ActOnItem trigger we just sent fired
    // (that's the one that will actually change world state --
    // an additional UseItemOnItem can provide flavor for a
    // particular item with the required ability.)
    let interaction_fired = triggers_contain_condition(&fired, |cond| match cond {
        TriggerCondition::ActOnItem { action, target_id } => {
            *action == sent_interaction && *target_id == sent_target_id
        },
        TriggerCondition::UseItem { ability, .. } => *ability == used_ability,
        TriggerCondition::UseItemOnItem {
            interaction,
            target_id,
            tool_id,
        } => *interaction == sent_interaction && *target_id == sent_target_id && *tool_id == sent_tool_id,
        _ => false,
    });

    if !interaction_fired {
        view.push(ViewItem::ActionFailure(
            world
                .spin_core(
                    CoreSpinnerType::NoEffect,
                    "That appears to have had no effect, Captain.",
                )
                .to_string(),
        ));
        info!(
            "No matching trigger for {interaction:?} {target_name} ({}) with {tool_name} ({})",
            item_symbol_from_id(&world.items, target_id),
            item_symbol_from_id(&world.items, tool_id)
        );
    }
    if tool_is_consumable {
        let uses_left = consume(world, &tool_id, used_ability)?;
        if let Some(uses_left) = uses_left {
            if uses_left == 0 {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} has no more uses left.",
                    tool_name.item_style()
                )));
            } else {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} has {} use{} left",
                    tool_name.item_style(),
                    uses_left,
                    if uses_left == 1 { "" } else { "s" }
                )));
            }
        }
    }
    Ok(())
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
pub fn turn_on_handler(world: &mut AmbleWorld, view: &mut View, item_pattern: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    if let Some(entity) = find_world_object(&current_room.contents, &world.items, &world.npcs, item_pattern) {
        if let Some(item) = entity.item() {
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
                    TriggerCondition::UseItem { item_id, ability } => {
                        *item_id == sent_id && *ability == ItemAbility::TurnOn
                    },
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
        } else if let Some(npc) = entity.npc() {
            info!("Player tried to turn on an NPC {} ({})", npc.name(), npc.symbol());
            view.push(ViewItem::ActionFailure(format!(
                "{} is impervious to your attempt at seduction.",
                npc.name().npc_style()
            )));
        }
    } else {
        entity_not_found(world, view, item_pattern);
    }
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
pub fn open_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    // search player's location for an item matching search
    let room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = room.contents.union(&world.player.inventory).copied().collect();
    let (container_id, name) =
        if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, pattern) {
            if let Some(item) = entity.item() {
                (item.id(), item.name().to_string())
            } else {
                warn!("Player attempted to open a non-Item WorldEntity by searching ({pattern})");
                view.push(ViewItem::Error(format!(
                    "{} isn't an item. You can't open it.",
                    pattern.error_style()
                )));
                return Ok(());
            }
        } else {
            entity_not_found(world, view, pattern);
            return Ok(());
        };

    if let Some(target_item) = world.get_item_mut(container_id) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} can't be opened.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked) => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} is locked. You'll have to unlock it first.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::TransparentLocked) => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} is locked. You'll have to unlock it first.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already open.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Closed | ContainerState::TransparentClosed) => {
                target_item.container_state = Some(ContainerState::Open);
                view.push(ViewItem::ActionSuccess(format!(
                    "You opened the {}.\n",
                    target_item.name().item_style()
                )));
                info!(
                    "{} opened the {} ({})",
                    world.player.name(),
                    name,
                    item_symbol_from_id(&world.items, container_id)
                );
                check_triggers(world, view, &[TriggerCondition::Open(container_id)])?;
            },
        }
    }
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
pub fn close_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = room.contents.union(&world.player.inventory).copied().collect();
    let (uuid, name) = if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, pattern) {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Close({pattern}) matched a non-Item WorldEntity");
            view.push(ViewItem::Error(format!(
                "You do not see a {} to close.",
                pattern.error_style()
            )));
            return Ok(());
        }
    } else {
        entity_not_found(world, view, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(uuid) {
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
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open) => {
                target_item.container_state = Some(ContainerState::Closed);
                view.push(ViewItem::ActionSuccess(format!(
                    "You closed the {}.\n",
                    target_item.name().item_style()
                )));
                info!(
                    "{} closed the {} ({})",
                    world.player.name(),
                    name,
                    item_symbol_from_id(&world.items, uuid)
                );
            },
        }
    }
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
pub fn lock_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let (uuid, name) = if let Some(entity) = find_world_object(&room.contents, &world.items, &world.npcs, pattern) {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Lock({pattern}) matched a non-Item WorldEntity");
            view.push(ViewItem::Error(format!(
                "You don't see a {} here to lock.",
                pattern.error_style()
            )));
            return Ok(());
        }
    } else {
        entity_not_found(world, view, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(uuid) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} isn't something that can be locked.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked | ContainerState::TransparentLocked) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already locked.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open | ContainerState::Closed | ContainerState::TransparentClosed) => {
                target_item.container_state = Some(ContainerState::Locked);
                view.push(ViewItem::ActionSuccess(format!(
                    "You locked the {}.\n",
                    target_item.name().item_style()
                )));
                info!(
                    "{} locked the {} ({})",
                    world.player.name(),
                    name,
                    item_symbol_from_id(&world.items, uuid)
                );
            },
        }
    }
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
pub fn unlock_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let (container_id, container_name) =
        if let Some(entity) = find_world_object(&room.contents, &world.items, &world.npcs, pattern) {
            if let Some(item) = entity.item() {
                (item.id(), item.name().to_string())
            } else {
                warn!("Command:Unlock({pattern}) matched a non-Item (NPC) WorldEntity");
                view.push(ViewItem::Error(format!(
                    "You don't see a {} here to unlock.",
                    pattern.error_style()
                )));
                return Ok(());
            }
        } else {
            entity_not_found(world, view, pattern);
            return Ok(());
        };

    // Check player inventory for valid key
    let has_valid_key = world.player.inventory.iter().any(|id| {
        world.items.get(id).is_some_and(|i| {
            i.abilities.iter().any(|a| match a {
                ItemAbility::Unlock(Some(target)) => *target == container_id,
                ItemAbility::Unlock(None) => true, // universal key
                _ => false,
            })
        })
    });

    if let Some(target_item) = world.get_item_mut(container_id) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} doesn't have a lock.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open | ContainerState::Closed | ContainerState::TransparentClosed) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already unlocked.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked | ContainerState::TransparentLocked) => {
                if has_valid_key {
                    // If it was transparent locked, make it transparent closed, otherwise regular closed
                    target_item.container_state =
                        if target_item.container_state == Some(ContainerState::TransparentLocked) {
                            Some(ContainerState::TransparentClosed)
                        } else {
                            Some(ContainerState::Closed)
                        };
                    view.push(ViewItem::ActionSuccess(format!(
                        "You unlocked the {}.\n",
                        target_item.name().item_style()
                    )));
                    info!(
                        "{} unlocked the {} ({})",
                        world.player.name(),
                        container_name,
                        item_symbol_from_id(&world.items, container_id)
                    );
                    check_triggers(world, view, &[TriggerCondition::Unlock(container_id)])?;
                } else {
                    view.push(ViewItem::ActionFailure(format!(
                        "You don't have anything that can unlock the {}.",
                        target_item.name().item_style()
                    )));
                }
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item, ItemAbility, ItemInteractionType},
        room::Room,
        trigger::{Trigger, TriggerAction, TriggerCondition},
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
            portable: true,
            container_state: Some(ContainerState::Locked),
            restricted: false,
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
            portable: true,
            container_state: None,
            restricted: false,
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
            portable: false,
            container_state: None,
            restricted: false,
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
            portable: true,
            container_state: None,
            restricted: false,
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
            actions: vec![TriggerAction::UnlockItem(container_id)],
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
            actions: vec![TriggerAction::UnlockItem(container_id)],
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
            actions: vec![TriggerAction::UnlockItem(container_id)],
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
        open_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
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
