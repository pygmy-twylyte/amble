//! Placement helpers for populating rooms, containers, and inventories.
//!
//! WorldDef items and NPCs already carry location data; these helpers apply
//! those locations to room contents and inventories after the player is built.

use anyhow::{Context, Result, anyhow};
use log::info;

use crate::item::ItemHolder;
use crate::world::{Location, WorldObject};
use crate::{AmbleWorld, Id};

/// Place items in their starting locations.
///
/// # Errors
/// - on failed lookups of items, rooms, or NPCs in the world
pub fn place_items(world: &mut AmbleWorld) -> Result<()> {
    info!("building item location lists for placement stage");
    let mut room_placements: Vec<(Id, Id)> = Vec::new();
    let mut chest_placements: Vec<(Id, Id)> = Vec::new();
    let mut npc_placements: Vec<(Id, Id)> = Vec::new();
    let mut inventory: Vec<Id> = Vec::new();
    let mut unspawned = 0;

    for item in world.items.values() {
        match &item.location {
            Location::Room(room_id) => room_placements.push((room_id.clone(), item.id.clone())),
            Location::Item(chest_id) => chest_placements.push((chest_id.clone(), item.id.clone())),
            Location::Npc(npc_id) => npc_placements.push((npc_id.clone(), item.id.clone())),
            Location::Inventory => inventory.push(item.id.clone()),
            Location::Nowhere => unspawned += 1,
        }
    }

    // Place items into containers first to allow nested objects to populate correctly.
    info!("placing {} items into containers", chest_placements.len());
    for (chest_id, item_id) in chest_placements {
        let chest = world
            .items
            .get_mut(&chest_id)
            .with_context(|| format!("container item id {chest_id} not found in world.items"))?;
        chest.add_item(item_id);
    }

    info!("placing {} items into rooms", room_placements.len());
    for (room_id, item_id) in room_placements {
        let room = world
            .rooms
            .get_mut(&room_id)
            .with_context(|| format!("room id {room_id} not found in world.rooms"))?;
        room.contents.insert(item_id);
    }

    info!("placing {} items into NPC inventories", npc_placements.len());
    for (npc_id, item_id) in npc_placements {
        let npc = world
            .npcs
            .get_mut(&npc_id)
            .with_context(|| format!("npc id {npc_id} not found in world.npcs"))?;
        npc.add_item(item_id);
    }

    info!("placing {} items into player inventory", inventory.len());
    for item_id in inventory {
        world.player.add_item(item_id);
    }

    info!("{unspawned} items remain unspawned (Location::Nowhere)");
    Ok(())
}

/// Place NPCs in their starting rooms.
///
/// # Errors
/// - on invalid placement locations
pub fn place_npcs(world: &mut AmbleWorld) -> Result<()> {
    let mut placements: Vec<(Id, Id)> = Vec::new();
    let mut unspawned = 0;

    for npc in world.npcs.values() {
        match &npc.location {
            Location::Room(room_id) => placements.push((npc.id.clone(), room_id.clone())),
            Location::Nowhere => unspawned += 1,
            _ => {
                return Err(anyhow!(
                    "NPC {} ({}) bad location ({:?}) - must be Room or Nowhere",
                    npc.name(),
                    npc.id(),
                    npc.location()
                ));
            },
        }
    }

    for (npc_id, room_id) in &placements {
        let room = world
            .rooms
            .get_mut(room_id)
            .with_context(|| format!("looking up {room_id} to place {npc_id}"))?;
        room.npcs.insert(npc_id.clone());
    }

    info!("{} NPCs placed into their starting rooms", placements.len());
    info!("{unspawned} NPCs remain unspawned (Location::Nowhere)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::item::{ContainerState, Item, Movability};
    use crate::world::Location;
    use crate::{AmbleWorld, Room, idgen};

    use super::place_items;

    #[test]
    fn test_transparent_container_loading() {
        let mut world = AmbleWorld::new_empty();

        let room_id = idgen::new_id();
        let room = Room {
            id: room_id.clone(),
            symbol: "test_room".into(),
            name: "Test Room".into(),
            base_description: "A test room".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room_id.clone(), room);

        let container_id = idgen::new_id();
        let container = Item {
            id: container_id.clone(),
            symbol: "test_container".into(),
            name: "Test Container".into(),
            description: "A test container".into(),
            location: Location::Room(room_id.clone()),
            movability: Movability::Free,
            container_state: Some(ContainerState::TransparentLocked),
            contents: HashSet::new(),
            abilities: HashSet::new(),
            consumable: None,
            interaction_requires: HashMap::new(),
            text: None,
        };

        let item_id = idgen::new_id();
        let item = Item {
            id: item_id.clone(),
            symbol: "test_item".into(),
            name: "Test Item".into(),
            description: "A test item".into(),
            location: Location::Item(container_id.clone()),
            container_state: None,
            movability: Movability::Free,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            consumable: None,
            interaction_requires: HashMap::new(),
            text: None,
        };

        world.items.insert(container_id.clone(), container);
        world.items.insert(item_id.clone(), item);

        place_items(&mut world).unwrap();

        let container = world.items.get(&container_id).unwrap();
        assert!(container.contents.contains(&item_id));
        assert_eq!(container.container_state, Some(ContainerState::TransparentLocked));
    }
}
