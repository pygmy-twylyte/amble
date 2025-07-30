//! Trigger module --
//!
//! Upon each run through the REPL loop, world Triggers are checked.
//! If all of a Trigger's TriggerConditions are met, a series of TriggerActions are fired.

pub mod action;

pub use action::*;

use std::collections::HashSet;

use crate::{
    AmbleWorld, ItemHolder, Location,
    item::{ItemAbility, ItemInteractionType},
    npc::NpcState,
    spinners::SpinnerType,
};
use anyhow::Result;

use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A specified response to a particular set of game conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub conditions: Vec<TriggerCondition>,
    pub actions: Vec<TriggerAction>,
    pub only_once: bool,
    pub fired: bool,
}

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
    fn matches_event_in(&self, events: &[TriggerCondition]) -> bool {
        events.contains(self)
    }

    fn is_ongoing(&self, world: &AmbleWorld) -> bool {
        let player_flag_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match self {
            Self::ContainerHasItem { container_id, item_id } => {
                if let Some(item) = world.items.get(item_id) {
                    item.location == Location::Item(*container_id)
                } else {
                    false
                }
            },
            Self::HasFlag(flag) => player_flag_set(flag),
            Self::MissingFlag(flag) => !player_flag_set(flag),
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
/// Determines if a matching trigger condition exists in a list of triggers.
/// Useful to see if a `TriggerCondition` just sent to `check_triggers` did anything.
pub fn triggers_contain_condition<F>(list: &[&Trigger], matcher: F) -> bool
where
    F: Fn(&TriggerCondition) -> bool,
{
    list.iter().any(|t| t.conditions.iter().any(&matcher))
}

/// Determine which triggers meet conditions to fire now, fire them, and return a list of fired triggers.
///
/// # Errors
/// - on any failed uuid lookup during trigger dispatch
pub fn check_triggers<'a>(world: &'a mut AmbleWorld, events: &[TriggerCondition]) -> Result<Vec<&'a Trigger>> {
    // collect map of indices to triggers that should fire now
    let to_fire: Vec<_> = world
        .triggers
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.only_once || !t.fired)
        .filter(|(_, t)| {
            t.conditions
                .iter()
                .all(|c| c.matches_event_in(events) || c.is_ongoing(world))
        })
        .map(|(i, _)| i)
        .collect();

    // mark each trigger as fired if a one-off and log it
    for i in &to_fire {
        let trigger = &mut world.triggers[*i];
        info!("Trigger fired: {}", trigger.name);
        if trigger.only_once {
            trigger.fired = true;
        }

        let actions = trigger.actions.clone();
        for action in actions {
            dispatch_action(world, &action)?;
        }
    }

    let fired_triggers: Vec<&Trigger> = to_fire.iter().map(|i| &world.triggers[*i]).collect();
    Ok(fired_triggers)
}
