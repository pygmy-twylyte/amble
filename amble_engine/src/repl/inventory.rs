//! `repl::inventory` module
//!
//! Contains repl loop handlers for commands that affect player inventory

use std::collections::HashSet;

use crate::{
    AmbleWorld, ItemHolder, Location, View, ViewItem, WorldObject,
    helpers::{item_symbol_from_id, npc_symbol_from_id, room_symbol_from_id},
    item::ItemInteractionType,
    repl::{WorldEntity, entity_not_found, find_world_object},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers},
    world::nearby_reachable_items,
};

use anyhow::{Result, anyhow, bail};
use colored::Colorize;
use log::{error, info, warn};
use uuid::Uuid;

/// Drops an item from inventory in the current room.
pub fn drop_handler(world: &mut AmbleWorld, view: &mut View, thing: &str) -> Result<()> {
    if let Some(entity) = find_world_object(&world.player.inventory, &world.items, &world.npcs, thing) {
        if let Some(item) = entity.item() {
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
/// Removes an item from current room and adds it to inventory.
pub fn take_handler(world: &mut AmbleWorld, view: &mut View, thing: &str) -> Result<()> {
    let take_verb = world.spin_spinner(SpinnerType::TakeVerb, "take");
    let room_id = world.player_room_ref()?.id();
    let scope = nearby_reachable_items(world, room_id)?;

    if let Some(entity) = find_world_object(&scope, &world.items, &world.npcs, thing) {
        if entity.is_not_item() {
            unexpected_entity(
                entity,
                view,
                &format!(
                    "That's not an item. You can't {} it.",
                    world.spin_spinner(SpinnerType::TakeVerb, "take")
                ),
            );
        }
        if let Some(item) = entity.item() {
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
                                item_symbol_from_id(&world.items, container_id),
                                item_symbol_from_id(&world.items, loot_id)
                            );
                        }
                    },
                    Location::Room(room_id) => {
                        if let Some(room) = world.rooms.get_mut(&room_id) {
                            room.remove_item(loot_id);
                        } else {
                            bail!(
                                "room ({}) not found during Take({})",
                                room_symbol_from_id(&world.rooms, room_id),
                                item_symbol_from_id(&world.items, loot_id)
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

/// Indicates whether the vessel we're taking from is an NPC or another item.
#[derive(Debug, Copy, Clone)]
pub enum VesselType {
    Item,
    Npc,
}

/// Removes an item from a vessel (NPC or container item) and adds to inventory.
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
    Ok(())
}

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

/// Update world state to move item from a vessel (NPC or container item) into inventory.
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
                vessel.remove_item(loot_id);
            }
        },
    }

    // Add item to player inventory
    world.player.add_item(loot_id);

    // Report and log success
    let take_verb = world.spin_spinner(SpinnerType::TakeVerb, "take");
    view.push(ViewItem::ActionSuccess(format!(
        "You {take_verb} the {}.",
        loot_name.item_style()
    )));
    info!(
        "{} took {} ({}) from {} ({})",
        world.player.name(),
        loot_name,
        item_symbol_from_id(&world.items, loot_id),
        vessel_name,
        match vessel_type {
            VesselType::Item => item_symbol_from_id(&world.items, vessel_id),
            VesselType::Npc => npc_symbol_from_id(&world.npcs, vessel_id),
        }
    );
}

/// Removes an item from inventory and places it in a nearby container.
pub fn put_in_handler(world: &mut AmbleWorld, view: &mut View, item: &str, container: &str) -> Result<()> {
    // get uuid of item and container
    let (item_id, item_name) =
        if let Some(entity) = find_world_object(&world.player.inventory, &world.items, &world.npcs, item) {
            if let Some(item) = entity.item() {
                if let Some(reason) = item.take_denied_reason() {
                    view.push(ViewItem::ActionFailure(reason));
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
        item_symbol_from_id(&world.items, item_id),
        vessel_name,
        item_symbol_from_id(&world.items, vessel_id)
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
    Ok(())
}

/// Handle situation where an NPC uuid is found where only items should be.
/// (Typically when `find_world_object` matches an NPC when an item is expected.)
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
