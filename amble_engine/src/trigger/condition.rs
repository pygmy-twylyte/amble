//! condition.rs -- `TriggerCondition` Module
//!
//! Implements various player actions and game state that can be detected
//! by a Trigger, resulting in some `TriggerActions` firing.

use std::collections::HashSet;

use rand::random_bool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AmbleWorld, ItemHolder, Location,
    item::{ItemAbility, ItemInteractionType},
    npc::NpcState,
    player::Flag,
    spinners::SpinnerType,
};

/// Game states and player actions that can be detected by a `Trigger`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerCondition {
    ActOnItem {
        target_id: Uuid,
        action: ItemInteractionType,
    },
    Ambient {
        room_ids: HashSet<Uuid>, // empty = applies everywhere
        spinner: SpinnerType,
    },
    Chance {
        one_in: f64,
    },
    ContainerHasItem {
        container_id: Uuid,
        item_id: Uuid,
    },
    Drop(Uuid),
    Enter(Uuid),
    GiveToNpc {
        item_id: Uuid,
        npc_id: Uuid,
    },
    HasItem(Uuid),
    HasFlag(String),
    FlagInProgress(String),
    FlagComplete(String),
    HasVisited(Uuid),
    InRoom(Uuid),
    Insert {
        item: Uuid,
        container: Uuid,
    },
    Leave(Uuid),
    LookAt(Uuid),
    MissingFlag(String),
    MissingItem(Uuid),
    NpcHasItem {
        npc_id: Uuid,
        item_id: Uuid,
    },
    NpcInState {
        npc_id: Uuid,
        mood: NpcState,
    },
    Open(Uuid),
    Take(Uuid),
    TakeFromNpc {
        item_id: Uuid,
        npc_id: Uuid,
    },
    TalkToNpc(Uuid),
    UseItem {
        item_id: Uuid,
        ability: ItemAbility,
    },
    UseItemOnItem {
        interaction: ItemInteractionType,
        target_id: Uuid,
        tool_id: Uuid,
    },
    Unlock(Uuid),
    WithNpc(Uuid),
}

impl TriggerCondition {
    pub fn matches_event_in(&self, events: &[TriggerCondition]) -> bool {
        events.contains(self)
    }

    /// Returns a random boolean according the parameters of a Chance trigger.
    ///
    /// This allows us to check chance conditions without having to pass an AmbleWorld
    /// reference, avoid some conflicts with the borrow checker. Returns true if called
    /// on a any other type of TriggerCondition.
    pub fn chance_value(&self) -> bool {
        match self {
            Self::Chance { one_in } => random_bool(1.0 / *one_in),
            _ => true,
        }
    }

    /// Returns true if a state check is true (or if the Chance trigger returns true in that case.)
    pub fn is_ongoing(&self, world: &AmbleWorld) -> bool {
        let player_flag_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match self {
            Self::Chance { one_in } => random_bool(1.0 / *one_in),
            Self::ContainerHasItem { container_id, item_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| item.location == Location::Item(*container_id)),
            Self::HasFlag(flag) => player_flag_set(flag),
            Self::MissingFlag(flag) => !player_flag_set(flag),
            Self::FlagInProgress(flag) => world
                .player
                .flags
                .get(&Flag::Simple {
                    name: flag.into(),
                    turn_set: usize::MAX, /* dummy - not used in hash */
                })
                .is_some_and(|f| !f.is_complete()),
            Self::FlagComplete(flag) => world
                .player
                .flags
                .get(&Flag::Simple {
                    name: flag.into(),
                    turn_set: usize::MAX,
                })
                .is_some_and(Flag::is_complete),
            Self::HasVisited(room_id) => world.rooms.get(room_id).is_some_and(|r| r.visited),
            Self::InRoom(room_id) => world.player.location.room_id().map_or(false, |id| *room_id == id),
            Self::NpcHasItem { npc_id, item_id } => {
                world.npcs.get(npc_id).is_some_and(|npc| npc.contains_item(*item_id))
            },
            Self::NpcInState { npc_id, mood } => world.npcs.get(npc_id).is_some_and(|npc| npc.state == *mood),
            Self::HasItem(item_id) => world.player.contains_item(*item_id),
            Self::MissingItem(item_id) => !world.player.contains_item(*item_id),
            Self::WithNpc(npc_id) => world
                .npcs
                .get(npc_id)
                .is_some_and(|npc| npc.location == world.player.location),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item},
        npc::{Npc, NpcState},
        player::Flag,
        room::Room,
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn build_test_world() -> (AmbleWorld, Uuid, Uuid) {
        let mut world = AmbleWorld::new_empty();
        let room1_id = Uuid::new_v4();
        let room2_id = Uuid::new_v4();

        let room1 = Room {
            id: room1_id,
            symbol: "r1".into(),
            name: "Room1".into(),
            base_description: "Room1".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        let room2 = Room {
            id: room2_id,
            symbol: "r2".into(),
            name: "Room2".into(),
            base_description: "Room2".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room1_id, room1);
        world.rooms.insert(room2_id, room2);
        world.player.location = Location::Room(room1_id);
        (world, room1_id, room2_id)
    }

    fn make_item(id: Uuid, location: Location, container_state: Option<ContainerState>) -> Item {
        Item {
            id,
            symbol: "it".into(),
            name: "Item".into(),
            description: "".into(),
            location,
            portable: true,
            container_state,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        }
    }

    fn make_npc(id: Uuid, location: Location, state: NpcState) -> Npc {
        Npc {
            id,
            symbol: "n".into(),
            name: "Npc".into(),
            description: "".into(),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state,
            movement: None,
        }
    }

    #[test]
    fn matches_event_in_detects_matching_event() {
        let (_, room1_id, room2_id) = build_test_world();
        let events = vec![TriggerCondition::Enter(room1_id)];
        assert!(TriggerCondition::Enter(room1_id).matches_event_in(&events));
        assert!(!TriggerCondition::Enter(room2_id).matches_event_in(&events));
    }

    #[test]
    fn look_at_condition_matches_event() {
        let item_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let events = vec![TriggerCondition::LookAt(item_id)];
        assert!(TriggerCondition::LookAt(item_id).matches_event_in(&events));
        assert!(!TriggerCondition::LookAt(other_id).matches_event_in(&events));
    }

    #[test]
    fn look_at_condition_is_not_ongoing() {
        let (world, _, _) = build_test_world();
        let item_id = Uuid::new_v4();
        assert!(!TriggerCondition::LookAt(item_id).is_ongoing(&world));
    }

    #[test]
    fn is_ongoing_detects_player_location() {
        let (world, room1_id, room2_id) = build_test_world();
        assert!(TriggerCondition::InRoom(room1_id).is_ongoing(&world));
        assert!(!TriggerCondition::InRoom(room2_id).is_ongoing(&world));
    }

    #[test]
    fn flag_conditions_reflect_player_flags() {
        let mut world = build_test_world().0;
        world.player.flags.insert(Flag::simple("a", world.turn_count));
        assert!(TriggerCondition::HasFlag("a".into()).is_ongoing(&world));
        assert!(!TriggerCondition::MissingFlag("a".into()).is_ongoing(&world));
        assert!(TriggerCondition::MissingFlag("b".into()).is_ongoing(&world));
    }

    #[test]
    fn sequence_flag_progress_and_complete() {
        let mut world = build_test_world().0;
        world
            .player
            .flags
            .insert(Flag::sequence("quest", Some(2), world.turn_count));
        world.player.advance_flag("quest");
        assert!(TriggerCondition::FlagInProgress("quest".into()).is_ongoing(&world));
        world.player.advance_flag("quest");
        assert!(TriggerCondition::FlagComplete("quest".into()).is_ongoing(&world));
    }

    #[test]
    fn has_visited_detects_room_visits() {
        let (mut world, room1_id, room2_id) = build_test_world();
        world.rooms.get_mut(&room1_id).unwrap().visited = true;
        assert!(TriggerCondition::HasVisited(room1_id).is_ongoing(&world));
        assert!(!TriggerCondition::HasVisited(room2_id).is_ongoing(&world));
    }

    #[test]
    fn npc_item_and_state_conditions() {
        let (mut world, room1_id, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let mut npc = make_npc(npc_id, Location::Room(room1_id), NpcState::Happy);
        npc.inventory.insert(item_id);
        world.npcs.insert(npc_id, npc);
        world
            .items
            .insert(item_id, make_item(item_id, Location::Npc(npc_id), None));
        assert!(TriggerCondition::NpcHasItem { npc_id, item_id }.is_ongoing(&world));
        assert!(
            TriggerCondition::NpcInState {
                npc_id,
                mood: NpcState::Happy
            }
            .is_ongoing(&world)
        );
        assert!(
            !TriggerCondition::NpcInState {
                npc_id,
                mood: NpcState::Mad
            }
            .is_ongoing(&world)
        );
    }

    #[test]
    fn player_inventory_item_conditions() {
        let (mut world, _, _) = build_test_world();
        let item_id = Uuid::new_v4();
        world
            .items
            .insert(item_id, make_item(item_id, Location::Inventory, None));
        world.player.inventory.insert(item_id);
        assert!(TriggerCondition::HasItem(item_id).is_ongoing(&world));
        assert!(!TriggerCondition::MissingItem(item_id).is_ongoing(&world));
        let other_id = Uuid::new_v4();
        world
            .items
            .insert(other_id, make_item(other_id, Location::Nowhere, None));
        assert!(TriggerCondition::MissingItem(other_id).is_ongoing(&world));
    }

    #[test]
    fn with_npc_condition_detects_presence() {
        let (mut world, room1_id, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        world.rooms.get_mut(&room1_id).unwrap().npcs.insert(npc_id);
        world
            .npcs
            .insert(npc_id, make_npc(npc_id, Location::Room(room1_id), NpcState::Normal));
        assert!(TriggerCondition::WithNpc(npc_id).is_ongoing(&world));
    }

    #[test]
    fn container_has_item_condition_detects_item() {
        let (mut world, room1_id, _) = build_test_world();
        let container_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let mut container = make_item(container_id, Location::Room(room1_id), Some(ContainerState::Open));
        container.contents.insert(item_id);
        world.items.insert(container_id, container);
        world.rooms.get_mut(&room1_id).unwrap().contents.insert(container_id);
        world
            .items
            .insert(item_id, make_item(item_id, Location::Item(container_id), None));
        assert!(TriggerCondition::ContainerHasItem { container_id, item_id }.is_ongoing(&world));
    }
}
