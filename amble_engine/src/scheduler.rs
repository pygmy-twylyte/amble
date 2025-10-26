//! Event Scheduler
//!
//! Simple future one-off or recurring events can be accomplished using flags and their associated
//! "turnstamps" (turn on which they were set.) This system will allow for more complicated series
//! of events scheduled at arbitrary times in the future. Because it is owned by `AmbleWorld`, all
//! scheduled events should persist correctly across saves.
//!
//! ### Designer Note:
//! This implementation as a priority queue using a binary heap is almost certainly overkill for most
//! likely use cases of this engine. (To be honest, I'm largely using it just to gain experience using
//! the `std::collections::BinaryHeap`). If problematic, a simpler Vec with a filter or partition on
//! turn due would be sufficient.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use log::info;
use serde::{Deserialize, Serialize};

use crate::trigger::{TriggerAction, TriggerCondition};

#[cfg(test)]
const PLACEHOLDER_THRESHOLD: usize = 4;
#[cfg(not(test))]
const PLACEHOLDER_THRESHOLD: usize = 64;

/// The event scheduler.
///
/// Uses a reversed binary heap to maintain a priority queue for upcoming events.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Scheduler {
    pub heap: BinaryHeap<Reverse<(usize, usize)>>, /* (turn_due, event_idx) */
    pub events: Vec<ScheduledEvent>,
}
impl Scheduler {
    /// Schedule some `TriggerActions` to fire a specified number of turns in the future.
    pub fn schedule_in(&mut self, now: usize, turns_ahead: usize, actions: Vec<TriggerAction>, note: Option<String>) {
        let idx = self.events.len();
        let on_turn = now + turns_ahead;
        let log_msg = match &note {
            Some(msg) => msg.as_str(),
            None => "<no note provided>",
        };
        info!("scheduling event (turn now/due = {now}/{on_turn}): \"{log_msg}\"");
        self.heap.push(Reverse((on_turn, idx)));
        self.events.push(ScheduledEvent {
            on_turn,
            actions,
            note,
            condition: None,
            on_false: OnFalsePolicy::Cancel,
        });
    }

    /// Schedule some `TriggerActions` to fire on a specific turn.
    pub fn schedule_on(&mut self, on_turn: usize, actions: Vec<TriggerAction>, note: Option<String>) {
        let idx = self.events.len();
        let log_msg = match &note {
            Some(note) => note.as_str(),
            None => "<no note provided>",
        };
        info!("scheduling event (turn due = {on_turn}): \"{log_msg}\"");
        self.heap.push(Reverse((on_turn, idx)));
        self.events.push(ScheduledEvent {
            on_turn,
            actions,
            note,
            condition: None,
            on_false: OnFalsePolicy::Cancel,
        });
    }

    /// Schedule actions in the future with an optional condition and on-false policy.
    ///
    /// Primarily used by conditional scheduling trigger actions; events that fail
    /// the condition at execution time can be retried according to `on_false`.
    pub fn schedule_in_if(
        &mut self,
        now: usize,
        turns_ahead: usize,
        condition: Option<EventCondition>,
        on_false: OnFalsePolicy,
        actions: Vec<TriggerAction>,
        note: Option<String>,
    ) {
        let idx = self.events.len();
        let on_turn = now + turns_ahead;
        let log_msg = match &note {
            Some(msg) => msg.as_str(),
            None => "<no note provided>",
        };
        info!("scheduling conditional event (turn now/due = {now}/{on_turn}): \"{log_msg}\"");
        self.heap.push(Reverse((on_turn, idx)));
        self.events.push(ScheduledEvent {
            on_turn,
            actions,
            note,
            condition,
            on_false,
        });
    }

    /// Schedule actions on a specific turn with an optional condition and on-false policy.
    pub fn schedule_on_if(
        &mut self,
        on_turn: usize,
        condition: Option<EventCondition>,
        on_false: OnFalsePolicy,
        actions: Vec<TriggerAction>,
        note: Option<String>,
    ) {
        let idx = self.events.len();
        let log_msg = match &note {
            Some(note) => note.as_str(),
            None => "<no note provided>",
        };
        info!("scheduling conditional event (turn due = {on_turn}): \"{log_msg}\"");
        self.heap.push(Reverse((on_turn, idx)));
        self.events.push(ScheduledEvent {
            on_turn,
            actions,
            note,
            condition,
            on_false,
        });
    }

    /// Pop the next due event, if any.
    ///
    /// Returns `None` when the earliest scheduled event is still in the future.
    pub fn pop_due(&mut self, now: usize) -> Option<ScheduledEvent> {
        if let Some(Reverse((turn_due, idx))) = self.heap.peek().copied() {
            if now >= turn_due {
                self.heap.pop();
                // "take" instead of "remove" keeps indices stable for the heap entries
                // leaves default placeholders
                let event = std::mem::take(&mut self.events[idx]);
                self.compact_if_needed();
                return Some(event);
            }
        }
        None
    }

    /// Rebuild the underlying storage when too many placeholder tombstones accumulate.
    fn compact_if_needed(&mut self) {
        let placeholder_count = self.events.iter().filter(|e| e.is_placeholder()).count();
        if placeholder_count > PLACEHOLDER_THRESHOLD {
            let old_events = std::mem::take(&mut self.events);
            let mut index_map = vec![0; old_events.len()];
            for (old_idx, event) in old_events.into_iter().enumerate() {
                if event.is_placeholder() {
                    continue;
                }
                let new_idx = self.events.len();
                index_map[old_idx] = new_idx;
                self.events.push(event);
            }
            let mut new_heap = BinaryHeap::with_capacity(self.heap.len());
            while let Some(Reverse((turn_due, old_idx))) = self.heap.pop() {
                let new_idx = index_map[old_idx];
                new_heap.push(Reverse((turn_due, new_idx)));
            }
            self.heap = new_heap;
        }
    }
}

/// An event (sequence of `TriggerActions`) scheduled for a particular turn.
///
/// ### Fields:
/// `on_turn` = turn on which to fire
/// actions = list of `TriggerActions` to take when the turn arrives
/// note = description of event (for logging)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub on_turn: usize,
    pub actions: Vec<TriggerAction>,
    pub note: Option<String>,
    /// Optional condition that must be true for the event to fire.
    pub condition: Option<EventCondition>,
    /// Policy to apply when the condition evaluates to false.
    pub on_false: OnFalsePolicy,
}

impl ScheduledEvent {
    /// Placeholder events mark consumed slots within the scheduler. Determine whether this is one of them.
    fn is_placeholder(&self) -> bool {
        self.on_turn == 0 && self.actions.is_empty() && self.note.is_none() && self.condition.is_none()
    }
}

impl Default for ScheduledEvent {
    fn default() -> Self {
        Self {
            on_turn: 0,
            actions: Vec::new(),
            note: None,
            condition: None,
            on_false: OnFalsePolicy::Cancel,
        }
    }
}

/// Policy controlling behavior when a scheduled event's condition is false.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OnFalsePolicy {
    /// Cancel the event and do not retry.
    #[default]
    Cancel,
    /// Retry after the specified number of turns.
    RetryAfter(usize),
    /// Retry on the next turn.
    RetryNextTurn,
}

/// Condition for scheduled events. Can wrap a `TriggerCondition` or combine multiple.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventCondition {
    /// Single trigger condition; evaluated via existing machinery.
    Trigger(TriggerCondition),
    /// All subconditions must be true.
    All(Vec<EventCondition>),
    /// Any subcondition must be true.
    Any(Vec<EventCondition>),
}

impl EventCondition {
    /// Evaluate the condition against the current world state.
    pub fn eval(&self, world: &crate::world::AmbleWorld) -> bool {
        match self {
            EventCondition::Trigger(tc) => tc.is_ongoing(world),
            EventCondition::All(conds) => conds.iter().all(|c| c.eval(world)),
            EventCondition::Any(conds) => conds.iter().any(|c| c.eval(world)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trigger::TriggerAction;

    fn create_test_action() -> TriggerAction {
        TriggerAction::ShowMessage("Test message".to_string())
    }

    fn create_test_actions(count: usize) -> Vec<TriggerAction> {
        (0..count)
            .map(|i| TriggerAction::ShowMessage(format!("Message {}", i)))
            .collect()
    }

    #[test]
    fn scheduler_new_is_empty() {
        let scheduler = Scheduler::default();
        assert!(scheduler.heap.is_empty());
        assert!(scheduler.events.is_empty());
    }

    #[test]
    fn schedule_in_adds_event_correctly() {
        let mut scheduler = Scheduler::default();
        let actions = vec![create_test_action()];
        let note = Some("Test event".to_string());

        scheduler.schedule_in(5, 3, actions.clone(), note.clone());

        assert_eq!(scheduler.events.len(), 1);
        assert_eq!(scheduler.heap.len(), 1);

        let event = &scheduler.events[0];
        assert_eq!(event.on_turn, 8); // 5 + 3
        assert_eq!(event.actions.len(), 1);
        assert_eq!(event.note, note);
    }

    #[test]
    fn schedule_on_adds_event_correctly() {
        let mut scheduler = Scheduler::default();
        let actions = vec![create_test_action()];
        let note = Some("Direct schedule test".to_string());

        scheduler.schedule_on(10, actions.clone(), note.clone());

        assert_eq!(scheduler.events.len(), 1);
        assert_eq!(scheduler.heap.len(), 1);

        let event = &scheduler.events[0];
        assert_eq!(event.on_turn, 10);
        assert_eq!(event.actions.len(), 1);
        assert_eq!(event.note, note);
    }

    #[test]
    fn schedule_multiple_events() {
        let mut scheduler = Scheduler::default();

        scheduler.schedule_in(0, 5, vec![create_test_action()], Some("Event 1".to_string()));
        scheduler.schedule_in(0, 3, vec![create_test_action()], Some("Event 2".to_string()));
        scheduler.schedule_on(10, vec![create_test_action()], Some("Event 3".to_string()));

        assert_eq!(scheduler.events.len(), 3);
        assert_eq!(scheduler.heap.len(), 3);
    }

    #[test]
    fn pop_due_returns_none_when_nothing_due() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_in(5, 5, vec![create_test_action()], None);

        let result = scheduler.pop_due(8); // Event due on turn 10
        assert!(result.is_none());
        assert_eq!(scheduler.heap.len(), 1); // Event should still be in heap
    }

    #[test]
    fn pop_due_returns_event_when_due() {
        let mut scheduler = Scheduler::default();
        let actions = vec![create_test_action()];
        let note = Some("Due event".to_string());

        scheduler.schedule_in(5, 3, actions.clone(), note.clone());

        let result = scheduler.pop_due(8); // Event due exactly on turn 8
        assert!(result.is_some());

        let event = result.unwrap();
        assert_eq!(event.on_turn, 8);
        assert_eq!(event.note, note);
        assert_eq!(event.actions.len(), 1);

        // Heap should now be empty
        assert!(scheduler.heap.is_empty());
    }

    #[test]
    fn pop_due_returns_event_when_overdue() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_in(5, 3, vec![create_test_action()], Some("Overdue event".to_string()));

        let result = scheduler.pop_due(10); // Event was due on turn 8, now turn 10
        assert!(result.is_some());

        let event = result.unwrap();
        assert_eq!(event.on_turn, 8);
    }

    #[test]
    fn events_fire_in_correct_order() {
        let mut scheduler = Scheduler::default();

        // Schedule events in reverse chronological order
        scheduler.schedule_on(15, create_test_actions(1), Some("Third".to_string()));
        scheduler.schedule_on(5, create_test_actions(1), Some("First".to_string()));
        scheduler.schedule_on(10, create_test_actions(1), Some("Second".to_string()));

        // Pop events in chronological order
        let first = scheduler.pop_due(5).unwrap();
        assert_eq!(first.note, Some("First".to_string()));
        assert_eq!(first.on_turn, 5);

        let second = scheduler.pop_due(10).unwrap();
        assert_eq!(second.note, Some("Second".to_string()));
        assert_eq!(second.on_turn, 10);

        let third = scheduler.pop_due(15).unwrap();
        assert_eq!(third.note, Some("Third".to_string()));
        assert_eq!(third.on_turn, 15);

        // Nothing left
        assert!(scheduler.pop_due(20).is_none());
    }

    #[test]
    fn events_with_same_turn_fire_in_fifo_order() {
        let mut scheduler = Scheduler::default();

        // Schedule multiple events for the same turn
        scheduler.schedule_on(10, create_test_actions(1), Some("First scheduled".to_string()));
        scheduler.schedule_on(10, create_test_actions(1), Some("Second scheduled".to_string()));
        scheduler.schedule_on(10, create_test_actions(1), Some("Third scheduled".to_string()));

        // They should come out in FIFO order (first scheduled, first fired)
        let first = scheduler.pop_due(10).unwrap();
        assert_eq!(first.note, Some("First scheduled".to_string()));

        let second = scheduler.pop_due(10).unwrap();
        assert_eq!(second.note, Some("Second scheduled".to_string()));

        let third = scheduler.pop_due(10).unwrap();
        assert_eq!(third.note, Some("Third scheduled".to_string()));
    }

    #[test]
    fn pop_due_multiple_events_same_turn() {
        let mut scheduler = Scheduler::default();

        scheduler.schedule_on(5, create_test_actions(1), Some("Event A".to_string()));
        scheduler.schedule_on(5, create_test_actions(1), Some("Event B".to_string()));
        scheduler.schedule_on(10, create_test_actions(1), Some("Event C".to_string()));

        // Pop all events due on turn 5
        let mut events_turn_5 = Vec::new();
        while let Some(event) = scheduler.pop_due(5) {
            if event.on_turn == 5 {
                events_turn_5.push(event);
            } else {
                break;
            }
        }

        assert_eq!(events_turn_5.len(), 2);
        assert!(events_turn_5.iter().any(|e| e.note == Some("Event A".to_string())));
        assert!(events_turn_5.iter().any(|e| e.note == Some("Event B".to_string())));

        // Event C should still be in scheduler
        let event_c = scheduler.pop_due(10).unwrap();
        assert_eq!(event_c.note, Some("Event C".to_string()));
    }

    #[test]
    fn schedule_with_no_note() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_in(0, 5, vec![create_test_action()], None);

        let event = scheduler.pop_due(5).unwrap();
        assert_eq!(event.note, None);
    }

    #[test]
    fn schedule_with_empty_actions() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_in(0, 5, vec![], Some("Empty actions".to_string()));

        let event = scheduler.pop_due(5).unwrap();
        assert!(event.actions.is_empty());
        assert_eq!(event.note, Some("Empty actions".to_string()));
    }

    #[test]
    fn schedule_with_multiple_actions() {
        let mut scheduler = Scheduler::default();
        let actions = create_test_actions(5);

        scheduler.schedule_in(0, 3, actions.clone(), Some("Multi-action event".to_string()));

        let event = scheduler.pop_due(3).unwrap();
        assert_eq!(event.actions.len(), 5);
    }

    #[test]
    fn scheduled_event_default() {
        let event = ScheduledEvent::default();
        assert_eq!(event.on_turn, 0);
        assert!(event.actions.is_empty());
        assert_eq!(event.note, None);
    }

    #[test]
    fn mem_take_leaves_default_placeholder() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_in(0, 5, vec![create_test_action()], Some("Test".to_string()));

        let _event = scheduler.pop_due(5).unwrap();

        // The event vector should still have the placeholder
        assert_eq!(scheduler.events.len(), 1);
        let placeholder = &scheduler.events[0];
        assert_eq!(placeholder.on_turn, 0);
        assert!(placeholder.actions.is_empty());
        assert_eq!(placeholder.note, None);
    }

    #[test]
    fn compact_events_when_placeholder_threshold_exceeded() {
        let mut scheduler = Scheduler::default();

        for i in 1..=6 {
            scheduler.schedule_on(i, create_test_actions(1), Some(format!("Event {}", i)));
        }

        for turn in 1..=5 {
            let ev = scheduler.pop_due(turn).unwrap();
            assert_eq!(ev.note, Some(format!("Event {}", turn)));
        }

        assert_eq!(scheduler.events.len(), 1);
        assert_eq!(scheduler.heap.len(), 1);
        assert_eq!(scheduler.events[0].note, Some("Event 6".to_string()));

        let final_event = scheduler.pop_due(6).unwrap();
        assert_eq!(final_event.note, Some("Event 6".to_string()));
    }

    #[test]
    fn edge_case_turn_zero() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_on(0, vec![create_test_action()], Some("Turn zero".to_string()));

        let event = scheduler.pop_due(0).unwrap();
        assert_eq!(event.on_turn, 0);
    }

    #[test]
    fn edge_case_large_turn_numbers() {
        let mut scheduler = Scheduler::default();
        let large_turn = usize::MAX - 1000;

        scheduler.schedule_on(large_turn, vec![create_test_action()], Some("Large turn".to_string()));

        let event = scheduler.pop_due(large_turn).unwrap();
        assert_eq!(event.on_turn, large_turn);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut scheduler = Scheduler::default();
        scheduler.schedule_in(5, 10, create_test_actions(3), Some("Serialization test".to_string()));
        scheduler.schedule_on(20, create_test_actions(2), None);

        // Serialize
        let serialized = serde_json::to_string(&scheduler).expect("Failed to serialize");

        // Deserialize
        let deserialized: Scheduler = serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Verify structure is preserved
        assert_eq!(deserialized.events.len(), scheduler.events.len());
        assert_eq!(deserialized.heap.len(), scheduler.heap.len());

        // Verify functionality is preserved
        let mut des_scheduler = deserialized;
        let event1 = des_scheduler.pop_due(15).unwrap();
        assert_eq!(event1.on_turn, 15);
        assert_eq!(event1.actions.len(), 3);

        let event2 = des_scheduler.pop_due(20).unwrap();
        assert_eq!(event2.on_turn, 20);
        assert_eq!(event2.actions.len(), 2);
        assert_eq!(event2.note, None);
    }
}
