use amble_engine as ae;

#[test]
fn toml_schedule_in_if_reschedules_then_fires() {
    use ae::View;
    use ae::loader::SymbolTable;
    use ae::loader::triggers::RawTrigger;
    use ae::loader::triggers::build_triggers;
    use ae::loader::triggers::raw_action::{RawEventCondition, RawOnFalsePolicy, RawTriggerAction};
    use ae::loader::triggers::raw_condition::RawTriggerCondition;
    use ae::world::AmbleWorld;

    // Prepare symbols (not used for HasFlag condition)
    let symbols = SymbolTable::default();

    // Build a trigger with a conditional scheduled action
    let raw_trigger = RawTrigger {
        name: "test-sched-if".into(),
        conditions: vec![],
        actions: vec![RawTriggerAction::ScheduleInIf {
            turns_ahead: 1,
            condition: RawEventCondition::Trigger(RawTriggerCondition::HasFlag { flag: "f".into() }),
            on_false: RawOnFalsePolicy::RetryNextTurn,
            actions: vec![RawTriggerAction::ShowMessage { text: "fired".into() }],
            note: Some("cond-test".into()),
        }],
        only_once: false,
    };

    // Convert raw trigger
    let triggers = build_triggers(&[raw_trigger], &symbols).expect("to_trigger ok");
    let trig = &triggers[0];

    // Create world and ensure condition false initially (flag missing)
    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();

    // Dispatch the schedule action
    assert_eq!(trig.actions.len(), 1);
    ae::trigger::dispatch_action(&mut world, &mut view, &trig.actions[0]).expect("dispatch");

    // Verify event scheduled with condition and policy
    assert_eq!(world.scheduler.events.len(), 1);
    let ev = &world.scheduler.events[0];
    assert!(ev.condition.is_some());

    // Turn advance to due time; since condition false, it should reschedule to next turn
    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check schedule");
    // No message yet
    assert!(
        view.items
            .iter()
            .all(|vi| !matches!(vi, ae::ViewItem::TriggeredEvent(_)))
    );
    // A new event should be queued (original popped, new scheduled)
    assert!(world.scheduler.heap.len() >= 1);

    // Now set flag to satisfy condition and process next turn to fire
    world
        .player
        .flags
        .insert(ae::player::Flag::simple("f", world.turn_count));
    world.turn_count = 2;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check schedule 2");
    // Should have displayed the message
    assert!(
        view.items
            .iter()
            .any(|vi| matches!(vi, ae::ViewItem::TriggeredEvent(msg) if msg.contains("fired")))
    );
}

#[test]
fn toml_schedule_in_if_retry_after() {
    use ae::View;
    use ae::loader::SymbolTable;
    use ae::loader::triggers::RawTrigger;
    use ae::loader::triggers::build_triggers;
    use ae::loader::triggers::raw_action::{RawEventCondition, RawOnFalsePolicy, RawTriggerAction};
    use ae::world::AmbleWorld;

    let symbols = SymbolTable::default();

    // Schedule for next turn; if missing flag, retry 2 turns later
    let raw_trigger = RawTrigger {
        name: "retry-after".into(),
        conditions: vec![],
        actions: vec![RawTriggerAction::ScheduleInIf {
            turns_ahead: 1,
            condition: RawEventCondition::Trigger(ae::loader::triggers::raw_condition::RawTriggerCondition::HasFlag {
                flag: "g".into(),
            }),
            on_false: RawOnFalsePolicy::RetryAfter { turns: 2 },
            actions: vec![RawTriggerAction::ShowMessage {
                text: "retry-fired".into(),
            }],
            note: Some("retry-after-note".into()),
        }],
        only_once: false,
    };
    let triggers = build_triggers(&[raw_trigger], &symbols).expect("to_trigger ok");

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();

    ae::trigger::dispatch_action(&mut world, &mut view, &triggers[0].actions[0]).expect("dispatch");
    assert_eq!(world.scheduler.events.len(), 1);

    // Due at turn 1; condition false -> rescheduled for turn 3
    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 1");
    assert!(
        view.items
            .iter()
            .all(|vi| !matches!(vi, ae::ViewItem::TriggeredEvent(_)))
    );

    // Next turn (2) should not yet fire
    world.turn_count = 2;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 2");
    assert!(
        view.items
            .iter()
            .all(|vi| !matches!(vi, ae::ViewItem::TriggeredEvent(_)))
    );

    // Set flag and reach rescheduled due turn (3) -> should fire
    world
        .player
        .flags
        .insert(ae::player::Flag::simple("g", world.turn_count));
    world.turn_count = 3;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 3");
    assert!(
        view.items
            .iter()
            .any(|vi| matches!(vi, ae::ViewItem::TriggeredEvent(msg) if msg.contains("retry-fired")))
    );
}

#[test]
fn toml_schedule_on_if_cancel() {
    use ae::View;
    use ae::loader::SymbolTable;
    use ae::loader::triggers::RawTrigger;
    use ae::loader::triggers::build_triggers;
    use ae::loader::triggers::raw_action::{RawEventCondition, RawOnFalsePolicy, RawTriggerAction};
    use ae::world::AmbleWorld;

    let symbols = SymbolTable::default();
    let raw_trigger = RawTrigger {
        name: "cancel-on-false".into(),
        conditions: vec![],
        actions: vec![RawTriggerAction::ScheduleOnIf {
            on_turn: 5,
            condition: RawEventCondition::Trigger(ae::loader::triggers::raw_condition::RawTriggerCondition::HasFlag {
                flag: "h".into(),
            }),
            on_false: RawOnFalsePolicy::Cancel,
            actions: vec![RawTriggerAction::ShowMessage {
                text: "cancel-should-not-fire".into(),
            }],
            note: Some("cancel-test".into()),
        }],
        only_once: false,
    };
    let triggers = build_triggers(&[raw_trigger], &symbols).expect("to_trigger ok");

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();

    // Schedule absolute turn
    ae::trigger::dispatch_action(&mut world, &mut view, &triggers[0].actions[0]).expect("dispatch");

    // At turn 5, condition false -> cancels
    world.turn_count = 5;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 5");
    assert!(
        view.items
            .iter()
            .all(|vi| !matches!(vi, ae::ViewItem::TriggeredEvent(_)))
    );

    // Even if condition becomes true later, event was canceled, should not fire
    world
        .player
        .flags
        .insert(ae::player::Flag::simple("h", world.turn_count));
    world.turn_count = 6;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 6");
    assert!(
        view.items
            .iter()
            .all(|vi| !matches!(vi, ae::ViewItem::TriggeredEvent(msg) if msg.contains("cancel-should-not-fire")))
    );
}

#[test]
fn toml_schedule_nested_all_any() {
    use ae::View;
    use ae::loader::SymbolTable;
    use ae::loader::triggers::RawTrigger;
    use ae::loader::triggers::build_triggers;
    use ae::loader::triggers::raw_action::{RawEventCondition, RawOnFalsePolicy, RawTriggerAction};
    use ae::world::AmbleWorld;

    let symbols = SymbolTable::default();
    // condition = all[ hasFlag a, any[ hasFlag b, hasFlag c ] ]
    let cond = RawEventCondition::All {
        all: vec![
            RawEventCondition::Trigger(ae::loader::triggers::raw_condition::RawTriggerCondition::HasFlag {
                flag: "a".into(),
            }),
            RawEventCondition::Any {
                any: vec![
                    RawEventCondition::Trigger(ae::loader::triggers::raw_condition::RawTriggerCondition::HasFlag {
                        flag: "b".into(),
                    }),
                    RawEventCondition::Trigger(ae::loader::triggers::raw_condition::RawTriggerCondition::HasFlag {
                        flag: "c".into(),
                    }),
                ],
            },
        ],
    };
    let raw_trigger = RawTrigger {
        name: "nested-all-any".into(),
        conditions: vec![],
        actions: vec![RawTriggerAction::ScheduleInIf {
            turns_ahead: 1,
            condition: cond,
            on_false: RawOnFalsePolicy::Cancel,
            actions: vec![RawTriggerAction::ShowMessage {
                text: "nested-fired".into(),
            }],
            note: None,
        }],
        only_once: false,
    };
    let triggers = build_triggers(&[raw_trigger], &symbols).expect("to_trigger ok");

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();
    // Satisfy a and c (any[b,c] → true), then fire
    world.player.flags.insert(ae::player::Flag::simple("a", 0));
    world.player.flags.insert(ae::player::Flag::simple("c", 0));

    ae::trigger::dispatch_action(&mut world, &mut view, &triggers[0].actions[0]).expect("dispatch");
    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check");
    assert!(
        view.items
            .iter()
            .any(|vi| matches!(vi, ae::ViewItem::TriggeredEvent(msg) if msg.contains("nested-fired")))
    );
}
