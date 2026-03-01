//! Trigger orchestration and dispatch.
//!
//! Coordinates evaluation of trigger conditions and executes the associated
//! actions when criteria are satisfied during the REPL loop.

pub mod action;
pub mod condition;

pub use action::*;
pub use condition::*;

use crate::{AmbleWorld, View, helpers::plural_s};
use anyhow::Result;

use crate::scheduler::EventCondition;
use log::info;
use serde::{Deserialize, Serialize};

/// A specified response to a particular set of game conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub conditions: EventCondition,
    pub actions: Vec<ScriptedAction>,
    pub only_once: bool,
    pub fired: bool,
}

/// Evaluate triggers against recent events and world state, execute any matching actions, and return the fired set.
/// - The provided `events` slice represents instantaneous "event" conditions (e.g., player enters a room).
/// - Persistent predicates (e.g. player is missing an item) are checked via [`TriggerCondition::is_ongoing`].
/// - Each `Trigger` whose conditions are met has its actions dispatched in order, respecting the `only_once` flag.
///
/// # Errors
/// - Propagates failures from action dispatch such as missing id references.
pub fn check_triggers<'a>(
    world: &'a mut AmbleWorld,
    view: &mut View,
    events: &[TriggerCondition],
) -> Result<Vec<&'a Trigger>> {
    let fire_plan = make_fire_plan(world, events);
    log_firing_triggers(&world.triggers, &fire_plan);
    fire_planned_actions(world, view, &fire_plan)?;
    mark_fired_triggers(&mut world.triggers, &fire_plan);
    let fired: Vec<&Trigger> = fire_plan.trig_indices.iter().map(|i| &world.triggers[*i]).collect();
    Ok(fired)
}

/// Determines if a matching trigger condition exists in a list of triggers.
/// Useful to see if a `TriggerCondition` just sent to `check_triggers` did anything.
pub fn triggers_contain_condition<F>(list: &[&Trigger], matcher: F) -> bool
where
    F: Fn(&TriggerCondition) -> bool,
{
    list.iter().any(|t| t.conditions.any_trigger(|c| matcher(c)))
}

/// A plan containing aggregated (indices to) `Triggers` and their `TriggerActions` to be used for
/// dispatch and logging this turn.
pub struct FirePlan {
    trig_indices: Vec<usize>,
    action_list: Vec<ScriptedAction>,
}

/// Construct a `FirePlan` from triggers and current world state.
fn make_fire_plan(world: &AmbleWorld, events: &[TriggerCondition]) -> FirePlan {
    let trig_indices: Vec<_> = world
        .triggers
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            (!t.only_once || !t.fired)
                && !t
                    .conditions
                    .any_trigger(|cond| matches!(cond, TriggerCondition::Ambient { .. }))
                && t.conditions.eval_with_events(world, events)
        })
        .map(|(idx, _)| idx)
        .collect();

    let action_list: Vec<_> = trig_indices
        .iter()
        .flat_map(|i| world.triggers[*i].actions.clone())
        .collect();

    FirePlan {
        trig_indices,
        action_list,
    }
}

/// Fire each of the `TriggerActions` in a `FirePlan`.
///
/// # Errors
/// - propagated from individual action handlers
fn fire_planned_actions(world: &mut AmbleWorld, view: &mut View, plan: &FirePlan) -> Result<()> {
    for action in &plan.action_list {
        dispatch_action(world, view, action)?;
    }
    Ok(())
}

/// Mark all triggers in a `FirePlan` as fired.
fn mark_fired_triggers(triggers: &mut [Trigger], plan: &FirePlan) {
    plan.trig_indices.iter().for_each(|i| triggers[*i].fired = true)
}

/// Enter the name of each `Trigger` that is firing into the log.
fn log_firing_triggers(triggers: &[Trigger], plan: &FirePlan) {
    let names = plan
        .trig_indices
        .iter()
        .map(|i| triggers[*i].name.clone())
        .collect::<Vec<_>>()
        .join(",");
    let count = plan.trig_indices.iter().count();
    info!("{count} Trigger{} firing: {names}", plural_s(count as isize));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RoomId;
    use crate::{
        room::Room,
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};

    fn build_test_world() -> (AmbleWorld, RoomId, RoomId) {
        let mut world = AmbleWorld::new_empty();
        let room1_id = crate::idgen::new_room_id();
        let room2_id = crate::idgen::new_room_id();

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

    #[test]
    fn check_triggers_moves_player_and_marks_trigger() {
        let (mut world, start_id, dest_id) = build_test_world();
        let mut view = View::new();
        let trigger = Trigger {
            name: "move".into(),
            conditions: EventCondition::Trigger(TriggerCondition::Enter(start_id.clone())),
            actions: vec![ScriptedAction::new(TriggerAction::PushPlayerTo(dest_id.clone()))],
            only_once: true,
            fired: false,
        };
        world.triggers.push(trigger);
        let events = vec![TriggerCondition::Enter(start_id.clone())];
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
            conditions: EventCondition::Trigger(TriggerCondition::Enter(room1_id.clone())),
            actions: vec![],
            only_once: false,
            fired: false,
        };
        let trigger2 = Trigger {
            name: "t2".into(),
            conditions: EventCondition::Trigger(TriggerCondition::Enter(room2_id.clone())),
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
            |c| matches!(c, TriggerCondition::Enter(id) if *id == crate::idgen::new_id())
        ));
    }
}
