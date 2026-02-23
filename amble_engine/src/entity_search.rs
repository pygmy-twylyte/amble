//! Entity Search Module
//!
//! Purpose: In many handlers across the codebase, there is a need to take a string
//! from user input and match it to a nearby item or NPC -- making a natural choice
//! for a unified handler for that job.
//!
//! While this is straightforward and can be accomplished in a few handfuls of lines
//! on the surface, it is complicated by a variety of needs in terms of search scope:
//!
//! - some searches will only want items, some will want NPCS, some will want either
//! - visible items and reachable items are two different things (consider an item in a locked transparent container)
//!
//! Callers send:
//! - an immutable `AmbleWorld` reference
//! - the search string
//! - the [`SearchScope`] of interest
//!
//! Also, callers get in return:
//! - the Id of the found entity, OR
//! - the reason that an entity's ID is not being returned (`SearchError`)
//!
//! General usage:
//! `find_item_match(...)` when only interested in items for a given command
//! `find_npc_match(...)` when only interested in npcs
//! `find_entity_match(...)` when the input could refer to either an item or an npc

use std::collections::HashSet;

pub use crate::EntityId;
use crate::{ItemId, NpcId, RoomId};
use thiserror::Error;

use crate::{
    AmbleWorld,
    repl::find_world_object,
    world::{nearby_reachable_items, nearby_vessel_items, nearby_visible_items},
};

/// Represents the scope of a requested search by the caller and includes the location to search.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchScope {
    /// All items and NPCs within the player's sight
    AllVisible(RoomId),
    /// All items and NPCs that the player can touch
    AllTouchable(RoomId),
    /// Items the player can look at.
    VisibleItems(RoomId),
    /// NPCs the player can see.
    VisibleNpcs(RoomId),
    /// Items the player can touch (but not necessarily move) in room or inventory
    TouchableItems(RoomId),
    /// NPCs the player can touch.
    TouchableNpcs(RoomId),
    /// Nearby container items or NPCs which can potentially offer or accept an item.
    NearbyVessels(RoomId),
    /// Only items in player's inventory.
    Inventory,
    /// Only items in an NPC's inventory.
    NpcInventory(NpcId),
}

/// Possible errors / situations causing a failed entity search.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("no entity in scope name matching user input '{0}'")]
    NoMatchingName(String),
    #[error("no npc found with the supplied id {0}")]
    InvalidNpcId(NpcId),
    #[error("found no room with the supplied id ({0})")]
    InvalidRoomId(RoomId),
    #[error("invalid {0} search scope: includes only {1}s")]
    InvalidScope(String, String),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// Find an `Item` with name matching `pattern` in the given `SearchScope` and return its id.
///
/// # Errors
/// - if no match found in the specified scope
/// - if an invalid scope for an item search is specified
/// - if the supplied room or NPC ids are invalid
pub fn find_item_match(world: &AmbleWorld, pattern: &str, scope: SearchScope) -> Result<ItemId, SearchError> {
    // construct a HashSet of item ids in scope for this search
    let haystack: HashSet<_> = match scope {
        SearchScope::VisibleItems(room_id) | SearchScope::AllVisible(room_id) => {
            let room_items = nearby_visible_items(world, &room_id).map_err(|_| SearchError::InvalidRoomId(room_id))?;
            room_items.union(&world.player.inventory).cloned().collect()
        },
        SearchScope::TouchableItems(room_id) | SearchScope::AllTouchable(room_id) => {
            let room_items =
                nearby_reachable_items(world, &room_id).map_err(|_| SearchError::InvalidRoomId(room_id))?;
            room_items.union(&world.player.inventory).cloned().collect()
        },
        SearchScope::NearbyVessels(room_id) => {
            nearby_vessel_items(world, &room_id).map_err(|_| SearchError::InvalidRoomId(room_id.clone()))?
        },
        SearchScope::Inventory => world.player.inventory.clone(),
        SearchScope::NpcInventory(npc_id) => {
            let npc = world.npcs.get(&npc_id).ok_or(SearchError::InvalidNpcId(npc_id))?;
            npc.inventory.clone()
        },
        SearchScope::VisibleNpcs(_) | SearchScope::TouchableNpcs(_) => {
            return Err(SearchError::InvalidScope("item".to_string(), "NPC".to_string()));
        },
    };

    let Some(entity) = find_world_object(
        haystack.iter(),
        std::iter::empty::<&NpcId>(),
        &world.items,
        &world.npcs,
        pattern,
    ) else {
        return Err(SearchError::NoMatchingName(pattern.to_string()));
    };
    match entity.entity_id() {
        EntityId::Item(item_id) => Ok(item_id),
        EntityId::Npc(_) => Err(SearchError::InvalidScope("item".into(), "NPC".into())),
    }
}

/// Find either an `NPC` or an `Item` with name matching `pattern` within the `SearchScope`.
///
/// # Panics
/// - never: the call to `expect` is guarded by an `is_ok()`
///
/// # Errors
/// - `SearchError` variants of any type, if no match is found or if the scope type or supplied ids are invalid
pub fn find_entity_match(world: &AmbleWorld, pattern: &str, scope: SearchScope) -> Result<EntityId, SearchError> {
    // return any item matched first -- these will account for most searches
    // no_match and scope errors are ignored on this pass because they may
    // not be errors when run against the NPCs
    match find_item_match(world, pattern, scope.clone()) {
        Ok(item_id) => return Ok(EntityId::Item(item_id)),
        Err(SearchError::NoMatchingName(_) | SearchError::InvalidScope(_, _)) => (),
        Err(e) => return Err(e),
    }

    // less often looking for an NPC match -- return that now if found
    match find_npc_match(world, pattern, scope) {
        Ok(npc_id) => Ok(EntityId::Npc(npc_id)),
        Err(e) => Err(e),
    }
}

/// Find an `Npc` with name matching `pattern` in the specified `SearchScope`.
///
/// # Errors
/// - if no match found in the specified scope
/// - if an invalid scope for an npc search is specified
/// - if the supplied room id is invalid
pub fn find_npc_match(world: &AmbleWorld, pattern: &str, scope: SearchScope) -> Result<NpcId, SearchError> {
    let haystack = match scope {
        // currently there is no distinction between NPCs you can see and those you could touch
        // both scopes kept for now as this may change in the future
        SearchScope::VisibleNpcs(room_id)
        | SearchScope::TouchableNpcs(room_id)
        | SearchScope::NearbyVessels(room_id)
        | SearchScope::AllVisible(room_id)
        | SearchScope::AllTouchable(room_id) => {
            let room = world.rooms.get(&room_id).ok_or(SearchError::InvalidRoomId(room_id))?;
            room.npcs.clone()
        },
        SearchScope::VisibleItems(_)
        | SearchScope::TouchableItems(_)
        | SearchScope::Inventory
        | SearchScope::NpcInventory(_) => {
            return Err(SearchError::InvalidScope("npc".into(), "item".into()));
        },
    };

    let Some(entity) = find_world_object(
        std::iter::empty::<&ItemId>(),
        haystack.iter(),
        &world.items,
        &world.npcs,
        pattern,
    ) else {
        return Err(SearchError::NoMatchingName(pattern.to_string()));
    };

    match entity.entity_id() {
        EntityId::Npc(npc_id) => Ok(npc_id),
        EntityId::Item(_) => Err(SearchError::InvalidScope("npc".into(), "item".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        AmbleWorld, Item, Npc, Room,
        health::HealthState,
        item::{ContainerState, Movability},
        npc::NpcState,
        world::Location,
    };
    use std::collections::{HashMap, HashSet};

    fn insert_room(world: &mut AmbleWorld, name: &str) -> RoomId {
        let room_id = crate::idgen::new_room_id();
        let room = Room {
            id: room_id.clone(),
            symbol: format!("room_{room_id}"),
            name: name.to_string(),
            base_description: format!("{name} description"),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room_id.clone(), room);
        room_id
    }

    fn insert_item(
        world: &mut AmbleWorld,
        name: &str,
        location: Location,
        container_state: Option<ContainerState>,
    ) -> ItemId {
        let item_id: ItemId = crate::idgen::new_id().into();
        let item = Item {
            id: item_id.clone(),
            symbol: format!("item_{item_id}"),
            name: name.to_string(),
            description: format!("{name} item"),
            location,
            visibility: crate::item::ItemVisibility::Listed,
            visible_when: None,
            aliases: Vec::new(),
            movability: Movability::Free,
            container_state,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(item_id.clone(), item);
        item_id
    }

    fn insert_npc(world: &mut AmbleWorld, name: &str, location: Location) -> NpcId {
        let npc_id: NpcId = crate::idgen::new_id().into();
        let npc = Npc {
            id: npc_id.clone(),
            symbol: format!("npc_{npc_id}"),
            name: name.to_string(),
            description: format!("{name} npc"),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: NpcState::Normal,
            movement: None,
            health: HealthState::new(),
        };
        world.npcs.insert(npc_id.clone(), npc);
        npc_id
    }

    #[test]
    fn find_item_match_includes_inventory_items_in_visible_scope() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Atrium");
        let coin_id = insert_item(&mut world, "Lucky Coin", Location::Inventory, None);
        world.player.inventory.insert(coin_id.clone());

        let result = find_item_match(&world, "coin", SearchScope::VisibleItems(room_id)).unwrap();
        assert_eq!(result, coin_id);
    }

    #[test]
    fn find_item_match_returns_nearby_vessels() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Vault");
        let chest_id = insert_item(
            &mut world,
            "Ancient Chest",
            Location::Room(room_id.clone()),
            Some(ContainerState::Open),
        );
        world.rooms.get_mut(&room_id).unwrap().contents.insert(chest_id.clone());

        let result = find_item_match(&world, "chest", SearchScope::NearbyVessels(room_id)).unwrap();
        assert_eq!(result, chest_id);
    }

    #[test]
    fn find_item_match_errors_when_room_missing() {
        let world = AmbleWorld::new_empty();
        let room_id = crate::idgen::new_room_id();

        let err = find_item_match(&world, "coin", SearchScope::VisibleItems(room_id.clone())).unwrap_err();
        match err {
            SearchError::InvalidRoomId(id) => assert_eq!(id, room_id),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn find_item_match_rejects_npc_scopes() {
        let world = AmbleWorld::new_empty();
        let scope = SearchScope::VisibleNpcs(crate::idgen::new_room_id());

        let err = find_item_match(&world, "coin", scope).unwrap_err();
        match err {
            SearchError::InvalidScope(kind, only) => {
                assert_eq!(kind, "item");
                assert_eq!(only, "NPC");
            },
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn find_npc_match_returns_visible_npc() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Garden");
        let npc_id = insert_npc(&mut world, "Caretaker", Location::Room(room_id.clone()));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id.clone());

        let result = find_npc_match(&world, "take", SearchScope::VisibleNpcs(room_id)).unwrap();
        assert_eq!(result, npc_id);
    }

    #[test]
    fn find_npc_match_errors_when_room_missing() {
        let world = AmbleWorld::new_empty();
        let room_id = crate::idgen::new_room_id();

        let err = find_npc_match(&world, "caretaker", SearchScope::VisibleNpcs(room_id.clone())).unwrap_err();
        match err {
            SearchError::InvalidRoomId(id) => assert_eq!(id, room_id),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn find_npc_match_rejects_item_scopes() {
        let world = AmbleWorld::new_empty();

        let err = find_npc_match(&world, "caretaker", SearchScope::Inventory).unwrap_err();
        match err {
            SearchError::InvalidScope(kind, only) => {
                assert_eq!(kind, "npc");
                assert_eq!(only, "item");
            },
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn find_entity_match_prefers_items() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Vault");
        let vessel_id = insert_item(
            &mut world,
            "Guardian Chest",
            Location::Room(room_id.clone()),
            Some(ContainerState::Open),
        );
        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .contents
            .insert(vessel_id.clone());
        let npc_id = insert_npc(&mut world, "Guardian", Location::Room(room_id.clone()));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id.clone());

        let result = find_entity_match(&world, "guardian", SearchScope::NearbyVessels(room_id)).unwrap();
        assert_eq!(result, EntityId::Item(vessel_id));
    }

    #[test]
    fn find_entity_match_returns_npc_when_no_item_found() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Library");
        let npc_id = insert_npc(&mut world, "Archivist", Location::Room(room_id.clone()));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id.clone());

        let result = find_entity_match(&world, "archivist", SearchScope::VisibleNpcs(room_id)).unwrap();
        assert_eq!(result, EntityId::Npc(npc_id));
    }

    #[test]
    fn find_entity_match_returns_no_matching_name_when_none_found() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Workshop");
        let vessel_id = insert_item(
            &mut world,
            "Toolbox",
            Location::Room(room_id.clone()),
            Some(ContainerState::Open),
        );
        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .contents
            .insert(vessel_id.clone());
        let npc_id = insert_npc(&mut world, "Mechanic", Location::Room(room_id.clone()));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id.clone());

        let err = find_entity_match(&world, "lantern", SearchScope::NearbyVessels(room_id)).unwrap_err();
        match err {
            SearchError::NoMatchingName(term) => assert_eq!(term, "lantern"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
