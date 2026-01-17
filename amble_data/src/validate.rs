use std::collections::HashSet;
use std::fmt;

use crate::*;

/// Validation error for malformed or missing references in a WorldDef.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateId { kind: &'static str, id: String },
    MissingReference { kind: &'static str, id: String, context: String },
    InvalidValue { context: String },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::DuplicateId { kind, id } => {
                write!(f, "duplicate {kind} id '{id}'")
            },
            ValidationError::MissingReference { kind, id, context } => {
                write!(f, "missing {kind} '{id}' ({context})")
            },
            ValidationError::InvalidValue { context } => {
                write!(f, "invalid value ({context})")
            },
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate cross-references and basic invariants in a WorldDef.
///
/// ```
/// use amble_data::{GameDef, PlayerDef, RoomDef, ScoringDef, WorldDef, validate_world};
///
/// let world = WorldDef {
///     game: GameDef {
///         title: "Demo".into(),
///         intro: "Intro".into(),
///         player: PlayerDef {
///             name: "Player".into(),
///             description: "A hero".into(),
///             start_room: "start".into(),
///             max_hp: 10,
///         },
///         scoring: ScoringDef::default(),
///     },
///     rooms: vec![RoomDef {
///         id: "start".into(),
///         name: "Start".into(),
///         desc: "A room.".into(),
///         visited: false,
///         exits: Vec::new(),
///         overlays: Vec::new(),
///     }],
///     ..WorldDef::default()
/// };
/// assert!(validate_world(&world).is_empty());
/// ```
pub fn validate_world(world: &WorldDef) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let mut rooms = HashSet::new();
    let mut items = HashSet::new();
    let mut npcs = HashSet::new();
    let mut spinners = HashSet::new();
    let mut goals = HashSet::new();

    track_ids(
        "room",
        world.rooms.iter().map(|r| r.id.as_str()),
        &mut rooms,
        &mut errors,
    );
    track_ids(
        "item",
        world.items.iter().map(|i| i.id.as_str()),
        &mut items,
        &mut errors,
    );
    track_ids("npc", world.npcs.iter().map(|n| n.id.as_str()), &mut npcs, &mut errors);
    track_ids(
        "spinner",
        world.spinners.iter().map(|s| s.id.as_str()),
        &mut spinners,
        &mut errors,
    );
    track_ids(
        "goal",
        world.goals.iter().map(|g| g.id.as_str()),
        &mut goals,
        &mut errors,
    );

    // Store ID sets once so we can check cross-references cheaply.
    let ids = IdSets {
        rooms: &rooms,
        items: &items,
        npcs: &npcs,
        spinners: &spinners,
        goals: &goals,
    };

    if world.game.player.start_room.trim().is_empty() {
        errors.push(ValidationError::InvalidValue {
            context: "game player start room missing".to_string(),
        });
    } else {
        check_ref(
            "room",
            &world.game.player.start_room,
            ids.rooms,
            "game player start room".to_string(),
            &mut errors,
        );
    }

    if world.game.scoring.ranks.is_empty() {
        errors.push(ValidationError::InvalidValue {
            context: "scoring ranks empty".to_string(),
        });
    }

    for rank in &world.game.scoring.ranks {
        if !(0.0..=100.0).contains(&rank.threshold) {
            errors.push(ValidationError::InvalidValue {
                context: format!(
                    "scoring rank '{}' threshold out of range ({})",
                    rank.name, rank.threshold
                ),
            });
        }
    }

    for room in &world.rooms {
        for exit in &room.exits {
            check_ref(
                "room",
                &exit.to,
                ids.rooms,
                format!("room '{}' exit '{}'", room.id, exit.direction),
                &mut errors,
            );
            for item in &exit.required_items {
                check_ref(
                    "item",
                    item,
                    ids.items,
                    format!("room '{}' exit '{}'", room.id, exit.direction),
                    &mut errors,
                );
            }
        }
        for overlay in &room.overlays {
            for cond in &overlay.conditions {
                validate_overlay_condition(cond, &ids, &mut errors, &room.id);
            }
        }
    }

    for item in &world.items {
        validate_location(&item.location, &ids, &mut errors, &format!("item '{}'", item.id));
        for ability in &item.abilities {
            validate_item_ability(ability, &ids, &mut errors, &format!("item '{}'", item.id));
        }
        for ability in item.interaction_requires.values() {
            validate_item_ability(ability, &ids, &mut errors, &format!("item '{}'", item.id));
        }
        if let Some(consumable) = &item.consumable {
            for ability in &consumable.consume_on {
                validate_item_ability(ability, &ids, &mut errors, &format!("item '{}'", item.id));
            }
            match &consumable.when_consumed {
                ConsumeTypeDef::Despawn => {},
                ConsumeTypeDef::ReplaceInventory { replacement }
                | ConsumeTypeDef::ReplaceCurrentRoom { replacement } => {
                    check_ref(
                        "item",
                        replacement,
                        ids.items,
                        format!("item '{}' consumable replacement", item.id),
                        &mut errors,
                    );
                },
            }
        }
    }

    for npc in &world.npcs {
        validate_location(&npc.location, &ids, &mut errors, &format!("npc '{}'", npc.id));
        if let Some(movement) = &npc.movement {
            for room in &movement.rooms {
                check_ref(
                    "room",
                    room,
                    ids.rooms,
                    format!("npc '{}' movement", npc.id),
                    &mut errors,
                );
            }
        }
    }

    for goal in &world.goals {
        if let Some(cond) = &goal.activate_when {
            validate_goal_condition(cond, &ids, &mut errors, &format!("goal '{}'", goal.id));
        }
        validate_goal_condition(&goal.finished_when, &ids, &mut errors, &format!("goal '{}'", goal.id));
        if let Some(cond) = &goal.failed_when {
            validate_goal_condition(cond, &ids, &mut errors, &format!("goal '{}'", goal.id));
        }
    }

    for trigger in &world.triggers {
        validate_event(
            &trigger.event,
            &ids,
            &mut errors,
            &format!("trigger '{}'", trigger.name),
        );
        validate_condition_expr(
            &trigger.conditions,
            &ids,
            &mut errors,
            &format!("trigger '{}'", trigger.name),
        );
        for action in &trigger.actions {
            validate_action(action, &ids, &mut errors, &format!("trigger '{}'", trigger.name));
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn room(id: &str) -> RoomDef {
        RoomDef {
            id: id.to_string(),
            name: format!("Room {id}"),
            desc: "Test room".into(),
            visited: false,
            exits: Vec::new(),
            overlays: Vec::new(),
        }
    }

    fn base_world() -> WorldDef {
        WorldDef {
            game: GameDef {
                title: "Demo".into(),
                intro: "Intro".into(),
                player: PlayerDef {
                    name: "Player".into(),
                    description: "A hero".into(),
                    start_room: "start".into(),
                    max_hp: 10,
                },
                scoring: ScoringDef::default(),
                ..GameDef::default()
            },
            rooms: vec![room("start")],
            ..WorldDef::default()
        }
    }

    fn item_in_room(id: &str, room_id: &str) -> ItemDef {
        ItemDef {
            id: id.to_string(),
            name: format!("Item {id}"),
            desc: "Test item".into(),
            movability: Movability::Free,
            container_state: None,
            location: LocationRef::Room(room_id.to_string()),
            abilities: Vec::new(),
            interaction_requires: BTreeMap::new(),
            text: None,
            consumable: None,
        }
    }

    fn trigger_with_condition(name: &str, condition: ConditionExpr) -> TriggerDef {
        TriggerDef {
            name: name.to_string(),
            note: None,
            only_once: false,
            event: EventDef::Always,
            conditions: condition,
            actions: Vec::new(),
        }
    }

    #[test]
    fn duplicate_ids_are_reported() {
        let mut world = base_world();
        world.rooms = vec![room("same"), room("same")];

        let errors = validate_world(&world);
        assert!(
            errors
                .iter()
                .any(|err| matches!(err, ValidationError::DuplicateId { kind, id } if *kind == "room" && id == "same"))
        );
    }

    #[test]
    fn missing_references_are_reported() {
        let mut world = base_world();
        world.items = vec![item_in_room("lantern", "missing_room")];

        let errors = validate_world(&world);
        assert!(errors.iter().any(|err| matches!(err, ValidationError::MissingReference { kind, id, .. } if *kind == "room" && id == "missing_room")));
    }

    #[test]
    fn invalid_chance_percent_is_reported() {
        let mut world = base_world();
        world.triggers = vec![trigger_with_condition(
            "chance",
            ConditionExpr::Pred(ConditionDef::ChancePercent { percent: 0.0 }),
        )];

        let errors = validate_world(&world);
        assert!(
            errors
                .iter()
                .any(|err| matches!(err, ValidationError::InvalidValue { .. }))
        );
    }
}

struct IdSets<'a> {
    rooms: &'a HashSet<String>,
    items: &'a HashSet<String>,
    npcs: &'a HashSet<String>,
    spinners: &'a HashSet<String>,
    goals: &'a HashSet<String>,
}

fn track_ids<'a>(
    kind: &'static str,
    ids: impl Iterator<Item = &'a str>,
    set: &mut HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    for id in ids {
        if !set.insert(id.to_string()) {
            errors.push(ValidationError::DuplicateId {
                kind,
                id: id.to_string(),
            });
        }
    }
}

fn check_ref(kind: &'static str, id: &str, set: &HashSet<String>, context: String, errors: &mut Vec<ValidationError>) {
    if !set.contains(id) {
        errors.push(ValidationError::MissingReference {
            kind,
            id: id.to_string(),
            context,
        });
    }
}

fn validate_location(loc: &LocationRef, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    match loc {
        LocationRef::Inventory | LocationRef::Nowhere => {},
        LocationRef::Room(room) => {
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        LocationRef::Item(item) => {
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        LocationRef::Npc(npc) => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
        },
    }
}

fn validate_overlay_condition(
    cond: &OverlayCondDef,
    ids: &IdSets<'_>,
    errors: &mut Vec<ValidationError>,
    room_id: &str,
) {
    let context = format!("room '{room_id}' overlay");
    match cond {
        OverlayCondDef::FlagSet { .. } | OverlayCondDef::FlagUnset { .. } | OverlayCondDef::FlagComplete { .. } => {},
        OverlayCondDef::ItemPresent { item }
        | OverlayCondDef::ItemAbsent { item }
        | OverlayCondDef::PlayerHasItem { item }
        | OverlayCondDef::PlayerMissingItem { item } => {
            check_ref("item", item, ids.items, context, errors);
        },
        OverlayCondDef::NpcPresent { npc }
        | OverlayCondDef::NpcAbsent { npc }
        | OverlayCondDef::NpcInState { npc, .. } => {
            check_ref("npc", npc, ids.npcs, context, errors);
        },
        OverlayCondDef::ItemInRoom { item, room } => {
            check_ref("item", item, ids.items, context.clone(), errors);
            check_ref("room", room, ids.rooms, context, errors);
        },
    }
}

fn validate_item_ability(ability: &ItemAbility, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    if let ItemAbility::Unlock(Some(item)) = ability {
        check_ref("item", item, ids.items, context.to_string(), errors);
    }
}

fn validate_goal_condition(cond: &GoalCondition, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    match cond {
        GoalCondition::HasItem { item } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        GoalCondition::ReachedRoom { room } => {
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        GoalCondition::GoalComplete { goal_id } => {
            check_ref("goal", goal_id, ids.goals, context.to_string(), errors);
        },
        GoalCondition::FlagComplete { .. }
        | GoalCondition::FlagInProgress { .. }
        | GoalCondition::HasFlag { .. }
        | GoalCondition::MissingFlag { .. } => {},
    }
}

fn validate_event(event: &EventDef, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    match event {
        EventDef::Always | EventDef::PlayerDeath => {},
        EventDef::EnterRoom { room } | EventDef::LeaveRoom { room } => {
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        EventDef::TakeItem { item }
        | EventDef::DropItem { item }
        | EventDef::LookAtItem { item }
        | EventDef::OpenItem { item }
        | EventDef::UnlockItem { item }
        | EventDef::TouchItem { item } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        EventDef::TalkToNpc { npc } | EventDef::NpcDeath { npc } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
        },
        EventDef::UseItem { item, ability } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
            validate_item_ability(ability, ids, errors, context);
        },
        EventDef::UseItemOnItem {
            tool,
            target,
            interaction: _,
        } => {
            check_ref("item", tool, ids.items, context.to_string(), errors);
            check_ref("item", target, ids.items, context.to_string(), errors);
        },
        EventDef::ActOnItem { target, action: _ } => {
            check_ref("item", target, ids.items, context.to_string(), errors);
        },
        EventDef::GiveToNpc { item, npc } | EventDef::TakeFromNpc { item, npc } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
        },
        EventDef::InsertItemInto { item, container } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
            check_ref("item", container, ids.items, context.to_string(), errors);
        },
        EventDef::Ingest { item, .. } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
    }
}

fn validate_condition_expr(expr: &ConditionExpr, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    match expr {
        ConditionExpr::All(kids) | ConditionExpr::Any(kids) => {
            for kid in kids {
                validate_condition_expr(kid, ids, errors, context);
            }
        },
        ConditionExpr::Pred(cond) => {
            validate_condition(cond, ids, errors, context);
        },
    }
}

fn validate_condition(cond: &ConditionDef, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    match cond {
        ConditionDef::HasItem { item } | ConditionDef::MissingItem { item } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        ConditionDef::HasVisited { room } | ConditionDef::PlayerInRoom { room } => {
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        ConditionDef::WithNpc { npc } | ConditionDef::NpcInState { npc, .. } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
        },
        ConditionDef::NpcHasItem { npc, item } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        ConditionDef::ContainerHasItem { container, item } => {
            check_ref("item", container, ids.items, context.to_string(), errors);
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        ConditionDef::Ambient { spinner, rooms } => {
            check_ref("spinner", spinner, ids.spinners, context.to_string(), errors);
            if let Some(rooms) = rooms {
                for room in rooms {
                    check_ref("room", room, ids.rooms, context.to_string(), errors);
                }
            }
        },
        ConditionDef::ChancePercent { percent } => {
            if *percent <= 0.0 {
                errors.push(ValidationError::InvalidValue {
                    context: format!("{context}: chance percent <= 0"),
                });
            }
        },
        ConditionDef::HasFlag { .. }
        | ConditionDef::MissingFlag { .. }
        | ConditionDef::FlagInProgress { .. }
        | ConditionDef::FlagComplete { .. } => {},
    }
}

fn validate_action(action: &ActionDef, ids: &IdSets<'_>, errors: &mut Vec<ValidationError>, context: &str) {
    match &action.action {
        ActionKind::ShowMessage { .. }
        | ActionKind::DenyRead { .. }
        | ActionKind::AddFlag { .. }
        | ActionKind::AdvanceFlag { .. }
        | ActionKind::RemoveFlag { .. }
        | ActionKind::ResetFlag { .. }
        | ActionKind::AwardPoints { .. }
        | ActionKind::DamagePlayer { .. }
        | ActionKind::DamagePlayerOT { .. }
        | ActionKind::HealPlayer { .. }
        | ActionKind::HealPlayerOT { .. }
        | ActionKind::RemovePlayerEffect { .. } => {},
        ActionKind::DamageNpc { npc, .. }
        | ActionKind::DamageNpcOT { npc, .. }
        | ActionKind::HealNpc { npc, .. }
        | ActionKind::HealNpcOT { npc, .. }
        | ActionKind::RemoveNpcEffect { npc, .. }
        | ActionKind::SetNpcActive { npc, .. }
        | ActionKind::SetNpcState { npc, .. }
        | ActionKind::NpcSays { npc, .. }
        | ActionKind::NpcSaysRandom { npc }
        | ActionKind::NpcRefuseItem { npc, .. } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
        },
        ActionKind::GiveItemToPlayer { npc, item } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        ActionKind::PushPlayerTo { room } => {
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        ActionKind::AddSpinnerWedge { spinner, .. } | ActionKind::SpinnerMessage { spinner } => {
            check_ref("spinner", spinner, ids.spinners, context.to_string(), errors);
        },
        ActionKind::SpawnItemCurrentRoom { item }
        | ActionKind::SpawnItemInInventory { item }
        | ActionKind::DespawnItem { item }
        | ActionKind::LockItem { item }
        | ActionKind::UnlockItem { item }
        | ActionKind::SetContainerState { item, .. }
        | ActionKind::SetItemDescription { item, .. }
        | ActionKind::SetItemMovability { item, .. } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
        },
        ActionKind::SpawnItemInRoom { item, room } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        ActionKind::SpawnNpcInRoom { npc, room } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
            check_ref("room", room, ids.rooms, context.to_string(), errors);
        },
        ActionKind::SpawnItemInContainer { item, container } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
            check_ref("item", container, ids.items, context.to_string(), errors);
        },
        ActionKind::DespawnNpc { npc } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
        },
        ActionKind::ReplaceItem { old_item, new_item } | ActionKind::ReplaceDropItem { old_item, new_item } => {
            check_ref("item", old_item, ids.items, context.to_string(), errors);
            check_ref("item", new_item, ids.items, context.to_string(), errors);
        },
        ActionKind::LockExit { from_room, .. } | ActionKind::UnlockExit { from_room, .. } => {
            check_ref("room", from_room, ids.rooms, context.to_string(), errors);
        },
        ActionKind::RevealExit { exit_from, exit_to, .. } | ActionKind::SetBarredMessage { exit_from, exit_to, .. } => {
            check_ref("room", exit_from, ids.rooms, context.to_string(), errors);
            check_ref("room", exit_to, ids.rooms, context.to_string(), errors);
        },
        ActionKind::ModifyItem { item, patch } => {
            check_ref("item", item, ids.items, context.to_string(), errors);
            for ability in &patch.add_abilities {
                validate_item_ability(ability, ids, errors, context);
            }
            for ability in &patch.remove_abilities {
                validate_item_ability(ability, ids, errors, context);
            }
        },
        ActionKind::ModifyRoom { room, patch } => {
            check_ref("room", room, ids.rooms, context.to_string(), errors);
            for room_id in &patch.remove_exits {
                check_ref("room", room_id, ids.rooms, context.to_string(), errors);
            }
            for exit in &patch.add_exits {
                check_ref("room", &exit.to, ids.rooms, context.to_string(), errors);
                for item in &exit.required_items {
                    check_ref("item", item, ids.items, context.to_string(), errors);
                }
            }
        },
        ActionKind::ModifyNpc { npc, patch } => {
            check_ref("npc", npc, ids.npcs, context.to_string(), errors);
            if let Some(movement) = &patch.movement {
                if let Some(route) = &movement.route {
                    for room in route {
                        check_ref("room", room, ids.rooms, context.to_string(), errors);
                    }
                }
                if let Some(random_rooms) = &movement.random_rooms {
                    for room in random_rooms {
                        check_ref("room", room, ids.rooms, context.to_string(), errors);
                    }
                }
            }
        },
        ActionKind::Conditional { condition, actions } => {
            validate_condition_expr(condition, ids, errors, context);
            for action in actions {
                validate_action(action, ids, errors, context);
            }
        },
        ActionKind::ScheduleIn { actions, .. } | ActionKind::ScheduleOn { actions, .. } => {
            for action in actions {
                validate_action(action, ids, errors, context);
            }
        },
        ActionKind::ScheduleInIf { condition, actions, .. } | ActionKind::ScheduleOnIf { condition, actions, .. } => {
            validate_condition_expr(condition, ids, errors, context);
            for action in actions {
                validate_action(action, ids, errors, context);
            }
        },
    }
}
