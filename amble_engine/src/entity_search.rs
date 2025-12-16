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

use std::collections::HashSet;

use thiserror::Error;
use uuid::Uuid;

use crate::{
    AmbleWorld,
    repl::find_world_object,
    world::{nearby_reachable_items, nearby_vessel_items, nearby_visible_items},
};

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
    let Some(entity) = find_world_object(&haystack, &world.items, &world.npcs, pattern) else {
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
