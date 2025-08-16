//! Data structures representing the game world.
//!
//! This module defines [`AmbleWorld`] and related types used at runtime to
//! track the current state of the adventure.

use crate::AMBLE_VERSION;
use crate::item::ContainerState;
use crate::npc::Npc;
use crate::spinners::SpinnerType;
use crate::trigger::Trigger;
use crate::{Goal, Item, Player, Room};

use anyhow::{Context, Result, anyhow};
use gametools::Spinner;
use log::info;
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use variantly::Variantly;

/// Kinds of places where a `WorldObject` may be located.
/// Because Rooms *are* the locations, their location is always `Nowhere`
/// Unspawned/despawned items and NPCs are also located `Nowhere`
#[derive(Copy, Debug, Default, Clone, Serialize, Deserialize, Variantly, PartialEq, Eq)]
pub enum Location {
    Item(Uuid),
    Inventory,
    #[default]
    Nowhere,
    Npc(Uuid),
    Room(Uuid),
}

/// Methods common to any object in the world.
pub trait WorldObject {
    fn id(&self) -> Uuid;
    fn symbol(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn location(&self) -> &Location;
}

/// Complete state of the running game.
///
/// `AmbleWorld` contains every room, item, NPC and trigger currently active, as
/// well as the player character. It is created during loading and then mutated
/// throughout gameplay.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AmbleWorld {
    pub rooms: HashMap<Uuid, Room>,
    pub items: HashMap<Uuid, Item>,
    pub triggers: Vec<Trigger>,
    pub player: Player,
    pub spinners: HashMap<SpinnerType, Spinner<String>>,
    pub npcs: HashMap<Uuid, Npc>,
    pub max_score: usize,
    pub goals: Vec<Goal>,
    pub version: String,
}
impl AmbleWorld {
    /// Create a new empty world with a default player.
    pub fn new_empty() -> AmbleWorld {
        let world = Self {
            rooms: HashMap::new(),
            npcs: HashMap::new(),
            items: HashMap::new(),
            triggers: Vec::new(),
            player: Player::default(),
            spinners: HashMap::new(),
            max_score: 0,
            goals: Vec::new(),
            version: AMBLE_VERSION.to_string(),
        };
        info!("new, empty 'AmbleWorld' created");
        world
    }

    /// Returns a random string (&'static str) from the selected spinner type, or a supplied default.
    pub fn spin_spinner(&self, spin_type: SpinnerType, default: &'static str) -> String {
        self.spinners
            .get(&spin_type)
            .and_then(gametools::Spinner::spin)
            .unwrap_or(default.to_string())
    }

    /// Obtain a reference to the room the player occupies.
    /// # Errors
    /// - if player isn't in a Room or the Room's uuid is not found
    pub fn player_room_ref(&self) -> Result<&Room> {
        match self.player.location {
            Location::Room(uuid) => self
                .rooms
                .get(&uuid)
                .ok_or_else(|| anyhow!("player's room UUID ({}) not found in world", uuid)),
            _ => Err(anyhow!("player not in a room - located at {:?}", self.player.location)),
        }
    }

    /// Obtain a mutable reference to the room the player occupies.
    /// # Errors
    /// - if player is not in a room or room's UUID is not found
    pub fn player_room_mut(&mut self) -> Result<&mut Room> {
        match self.player.location {
            Location::Room(uuid) => self
                .rooms
                .get_mut(&uuid)
                .ok_or_else(|| anyhow!("player's room UUID ({}) not found in world", uuid)),
            _ => Err(anyhow!("player not in a room - located at {:?}", self.player.location)),
        }
    }

    /// Get mutable reference to a world item.
    pub fn get_item_mut(&mut self, item_id: Uuid) -> Option<&mut Item> {
        self.items.get_mut(&item_id)
    }
}

/// Constructs a set of all potentially take-able / viewable item (uuids) in a room.
/// Non-portable or restricted items not filtered here -- player discovers
/// that on their own. The scope includes items in room, and items in open containers.
/// Items in closed or locked containers and NPCs are excluded.
///
/// # Errors
/// - if supplied `room_id` is invalid
pub fn nearby_reachable_items(world: &AmbleWorld, room_id: Uuid) -> Result<HashSet<Uuid>> {
    let current_room = world
        .rooms
        .get(&room_id)
        .with_context(|| format!("{room_id} room id not found"))?;
    let room_items = &current_room.contents;
    let mut contained_items = HashSet::new();
    for item_id in room_items {
        if let Some(item) = world.items.get(item_id)
            && item.container_state == Some(ContainerState::Open)
        {
            contained_items.extend(&item.contents);
        }
    }
    Ok(room_items.union(&contained_items).copied().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item, ItemAbility},
        npc::{Npc, NpcState},
        player::{Flag, Player},
        room::Room,
        spinners::SpinnerType,
    };
    use gametools::{Spinner, Wedge};
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn create_test_item(id: Uuid, location: Location) -> Item {
        Item {
            id,
            symbol: format!("item_{}", id.simple()),
            name: format!("Item {}", id.simple()),
            description: "A test item".into(),
            location,
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        }
    }

    fn create_test_room(id: Uuid) -> Room {
        Room {
            id,
            symbol: format!("room_{}", id.simple()),
            name: format!("Room {}", id.simple()),
            base_description: "A test room".into(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        }
    }

    fn create_test_npc(id: Uuid, location: Location) -> Npc {
        Npc {
            id,
            symbol: format!("npc_{}", id.simple()),
            name: format!("NPC {}", id.simple()),
            description: "A test NPC".into(),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: NpcState::Normal,
        }
    }

    #[test]
    fn location_variants_work() {
        let item_id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        let npc_id = Uuid::new_v4();

        assert_eq!(Location::Item(item_id), Location::Item(item_id));
        assert_eq!(Location::Room(room_id), Location::Room(room_id));
        assert_eq!(Location::Npc(npc_id), Location::Npc(npc_id));
        assert_eq!(Location::Inventory, Location::Inventory);
        assert_eq!(Location::Nowhere, Location::Nowhere);

        assert_ne!(Location::Inventory, Location::Nowhere);
        assert_ne!(Location::Room(room_id), Location::Item(item_id));
    }

    #[test]
    fn location_default_is_nowhere() {
        assert_eq!(Location::default(), Location::Nowhere);
    }

    #[test]
    fn location_is_nowhere_works() {
        assert!(Location::Nowhere.is_nowhere());
        assert!(!Location::Inventory.is_nowhere());
        assert!(!Location::Room(Uuid::new_v4()).is_nowhere());
    }

    #[test]
    fn location_is_not_nowhere_works() {
        assert!(!Location::Nowhere.is_not_nowhere());
        assert!(Location::Inventory.is_not_nowhere());
        assert!(Location::Room(Uuid::new_v4()).is_not_nowhere());
    }

    #[test]
    fn location_unwrap_room_works() {
        let room_id = Uuid::new_v4();
        let location = Location::Room(room_id);
        assert_eq!(location.unwrap_room(), room_id);
    }

    #[test]
    #[should_panic]
    fn location_unwrap_room_panics_on_non_room() {
        Location::Inventory.unwrap_room();
    }

    #[test]
    fn location_room_ref_works() {
        let room_id = Uuid::new_v4();
        let location = Location::Room(room_id);
        assert_eq!(location.room_ref(), Some(&room_id));

        assert_eq!(Location::Inventory.room_ref(), None);
        assert_eq!(Location::Nowhere.room_ref(), None);
    }

    #[test]
    fn amble_world_new_empty_creates_valid_world() {
        let world = AmbleWorld::new_empty();

        assert!(world.rooms.is_empty());
        assert!(world.items.is_empty());
        assert!(world.triggers.is_empty());
        assert!(world.npcs.is_empty());
        assert!(world.goals.is_empty());
        assert!(world.spinners.is_empty());
        assert_eq!(world.max_score, 0);
        assert_eq!(world.version, crate::AMBLE_VERSION);
        assert_eq!(world.player.name, "The Candidate");
    }

    #[test]
    fn amble_world_spin_spinner_returns_result_or_default() {
        let mut world = AmbleWorld::new_empty();

        // Test with no spinner
        let result = world.spin_spinner(SpinnerType::Movement, "default");
        assert_eq!(result, "default");

        // Test with spinner
        let spinner = Spinner::new(vec![Wedge::new("custom result".into())]);
        world.spinners.insert(SpinnerType::Movement, spinner);

        let result = world.spin_spinner(SpinnerType::Movement, "default");
        assert_eq!(result, "custom result");
    }

    #[test]
    fn amble_world_player_room_ref_works() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let room = create_test_room(room_id);
        world.rooms.insert(room_id, room);
        world.player.location = Location::Room(room_id);

        let room_ref = world.player_room_ref().unwrap();
        assert_eq!(room_ref.id, room_id);
    }

    #[test]
    fn amble_world_player_room_ref_errors_when_not_in_room() {
        let world = AmbleWorld::new_empty();
        // Player defaults to Room location but room doesn't exist
        assert!(world.player_room_ref().is_err());
    }

    #[test]
    fn amble_world_player_room_ref_errors_when_player_in_inventory() {
        let mut world = AmbleWorld::new_empty();
        world.player.location = Location::Inventory;
        assert!(world.player_room_ref().is_err());
    }

    #[test]
    fn amble_world_player_room_mut_works() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let room = create_test_room(room_id);
        world.rooms.insert(room_id, room);
        world.player.location = Location::Room(room_id);

        let room_mut = world.player_room_mut().unwrap();
        room_mut.visited = true;
        assert!(world.rooms.get(&room_id).unwrap().visited);
    }

    #[test]
    fn amble_world_player_room_mut_errors_when_not_in_room() {
        let mut world = AmbleWorld::new_empty();
        world.player.location = Location::Inventory;
        assert!(world.player_room_mut().is_err());
    }

    #[test]
    fn amble_world_get_item_mut_works() {
        let mut world = AmbleWorld::new_empty();
        let item_id = Uuid::new_v4();
        let item = create_test_item(item_id, Location::Nowhere);
        world.items.insert(item_id, item);

        let item_mut = world.get_item_mut(item_id).unwrap();
        item_mut.restricted = true;
        assert!(world.items.get(&item_id).unwrap().restricted);
    }

    #[test]
    fn amble_world_get_item_mut_returns_none_for_nonexistent() {
        let mut world = AmbleWorld::new_empty();
        assert!(world.get_item_mut(Uuid::new_v4()).is_none());
    }

    #[test]
    fn nearby_reachable_items_includes_room_items() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let mut room = create_test_room(room_id);

        let item_id = Uuid::new_v4();
        let item = create_test_item(item_id, Location::Room(room_id));
        room.contents.insert(item_id);

        world.rooms.insert(room_id, room);
        world.items.insert(item_id, item);

        let reachable = nearby_reachable_items(&world, room_id).unwrap();
        assert!(reachable.contains(&item_id));
    }

    #[test]
    fn nearby_reachable_items_includes_open_container_contents() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let mut room = create_test_room(room_id);

        let container_id = Uuid::new_v4();
        let mut container = create_test_item(container_id, Location::Room(room_id));
        container.container_state = Some(ContainerState::Open);

        let item_in_container_id = Uuid::new_v4();
        let item_in_container = create_test_item(item_in_container_id, Location::Item(container_id));
        container.contents.insert(item_in_container_id);

        room.contents.insert(container_id);

        world.rooms.insert(room_id, room);
        world.items.insert(container_id, container);
        world.items.insert(item_in_container_id, item_in_container);

        let reachable = nearby_reachable_items(&world, room_id).unwrap();
        assert!(reachable.contains(&container_id));
        assert!(reachable.contains(&item_in_container_id));
    }

    #[test]
    fn nearby_reachable_items_excludes_closed_container_contents() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let mut room = create_test_room(room_id);

        let container_id = Uuid::new_v4();
        let mut container = create_test_item(container_id, Location::Room(room_id));
        container.container_state = Some(ContainerState::Closed);

        let item_in_container_id = Uuid::new_v4();
        let item_in_container = create_test_item(item_in_container_id, Location::Item(container_id));
        container.contents.insert(item_in_container_id);

        room.contents.insert(container_id);

        world.rooms.insert(room_id, room);
        world.items.insert(container_id, container);
        world.items.insert(item_in_container_id, item_in_container);

        let reachable = nearby_reachable_items(&world, room_id).unwrap();
        assert!(reachable.contains(&container_id));
        assert!(!reachable.contains(&item_in_container_id));
    }

    #[test]
    fn nearby_reachable_items_excludes_locked_container_contents() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let mut room = create_test_room(room_id);

        let container_id = Uuid::new_v4();
        let mut container = create_test_item(container_id, Location::Room(room_id));
        container.container_state = Some(ContainerState::Locked);

        let item_in_container_id = Uuid::new_v4();
        let item_in_container = create_test_item(item_in_container_id, Location::Item(container_id));
        container.contents.insert(item_in_container_id);

        room.contents.insert(container_id);

        world.rooms.insert(room_id, room);
        world.items.insert(container_id, container);
        world.items.insert(item_in_container_id, item_in_container);

        let reachable = nearby_reachable_items(&world, room_id).unwrap();
        assert!(reachable.contains(&container_id));
        assert!(!reachable.contains(&item_in_container_id));
    }

    #[test]
    fn nearby_reachable_items_errors_for_invalid_room() {
        let world = AmbleWorld::new_empty();
        let result = nearby_reachable_items(&world, Uuid::new_v4());
        assert!(result.is_err());
    }

    #[test]
    fn nearby_reachable_items_handles_empty_room() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let room = create_test_room(room_id);
        world.rooms.insert(room_id, room);

        let reachable = nearby_reachable_items(&world, room_id).unwrap();
        assert!(reachable.is_empty());
    }

    #[test]
    fn nearby_reachable_items_handles_non_container_items() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let mut room = create_test_room(room_id);

        let item_id = Uuid::new_v4();
        let item = create_test_item(item_id, Location::Room(room_id));
        room.contents.insert(item_id);

        world.rooms.insert(room_id, room);
        world.items.insert(item_id, item);

        let reachable = nearby_reachable_items(&world, room_id).unwrap();
        assert_eq!(reachable.len(), 1);
        assert!(reachable.contains(&item_id));
    }

    #[test]
    fn world_object_trait_implemented_for_player() {
        let player = Player::default();
        assert!(!player.id().is_nil());
        assert_eq!(player.symbol(), "the_candidate");
        assert_eq!(player.name(), "The Candidate");
        assert_eq!(player.description(), "default");
        assert_eq!(player.location(), &Location::default());
    }

    #[test]
    fn amble_world_serialization_includes_version() {
        let world = AmbleWorld::new_empty();
        assert_eq!(world.version, crate::AMBLE_VERSION);
    }
}
