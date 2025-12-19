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
//! Caller should only need to send:
//! - an immutable AmbleWorld reference
//! - the search string
//! - the search scope
//!
//! Also, callers will generally need in return:
//! - the Uuid of the found entity, OR
//! - the reason that an entity's ID is not being returned (`SearchError`)
//!
//! TODO?: For now, we'll just duplicate search functionality currently existing. Ultimately, the search scopes could
//! become more task-specific and / or some validation steps may be moved here.

use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use thiserror::Error;
use uuid::Uuid;

use crate::{
    AmbleWorld, Item, Npc,
    repl::find_world_object,
    world::{nearby_reachable_items, nearby_vessel_items, nearby_visible_items},
};

/// Empty item map used in NPC-only searches.
pub static NO_ITEMS: OnceLock<HashMap<Uuid, Item>> = OnceLock::new();
/// Empty NPC map used in item-only searches.
pub static NO_NPCS: OnceLock<HashMap<Uuid, Npc>> = OnceLock::new();

/// Represents the scope of a requested search by the caller and includes the location to search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchScope {
    /// Items the player can look at.
    VisibleItems(Uuid),
    /// NPCs the player can see.
    VisibleNpcs(Uuid),
    /// Items the player can touch (but not necessarily move) in room or inventory
    TouchableItems(Uuid),
    /// NPCs the player can touch.
    TouchableNpcs(Uuid),
    /// Nearby container items or NPCs which can potentially offer or accept an item.
    NearbyVessels(Uuid),
    /// Only items in inventory.
    Inventory,
}

/// Possible errors / situations causing a failed entity search.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("no entity in scope name matching user input '{0}'")]
    NoMatchingName(String),
    #[error("found no room with the supplied UUID ({0})")]
    InvalidRoomId(Uuid),
    #[error("search for an item matching '{input}' found NPC '{npc_name}'")]
    MatchedNonItem { input: String, npc_name: String },
    #[error("search for an NPC matching '{input}' found item '{item_name}'")]
    MatchedNonNpc { input: String, item_name: String },
    #[error("invalid {0} search scope: includes only {1}s")]
    InvalidScope(String, String),
    #[error("unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

/// Find an `Item` with name matching `pattern` in the given `SearchScope` and return its id.
pub fn find_item_match(world: &AmbleWorld, pattern: &str, scope: SearchScope) -> Result<Uuid, SearchError> {
    // construct a HashSet of item UUIDs in scope for this search
    let haystack: HashSet<_> = match scope {
        SearchScope::VisibleItems(room_id) => {
            let room_items = nearby_visible_items(world, room_id).map_err(|_| SearchError::InvalidRoomId(room_id))?;
            room_items.union(&world.player.inventory).copied().collect()
        },
        SearchScope::TouchableItems(room_id) => {
            let room_items = nearby_reachable_items(world, room_id).map_err(|_| SearchError::InvalidRoomId(room_id))?;
            room_items.union(&world.player.inventory).copied().collect()
        },
        SearchScope::NearbyVessels(room_id) => {
            let room_items = nearby_vessel_items(world, room_id).map_err(|_| SearchError::InvalidRoomId(room_id))?;
            let current_room = world.rooms.get(&room_id).ok_or(SearchError::InvalidRoomId(room_id))?;
            room_items.union(&current_room.npcs).copied().collect()
        },
        SearchScope::Inventory => world.player.inventory.clone(),
        SearchScope::VisibleNpcs(_) | SearchScope::TouchableNpcs(_) => {
            return Err(SearchError::InvalidScope("item".to_string(), "NPC".to_string()));
        },
    };

    // find any world object in scope that matches the input pattern, return error if none found
    let no_npcs = NO_NPCS.get_or_init(|| HashMap::new());
    let Some(entity) = find_world_object(&haystack, &world.items, no_npcs, pattern) else {
        return Err(SearchError::NoMatchingName(pattern.to_string()));
    };

    // if the entity we found isn't an item somehow (though scope rules should prevent this), error
    if entity.is_not_item() {
        return Err(SearchError::MatchedNonItem {
            input: pattern.to_string(),
            npc_name: entity.name().to_string(),
        });
    }

    // entity is a matching item -- return its UUID
    Ok(entity.id())
}

/// Find an `Npc` with name matching `pattern` in the specified `SearchScope`.
pub fn find_npc_match(world: &AmbleWorld, pattern: &str, scope: SearchScope) -> Result<Uuid, SearchError> {
    let haystack = match scope {
        // currently there is no distinction between NPCs you can see and those you could touch
        // both scopes kept for now as this may change in the future
        SearchScope::VisibleNpcs(uuid) | SearchScope::TouchableNpcs(uuid) | SearchScope::NearbyVessels(uuid) => {
            let room = world.rooms.get(&uuid).ok_or(SearchError::InvalidRoomId(uuid))?;
            room.npcs.clone()
        },
        SearchScope::VisibleItems(_) | SearchScope::TouchableItems(_) | SearchScope::Inventory => {
            return Err(SearchError::InvalidScope("npc".into(), "item".into()));
        },
    };

    let no_items = NO_ITEMS.get_or_init(|| HashMap::new());
    let Some(entity) = find_world_object(&haystack, no_items, &world.npcs, pattern) else {
        return Err(SearchError::NoMatchingName(pattern.to_string()));
    };

    if entity.is_not_npc() {
        return Err(SearchError::MatchedNonNpc {
            input: pattern.to_owned(),
            item_name: entity.name().to_owned(),
        });
    }

    Ok(entity.id())
}

/// Find either an `NPC` or an `Item` with name matching `pattern` within the SearchScope.
pub fn find_entity_match(world: &AmbleWorld, pattern: &str, scope: SearchScope) -> Result<Uuid, SearchError> {
    // return any item matched first -- these will account for most searches
    let item_match = find_item_match(world, pattern, scope);
    if item_match.is_ok() {
        return item_match;
    }

    // less often looking for an NPC match -- return that now if found
    let npc_match = find_npc_match(world, pattern, scope);
    if npc_match.is_ok() {
        return npc_match;
    }

    Err(SearchError::NoMatchingName(pattern.to_string()))
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
    use uuid::Uuid;

    fn insert_room(world: &mut AmbleWorld, name: &str) -> Uuid {
        let room_id = Uuid::new_v4();
        let room = Room {
            id: room_id,
            symbol: format!("room_{}", room_id.simple()),
            name: name.to_string(),
            base_description: format!("{name} description"),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room_id, room);
        room_id
    }

    fn insert_item(
        world: &mut AmbleWorld,
        name: &str,
        location: Location,
        container_state: Option<ContainerState>,
    ) -> Uuid {
        let item_id = Uuid::new_v4();
        let item = Item {
            id: item_id,
            symbol: format!("item_{}", item_id.simple()),
            name: name.to_string(),
            description: format!("{name} item"),
            location,
            movability: Movability::Free,
            container_state,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(item_id, item);
        item_id
    }

    fn insert_npc(world: &mut AmbleWorld, name: &str, location: Location) -> Uuid {
        let npc_id = Uuid::new_v4();
        let npc = Npc {
            id: npc_id,
            symbol: format!("npc_{}", npc_id.simple()),
            name: name.to_string(),
            description: format!("{name} npc"),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: NpcState::Normal,
            movement: None,
            health: HealthState::new(),
        };
        world.npcs.insert(npc_id, npc);
        npc_id
    }

    #[test]
    fn find_item_match_includes_inventory_items_in_visible_scope() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Atrium");
        let coin_id = insert_item(&mut world, "Lucky Coin", Location::Inventory, None);
        world.player.inventory.insert(coin_id);

        let result = find_item_match(&world, "coin", SearchScope::VisibleItems(room_id)).unwrap();
        assert_eq!(result, coin_id);
    }

    #[test]
    fn find_item_match_returns_nearby_vessels() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Vault");
        let chest_id = insert_item(&mut world, "Ancient Chest", Location::Room(room_id), Some(ContainerState::Open));
        world.rooms.get_mut(&room_id).unwrap().contents.insert(chest_id);

        let result = find_item_match(&world, "chest", SearchScope::NearbyVessels(room_id)).unwrap();
        assert_eq!(result, chest_id);
    }

    #[test]
    fn find_item_match_errors_when_room_missing() {
        let world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();

        let err = find_item_match(&world, "coin", SearchScope::VisibleItems(room_id)).unwrap_err();
        match err {
            SearchError::InvalidRoomId(id) => assert_eq!(id, room_id),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn find_item_match_rejects_npc_scopes() {
        let world = AmbleWorld::new_empty();
        let scope = SearchScope::VisibleNpcs(Uuid::new_v4());

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
        let npc_id = insert_npc(&mut world, "Caretaker", Location::Room(room_id));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let result = find_npc_match(&world, "take", SearchScope::VisibleNpcs(room_id)).unwrap();
        assert_eq!(result, npc_id);
    }

    #[test]
    fn find_npc_match_errors_when_room_missing() {
        let world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();

        let err = find_npc_match(&world, "caretaker", SearchScope::VisibleNpcs(room_id)).unwrap_err();
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
        let vessel_id = insert_item(&mut world, "Guardian Chest", Location::Room(room_id), Some(ContainerState::Open));
        world.rooms.get_mut(&room_id).unwrap().contents.insert(vessel_id);
        let npc_id = insert_npc(&mut world, "Guardian", Location::Room(room_id));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let result = find_entity_match(&world, "guardian", SearchScope::NearbyVessels(room_id)).unwrap();
        assert_eq!(result, vessel_id);
    }

    #[test]
    fn find_entity_match_returns_npc_when_no_item_found() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Library");
        let npc_id = insert_npc(&mut world, "Archivist", Location::Room(room_id));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let result = find_entity_match(&world, "archivist", SearchScope::VisibleNpcs(room_id)).unwrap();
        assert_eq!(result, npc_id);
    }

    #[test]
    fn find_entity_match_returns_no_matching_name_when_none_found() {
        let mut world = AmbleWorld::new_empty();
        let room_id = insert_room(&mut world, "Workshop");
        let vessel_id = insert_item(&mut world, "Toolbox", Location::Room(room_id), Some(ContainerState::Open));
        world.rooms.get_mut(&room_id).unwrap().contents.insert(vessel_id);
        let npc_id = insert_npc(&mut world, "Mechanic", Location::Room(room_id));
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let err = find_entity_match(&world, "lantern", SearchScope::NearbyVessels(room_id)).unwrap_err();
        match err {
            SearchError::NoMatchingName(term) => assert_eq!(term, "lantern"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
