//! `repl::inventory` module
//!
//! Contains repl loop handlers for commands that affect player inventory

use std::collections::HashSet;

use crate::{
    AmbleWorld, ItemHolder, WorldObject,
    item::ItemInteractionType,
    repl::{entity_not_found, find_world_object},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
};

use anyhow::{Result, anyhow};
use colored::Colorize;
use log::{error, info, warn};
use uuid::Uuid;

/// Drops an item from inventory in the current room.
pub fn drop_handler(world: &mut AmbleWorld, thing: &str) -> Result<()> {
    if let Some(entity) =
        find_world_object(&world.player.inventory, &world.items, &world.npcs, thing)
    {
        if let Some(item) = entity.item() {
            let item_id = item.id();
            let room_id = world.player_room_ref()?.id();
            if item.portable {
                if let Some(dropped) = world.items.get_mut(&item_id) {
                    dropped.set_location_room(room_id);
                    if let Some(room) = world.rooms.get_mut(&room_id) {
                        room.add_item(dropped.id());
                        info!(
                            "{} dropped {}({}) in {}({})",
                            world.player.name(),
                            dropped.name(),
                            dropped.id(),
                            room.name(),
                            room.id()
                        );
                    }
                    world.player.remove_item(item_id);
                    println!("You dropped the {}.\n", dropped.name().item_style());
                    check_triggers(world, &[TriggerCondition::Drop(item_id)])?;
                }
            } else {
                // item not portable
                println!("You can't drop the {}.", item.name().item_style());
                return Ok(());
            }
            Ok(())
        } else {
            // entity is not an item
            println!(
                "{}? That's not a thing that one can drop.",
                thing.error_style()
            );
            Ok(())
        }
    } else {
        entity_not_found(world, thing);
        Ok(())
    }
}

/// Removes an item from current room and adds it to inventory.
pub fn take_handler(world: &mut AmbleWorld, thing: &str) -> Result<()> {
    let take_verb = world.spin_spinner(SpinnerType::TakeVerb, "take");
    let current_room = world.player_room_ref()?;

    if let Some(entity) =
        find_world_object(&current_room.contents, &world.items, &world.npcs, thing)
    {
        if entity.is_not_item() {
            // entity found isn't an Item
            warn!("Player tried to take a non-Item from a room: {entity:?}");
            println!(
                "That's not an item. You can't {} it.",
                world.spin_spinner(SpinnerType::TakeVerb, "take")
            );
        }
        if let Some(item) = entity.item() {
            // check to make sure special handling isn't necessary
            if let Some(ability) = item.requires_capability_for(ItemInteractionType::Handle) {
                println!(
                    "You can't pick it up barehanded. Use something to {} it.",
                    ability.to_string().bold()
                );
                info!(
                    "Blocked attempt to take {} ({}) without item that can \"{ability}\"",
                    item.name(),
                    item.id()
                );
                return Ok(());
            }
            if item.portable {
                // extract item uuid
                let uuid = item.id();
                // update item location and copy to player inventory
                if let Some(moved_item) = world.items.get_mut(&uuid) {
                    moved_item.set_location_inventory();
                    world.player.inventory.insert(moved_item.id());
                    println!("You {take_verb} the {}.\n", moved_item.name().item_style());
                    info!(
                        "{} took the {} ({})",
                        world.player.name,
                        moved_item.name(),
                        moved_item.id()
                    );
                }
                // remove item from room
                world.player_room_mut()?.contents.remove(&uuid);
                check_triggers(world, &[TriggerCondition::Take(uuid)])?;
            } else {
                println!(
                    "You can't {take_verb} the {}. It's not portable.\n",
                    item.name().error_style()
                );
                info!(
                    "{} attempted to take fixed item {} ({})",
                    world.player.name(),
                    item.name(),
                    item.id()
                );
            }
        }
    } else {
        entity_not_found(world, thing);
    }
    Ok(())
}

/// Indicates whether the vessel we're taking from is an NPC or another item.
#[derive(Debug, Copy, Clone)]
pub enum VesselType {
    Item,
    Npc,
}

/// Removes an item from a vessel (NPC or container item) and adds to inventory.
pub fn take_from_handler(
    world: &mut AmbleWorld,
    item_pattern: &str,
    vessel_pattern: &str,
) -> Result<()> {
    // find vessel id from containers and NPCs in room
    let current_room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = current_room
        .contents
        .union(&current_room.npcs)
        .copied()
        .collect();

    // extract metadata for the npc or container we're transferring from
    let (vessel_id, vessel_name, vessel_type) = if let Some(entity) =
        find_world_object(&search_scope, &world.items, &world.npcs, vessel_pattern)
    {
        if let Some(vessel) = entity.item() {
            if vessel.is_accessible() {
                // it's a container and it's open
                (vessel.id(), vessel.name().to_string(), VesselType::Item)
            } else {
                // tell player why vessel can't be accessed
                if let Some(reason) = vessel.access_denied_reason() {
                    println!("{reason} You can't take anything from it.");
                }
                return Ok(());
            }
        } else if let Some(npc) = entity.npc() {
            (npc.id(), npc.name().to_string(), VesselType::Npc)
        } else {
            println!(
                "{} isn't an nearby item or NPC. You can't take anything from it.",
                vessel_pattern.error_style()
            );
            return Ok(());
        }
    } else {
        entity_not_found(world, vessel_pattern);
        return Ok(());
    };

    // Validate and execute transfer of loot from container item or NPC
    match vessel_type {
        VesselType::Item => {
            validate_and_transfer_from_item(world, item_pattern, vessel_id, &vessel_name)?;
        }
        VesselType::Npc => {
            validate_and_transfer_from_npc(world, item_pattern, vessel_id, &vessel_name)?;
        }
    }
    Ok(())
}

fn validate_and_transfer_from_npc(
    world: &mut AmbleWorld,
    item_pattern: &str,
    vessel_id: Uuid,
    vessel_name: &str,
) -> Result<(), anyhow::Error> {
    let container = world
        .npcs
        .get(&vessel_id)
        .ok_or(anyhow!("container {} lookup failed", vessel_id))?;
    let (loot_id, loot_name) = if let Some(entity) = find_world_object(
        &container.inventory,
        &world.items,
        &world.npcs,
        item_pattern,
    ) {
        if let Some(loot) = entity.item() {
            if loot.restricted {
                println!(
                    "Sorry, you can't take {} from {}.\n(But it may be given to you under the right conditions.)",
                    loot.name().item_style(),
                    vessel_name.npc_style()
                );
                return Ok(());
            }
            (loot.id(), loot.name().to_string())
        } else {
            warn!(
                "Non-item WorldEntity found inside NPC '{vessel_name}' ({vessel_id})",
            );
            println!(
                "{} shouldn't have that. You can't have it either.",
                vessel_name.npc_style()
            );
            return Ok(());
        }
    } else {
        println!(
            "{} doesn't have any {} to take.",
            vessel_name.npc_style(),
            item_pattern.error_style(),
        );
        return Ok(());
    };
    transfer_to_player(
        world,
        VesselType::Npc,
        vessel_id,
        vessel_name,
        loot_id,
        &loot_name,
    );
    check_triggers(
        world,
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

fn validate_and_transfer_from_item(
    world: &mut AmbleWorld,
    item_pattern: &str,
    vessel_id: Uuid,
    vessel_name: &str,
) -> Result<(), anyhow::Error> {
    let container = world
        .items
        .get(&vessel_id)
        .ok_or(anyhow!("container {} lookup failed", vessel_id))?;
    let (loot_id, loot_name) = if let Some(entity) =
        find_world_object(&container.contents, &world.items, &world.npcs, item_pattern)
    {
        if let Some(loot) = entity.item() {
            if loot.portable && !loot.restricted {
                (loot.id(), loot.name().to_string())
            } else {
                println!("Sorry, the {} can't be removed.", loot.name().item_style());
                return Ok(());
            }
        } else {
            warn!("NPC WorldEntity found inside container '{vessel_name}' ({vessel_id})");
            println!(
                "That shouldn't be in the {} and can't be taken. Just ignore it.",
                vessel_name.item_style()
            );
            return Ok(());
        }
    } else {
        println!(
            "You don't see any {} in the {} to take.",
            item_pattern.error_style(),
            vessel_name.item_style()
        );
        return Ok(());
    };
    transfer_to_player(
        world,
        VesselType::Item,
        vessel_id,
        vessel_name,
        loot_id,
        &loot_name,
    );
    check_triggers(world, &[TriggerCondition::Take(loot_id)])?;
    Ok(())
}

/// Update world state to move item from a vessel (NPC or container item) into inventory.
pub fn transfer_to_player(
    world: &mut AmbleWorld,
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
        }
        VesselType::Npc => {
            if let Some(vessel) = world.npcs.get_mut(&vessel_id) {
                vessel.remove_item(loot_id);
            }
        }
    }

    // Add item to player inventory
    world.player.add_item(loot_id);

    // Report and log success
    let take_verb = world.spin_spinner(SpinnerType::TakeVerb, "take");
    println!("You {take_verb} the {}.\n", loot_name.item_style());
    info!(
        "{} took {} ({}) from {} ({})",
        world.player.name(),
        loot_name,
        loot_id,
        vessel_name,
        vessel_id
    );
}

/// Removes an item from inventory and places it in a nearby container.
pub fn put_in_handler(world: &mut AmbleWorld, item: &str, container: &str) -> Result<()> {
    // get uuid of item and container
    let (item_id, item_name) = if let Some(entity) =
        find_world_object(&world.player.inventory, &world.items, &world.npcs, item)
    {
        if let Some(item) = entity.item() {
            if item.portable {
                (item.id(), item.name().to_string())
            } else {
                println!(
                    "You can't take the {}; it isn't portable.",
                    item.name().item_style()
                );
                info!(
                    "The Candidate tried to take fixed item '{}' ({})",
                    item.name(),
                    item.id()
                );
                return Ok(());
            }
        } else {
            println!("Nothing in inventory matches {}.", item.error_style());
            return Ok(());
        }
    } else {
        entity_not_found(world, item);
        return Ok(());
    };

    let room = world.player_room_ref()?;
    let (vessel_id, vessel_name) = if let Some(entity) =
        find_world_object(&room.contents, &world.items, &world.npcs, container)
    {
        if let Some(vessel) = entity.item() {
            if let Some(reason) = vessel.access_denied_reason() {
                println!("{reason} You can't put anything in it.");
                return Ok(());
            }
            (vessel.id(), vessel.name().to_string())
        } else {
            error!(
                "a non-item (= NPC) WorldEntity was found in contents of room '{}' ({})",
                room.name(),
                room.id()
            );
            println!(
                "You don't see a container by the name {}",
                container.error_style()
            );
            return Ok(());
        }
    } else {
        entity_not_found(world, container);
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
    println!(
        "You put the {} in the {}.\n",
        item_name.item_style(),
        vessel_name.item_style()
    );
    info!(
        "{} put {} ({}) into {} ({})",
        world.player.name(),
        item_name,
        item_id,
        vessel_name,
        vessel_id
    );
    check_triggers(
        world,
        &[
            TriggerCondition::Insert {
                item: item_id,
                container: vessel_id,
            },
            TriggerCondition::Drop(item_id),
        ],
    )?;
    Ok(())
}
