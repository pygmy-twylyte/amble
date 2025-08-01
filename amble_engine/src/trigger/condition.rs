//! condition.rs -- `TriggerCondition` Module
//!
//! Implements various player actions and game state that can be detected
//! by a Trigger, resulting in some `TriggerActions` firing.

use std::collections::HashSet;

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
    Ambient {
        room_ids: HashSet<Uuid>, // empty = applies everywhere
        spinner: SpinnerType,
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

    pub fn is_ongoing(&self, world: &AmbleWorld) -> bool {
        let player_flag_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match self {
            Self::ContainerHasItem { container_id, item_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| item.location == Location::Item(*container_id)),
            Self::HasFlag(flag) => player_flag_set(flag),
            Self::MissingFlag(flag) => !player_flag_set(flag),
            Self::FlagInProgress(flag) => world
                .player
                .flags
                .get(&Flag::Simple { name: flag.into() })
                .is_some_and(|f| !f.is_complete()),
            Self::FlagComplete(flag) => world
                .player
                .flags
                .get(&Flag::Simple { name: flag.into() })
                .is_some_and(Flag::is_complete),
            Self::HasVisited(room_id) => world.rooms.get(room_id).is_some_and(|r| r.visited),
            Self::InRoom(room_id) => *room_id == world.player.location.unwrap_room(),
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
