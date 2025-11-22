//! Trigger orchestration and dispatch.
//!
//! Coordinates evaluation of trigger conditions and executes the associated
//! actions when criteria are satisfied during the REPL loop.

pub mod action;
pub mod condition;

pub use action::*;
pub use condition::*;

use crate::{AmbleWorld, View};
use anyhow::Result;

use log::info;
use serde::{Deserialize, Serialize};

/// A specified response to a particular set of game conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub conditions: Vec<TriggerCondition>,
    pub actions: Vec<ScriptedAction>,
    pub only_once: bool,
    pub fired: bool,
}

/// Determines if a matching trigger condition exists in a list of triggers.
/// Useful to see if a `TriggerCondition` just sent to `check_triggers` did anything.
pub fn triggers_contain_condition<F>(list: &[&Trigger], matcher: F) -> bool
where
    F: Fn(&TriggerCondition) -> bool,
{
    list.iter().any(|t| t.conditions.iter().any(&matcher))
}

/// Evaluate triggers against recent events, execute matching actions, and return the fired set.
///
/// The provided `events` slice represents instantaneous conditions (e.g., player just entered a room).
/// Persistent predicates are checked via [`TriggerCondition::is_ongoing`]. Each matching trigger
/// has its actions dispatched in order, respecting the `only_once` flag.
///
/// # Errors
/// - Propagates failures from action dispatch such as missing UUID references.
pub fn check_triggers<'a>(
    world: &'a mut AmbleWorld,
    view: &mut View,
    events: &[TriggerCondition],
) -> Result<Vec<&'a Trigger>> {
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
            dispatch_action(world, view, &action)?;
        }
    }

    let fired_triggers: Vec<&Trigger> = to_fire.iter().map(|i| &world.triggers[*i]).collect();
    Ok(fired_triggers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
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

    #[test]
    fn check_triggers_moves_player_and_marks_trigger() {
        let (mut world, start_id, dest_id) = build_test_world();
        let mut view = View::new();
        let trigger = Trigger {
            name: "move".into(),
            conditions: vec![TriggerCondition::Enter(start_id)],
            actions: vec![ScriptedAction::new(TriggerAction::PushPlayerTo(dest_id))],
            only_once: true,
            fired: false,
        };
        world.triggers.push(trigger);
        let events = vec![TriggerCondition::Enter(start_id)];
        let fired = check_triggers(&mut world, &mut view, &events).expect("check_triggers failed");
        assert_eq!(fired.len(), 1);
        assert!(triggers_contain_condition(
            &fired,
            |c| matches!(c, TriggerCondition::Enter(id) if *id == start_id)
        ));
        drop(fired);
        assert_eq!(world.player.location, Location::Room(dest_id));
        assert!(world.triggers[0].fired);
    }

    #[test]
    fn triggers_contain_condition_finds_matches() {
        let (mut world, room1_id, room2_id) = build_test_world();
        let trigger1 = Trigger {
            name: "t1".into(),
            conditions: vec![TriggerCondition::Enter(room1_id)],
            actions: vec![],
            only_once: false,
            fired: false,
        };
        let trigger2 = Trigger {
            name: "t2".into(),
            conditions: vec![TriggerCondition::Enter(room2_id)],
            actions: vec![],
            only_once: false,
            fired: false,
        };
        world.triggers.push(trigger1);
        world.triggers.push(trigger2);
        let refs: Vec<&Trigger> = world.triggers.iter().collect();
        assert!(triggers_contain_condition(
            &refs,
            |c| matches!(c, TriggerCondition::Enter(id) if *id == room1_id)
        ));
        assert!(!triggers_contain_condition(
            &refs,
            |c| matches!(c, TriggerCondition::Enter(id) if *id == Uuid::new_v4())
        ));
    }
}
