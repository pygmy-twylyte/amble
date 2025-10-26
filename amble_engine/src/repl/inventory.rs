//! Player inventory management command handlers for the Amble game engine.
//!
//! This module contains handlers for all commands that manipulate items in the
//! player's inventory or transfer items between different containers in the world.
//! These commands form the core of the game's item interaction system.
//!
//! # Command Categories
//!
//! ## Basic Inventory Operations
//! - [`drop_handler`] - Remove items from inventory and place in current room
//! - [`take_handler`] - Pick up items from the current room into inventory
//!
//! ## Container Interactions
//! - [`take_from_handler`] - Remove items from containers or NPCs into inventory
//! - [`put_in_handler`] - Place inventory items into nearby containers
//!
//! ## Transfer Mechanics
//!
//! The module handles complex item transfer logic including:
//! - Location tracking (rooms, containers, NPCs, inventory)
//! - Portability restrictions (some items cannot be moved)
//! - Access permissions (locked containers, restricted items)
//! - Container state management (open/closed/locked)
//! - World consistency (preventing duplicate items)
//!
//! # Error Handling
//!
//! All handlers provide user-friendly error messages for common failure cases:
//! - Items not found or not available
//! - Containers that are locked or inaccessible
//! - Items that cannot be transferred due to restrictions
//! - Attempting to transfer non-items (like NPCs)
//!
//! # Trigger Integration
//!
//! Many inventory operations trigger game events:
//! - `TriggerCondition::Take` - When items are picked up
//! - `TriggerCondition::Drop` - When items are dropped or placed
//! - `TriggerCondition::Insert` - When items are put into containers
//! - `TriggerCondition::TakeFromNpc` - When taking items from NPCs
//!
//! These triggers can cause additional game effects like advancing storylines,
//! unlocking areas, or triggering NPC responses.

use std::collections::HashSet;

use crate::{
    AmbleWorld, ItemHolder, Location, View, ViewItem, WorldObject,
    helpers::symbol_or_unknown,
    item::ItemInteractionType,
    repl::{WorldEntity, entity_not_found, find_world_object},
    spinners::CoreSpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
    world::nearby_reachable_items,
};

use anyhow::{Result, anyhow, bail};
use colored::Colorize;
use log::{error, info, warn};
use uuid::Uuid;

/// Removes an item from the player's inventory and places it in the current room.
///
/// This command transfers an item from the player's inventory to the room they're
/// currently in, making it available for other interactions. Only portable items
/// can be dropped, and the action may trigger game events.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `thing` - Pattern string to match against inventory items
///
/// # Returns
///
/// Returns `Ok(())` on success or error if world state is inconsistent.
///
/// # Behavior
///
/// - Searches player inventory for items matching the pattern
/// - Verifies the item is portable (non-portable items cannot be dropped)
/// - Updates item location from inventory to current room
/// - Adds item to room's contents and removes from player inventory
/// - Triggers `TriggerCondition::Drop` for potential game effects
/// - Provides appropriate feedback messages for all outcomes
///
/// # Error Conditions
///
/// - Item not found in inventory
/// - Item is not portable (displays specific message)
/// - Pattern matches non-item entity (handled gracefully)
/// - World state corruption (returns error)
///
/// # Errors
/// Returns an error if the player's current room cannot be determined, if the item state
/// cannot be updated due to missing world entries, or if trigger evaluation fails.
pub fn drop_handler(world: &mut AmbleWorld, view: &mut View, thing: &str) -> Result<()> {
    if let Some(entity) = find_world_object(&world.player.inventory, &world.items, &world.npcs, thing) {
        if let Some(item) = entity.item() {
            world.turn_count += 1;
            let item_id = item.id();
            let room_id = world.player_room_ref()?.id();
            if item.portable {
                if let Some(dropped) = world.items.get_mut(&item_id) {
                    dropped.set_location_room(room_id);
                    if let Some(room) = world.rooms.get_mut(&room_id) {
                        room.add_item(dropped.id());
                        info!(
                            "{} dropped {} ({}) in {} ({})",
                            world.player.name(),
                            dropped.name(),
                            dropped.symbol(),
                            room.name(),
                            room.symbol()
                        );
                    }
                    world.player.remove_item(item_id);
                    view.push(ViewItem::ActionSuccess(format!(
                        "You dropped the {}.",
                        dropped.name().item_style()
                    )));
                    check_triggers(world, view, &[TriggerCondition::Drop(item_id)])?;
                }
            } else {
                // item not portable
                view.push(ViewItem::ActionFailure(format!(
                    "You can't drop the {}. It's not transferrable.",
                    item.name().item_style()
                )));
                return Ok(());
            }
            Ok(())
        } else {
            unexpected_entity(
                entity,
                view,
                &format!("{}? That's not a item that one can drop.", thing.error_style()),
            );
            Ok(())
        }
    } else {
        entity_not_found(world, view, thing);
        Ok(())
    }
}
/// Picks up an item from the current area and adds it to the player's inventory.
///
/// This command transfers an item from the player's current environment (room or
/// nearby containers) into their inventory. Items must be portable and not restricted,
/// and some items may require specific capabilities to handle safely.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `thing` - Pattern string to match against nearby items
///
/// # Returns
///
/// Returns `Ok(())` on success or error if world state is inconsistent.
///
/// # Behavior
///
/// - Searches nearby reachable items for matches to the pattern
/// - Checks if item requires special handling capabilities (like heat resistance)
/// - Verifies item is portable and not restricted
/// - Updates item location from current location to inventory
/// - Removes item from original container and adds to player inventory
/// - Triggers `TriggerCondition::Take` for potential game effects
/// - Uses randomized "take" verbs for variety in descriptions
///
/// # Access Control
///
/// Items may be denied if they:
/// - Require special capabilities the player lacks (e.g., heat-proof gloves)
/// - Are marked as restricted (cannot be transferred)
/// - Are not portable (fixed in place)
///
/// # Location Handling
///
/// Items can be taken from various locations:
/// - Room contents (lying on the ground)
/// - Open containers (boxes, chests, etc.)
/// - Other accessible locations
///
/// # Errors
/// Returns an error if the player's current room cannot be resolved, if world entities
/// referenced during transfer cannot be found, or if trigger evaluation fails.
pub fn take_handler(world: &mut AmbleWorld, view: &mut View, thing: &str) -> Result<()> {
    let take_verb = world.spin_core(CoreSpinnerType::TakeVerb, "take");
    let room_id = world.player_room_ref()?.id();
    let scope = nearby_reachable_items(world, room_id)?;

    if let Some(entity) = find_world_object(&scope, &world.items, &world.npcs, thing) {
        if entity.is_not_item() {
            unexpected_entity(
                entity,
                view,
                &format!(
                    "That's not an item. You can't {} it.",
                    world.spin_core(CoreSpinnerType::TakeVerb, "take")
                ),
            );
        }
        if let Some(item) = entity.item() {
            // counts as a turn even if action fails due to game-world reason (restricted, not portable, requires special handling)
            world.turn_count += 1;

            // check to make sure special handling isn't necessary
            if let Some(ability) = item.requires_capability_for(ItemInteractionType::Handle) {
                view.push(ViewItem::ActionFailure(format!(
                    "{}",
                    format!(
                        "You can't pick it up barehanded. Use something to {} it.",
                        ability.to_string().bold()
                    )
                    .denied_style()
                )));
                info!(
                    "Blocked attempt to take {} ({}) without item that can \"{ability}\"",
                    item.name(),
                    item.symbol()
                );
                return Ok(());
            }
            if item.portable && !item.restricted {
                // extract item uuid & original location
                let loot_id = item.id();
                let orig_loc = item.location;
                // update item location and copy to player inventory
                if let Some(moved_item) = world.items.get_mut(&loot_id) {
                    moved_item.set_location_inventory();
                    world.player.inventory.insert(moved_item.id());
                    view.push(ViewItem::ActionSuccess(format!(
                        "You {take_verb} the {}.",
                        moved_item.name().item_style()
                    )));
                    info!(
                        "{} took the {} ({})",
                        world.player.name,
                        moved_item.name(),
                        moved_item.symbol()
                    );
                }
                // remove item from original location
                match orig_loc {
                    Location::Item(container_id) => {
                        if let Some(container) = world.items.get_mut(&container_id) {
                            container.remove_item(loot_id);
                        } else {
                            bail!(
                                "container ({}) not found during Take({})",
                                symbol_or_unknown(&world.items, container_id),
                                symbol_or_unknown(&world.items, loot_id)
                            );
                        }
                    },
                    Location::Room(room_id) => {
                        if let Some(room) = world.rooms.get_mut(&room_id) {
                            room.remove_item(loot_id);
                        } else {
                            bail!(
                                "room ({}) not found during Take({})",
                                symbol_or_unknown(&world.rooms, room_id),
                                symbol_or_unknown(&world.items, loot_id)
                            );
                        }
                    },
                    _ => {
                        warn!("'take' matched an item at {orig_loc:?}: shouldn't be in scope");
                    },
                }
                check_triggers(world, view, &[TriggerCondition::Take(loot_id)])?;
            } else {
                let reason = if item.restricted { "restricted" } else { "not portable" };
                view.push(ViewItem::ActionFailure(format!(
                    "You can't {take_verb} the {}. It's {}.\n",
                    item.name().error_style(),
                    reason.italic()
                )));
                info!(
                    "{} denied ({reason}) item {} ({})",
                    world.player.name(),
                    item.name(),
                    item.symbol()
                );
            }
        }
    } else {
        entity_not_found(world, view, thing);
    }
    Ok(())
}

/// Specifies the type of container being accessed for item transfers.
///
/// This enum distinguishes between taking items from NPCs versus taking them
/// from container items, which require different validation and transfer logic.
#[derive(Debug, Copy, Clone)]
pub enum VesselType {
    Item,
    Npc,
}

/// Transfers an item from a container or NPC to the player's inventory.
///
/// This is one of the most complex inventory operations, handling transfers from
/// both container items (like chests or bags) and NPC inventories. It performs
/// extensive validation to ensure the transfer is valid and maintains world consistency.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item_pattern` - Pattern string to match against items in the container/NPC
/// * `vessel_pattern` - Pattern string to match against nearby containers or NPCs
///
/// # Returns
///
/// Returns `Ok(())` on success or error if world state is inconsistent.
///
/// # Validation Process
///
/// The function performs several validation steps:
/// 1. Identifies and validates the target container/NPC in the current room
/// 2. Checks container accessibility (not closed/locked)
/// 3. Searches for the requested item within the container/NPC
/// 4. Verifies the item can be transferred (portable, not restricted)
/// 5. Executes the complete transfer with world state updates
///
/// # Container Access
///
/// For container items:
/// - Must be in an accessible state (open, unlocked)
/// - Player receives appropriate feedback for locked/closed containers
///
/// For NPCs:
/// - Items in NPC inventory are accessible by default
/// - May trigger special NPC interactions or responses
///
/// # Trigger Effects
///
/// Different triggers fire depending on the source:
/// - `TriggerCondition::Take` - General item pickup trigger
/// - `TriggerCondition::TakeFromNpc` - Specific trigger for NPC interactions
///
/// This allows the game to respond differently to taking items from containers
/// versus taking them from NPCs.
///
/// This function handles some complex logic of validating and then transferring
/// items either from an NPC or from a container item. It must validate:
/// 1. The `vessel_pattern` matches a nearby container item or NPC (the "vessel")
/// 2. The vessel contents are accessible (not closed or locked).
/// 3. The vessel contains an item that matches `item_pattern`.
/// 4. Player has permission to take the item (`portable` and not `restricted`).
///
/// Then player inventory, vessel inventory/contents, and item location are all
/// updated to maintain consistent game state.
///
/// # Errors
/// Returns an error if the player is not currently in a valid room, if container or NPC
/// references cannot be resolved, or if trigger evaluation encounters invalid data.
pub fn take_from_handler(
    world: &mut AmbleWorld,
    view: &mut View,
    item_pattern: &str,
    vessel_pattern: &str,
) -> Result<()> {
    // find vessel id from containers and NPCs in room
    let current_room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = current_room.contents.union(&current_room.npcs).copied().collect();

    // extract metadata for the npc or container we're transferring from, which will be used
    // to determine the validation and transfer logic required.
    let (vessel_id, vessel_name, vessel_type) =
        if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, vessel_pattern) {
            if let Some(vessel) = entity.item() {
                if let Some(reason) = vessel.access_denied_reason() {
                    view.push(ViewItem::ActionFailure(format!(
                        "{reason} You can't take anything from it."
                    )));
                    return Ok(());
                }
                (vessel.id(), vessel.name().to_string(), VesselType::Item)
            } else if let Some(npc) = entity.npc() {
                (npc.id(), npc.name().to_string(), VesselType::Npc)
            } else {
                unexpected_entity(
                    entity,
                    view,
                    &format!("\"{vessel_pattern}\" isn't a nearby item or NPC that you can take from."),
                );
                return Ok(());
            }
        } else {
            entity_not_found(world, view, vessel_pattern);
            return Ok(());
        };

    // Validate and execute transfer of loot from container item or NPC
    match vessel_type {
        VesselType::Item => {
            validate_and_transfer_from_item(world, view, item_pattern, vessel_id, &vessel_name)?;
        },
        VesselType::Npc => {
            validate_and_transfer_from_npc(world, view, item_pattern, vessel_id, &vessel_name)?;
        },
    }
    world.turn_count += 1;
    Ok(())
}

/// Validates and executes transfer of an item from an NPC to the player.
///
/// This internal function handles the NPC-specific logic for item transfers,
/// including validation of the NPC's inventory and the target item's transferability.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item_pattern` - Pattern to match against items in the NPC's inventory
/// * `vessel_id` - UUID of the NPC being accessed
/// * `vessel_name` - Display name of the NPC for user feedback
///
/// # Returns
///
/// Returns `Ok(())` on successful transfer or validation failure.
///
/// # Validation
///
/// - Verifies the NPC exists and has the requested item
/// - Checks if the item can be transferred (not restricted)
/// - Handles cases where pattern matches non-items inappropriately
/// - Provides specific feedback for each failure case
///
/// # Transfer Process
///
/// On successful validation:
/// - Calls [`transfer_to_player`] to execute the actual transfer
/// - Triggers both general and NPC-specific trigger conditions
/// - Updates all necessary world state collections
pub(crate) fn validate_and_transfer_from_npc(
    world: &mut AmbleWorld,
    view: &mut View,
    item_pattern: &str,
    vessel_id: Uuid,
    vessel_name: &str,
) -> Result<(), anyhow::Error> {
    let container = world
        .npcs
        .get(&vessel_id)
        .ok_or(anyhow!("container {} lookup failed", vessel_id))?;
    let (loot_id, loot_name) =
        if let Some(entity) = find_world_object(&container.inventory, &world.items, &world.npcs, item_pattern) {
            if let Some(loot) = entity.item() {
                if let Some(reason) = loot.take_denied_reason() {
                    view.push(ViewItem::ActionFailure(reason.to_string()));
                    return Ok(());
                }
                (loot.id(), loot.name().to_string())
            } else {
                unexpected_entity(
                    entity,
                    view,
                    &format!("\"{item_pattern}\" matches an NPC. That would be kidnapping."),
                );
                return Ok(());
            }
        } else {
            view.push(ViewItem::ActionFailure(format!(
                "{} doesn't have any {} to take.",
                vessel_name.npc_style(),
                item_pattern.error_style(),
            )));
            return Ok(());
        };
    transfer_to_player(
        world,
        view,
        VesselType::Npc,
        vessel_id,
        vessel_name,
        loot_id,
        &loot_name,
    );
    check_triggers(
        world,
        view,
        &[
            TriggerCondition::Take(loot_id),
            TriggerCondition::TakeFromNpc {
                item_id: loot_id,
                npc_id: vessel_id,
            },
        ],
    )?;
    Ok(())
}

/// Validates and executes transfer of an item from a container to the player.
///
/// This internal function handles the container-specific logic for item transfers,
/// including validation of container contents and item accessibility.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item_pattern` - Pattern to match against items in the container
/// * `vessel_id` - UUID of the container item being accessed
/// * `vessel_name` - Display name of the container for user feedback
///
/// # Returns
///
/// Returns `Ok(())` on successful transfer or validation failure.
///
/// # Validation
///
/// - Verifies the container exists and contains the requested item
/// - Checks if the item can be transferred (not restricted)
/// - Handles inappropriate pattern matches gracefully
/// - Provides specific feedback for missing items or transfer restrictions
///
/// # Transfer Process
///
/// On successful validation:
/// - Calls [`transfer_to_player`] to execute the actual transfer
/// - Triggers general take conditions (container transfers don't have special triggers)
/// - Updates container contents and player inventory
pub(crate) fn validate_and_transfer_from_item(
    world: &mut AmbleWorld,
    view: &mut View,
    item_pattern: &str,
    vessel_id: Uuid,
    vessel_name: &str,
) -> Result<(), anyhow::Error> {
    let container = world
        .items
        .get(&vessel_id)
        .ok_or(anyhow!("container {} lookup failed", vessel_id))?;
    let (loot_id, loot_name) =
        if let Some(entity) = find_world_object(&container.contents, &world.items, &world.npcs, item_pattern) {
            if let Some(loot) = entity.item() {
                match loot.take_denied_reason() {
                    Some(reason) => {
                        view.push(ViewItem::ActionFailure(reason));
                        return Ok(());
                    },
                    None => (loot.id(), loot.name().to_string()),
                }
            } else {
                unexpected_entity(
                    entity,
                    view,
                    &format!("\"{item_pattern}\" matches an NPC. That would be kidnapping."),
                );
                return Ok(());
            }
        } else {
            view.push(ViewItem::ActionFailure(format!(
                "You don't see any {} in the {} to take.",
                item_pattern.error_style(),
                vessel_name.item_style()
            )));
            return Ok(());
        };
    transfer_to_player(
        world,
        view,
        VesselType::Item,
        vessel_id,
        vessel_name,
        loot_id,
        &loot_name,
    );
    check_triggers(world, view, &[TriggerCondition::Take(loot_id)])?;
    Ok(())
}

/// Executes the complete transfer of an item from a container/NPC to player inventory.
///
/// This function performs the actual world state updates required to move an item
/// from its current location (NPC inventory or container contents) to the player's
/// inventory. It maintains world consistency by updating all affected collections.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `vessel_type` - Whether transferring from an NPC or container item
/// * `vessel_id` - UUID of the source container or NPC
/// * `vessel_name` - Display name of the source for user feedback
/// * `loot_id` - UUID of the item being transferred
/// * `loot_name` - Display name of the item for user feedback
///
/// # World State Updates
///
/// The function updates multiple world state collections:
/// 1. **Item location** - Updates the item's location to inventory
/// 2. **Source cleanup** - Removes item from NPC inventory or container contents
/// 3. **Player inventory** - Adds item to player's inventory collection
/// 4. **User feedback** - Displays success message with randomized take verb
/// 5. **Audit logging** - Records the transfer with full details
///
/// # Consistency
///
/// This function is critical for maintaining world state consistency. It ensures
/// that items exist in exactly one location and that all collections accurately
/// reflect the current world state.
pub fn transfer_to_player(
    world: &mut AmbleWorld,
    view: &mut View,
    vessel_type: VesselType,
    vessel_id: Uuid,
    vessel_name: &str,
    loot_id: Uuid,
    loot_name: &str,
) {
    // Change item location to inventory
    if let Some(moving_item) = world.get_item_mut(loot_id) {
        moving_item.set_location_inventory();
    }

    // Remove item id from vessel
    match vessel_type {
        VesselType::Item => {
            if let Some(vessel) = world.get_item_mut(vessel_id) {
                vessel.remove_item(loot_id);
            }
        },
        VesselType::Npc => {
            if let Some(vessel) = world.npcs.get_mut(&vessel_id) {
                // set a movement pause after taking something from NPC
                vessel.pause_movement(world.turn_count, 4);
                vessel.remove_item(loot_id);
            }
        },
    }

    // Add item to player inventory
    world.player.add_item(loot_id);

    // Report and log success
    let take_verb = world.spin_core(CoreSpinnerType::TakeVerb, "take");
    view.push(ViewItem::ActionSuccess(format!(
        "You {take_verb} the {}.",
        loot_name.item_style()
    )));
    info!(
        "{} took {} ({}) from {} ({})",
        world.player.name(),
        loot_name,
        symbol_or_unknown(&world.items, loot_id),
        vessel_name,
        match vessel_type {
            VesselType::Item => symbol_or_unknown(&world.items, vessel_id),
            VesselType::Npc => symbol_or_unknown(&world.npcs, vessel_id),
        }
    );
}

/// Transfers an item from the player's inventory to a nearby container.
///
/// This command allows players to organize items by placing them in containers
/// like chests, bags, or other storage items. The container must be accessible
/// (unlocked and open) and in the current room.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `item` - Pattern string to match against inventory items
/// * `container` - Pattern string to match against nearby containers
///
/// # Returns
///
/// Returns `Ok(())` on success or error if world state is inconsistent.
///
/// # Validation Process
///
/// 1. **Item validation** - Verifies item exists in inventory and is transferable
/// 2. **Container validation** - Finds container in current room and checks accessibility
/// 3. **Transfer execution** - Updates all world state collections consistently
///
/// # Container Requirements
///
/// Target containers must be:
/// - Present in the current room
/// - Accessible (not locked or closed)
/// - Actually be a container (not a regular item)
///
/// # Trigger Effects
///
/// This action triggers multiple conditions:
/// - `TriggerCondition::Insert` - Specific to putting items in containers
/// - `TriggerCondition::Drop` - General item placement trigger
///
/// This allows the game to respond to both the specific act of organized storage
/// and the general act of item placement.
///
/// # Errors
/// Returns an error if the player's room or the target container cannot be resolved,
/// if world state updates fail due to missing entities, or if trigger evaluation fails.
pub fn put_in_handler(world: &mut AmbleWorld, view: &mut View, item: &str, container: &str) -> Result<()> {
    // get uuid of item and container
    let (item_id, item_name) =
        if let Some(entity) = find_world_object(&world.player.inventory, &world.items, &world.npcs, item) {
            if let Some(item) = entity.item() {
                if let Some(reason) = item.take_denied_reason() {
                    view.push(ViewItem::ActionFailure(reason));
                    world.turn_count += 1;
                    return Ok(());
                }
                (item.id(), item.name().to_string())
            } else {
                unexpected_entity(entity, view, &format!("No item in inventory matches \"{item}\"."));
                return Ok(());
            }
        } else {
            entity_not_found(world, view, item);
            return Ok(());
        };

    let room = world.player_room_ref()?;
    let (vessel_id, vessel_name) =
        if let Some(entity) = find_world_object(&room.contents, &world.items, &world.npcs, container) {
            if let Some(vessel) = entity.item() {
                if let Some(reason) = vessel.access_denied_reason() {
                    view.push(ViewItem::ActionFailure(format!(
                        "{reason} You can't put anything in it."
                    )));
                    world.turn_count += 1;
                    return Ok(());
                }
                (vessel.id(), vessel.name().to_string())
            } else {
                unexpected_entity(
                    entity,
                    view,
                    &format!("You don't see a container matching \"{container}\" here."),
                );
                return Ok(());
            }
        } else {
            entity_not_found(world, view, container);
            return Ok(());
        };

    // update item location and add to container
    if let Some(moved_item) = world.items.get_mut(&item_id) {
        moved_item.set_location_item(vessel_id);
    }
    if let Some(vessel) = world.items.get_mut(&vessel_id) {
        vessel.add_item(item_id);
    }
    // remove item from inventory
    world.player.inventory.remove(&item_id);
    // report and log success
    view.push(ViewItem::ActionSuccess(format!(
        "You put the {} in the {}.",
        item_name.item_style(),
        vessel_name.item_style()
    )));
    info!(
        "{} put {} ({}) into {} ({})",
        world.player.name(),
        item_name,
        symbol_or_unknown(&world.items, item_id),
        vessel_name,
        symbol_or_unknown(&world.items, vessel_id)
    );

    check_triggers(
        world,
        view,
        &[
            TriggerCondition::Insert {
                item: item_id,
                container: vessel_id,
            },
            TriggerCondition::Drop(item_id),
        ],
    )?;
    world.turn_count += 1;
    Ok(())
}

/// Handles cases where an NPC is found when searching for items.
///
/// This utility function provides appropriate error handling when the player's
/// search pattern matches an NPC in a context where only items are expected.
/// It provides user-friendly feedback and logs the unexpected situation for debugging.
///
/// # Parameters
///
/// * `entity` - The world entity that was unexpectedly found
/// * `view` - Mutable reference to the player's view for error messages
/// * `denial_msg` - Specific message explaining why the action cannot be performed
///
/// # Behavior
///
/// - Displays the denial message to the player
/// - Logs detailed error information for debugging
/// - Does not modify world state (safe error handling)
///
/// # Common Use Cases
///
/// This typically occurs when players try to:
/// - Take an NPC (which would be kidnapping)
/// - Put an NPC in a container
/// - Use item-specific commands on NPCs
pub fn unexpected_entity(entity: WorldEntity, view: &mut View, denial_msg: &str) {
    let (entity_name, entity_sym, entity_loc) = match entity {
        WorldEntity::Item(item) => (item.name(), item.symbol(), item.location()),
        WorldEntity::Npc(npc) => (npc.name(), npc.symbol(), npc.location()),
    };
    view.push(ViewItem::Error(denial_msg.to_string()));
    error!("entity '{entity_name}' ({entity_sym}) found in unexpected location {entity_loc:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ItemHolder,
        item::{ContainerState, Item},
        npc::{Npc, NpcState},
        room::Room,
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    struct TestWorld {
        world: AmbleWorld,
        view: View,
        room_id: Uuid,
        inv_item_id: Uuid,
        room_item_id: Uuid,
        chest_id: Uuid,
        gem_id: Uuid,
        npc_id: Uuid,
        npc_item_id: Uuid,
        restr_chest_item_id: Uuid,
        restr_npc_item_id: Uuid,
    }

    fn build_world() -> TestWorld {
        let mut world = AmbleWorld::new_empty();

        // set up room
        let room_id = Uuid::new_v4();
        let room = Room {
            id: room_id,
            symbol: "room".into(),
            name: "Test Room".into(),
            base_description: "".into(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room_id, room);
        world.player.location = Location::Room(room_id);

        // item in inventory
        let inv_item_id = Uuid::new_v4();
        let inv_item = Item {
            id: inv_item_id,
            symbol: "apple".into(),
            name: "Apple".into(),
            description: "".into(),
            location: Location::Inventory,
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(inv_item_id, inv_item);
        world.player.inventory.insert(inv_item_id);

        // item in room
        let room_item_id = Uuid::new_v4();
        let room_item = Item {
            id: room_item_id,
            symbol: "rock".into(),
            name: "Rock".into(),
            description: "".into(),
            location: Location::Room(room_id),
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(room_item_id, room_item);
        world.rooms.get_mut(&room_id).unwrap().add_item(room_item_id);

        // container item with loot
        let chest_id = Uuid::new_v4();
        let mut chest = Item {
            id: chest_id,
            symbol: "chest".into(),
            name: "Chest".into(),
            description: "".into(),
            location: Location::Room(room_id),
            portable: true,
            container_state: Some(ContainerState::Open),
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        let gem_id = Uuid::new_v4();
        let gem = Item {
            id: gem_id,
            symbol: "gem".into(),
            name: "Gem".into(),
            description: "".into(),
            location: Location::Item(chest_id),
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        let restricted_chest_item_id = Uuid::new_v4();
        let restricted_chest_item = Item {
            id: restricted_chest_item_id,
            symbol: "rci".into(),
            name: "Restricted Chest Item".into(),
            description: "".into(),
            location: Location::Item(chest_id),
            portable: true,
            container_state: None,
            restricted: true,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        chest.add_item(gem_id);
        chest.add_item(restricted_chest_item_id);
        world.items.insert(gem_id, gem);
        world.items.insert(chest_id, chest);
        world.items.insert(restricted_chest_item_id, restricted_chest_item);
        world.rooms.get_mut(&room_id).unwrap().add_item(chest_id);

        // npc with item
        let npc_id = Uuid::new_v4();
        let mut npc = Npc {
            id: npc_id,
            symbol: "bob".into(),
            name: "Bob".into(),
            description: "".into(),
            location: Location::Room(room_id),
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: NpcState::Normal,
            movement: None,
        };
        let npc_item_id = Uuid::new_v4();
        let npc_item = Item {
            id: npc_item_id,
            symbol: "coin".into(),
            name: "Coin".into(),
            description: "".into(),
            location: Location::Npc(npc_id),
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        let restricted_npc_item_id = Uuid::new_v4();
        let restricted_npc_item = Item {
            id: restricted_npc_item_id,
            symbol: "key".into(),
            name: "Restricted NPC Item".into(),
            description: "".into(),
            location: Location::Npc(npc_id),
            portable: true,
            container_state: None,
            restricted: true,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        npc.add_item(npc_item_id);
        npc.add_item(restricted_npc_item_id);
        world.items.insert(npc_item_id, npc_item);
        world.items.insert(restricted_npc_item_id, restricted_npc_item);

        world.npcs.insert(npc_id, npc);
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);
        let view = View::new();

        TestWorld {
            world,
            view,
            room_id,
            inv_item_id,
            room_item_id,
            chest_id,
            gem_id,
            npc_id,
            npc_item_id,
            restr_chest_item_id: restricted_chest_item_id,
            restr_npc_item_id: restricted_npc_item_id,
        }
    }

    #[test]
    fn drop_handler_drops_item_into_room() {
        let mut tw = build_world();
        let item_id = tw.inv_item_id;
        let room_id = tw.room_id;
        drop_handler(&mut tw.world, &mut tw.view, "apple").unwrap();
        assert!(!tw.world.player.inventory.contains(&item_id));
        assert!(tw.world.rooms.get(&room_id).unwrap().contents.contains(&item_id));
        assert_eq!(
            tw.world.items.get(&item_id).unwrap().location(),
            &Location::Room(room_id)
        );
    }

    #[test]
    fn take_handler_moves_item_to_inventory() {
        let mut tw = build_world();
        let item_id = tw.room_item_id;
        let room_id = tw.room_id;
        take_handler(&mut tw.world, &mut tw.view, "rock").unwrap();
        assert!(tw.world.player.inventory.contains(&item_id));
        assert!(!tw.world.rooms.get(&room_id).unwrap().contents.contains(&item_id));
        assert_eq!(tw.world.items.get(&item_id).unwrap().location(), &Location::Inventory);
    }

    #[test]
    fn take_from_handler_from_item() {
        let mut tw = build_world();
        let chest_id = tw.chest_id;
        let gem_id = tw.gem_id;
        take_from_handler(&mut tw.world, &mut tw.view, "gem", "chest").unwrap();
        assert!(tw.world.player.inventory.contains(&gem_id));
        assert!(!tw.world.items.get(&chest_id).unwrap().contents.contains(&gem_id));
        assert_eq!(tw.world.items.get(&gem_id).unwrap().location(), &Location::Inventory);
    }

    #[test]
    fn take_restricted_item_from_item_blocked() {
        let mut tw = build_world();
        let chest_id = tw.chest_id;
        let item_id = tw.restr_chest_item_id;
        take_from_handler(&mut tw.world, &mut tw.view, "restricted", "bob").unwrap();
        assert!(!tw.world.player.inventory.contains(&item_id));
        assert!(tw.world.items.get(&chest_id).unwrap().contents.contains(&item_id));
        assert_eq!(
            tw.world.items.get(&item_id).unwrap().location(),
            &Location::Item(chest_id)
        );
    }

    #[test]
    fn take_from_handler_from_npc() {
        let mut tw = build_world();
        let npc_id = tw.npc_id;
        let coin_id = tw.npc_item_id;
        take_from_handler(&mut tw.world, &mut tw.view, "coin", "bob").unwrap();
        assert!(tw.world.player.inventory.contains(&coin_id));
        assert!(!tw.world.npcs.get(&npc_id).unwrap().inventory.contains(&coin_id));
        assert_eq!(tw.world.items.get(&coin_id).unwrap().location(), &Location::Inventory);
    }

    #[test]
    fn take_restricted_item_from_npc_blocked() {
        let mut tw = build_world();
        let npc_id = tw.npc_id;
        let item_id = tw.restr_npc_item_id;
        take_from_handler(&mut tw.world, &mut tw.view, "restricted", "bob").unwrap();
        assert!(!tw.world.player.inventory.contains(&item_id));
        assert!(tw.world.npcs.get(&npc_id).unwrap().inventory.contains(&item_id));
        assert_eq!(tw.world.items.get(&item_id).unwrap().location(), &Location::Npc(npc_id));
    }

    #[test]
    fn validate_and_transfer_from_item_moves_loot() {
        let mut tw = build_world();
        let chest_id = tw.chest_id;
        let gem_id = tw.gem_id;
        validate_and_transfer_from_item(&mut tw.world, &mut tw.view, "gem", chest_id, "Chest").unwrap();
        assert!(tw.world.player.inventory.contains(&gem_id));
        assert!(!tw.world.items.get(&chest_id).unwrap().contents.contains(&gem_id));
        assert_eq!(tw.world.items.get(&gem_id).unwrap().location(), &Location::Inventory);
    }

    #[test]
    fn validate_and_transfer_from_npc_moves_loot() {
        let mut tw = build_world();
        let npc_id = tw.npc_id;
        let coin_id = tw.npc_item_id;
        validate_and_transfer_from_npc(&mut tw.world, &mut tw.view, "coin", npc_id, "Bob").unwrap();
        assert!(tw.world.player.inventory.contains(&coin_id));
        assert!(!tw.world.npcs.get(&npc_id).unwrap().inventory.contains(&coin_id));
        assert_eq!(tw.world.items.get(&coin_id).unwrap().location(), &Location::Inventory);
    }

    #[test]
    fn transfer_to_player_updates_world_from_item() {
        let mut tw = build_world();
        transfer_to_player(
            &mut tw.world,
            &mut tw.view,
            VesselType::Item,
            tw.chest_id,
            "Chest",
            tw.gem_id,
            "Gem",
        );
        assert!(tw.world.player.inventory.contains(&tw.gem_id));
        assert_eq!(tw.world.items.get(&tw.gem_id).unwrap().location(), &Location::Inventory);
        assert!(!tw.world.items.get(&tw.chest_id).unwrap().contents.contains(&tw.gem_id));
    }

    #[test]
    fn transfer_to_player_updates_world_from_npc() {
        let mut tw = build_world();
        transfer_to_player(
            &mut tw.world,
            &mut tw.view,
            VesselType::Npc,
            tw.npc_id,
            "Bob",
            tw.npc_item_id,
            "Coin",
        );
        assert!(tw.world.player.inventory.contains(&tw.npc_item_id));
        assert_eq!(
            tw.world.items.get(&tw.npc_item_id).unwrap().location(),
            &Location::Inventory
        );
        assert!(
            !tw.world
                .npcs
                .get(&tw.npc_id)
                .unwrap()
                .inventory
                .contains(&tw.npc_item_id)
        );
    }

    #[test]
    fn put_in_handler_moves_item_into_container() {
        let mut tw = build_world();
        put_in_handler(&mut tw.world, &mut tw.view, "apple", "chest").unwrap();
        assert!(!tw.world.player.inventory.contains(&tw.inv_item_id));
        assert!(
            tw.world
                .items
                .get(&tw.chest_id)
                .unwrap()
                .contents
                .contains(&tw.inv_item_id)
        );
        assert_eq!(
            tw.world.items.get(&tw.inv_item_id).unwrap().location(),
            &Location::Item(tw.chest_id)
        );
    }

    #[test]
    fn unexpected_entity_does_not_change_world() {
        let mut tw = build_world();
        let before = tw.world.npcs.get(&tw.npc_id).unwrap().location().clone();
        {
            let npc_ref = tw.world.npcs.get(&tw.npc_id).unwrap();
            unexpected_entity(WorldEntity::Npc(npc_ref), &mut tw.view, "nope");
        }
        let after = tw.world.npcs.get(&tw.npc_id).unwrap().location().clone();
        assert_eq!(before, after);
    }
}
