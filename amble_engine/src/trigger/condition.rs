//! Trigger condition evaluation.
//!
//! Defines the detectable game events and state predicates that drive the
//! trigger system, plus helpers for matching them against world state.

use std::collections::HashSet;

use crate::Id;
use rand::random_bool;
use serde::{Deserialize, Serialize};

use crate::{
    AmbleWorld, ItemHolder, Location,
    item::{IngestMode, ItemAbility, ItemInteractionType},
    npc::NpcState,
    player::Flag,
    spinners::SpinnerType,
};

/// Game states and player actions that can be detected by a `Trigger`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerCondition {
    ActOnItem {
        target_id: Id,
        action: ItemInteractionType,
    },
    Ambient {
        room_ids: HashSet<Id>, // empty = applies everywhere
        spinner: SpinnerType,
    },
    Chance {
        one_in: f64,
    },
    ContainerHasItem {
        container_id: Id,
        item_id: Id,
    },
    Drop(Id),
    Enter(Id),
    GiveToNpc {
        item_id: Id,
        npc_id: Id,
    },
    HasItem(Id),
    HasFlag(String),
    FlagInProgress(String),
    FlagComplete(String),
    HasVisited(Id),
    InRoom(Id),
    Ingest {
        item_id: Id,
        mode: IngestMode,
    },
    PlayerDeath,
    Insert {
        item: Id,
        container: Id,
    },
    Leave(Id),
    LookAt(Id),
    MissingFlag(String),
    MissingItem(Id),
    NpcDeath(Id),
    NpcHasItem {
        npc_id: Id,
        item_id: Id,
    },
    NpcInState {
        npc_id: Id,
        mood: NpcState,
    },
    Open(Id),
    Take(Id),
    TakeFromNpc {
        item_id: Id,
        npc_id: Id,
    },
    TalkToNpc(Id),
    Touch(Id),
    UseItem {
        item_id: Id,
        ability: ItemAbility,
    },
    UseItemOnItem {
        interaction: ItemInteractionType,
        target_id: Id,
        tool_id: Id,
    },
    Unlock(Id),
    WithNpc(Id),
}

impl TriggerCondition {
    /// Check whether this condition appears in a list of fired events.
    pub fn matches_event_in(&self, events: &[TriggerCondition]) -> bool {
        events.contains(self)
    }

    /// Returns a random boolean according the parameters of a Chance trigger.
    ///
    /// This allows us to check chance conditions without having to pass an `AmbleWorld`
    /// reference, avoid some conflicts with the borrow checker. Returns true if called
    /// on any other type of `TriggerCondition`.
    pub fn chance_value(&self) -> bool {
        match self {
            Self::Chance { one_in } => random_bool(1.0 / *one_in),
            _ => true,
        }
    }

    /// Evaluate non-event-driven conditions against the live world state.
    ///
    /// This covers ongoing predicates such as flags, inventory membership,
    /// and NPC states. For chance triggers it performs the random roll.
    pub fn is_ongoing(&self, world: &AmbleWorld) -> bool {
        let player_flag_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match self {
            Self::Chance { one_in } => random_bool(1.0 / *one_in),
            Self::ContainerHasItem { container_id, item_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| matches!(&item.location, Location::Item(id) if id == container_id)),
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
            Self::InRoom(room_id) => world.player.location.room_id().is_ok_and(|id| room_id == &id),
            Self::NpcHasItem { npc_id, item_id } => world
                .npcs
                .get(npc_id)
                .is_some_and(|npc| npc.contains_item(item_id.clone())),
            Self::NpcInState { npc_id, mood } => world.npcs.get(npc_id).is_some_and(|npc| npc.state == *mood),
            Self::HasItem(item_id) => world.player.contains_item(item_id.clone()),
            Self::MissingItem(item_id) => !world.player.contains_item(item_id.clone()),
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
    use crate::Id;
    use crate::{
        health::HealthState,
        item::{ContainerState, Item},
        npc::{Npc, NpcState},
        player::Flag,
        room::Room,
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};

    fn build_test_world() -> (AmbleWorld, Id, Id) {
        let mut world = AmbleWorld::new_empty();
        let room1_id = crate::idgen::new_id();
        let room2_id = crate::idgen::new_id();

        let room1 = Room {
            id: room1_id.clone(),
            symbol: "r1".into(),
            name: "Room1".into(),
            base_description: "Room1".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        let room2 = Room {
            id: room2_id.clone(),
            symbol: "r2".into(),
            name: "Room2".into(),
            base_description: "Room2".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room1_id.clone(), room1);
        world.rooms.insert(room2_id.clone(), room2);
        world.player.location = Location::Room(room1_id.clone());
        (world, room1_id, room2_id)
    }

    fn make_item(id: Id, location: Location, container_state: Option<ContainerState>) -> Item {
        Item {
            id,
            symbol: "it".into(),
            name: "Item".into(),
            description: String::new(),
            location,
            container_state,
            visibility: crate::item::ItemVisibility::Listed,
            visible_when: None,
            aliases: Vec::new(),
            movability: crate::item::Movability::Free,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        }
    }

    fn make_npc(id: Id, location: Location, state: NpcState) -> Npc {
        Npc {
            id,
            symbol: "n".into(),
            name: "Npc".into(),
            description: String::new(),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state,
            movement: None,
            health: HealthState::new_at_max(20),
        }
    }

    #[test]
    fn matches_event_in_detects_matching_event() {
        let (_, room1_id, room2_id) = build_test_world();
        let events = vec![TriggerCondition::Enter(room1_id.clone())];
        assert!(TriggerCondition::Enter(room1_id.clone()).matches_event_in(&events));
        assert!(!TriggerCondition::Enter(room2_id).matches_event_in(&events));
    }

    #[test]
    fn look_at_condition_matches_event() {
        let item_id = crate::idgen::new_id();
        let other_id = crate::idgen::new_id();
        let events = vec![TriggerCondition::LookAt(item_id.clone())];
        assert!(TriggerCondition::LookAt(item_id.clone()).matches_event_in(&events));
        assert!(!TriggerCondition::LookAt(other_id).matches_event_in(&events));
    }

    #[test]
    fn look_at_condition_is_not_ongoing() {
        let (world, _, _) = build_test_world();
        let item_id = crate::idgen::new_id();
        assert!(!TriggerCondition::LookAt(item_id).is_ongoing(&world));
    }

    #[test]
    fn is_ongoing_detects_player_location() {
        let (world, room1_id, room2_id) = build_test_world();
        assert!(TriggerCondition::InRoom(room1_id.clone()).is_ongoing(&world));
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
        assert!(TriggerCondition::HasVisited(room1_id.clone()).is_ongoing(&world));
        assert!(!TriggerCondition::HasVisited(room2_id).is_ongoing(&world));
    }

    #[test]
    fn npc_item_and_state_conditions() {
        let (mut world, room1_id, _) = build_test_world();
        let npc_id = crate::idgen::new_id();
        let item_id = crate::idgen::new_id();
        let mut npc = make_npc(npc_id.clone(), Location::Room(room1_id.clone()), NpcState::Happy);
        npc.inventory.insert(item_id.clone());
        world.npcs.insert(npc_id.clone(), npc);
        world.items.insert(
            item_id.clone(),
            make_item(item_id.clone(), Location::Npc(npc_id.clone()), None),
        );
        assert!(
            TriggerCondition::NpcHasItem {
                npc_id: npc_id.clone(),
                item_id: item_id.clone()
            }
            .is_ongoing(&world)
        );
        assert!(
            TriggerCondition::NpcInState {
                npc_id: npc_id.clone(),
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
        let item_id = crate::idgen::new_id();
        world
            .items
            .insert(item_id.clone(), make_item(item_id.clone(), Location::Inventory, None));
        world.player.inventory.insert(item_id.clone());
        assert!(TriggerCondition::HasItem(item_id.clone()).is_ongoing(&world));
        assert!(!TriggerCondition::MissingItem(item_id.clone()).is_ongoing(&world));
        let other_id = crate::idgen::new_id();
        world
            .items
            .insert(other_id.clone(), make_item(other_id.clone(), Location::Nowhere, None));
        assert!(TriggerCondition::MissingItem(other_id).is_ongoing(&world));
    }

    #[test]
    fn with_npc_condition_detects_presence() {
        let (mut world, room1_id, _) = build_test_world();
        let npc_id = crate::idgen::new_id();
        world.rooms.get_mut(&room1_id).unwrap().npcs.insert(npc_id.clone());
        world.npcs.insert(
            npc_id.clone(),
            make_npc(npc_id.clone(), Location::Room(room1_id.clone()), NpcState::Normal),
        );
        assert!(TriggerCondition::WithNpc(npc_id).is_ongoing(&world));
    }

    #[test]
    fn container_has_item_condition_detects_item() {
        let (mut world, room1_id, _) = build_test_world();
        let container_id = crate::idgen::new_id();
        let item_id = crate::idgen::new_id();
        let mut container = make_item(
            container_id.clone(),
            Location::Room(room1_id.clone()),
            Some(ContainerState::Open),
        );
        container.contents.insert(item_id.clone());
        world.items.insert(container_id.clone(), container);
        world
            .rooms
            .get_mut(&room1_id)
            .unwrap()
            .contents
            .insert(container_id.clone());
        world.items.insert(
            item_id.clone(),
            make_item(item_id.clone(), Location::Item(container_id.clone()), None),
        );
        assert!(TriggerCondition::ContainerHasItem { container_id, item_id }.is_ongoing(&world));
    }
}
