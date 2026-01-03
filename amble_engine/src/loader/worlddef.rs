//! WorldDef loader and conversion helpers.
//!
//! Converts the serialized `WorldDef` data model into runtime engine structs.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use gametools::{Spinner, Wedge};

use amble_data::{
    ActionDef, ActionKind, ConditionDef, ConditionExpr, ConsumableDef, ConsumeTypeDef,
    ContainerState as DefContainerState, EventDef, ExitDef, FlagDef, GoalCondition as DefGoalCondition, GoalDef,
    GoalGroup as DefGoalGroup, IngestMode, ItemAbility as DefItemAbility, ItemDef,
    ItemInteractionType as DefItemInteractionType, ItemPatchDef, LocationRef, Movability as DefMovability, NpcDef,
    NpcDialoguePatchDef, NpcMovementDef, NpcMovementPatchDef, NpcMovementTiming, NpcMovementType, NpcPatchDef,
    NpcState as DefNpcState, OnFalsePolicy as DefOnFalsePolicy, OverlayCondDef, OverlayDef, RoomDef, RoomExitPatchDef,
    RoomPatchDef, SpinnerDef, TriggerDef, WorldDef,
};

use crate::goal::{Goal, GoalCondition, GoalGroup};
use crate::health::HealthState;
use crate::item::{
    ConsumableOpts, ConsumeType, ContainerState, IngestMode as EngineIngestMode, Item, ItemAbility,
    ItemInteractionType, Movability,
};
use crate::loader::spinners::create_default_spinners;
use crate::npc::{MovementTiming, MovementType, Npc, NpcMovement, NpcState};
use crate::player::Flag;
use crate::room::{Exit, OverlayCondition, Room, RoomOverlay};
use crate::scheduler::{EventCondition, OnFalsePolicy};
use crate::spinners::SpinnerType;
use crate::trigger::{ScriptedAction, Trigger, TriggerAction, TriggerCondition};
use crate::world::{AmbleWorld, Location};

/// Load a `WorldDef` from a RON file.
pub fn load_worlddef(path: &Path) -> Result<WorldDef> {
    let text = fs::read_to_string(path).with_context(|| format!("reading worlddef from '{}'", path.display()))?;
    ron::from_str(&text).with_context(|| format!("parsing worlddef RON from '{}'", path.display()))
}

/// Convert a `WorldDef` into a populated `AmbleWorld` (player/scoring loaded separately).
pub fn build_world_from_def(def: &WorldDef) -> Result<AmbleWorld> {
    let mut world = AmbleWorld::new_empty();

    world.spinners = build_spinners(&def.spinners);

    for room_def in &def.rooms {
        let room = room_from_def(room_def)?;
        world.rooms.insert(room.id.clone(), room);
    }
    world.max_score = world.rooms.len();

    for npc_def in &def.npcs {
        let npc = npc_from_def(npc_def)?;
        world.npcs.insert(npc.id.clone(), npc);
    }

    for item_def in &def.items {
        let item = item_from_def(item_def)?;
        world.items.insert(item.id.clone(), item);
    }

    world.triggers = def.triggers.iter().map(trigger_from_def).collect::<Result<Vec<_>>>()?;

    world.goals = def.goals.iter().map(goal_from_def).collect::<Result<Vec<_>>>()?;

    Ok(world)
}

fn build_spinners(defs: &[SpinnerDef]) -> HashMap<SpinnerType, Spinner<String>> {
    let mut spinners = create_default_spinners();
    for def in defs {
        let spinner_type = SpinnerType::from_toml_key(&def.id);
        let wedges: Vec<Wedge<String>> = def
            .wedges
            .iter()
            .map(|w| Wedge::new_weighted(w.text.clone(), w.width))
            .collect();
        spinners.insert(spinner_type, Spinner::new(wedges));
    }
    spinners
}

fn room_from_def(def: &RoomDef) -> Result<Room> {
    let mut exits = HashMap::new();
    for exit in &def.exits {
        exits.insert(exit.direction.clone(), exit_from_def(exit));
    }
    let overlays = def.overlays.iter().map(overlay_from_def).collect::<Result<Vec<_>>>()?;

    Ok(Room {
        id: def.id.clone(),
        symbol: def.id.clone(),
        name: def.name.clone(),
        base_description: def.desc.clone(),
        overlays,
        location: Location::Nowhere,
        visited: def.visited,
        exits,
        contents: HashSet::new(),
        npcs: HashSet::new(),
    })
}

fn exit_from_def(def: &ExitDef) -> Exit {
    Exit {
        to: def.to.clone(),
        hidden: def.hidden,
        locked: def.locked,
        required_flags: def.required_flags.iter().map(|flag| Flag::simple(flag, 0)).collect(),
        required_items: def.required_items.iter().cloned().collect(),
        barred_message: def.barred_message.clone(),
    }
}

fn overlay_from_def(def: &OverlayDef) -> Result<RoomOverlay> {
    let conditions = def
        .conditions
        .iter()
        .map(overlay_condition_from_def)
        .collect::<Result<Vec<_>>>()?;
    Ok(RoomOverlay {
        conditions,
        text: def.text.clone(),
    })
}

fn overlay_condition_from_def(def: &OverlayCondDef) -> Result<OverlayCondition> {
    Ok(match def {
        OverlayCondDef::FlagSet { flag } => OverlayCondition::FlagSet { flag: flag.clone() },
        OverlayCondDef::FlagUnset { flag } => OverlayCondition::FlagUnset { flag: flag.clone() },
        OverlayCondDef::FlagComplete { flag } => OverlayCondition::FlagComplete { flag: flag.clone() },
        OverlayCondDef::ItemPresent { item } => OverlayCondition::ItemPresent { item_id: item.clone() },
        OverlayCondDef::ItemAbsent { item } => OverlayCondition::ItemAbsent { item_id: item.clone() },
        OverlayCondDef::PlayerHasItem { item } => OverlayCondition::PlayerHasItem { item_id: item.clone() },
        OverlayCondDef::PlayerMissingItem { item } => OverlayCondition::PlayerMissingItem { item_id: item.clone() },
        OverlayCondDef::NpcPresent { npc } => OverlayCondition::NpcPresent { npc_id: npc.clone() },
        OverlayCondDef::NpcAbsent { npc } => OverlayCondition::NpcAbsent { npc_id: npc.clone() },
        OverlayCondDef::NpcInState { npc, state } => OverlayCondition::NpcInState {
            npc_id: npc.clone(),
            mood: npc_state_from_def(state),
        },
        OverlayCondDef::ItemInRoom { item, room } => OverlayCondition::ItemInRoom {
            item_id: item.clone(),
            room_id: room.clone(),
        },
    })
}

fn item_from_def(def: &ItemDef) -> Result<Item> {
    let abilities = def.abilities.iter().map(item_ability_from_def).collect::<HashSet<_>>();
    let mut interaction_requires = HashMap::new();
    for (interaction, ability) in &def.interaction_requires {
        interaction_requires.insert(item_interaction_from_def(interaction), item_ability_from_def(ability));
    }
    let consumable = def.consumable.as_ref().map(consumable_from_def).transpose()?;

    Ok(Item {
        id: def.id.clone(),
        symbol: def.id.clone(),
        name: def.name.clone(),
        description: def.desc.clone(),
        location: location_from_ref(&def.location),
        movability: movability_from_def(&def.movability),
        container_state: def.container_state.as_ref().map(container_state_from_def),
        contents: HashSet::new(),
        abilities,
        interaction_requires,
        text: def.text.clone(),
        consumable,
    })
}

fn consumable_from_def(def: &ConsumableDef) -> Result<ConsumableOpts> {
    let consume_on = def.consume_on.iter().map(item_ability_from_def).collect::<HashSet<_>>();
    let when_consumed = match &def.when_consumed {
        ConsumeTypeDef::Despawn => ConsumeType::Despawn,
        ConsumeTypeDef::ReplaceInventory { replacement } => ConsumeType::ReplaceInventory {
            replacement: replacement.clone(),
        },
        ConsumeTypeDef::ReplaceCurrentRoom { replacement } => ConsumeType::ReplaceCurrentRoom {
            replacement: replacement.clone(),
        },
    };
    Ok(ConsumableOpts {
        uses_left: def.uses_left,
        consume_on,
        when_consumed,
    })
}

fn npc_from_def(def: &NpcDef) -> Result<Npc> {
    let dialogue = def
        .dialogue
        .iter()
        .map(|(state, lines)| (npc_state_from_def(state), lines.clone()))
        .collect::<HashMap<_, _>>();
    let movement = def.movement.as_ref().map(npc_movement_from_def).transpose()?;
    Ok(Npc {
        id: def.id.clone(),
        symbol: def.id.clone(),
        name: def.name.clone(),
        description: def.desc.clone(),
        location: location_from_ref(&def.location),
        inventory: HashSet::new(),
        dialogue,
        state: npc_state_from_def(&def.state),
        movement,
        health: HealthState::new_at_max(def.max_hp),
    })
}

fn npc_movement_from_def(def: &NpcMovementDef) -> Result<NpcMovement> {
    let active = def.active.unwrap_or(true);
    let loop_route = def.loop_route.unwrap_or(true);
    let timing = match def.timing {
        Some(NpcMovementTiming::EveryNTurns { turns }) => MovementTiming::EveryNTurns { turns },
        Some(NpcMovementTiming::OnTurn { turn }) => MovementTiming::OnTurn { turn },
        None => MovementTiming::EveryNTurns { turns: 1 },
    };
    let movement_type = match def.movement_type {
        NpcMovementType::Route => MovementType::Route {
            rooms: def.rooms.clone(),
            current_idx: 0,
            loop_route,
        },
        NpcMovementType::RandomSet => MovementType::RandomSet {
            rooms: def.rooms.iter().cloned().collect(),
        },
    };
    Ok(NpcMovement {
        movement_type,
        timing,
        active,
        last_moved_turn: 0,
        paused_until: None,
    })
}

fn trigger_from_def(def: &TriggerDef) -> Result<Trigger> {
    let event_condition = event_condition_from_def(&def.event);
    let mut conditions = condition_expr_from_def(&def.conditions);
    if let Some(event_tc) = event_condition {
        conditions = EventCondition::All(vec![EventCondition::Trigger(event_tc), conditions]);
    }
    let actions = def
        .actions
        .iter()
        .map(scripted_action_from_def)
        .collect::<Result<Vec<_>>>()?;
    Ok(Trigger {
        name: def.name.clone(),
        conditions,
        actions,
        only_once: def.only_once,
        fired: false,
    })
}

fn goal_from_def(def: &GoalDef) -> Result<Goal> {
    Ok(Goal {
        id: def.id.clone(),
        name: def.name.clone(),
        description: def.description.clone(),
        group: goal_group_from_def(def.group),
        activate_when: def.activate_when.as_ref().map(goal_condition_from_def),
        finished_when: goal_condition_from_def(&def.finished_when),
        failed_when: def.failed_when.as_ref().map(goal_condition_from_def),
    })
}

fn scripted_action_from_def(def: &ActionDef) -> Result<ScriptedAction> {
    let action = action_from_def(&def.action)?;
    Ok(ScriptedAction::with_priority(action, def.priority))
}

fn action_from_def(def: &ActionKind) -> Result<TriggerAction> {
    Ok(match def {
        ActionKind::ShowMessage { text } => TriggerAction::ShowMessage(text.clone()),
        ActionKind::AddFlag { flag } => TriggerAction::AddFlag(flag_from_def(flag)),
        ActionKind::AdvanceFlag { name } => TriggerAction::AdvanceFlag(name.clone()),
        ActionKind::RemoveFlag { name } => TriggerAction::RemoveFlag(name.clone()),
        ActionKind::ResetFlag { name } => TriggerAction::ResetFlag(name.clone()),
        ActionKind::AwardPoints { amount, reason } => TriggerAction::AwardPoints {
            amount: *amount,
            reason: reason.clone(),
        },
        ActionKind::DamagePlayer { amount, cause } => TriggerAction::DamagePlayer {
            amount: *amount,
            cause: cause.clone(),
        },
        ActionKind::DamagePlayerOT { amount, turns, cause } => TriggerAction::DamagePlayerOT {
            amount: *amount,
            turns: *turns,
            cause: cause.clone(),
        },
        ActionKind::HealPlayer { amount, cause } => TriggerAction::HealPlayer {
            amount: *amount,
            cause: cause.clone(),
        },
        ActionKind::HealPlayerOT { amount, turns, cause } => TriggerAction::HealPlayerOT {
            amount: *amount,
            turns: *turns,
            cause: cause.clone(),
        },
        ActionKind::RemovePlayerEffect { cause } => TriggerAction::RemovePlayerEffect { cause: cause.clone() },
        ActionKind::DamageNpc { npc, amount, cause } => TriggerAction::DamageNpc {
            npc_id: npc.clone(),
            amount: *amount,
            cause: cause.clone(),
        },
        ActionKind::DamageNpcOT {
            npc,
            amount,
            turns,
            cause,
        } => TriggerAction::DamageNpcOT {
            npc_id: npc.clone(),
            amount: *amount,
            turns: *turns,
            cause: cause.clone(),
        },
        ActionKind::HealNpc { npc, amount, cause } => TriggerAction::HealNpc {
            npc_id: npc.clone(),
            amount: *amount,
            cause: cause.clone(),
        },
        ActionKind::HealNpcOT {
            npc,
            amount,
            turns,
            cause,
        } => TriggerAction::HealNpcOT {
            npc_id: npc.clone(),
            amount: *amount,
            turns: *turns,
            cause: cause.clone(),
        },
        ActionKind::RemoveNpcEffect { npc, cause } => TriggerAction::RemoveNpcEffect {
            npc_id: npc.clone(),
            cause: cause.clone(),
        },
        ActionKind::SetNpcActive { npc, active } => TriggerAction::SetNpcActive {
            npc_id: npc.clone(),
            active: *active,
        },
        ActionKind::SetNpcState { npc, state } => TriggerAction::SetNPCState {
            npc_id: npc.clone(),
            state: npc_state_from_def(state),
        },
        ActionKind::NpcSays { npc, quote } => TriggerAction::NpcSays {
            npc_id: npc.clone(),
            quote: quote.clone(),
        },
        ActionKind::NpcSaysRandom { npc } => TriggerAction::NpcSaysRandom { npc_id: npc.clone() },
        ActionKind::NpcRefuseItem { npc, reason } => TriggerAction::NpcRefuseItem {
            npc_id: npc.clone(),
            reason: reason.clone(),
        },
        ActionKind::GiveItemToPlayer { npc, item } => TriggerAction::GiveItemToPlayer {
            npc_id: npc.clone(),
            item_id: item.clone(),
        },
        ActionKind::PushPlayerTo { room } => TriggerAction::PushPlayerTo(room.clone()),
        ActionKind::AddSpinnerWedge { spinner, text, width } => TriggerAction::AddSpinnerWedge {
            spinner: SpinnerType::from_toml_key(spinner),
            text: text.clone(),
            width: *width,
        },
        ActionKind::SpinnerMessage { spinner } => TriggerAction::SpinnerMessage {
            spinner: SpinnerType::from_toml_key(spinner),
        },
        ActionKind::DenyRead { reason } => TriggerAction::DenyRead(reason.clone()),
        ActionKind::SpawnItemCurrentRoom { item } => TriggerAction::SpawnItemCurrentRoom(item.clone()),
        ActionKind::SpawnItemInRoom { item, room } => TriggerAction::SpawnItemInRoom {
            item_id: item.clone(),
            room_id: room.clone(),
        },
        ActionKind::SpawnItemInInventory { item } => TriggerAction::SpawnItemInInventory(item.clone()),
        ActionKind::SpawnItemInContainer { item, container } => TriggerAction::SpawnItemInContainer {
            item_id: item.clone(),
            container_id: container.clone(),
        },
        ActionKind::SpawnNpcInRoom { npc, room } => TriggerAction::SpawnNpcInRoom {
            npc_id: npc.clone(),
            room_id: room.clone(),
        },
        ActionKind::DespawnItem { item } => TriggerAction::DespawnItem { item_id: item.clone() },
        ActionKind::DespawnNpc { npc } => TriggerAction::DespawnNpc { npc_id: npc.clone() },
        ActionKind::ReplaceItem { old_item, new_item } => TriggerAction::ReplaceItem {
            old_id: old_item.clone(),
            new_id: new_item.clone(),
        },
        ActionKind::ReplaceDropItem { old_item, new_item } => TriggerAction::ReplaceDropItem {
            old_id: old_item.clone(),
            new_id: new_item.clone(),
        },
        ActionKind::LockItem { item } => TriggerAction::LockItem(item.clone()),
        ActionKind::UnlockItem { item } => TriggerAction::UnlockItem(item.clone()),
        ActionKind::SetContainerState { item, state } => TriggerAction::SetContainerState {
            item_id: item.clone(),
            state: state.as_ref().map(container_state_from_def),
        },
        ActionKind::SetItemDescription { item, text } => TriggerAction::SetItemDescription {
            item_id: item.clone(),
            text: text.clone(),
        },
        ActionKind::SetItemMovability { item, movability } => TriggerAction::SetItemMovability {
            item_id: item.clone(),
            movability: movability_from_def(movability),
        },
        ActionKind::LockExit { from_room, direction } => TriggerAction::LockExit {
            from_room: from_room.clone(),
            direction: direction.clone(),
        },
        ActionKind::UnlockExit { from_room, direction } => TriggerAction::UnlockExit {
            from_room: from_room.clone(),
            direction: direction.clone(),
        },
        ActionKind::RevealExit {
            exit_from,
            exit_to,
            direction,
        } => TriggerAction::RevealExit {
            exit_from: exit_from.clone(),
            exit_to: exit_to.clone(),
            direction: direction.clone(),
        },
        ActionKind::SetBarredMessage {
            exit_from,
            exit_to,
            msg,
        } => TriggerAction::SetBarredMessage {
            exit_from: exit_from.clone(),
            exit_to: exit_to.clone(),
            msg: msg.clone(),
        },
        ActionKind::ModifyItem { item, patch } => TriggerAction::ModifyItem {
            item_id: item.clone(),
            patch: item_patch_from_def(patch),
        },
        ActionKind::ModifyRoom { room, patch } => TriggerAction::ModifyRoom {
            room_id: room.clone(),
            patch: room_patch_from_def(patch),
        },
        ActionKind::ModifyNpc { npc, patch } => TriggerAction::ModifyNpc {
            npc_id: npc.clone(),
            patch: npc_patch_from_def(patch),
        },
        ActionKind::Conditional { condition, actions } => TriggerAction::Conditional {
            condition: condition_expr_from_def(condition),
            actions: actions
                .iter()
                .map(scripted_action_from_def)
                .collect::<Result<Vec<_>>>()?,
        },
        ActionKind::ScheduleIn {
            turns_ahead,
            actions,
            note,
        } => TriggerAction::ScheduleIn {
            turns_ahead: *turns_ahead,
            actions: actions
                .iter()
                .map(scripted_action_from_def)
                .collect::<Result<Vec<_>>>()?,
            note: note.clone(),
        },
        ActionKind::ScheduleOn { on_turn, actions, note } => TriggerAction::ScheduleOn {
            on_turn: *on_turn,
            actions: actions
                .iter()
                .map(scripted_action_from_def)
                .collect::<Result<Vec<_>>>()?,
            note: note.clone(),
        },
        ActionKind::ScheduleInIf {
            turns_ahead,
            condition,
            on_false,
            actions,
            note,
        } => TriggerAction::ScheduleInIf {
            turns_ahead: *turns_ahead,
            condition: condition_expr_from_def(condition),
            on_false: on_false_from_def(on_false),
            actions: actions
                .iter()
                .map(scripted_action_from_def)
                .collect::<Result<Vec<_>>>()?,
            note: note.clone(),
        },
        ActionKind::ScheduleOnIf {
            on_turn,
            condition,
            on_false,
            actions,
            note,
        } => TriggerAction::ScheduleOnIf {
            on_turn: *on_turn,
            condition: condition_expr_from_def(condition),
            on_false: on_false_from_def(on_false),
            actions: actions
                .iter()
                .map(scripted_action_from_def)
                .collect::<Result<Vec<_>>>()?,
            note: note.clone(),
        },
    })
}

fn item_patch_from_def(def: &ItemPatchDef) -> crate::trigger::ItemPatch {
    crate::trigger::ItemPatch {
        name: def.name.clone(),
        desc: def.desc.clone(),
        text: def.text.clone(),
        movability: def.movability.as_ref().map(movability_from_def),
        container_state: def.container_state.as_ref().map(container_state_from_def),
        remove_container_state: def.remove_container_state,
        add_abilities: def.add_abilities.iter().map(item_ability_from_def).collect(),
        remove_abilities: def.remove_abilities.iter().map(item_ability_from_def).collect(),
    }
}

fn room_patch_from_def(def: &RoomPatchDef) -> crate::trigger::RoomPatch {
    crate::trigger::RoomPatch {
        name: def.name.clone(),
        desc: def.desc.clone(),
        remove_exits: def.remove_exits.clone(),
        add_exits: def.add_exits.iter().map(room_exit_patch_from_def).collect(),
    }
}

fn room_exit_patch_from_def(def: &RoomExitPatchDef) -> crate::trigger::RoomExitPatch {
    crate::trigger::RoomExitPatch {
        direction: def.direction.clone(),
        to: def.to.clone(),
        hidden: def.hidden,
        locked: def.locked,
        required_flags: def.required_flags.iter().map(|flag| Flag::simple(flag, 0)).collect(),
        required_items: def.required_items.iter().cloned().collect(),
        barred_message: def.barred_message.clone(),
    }
}

fn npc_patch_from_def(def: &NpcPatchDef) -> crate::trigger::NpcPatch {
    crate::trigger::NpcPatch {
        name: def.name.clone(),
        desc: def.desc.clone(),
        state: def.state.as_ref().map(npc_state_from_def),
        add_lines: def.add_lines.iter().map(npc_dialogue_patch_from_def).collect(),
        movement: def.movement.as_ref().map(npc_movement_patch_from_def),
    }
}

fn npc_dialogue_patch_from_def(def: &NpcDialoguePatchDef) -> crate::trigger::NpcDialoguePatch {
    crate::trigger::NpcDialoguePatch {
        state: npc_state_from_def(&def.state),
        line: def.line.clone(),
    }
}

fn npc_movement_patch_from_def(def: &NpcMovementPatchDef) -> crate::trigger::NpcMovementPatch {
    crate::trigger::NpcMovementPatch {
        route: def.route.clone(),
        random_rooms: def.random_rooms.as_ref().map(|rooms| rooms.iter().cloned().collect()),
        timing: def.timing.as_ref().map(npc_timing_patch_from_def),
        active: def.active,
        loop_route: def.loop_route,
    }
}

fn npc_timing_patch_from_def(def: &amble_data::NpcTimingPatchDef) -> MovementTiming {
    match def {
        amble_data::NpcTimingPatchDef::EveryNTurns { turns } => MovementTiming::EveryNTurns { turns: *turns },
        amble_data::NpcTimingPatchDef::OnTurn { turn } => MovementTiming::OnTurn { turn: *turn },
    }
}

fn goal_group_from_def(def: DefGoalGroup) -> GoalGroup {
    match def {
        DefGoalGroup::Required => GoalGroup::Required,
        DefGoalGroup::Optional => GoalGroup::Optional,
        DefGoalGroup::StatusEffect => GoalGroup::StatusEffect,
    }
}

fn goal_condition_from_def(def: &DefGoalCondition) -> GoalCondition {
    match def {
        DefGoalCondition::FlagComplete { flag } => GoalCondition::FlagComplete { flag: flag.clone() },
        DefGoalCondition::FlagInProgress { flag } => GoalCondition::FlagInProgress { flag: flag.clone() },
        DefGoalCondition::GoalComplete { goal_id } => GoalCondition::GoalComplete {
            goal_id: goal_id.clone(),
        },
        DefGoalCondition::HasItem { item } => GoalCondition::HasItem { item_id: item.clone() },
        DefGoalCondition::HasFlag { flag } => GoalCondition::HasFlag { flag: flag.clone() },
        DefGoalCondition::MissingFlag { flag } => GoalCondition::MissingFlag { flag: flag.clone() },
        DefGoalCondition::ReachedRoom { room } => GoalCondition::ReachedRoom { room_id: room.clone() },
    }
}

fn condition_expr_from_def(def: &ConditionExpr) -> EventCondition {
    match def {
        ConditionExpr::All(list) => EventCondition::All(list.iter().map(condition_expr_from_def).collect()),
        ConditionExpr::Any(list) => EventCondition::Any(list.iter().map(condition_expr_from_def).collect()),
        ConditionExpr::Pred(pred) => EventCondition::Trigger(condition_from_def(pred)),
    }
}

fn condition_from_def(def: &ConditionDef) -> TriggerCondition {
    match def {
        ConditionDef::HasFlag { flag } => TriggerCondition::HasFlag(flag.clone()),
        ConditionDef::MissingFlag { flag } => TriggerCondition::MissingFlag(flag.clone()),
        ConditionDef::FlagInProgress { flag } => TriggerCondition::FlagInProgress(flag.clone()),
        ConditionDef::FlagComplete { flag } => TriggerCondition::FlagComplete(flag.clone()),
        ConditionDef::HasItem { item } => TriggerCondition::HasItem(item.clone()),
        ConditionDef::MissingItem { item } => TriggerCondition::MissingItem(item.clone()),
        ConditionDef::HasVisited { room } => TriggerCondition::HasVisited(room.clone()),
        ConditionDef::PlayerInRoom { room } => TriggerCondition::InRoom(room.clone()),
        ConditionDef::WithNpc { npc } => TriggerCondition::WithNpc(npc.clone()),
        ConditionDef::NpcHasItem { npc, item } => TriggerCondition::NpcHasItem {
            npc_id: npc.clone(),
            item_id: item.clone(),
        },
        ConditionDef::NpcInState { npc, state } => TriggerCondition::NpcInState {
            npc_id: npc.clone(),
            mood: npc_state_from_def(state),
        },
        ConditionDef::ContainerHasItem { container, item } => TriggerCondition::ContainerHasItem {
            container_id: container.clone(),
            item_id: item.clone(),
        },
        ConditionDef::ChancePercent { percent } => {
            let one_in = if *percent <= 0.0 {
                f64::INFINITY
            } else {
                100.0 / *percent
            };
            TriggerCondition::Chance { one_in }
        },
        ConditionDef::Ambient { spinner, rooms } => TriggerCondition::Ambient {
            room_ids: rooms
                .as_ref()
                .map(|list| list.iter().cloned().collect())
                .unwrap_or_default(),
            spinner: SpinnerType::from_toml_key(spinner),
        },
    }
}

fn event_condition_from_def(def: &EventDef) -> Option<TriggerCondition> {
    Some(match def {
        EventDef::Always => return None,
        EventDef::EnterRoom { room } => TriggerCondition::Enter(room.clone()),
        EventDef::LeaveRoom { room } => TriggerCondition::Leave(room.clone()),
        EventDef::TakeItem { item } => TriggerCondition::Take(item.clone()),
        EventDef::DropItem { item } => TriggerCondition::Drop(item.clone()),
        EventDef::LookAtItem { item } => TriggerCondition::LookAt(item.clone()),
        EventDef::OpenItem { item } => TriggerCondition::Open(item.clone()),
        EventDef::UnlockItem { item } => TriggerCondition::Unlock(item.clone()),
        EventDef::TouchItem { item } => TriggerCondition::Touch(item.clone()),
        EventDef::TalkToNpc { npc } => TriggerCondition::TalkToNpc(npc.clone()),
        EventDef::UseItem { item, ability } => TriggerCondition::UseItem {
            item_id: item.clone(),
            ability: item_ability_from_def(ability),
        },
        EventDef::UseItemOnItem {
            tool,
            target,
            interaction,
        } => TriggerCondition::UseItemOnItem {
            interaction: item_interaction_from_def(interaction),
            target_id: target.clone(),
            tool_id: tool.clone(),
        },
        EventDef::ActOnItem { target, action } => TriggerCondition::ActOnItem {
            target_id: target.clone(),
            action: item_interaction_from_def(action),
        },
        EventDef::GiveToNpc { item, npc } => TriggerCondition::GiveToNpc {
            item_id: item.clone(),
            npc_id: npc.clone(),
        },
        EventDef::TakeFromNpc { item, npc } => TriggerCondition::TakeFromNpc {
            item_id: item.clone(),
            npc_id: npc.clone(),
        },
        EventDef::InsertItemInto { item, container } => TriggerCondition::Insert {
            item: item.clone(),
            container: container.clone(),
        },
        EventDef::Ingest { item, mode } => TriggerCondition::Ingest {
            item_id: item.clone(),
            mode: ingest_mode_from_def(mode),
        },
        EventDef::PlayerDeath => TriggerCondition::PlayerDeath,
        EventDef::NpcDeath { npc } => TriggerCondition::NpcDeath(npc.clone()),
    })
}

fn location_from_ref(loc: &LocationRef) -> Location {
    match loc {
        LocationRef::Inventory => Location::Inventory,
        LocationRef::Nowhere => Location::Nowhere,
        LocationRef::Room(id) => Location::Room(id.clone()),
        LocationRef::Item(id) => Location::Item(id.clone()),
        LocationRef::Npc(id) => Location::Npc(id.clone()),
    }
}

fn movability_from_def(def: &DefMovability) -> Movability {
    match def {
        DefMovability::Fixed { reason } => Movability::Fixed { reason: reason.clone() },
        DefMovability::Restricted { reason } => Movability::Restricted { reason: reason.clone() },
        DefMovability::Free => Movability::Free,
    }
}

fn container_state_from_def(def: &DefContainerState) -> ContainerState {
    match def {
        DefContainerState::Open => ContainerState::Open,
        DefContainerState::Closed => ContainerState::Closed,
        DefContainerState::Locked => ContainerState::Locked,
        DefContainerState::TransparentOpen => ContainerState::TransparentOpen,
        DefContainerState::TransparentClosed => ContainerState::TransparentClosed,
        DefContainerState::TransparentLocked => ContainerState::TransparentLocked,
    }
}

fn item_ability_from_def(def: &DefItemAbility) -> ItemAbility {
    match def {
        DefItemAbility::Attach => ItemAbility::Attach,
        DefItemAbility::Clean => ItemAbility::Clean,
        DefItemAbility::Cut => ItemAbility::Cut,
        DefItemAbility::CutWood => ItemAbility::CutWood,
        DefItemAbility::Drink => ItemAbility::Drink,
        DefItemAbility::Eat => ItemAbility::Eat,
        DefItemAbility::Extinguish => ItemAbility::Extinguish,
        DefItemAbility::Ignite => ItemAbility::Ignite,
        DefItemAbility::Inhale => ItemAbility::Inhale,
        DefItemAbility::Insulate => ItemAbility::Insulate,
        DefItemAbility::Magnify => ItemAbility::Magnify,
        DefItemAbility::Pluck => ItemAbility::Pluck,
        DefItemAbility::Pry => ItemAbility::Pry,
        DefItemAbility::Read => ItemAbility::Read,
        DefItemAbility::Repair => ItemAbility::Repair,
        DefItemAbility::Sharpen => ItemAbility::Sharpen,
        DefItemAbility::Smash => ItemAbility::Smash,
        DefItemAbility::TurnOn => ItemAbility::TurnOn,
        DefItemAbility::TurnOff => ItemAbility::TurnOff,
        DefItemAbility::Unlock(target) => ItemAbility::Unlock(target.clone()),
        DefItemAbility::Use => ItemAbility::Use,
    }
}

fn item_interaction_from_def(def: &DefItemInteractionType) -> ItemInteractionType {
    match def {
        DefItemInteractionType::Attach => ItemInteractionType::Attach,
        DefItemInteractionType::Break => ItemInteractionType::Break,
        DefItemInteractionType::Burn => ItemInteractionType::Burn,
        DefItemInteractionType::Extinguish => ItemInteractionType::Extinguish,
        DefItemInteractionType::Clean => ItemInteractionType::Clean,
        DefItemInteractionType::Cover => ItemInteractionType::Cover,
        DefItemInteractionType::Cut => ItemInteractionType::Cut,
        DefItemInteractionType::Handle => ItemInteractionType::Handle,
        DefItemInteractionType::Move => ItemInteractionType::Move,
        DefItemInteractionType::Open => ItemInteractionType::Open,
        DefItemInteractionType::Repair => ItemInteractionType::Repair,
        DefItemInteractionType::Sharpen => ItemInteractionType::Sharpen,
        DefItemInteractionType::Turn => ItemInteractionType::Turn,
        DefItemInteractionType::Unlock => ItemInteractionType::Unlock,
    }
}

fn npc_state_from_def(def: &DefNpcState) -> NpcState {
    match def {
        DefNpcState::Bored => NpcState::Bored,
        DefNpcState::Happy => NpcState::Happy,
        DefNpcState::Mad => NpcState::Mad,
        DefNpcState::Normal => NpcState::Normal,
        DefNpcState::Sad => NpcState::Sad,
        DefNpcState::Tired => NpcState::Tired,
        DefNpcState::Custom(value) => NpcState::Custom(value.clone()),
    }
}

fn ingest_mode_from_def(def: &IngestMode) -> EngineIngestMode {
    match def {
        IngestMode::Eat => EngineIngestMode::Eat,
        IngestMode::Drink => EngineIngestMode::Drink,
        IngestMode::Inhale => EngineIngestMode::Inhale,
    }
}

fn flag_from_def(def: &FlagDef) -> Flag {
    match def {
        FlagDef::Simple { name } => Flag::simple(name, 0),
        FlagDef::Sequence { name, end } => Flag::sequence(name, *end, 0),
    }
}

fn on_false_from_def(def: &DefOnFalsePolicy) -> OnFalsePolicy {
    match def {
        DefOnFalsePolicy::Cancel => OnFalsePolicy::Cancel,
        DefOnFalsePolicy::RetryAfter { turns } => OnFalsePolicy::RetryAfter(*turns),
        DefOnFalsePolicy::RetryNextTurn => OnFalsePolicy::RetryNextTurn,
    }
}
