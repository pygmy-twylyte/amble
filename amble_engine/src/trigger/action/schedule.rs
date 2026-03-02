use crate::{ItemId, RoomId};
use anyhow::Result;
use log::info;

use crate::scheduler::{EventCondition, OnFalsePolicy};
use crate::trigger::TriggerCondition;
use crate::view::View;
use crate::world::AmbleWorld;

use super::ScriptedAction;

/// Schedules a list of actions to fire after a specified number of turns.
///
/// # Errors
/// This action never produces an error; scheduling always succeeds.
pub fn schedule_in(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleIn({turns_ahead} turns, {} actions): \"{log_note}\"",
        actions.len()
    );

    world
        .scheduler
        .schedule_in(world.turn_count, turns_ahead + 1, actions.to_vec(), note);
    Ok(())
}

/// Schedules a list of actions to fire on a specific turn.
///
/// # Errors
/// This action never produces an error; scheduling always succeeds.
pub fn schedule_on(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleOn(turn {on_turn}, {} actions): \"{log_note}\"",
        actions.len()
    );

    world.scheduler.schedule_on(on_turn, actions.to_vec(), note);
    Ok(())
}

/// Schedule actions to fire in the future, gated by a condition.
///
/// # Errors
/// This action never produces an error; scheduling always succeeds.
pub fn schedule_in_if(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    condition: &EventCondition,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleInIf({turns_ahead} turns, {} actions, on_false={on_false:?}): \"{log_note}\"",
        actions.len()
    );
    world.scheduler.schedule_in_if(
        world.turn_count,
        turns_ahead + 1,
        Some(condition.clone()),
        on_false,
        actions.to_vec(),
        note,
    );
    Ok(())
}

/// Schedule actions to fire on a specific turn, gated by a condition.
///
/// # Errors
/// This action never produces an error; scheduling always succeeds.
pub fn schedule_on_if(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    condition: &EventCondition,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleOnIf(turn {on_turn}, {} actions, on_false={on_false:?}): \"{log_note}\"",
        actions.len()
    );
    world
        .scheduler
        .schedule_on_if(on_turn, Some(condition.clone()), on_false, actions.to_vec(), note);
    Ok(())
}

/// Convenience: schedule with condition that player is in any of the supplied rooms.
pub fn schedule_if_player_in_any(
    world: &mut AmbleWorld,
    view: &mut View,
    turns_ahead: usize,
    room_ids: impl IntoIterator<Item = RoomId>,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let mut conds = Vec::new();
    for r in room_ids {
        conds.push(EventCondition::Trigger(TriggerCondition::InRoom(r)));
    }
    let condition = if conds.len() == 1 {
        conds.remove(0)
    } else {
        EventCondition::Any(conds)
    };
    schedule_in_if(world, view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on an absolute turn with condition that player is in any of the supplied rooms.
pub fn schedule_on_if_player_in_any(
    world: &mut AmbleWorld,
    view: &mut View,
    on_turn: usize,
    room_ids: impl IntoIterator<Item = RoomId>,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let mut conds = Vec::new();
    for r in room_ids {
        conds.push(EventCondition::Trigger(TriggerCondition::InRoom(r)));
    }
    let condition = if conds.len() == 1 {
        conds.remove(0)
    } else {
        EventCondition::Any(conds)
    };
    schedule_on_if(world, view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if the player has a specific item.
pub fn schedule_in_if_player_has_item(
    world: &mut AmbleWorld,
    view: &mut View,
    turns_ahead: usize,
    item_id: ItemId,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasItem(item_id));
    schedule_in_if(world, view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if the player has a specific item.
pub fn schedule_on_if_player_has_item(
    world: &mut AmbleWorld,
    view: &mut View,
    on_turn: usize,
    item_id: ItemId,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasItem(item_id));
    schedule_on_if(world, view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if the player is missing a specific item.
pub fn schedule_in_if_player_missing_item(
    world: &mut AmbleWorld,
    view: &mut View,
    turns_ahead: usize,
    item_id: ItemId,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingItem(item_id));
    schedule_in_if(world, view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if the player is missing a specific item.
pub fn schedule_on_if_player_missing_item(
    world: &mut AmbleWorld,
    view: &mut View,
    on_turn: usize,
    item_id: ItemId,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingItem(item_id));
    schedule_on_if(world, view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if a flag is set.
pub fn schedule_in_if_flag_set(
    world: &mut AmbleWorld,
    view: &mut View,
    turns_ahead: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasFlag(flag.to_string()));
    schedule_in_if(world, view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if a flag is set.
pub fn schedule_on_if_flag_set(
    world: &mut AmbleWorld,
    view: &mut View,
    on_turn: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasFlag(flag.to_string()));
    schedule_on_if(world, view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if a flag is missing.
pub fn schedule_in_if_flag_missing(
    world: &mut AmbleWorld,
    view: &mut View,
    turns_ahead: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingFlag(flag.to_string()));
    schedule_in_if(world, view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if a flag is missing.
pub fn schedule_on_if_flag_missing(
    world: &mut AmbleWorld,
    view: &mut View,
    on_turn: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[ScriptedAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingFlag(flag.to_string()));
    schedule_on_if(world, view, on_turn, &condition, on_false, actions, note)
}
