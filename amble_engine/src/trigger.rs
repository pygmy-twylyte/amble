//! Trigger module --
//!
//! Upon each run through the REPL loop, world Triggers are checked.
//! If all of a Trigger's TriggerConditions are met, a series of TriggerActions are fired.

pub mod action;
pub mod condition;

pub use action::*;
pub use condition::*;

use crate::AmbleWorld;
use anyhow::Result;

use log::info;
use serde::{Deserialize, Serialize};

/// A specified response to a particular set of game conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub conditions: Vec<TriggerCondition>,
    pub actions: Vec<TriggerAction>,
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
