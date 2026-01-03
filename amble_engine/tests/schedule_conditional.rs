use amble_engine as ae;

fn stmt(action: ae::trigger::TriggerAction) -> ae::trigger::ScriptedAction {
    ae::trigger::ScriptedAction::new(action)
}

#[test]
fn schedule_in_if_reschedules_then_fires() {
    use ae::View;
    use ae::scheduler::{EventCondition, OnFalsePolicy};
    use ae::trigger::{TriggerAction, TriggerCondition, dispatch_action};
    use ae::world::AmbleWorld;

    let action = TriggerAction::ScheduleInIf {
        turns_ahead: 1,
        condition: EventCondition::Trigger(TriggerCondition::HasFlag("f".into())),
        on_false: OnFalsePolicy::RetryNextTurn,
        actions: vec![stmt(TriggerAction::ShowMessage("fired".into()))],
        note: Some("cond-test".into()),
    };

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();

    dispatch_action(&mut world, &mut view, &stmt(action)).expect("dispatch");

    assert_eq!(world.scheduler.events.len(), 1);
    let ev = &world.scheduler.events[0];
    assert!(ev.condition.is_some());

    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check schedule");
    assert!(
        view.items
            .iter()
            .all(|entry| { !matches!(&entry.view_item, ae::ViewItem::TriggeredEvent(_)) })
    );
    assert!(!world.scheduler.heap.is_empty());

    world
        .player
        .flags
        .insert(ae::player::Flag::simple("f", world.turn_count));
    world.turn_count = 2;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check schedule 2");
    assert!(view.items.iter().any(|entry| {
        matches!(
            &entry.view_item,
            ae::ViewItem::TriggeredEvent(msg) if msg.contains("fired")
        )
    }));
}

#[test]
fn schedule_in_if_retry_after() {
    use ae::View;
    use ae::scheduler::{EventCondition, OnFalsePolicy};
    use ae::trigger::{TriggerAction, TriggerCondition, dispatch_action};
    use ae::world::AmbleWorld;

    let action = TriggerAction::ScheduleInIf {
        turns_ahead: 1,
        condition: EventCondition::Trigger(TriggerCondition::HasFlag("g".into())),
        on_false: OnFalsePolicy::RetryAfter(2),
        actions: vec![stmt(TriggerAction::ShowMessage("retry-fired".into()))],
        note: Some("retry-after-note".into()),
    };

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();

    dispatch_action(&mut world, &mut view, &stmt(action)).expect("dispatch");
    assert_eq!(world.scheduler.events.len(), 1);

    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 1");
    assert!(
        view.items
            .iter()
            .all(|entry| { !matches!(&entry.view_item, ae::ViewItem::TriggeredEvent(_)) })
    );

    world.turn_count = 2;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 2");
    assert!(
        view.items
            .iter()
            .all(|entry| { !matches!(&entry.view_item, ae::ViewItem::TriggeredEvent(_)) })
    );

    world
        .player
        .flags
        .insert(ae::player::Flag::simple("g", world.turn_count));
    world.turn_count = 3;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 3");
    assert!(view.items.iter().any(|entry| {
        matches!(
            &entry.view_item,
            ae::ViewItem::TriggeredEvent(msg) if msg.contains("retry-fired")
        )
    }));
}

#[test]
fn schedule_on_if_cancel() {
    use ae::View;
    use ae::scheduler::{EventCondition, OnFalsePolicy};
    use ae::trigger::{TriggerAction, TriggerCondition, dispatch_action};
    use ae::world::AmbleWorld;

    let action = TriggerAction::ScheduleOnIf {
        on_turn: 5,
        condition: EventCondition::Trigger(TriggerCondition::HasFlag("h".into())),
        on_false: OnFalsePolicy::Cancel,
        actions: vec![stmt(TriggerAction::ShowMessage("cancel-should-not-fire".into()))],
        note: Some("cancel-test".into()),
    };

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();

    dispatch_action(&mut world, &mut view, &stmt(action)).expect("dispatch");

    world.turn_count = 5;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 5");
    assert!(
        view.items
            .iter()
            .all(|entry| { !matches!(&entry.view_item, ae::ViewItem::TriggeredEvent(_)) })
    );

    world
        .player
        .flags
        .insert(ae::player::Flag::simple("h", world.turn_count));
    world.turn_count = 6;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check 6");
    assert!(view.items.iter().all(|entry| {
        !matches!(
            &entry.view_item,
            ae::ViewItem::TriggeredEvent(msg) if msg.contains("cancel-should-not-fire")
        )
    }));
}

#[test]
fn schedule_nested_all_any() {
    use ae::View;
    use ae::scheduler::{EventCondition, OnFalsePolicy};
    use ae::trigger::{TriggerAction, TriggerCondition, dispatch_action};
    use ae::world::AmbleWorld;

    let cond = EventCondition::All(vec![
        EventCondition::Trigger(TriggerCondition::HasFlag("a".into())),
        EventCondition::Any(vec![
            EventCondition::Trigger(TriggerCondition::HasFlag("b".into())),
            EventCondition::Trigger(TriggerCondition::HasFlag("c".into())),
        ]),
    ]);
    let action = TriggerAction::ScheduleInIf {
        turns_ahead: 1,
        condition: cond,
        on_false: OnFalsePolicy::Cancel,
        actions: vec![stmt(TriggerAction::ShowMessage("nested-fired".into()))],
        note: None,
    };

    let mut world = AmbleWorld::new_empty();
    let mut view = View::new();
    world.player.flags.insert(ae::player::Flag::simple("a", 0));
    world.player.flags.insert(ae::player::Flag::simple("c", 0));

    dispatch_action(&mut world, &mut view, &stmt(action)).expect("dispatch");
    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).expect("check");
    assert!(view.items.iter().any(|entry| {
        matches!(
            &entry.view_item,
            ae::ViewItem::TriggeredEvent(msg) if msg.contains("nested-fired")
        )
    }));
}
