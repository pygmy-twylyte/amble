use std::collections::BTreeMap;

use amble_data::{
    ActionDef, ActionKind, ConditionDef, ConditionExpr, ConsumableDef, ConsumeTypeDef, ContainerState, EventDef,
    ExitDef, FlagDef, GameDef, GoalCondition, GoalDef, GoalGroup, IngestMode, ItemAbility, ItemDef,
    ItemInteractionType, ItemPatchDef, LocationRef, Movability, NpcDef, NpcDialoguePatchDef, NpcMovementDef,
    NpcMovementPatchDef, NpcMovementTiming, NpcMovementType, NpcPatchDef, NpcState, NpcTimingPatchDef, OnFalsePolicy,
    OverlayCondDef, OverlayDef, PlayerDef, RoomDef, RoomExitPatchDef, RoomPatchDef, ScoringDef, ScoringRankDef,
    SpinnerDef, SpinnerWedgeDef, TriggerDef, WorldDef,
};
use thiserror::Error;

use crate::{
    ActionAst, ActionStmt, ConditionAst, ConsumableAst, ConsumableWhenAst, ContainerStateAst, GameAst, GoalAst,
    GoalCondAst, GoalGroupAst, IngestModeAst, ItemAbilityAst, ItemAst, ItemLocationAst, MovabilityAst, NpcAst,
    NpcLocationAst, NpcMovementAst, NpcMovementTypeAst, NpcPatchAst, NpcStateValue, NpcTimingPatchAst, OnFalseAst,
    OverlayAst, OverlayCondAst, PlayerAst, RoomAst, RoomExitPatchAst, ScoringAst, ScoringRankAst, SpinnerAst,
    SpinnerWedgeAst, TriggerAst,
};

/// Errors emitted while lowering AST data into the WorldDef model.
#[derive(Debug, Error)]
pub enum WorldDefError {
    #[error("unknown item ability '{value}'")]
    UnknownItemAbility { value: String },
    #[error("unknown interaction '{value}'")]
    UnknownInteraction { value: String },
    #[error("unknown npc state '{value}'")]
    UnknownNpcState { value: String },
    #[error("invalid npc movement timing '{value}'")]
    InvalidNpcMovementTiming { value: String },
    #[error("invalid container state '{value}'")]
    InvalidContainerState { value: String },
    #[error("missing game block")]
    MissingGame,
    #[error("unsupported {kind}: {value}")]
    UnsupportedAst { kind: &'static str, value: String },
}

/// Convert parsed AST collections into a serialized `WorldDef`.
///
/// This performs structural mapping only; cross-reference validation is
/// handled separately via `amble_data::validate_world`.
///
/// # Errors
/// - Returns `WorldDefError` for unsupported or invalid AST values.
/// - Returns `WorldDefError::MissingGame` when no game block is provided.
pub fn worlddef_from_asts(
    game: Option<&GameAst>,
    triggers: &[TriggerAst],
    rooms: &[RoomAst],
    items: &[ItemAst],
    spinners: &[SpinnerAst],
    npcs: &[NpcAst],
    goals: &[GoalAst],
) -> Result<WorldDef, WorldDefError> {
    let game = game.ok_or(WorldDefError::MissingGame)?;
    let game = game_to_def(game)?;
    let rooms = rooms.iter().map(room_to_def).collect::<Result<Vec<_>, _>>()?;
    let items = items.iter().map(item_to_def).collect::<Result<Vec<_>, _>>()?;
    let spinners = spinners.iter().map(spinner_to_def).collect::<Result<Vec<_>, _>>()?;
    let npcs = npcs.iter().map(npc_to_def).collect::<Result<Vec<_>, _>>()?;
    let goals = goals.iter().map(goal_to_def).collect::<Result<Vec<_>, _>>()?;
    let triggers = triggers.iter().map(trigger_to_def).collect::<Result<Vec<_>, _>>()?;

    Ok(WorldDef {
        game,
        rooms,
        items,
        npcs,
        spinners,
        triggers,
        goals,
    })
}

fn game_to_def(game: &GameAst) -> Result<GameDef, WorldDefError> {
    let player = player_to_def(&game.player);
    let scoring = game.scoring.as_ref().map(scoring_to_def).transpose()?;
    Ok(GameDef {
        title: game.title.clone(),
        slug: game.slug.clone().unwrap_or_default(),
        author: game.author.clone().unwrap_or_default(),
        version: game.version.clone().unwrap_or_default(),
        blurb: game.blurb.clone().unwrap_or_default(),
        intro: game.intro.clone(),
        player,
        scoring: scoring.unwrap_or_default(),
    })
}

fn player_to_def(player: &PlayerAst) -> PlayerDef {
    PlayerDef {
        name: player.name.clone(),
        description: player.description.clone(),
        start_room: player.start_room.clone(),
        max_hp: player.max_hp,
    }
}

fn scoring_to_def(scoring: &ScoringAst) -> Result<ScoringDef, WorldDefError> {
    Ok(ScoringDef {
        report_title: scoring
            .report_title
            .clone()
            .unwrap_or_else(|| ScoringDef::default().report_title),
        ranks: scoring.ranks.iter().map(scoring_rank_to_def).collect::<Vec<_>>(),
    })
}

fn scoring_rank_to_def(rank: &ScoringRankAst) -> ScoringRankDef {
    ScoringRankDef {
        threshold: rank.threshold,
        name: rank.name.clone(),
        description: rank.description.clone(),
    }
}

fn room_to_def(room: &RoomAst) -> Result<RoomDef, WorldDefError> {
    let exits = room
        .exits
        .iter()
        .map(|(dir, exit)| {
            Ok(ExitDef {
                direction: dir.clone(),
                to: exit.to.clone(),
                hidden: exit.hidden,
                locked: exit.locked,
                required_flags: exit.required_flags.clone(),
                required_items: exit.required_items.clone(),
                barred_message: exit.barred_message.clone(),
            })
        })
        .collect::<Result<Vec<_>, WorldDefError>>()?;
    let overlays = room
        .overlays
        .iter()
        .map(overlay_to_def)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RoomDef {
        id: room.id.clone(),
        name: room.name.clone(),
        desc: room.desc.clone(),
        visited: room.visited,
        exits,
        overlays,
    })
}

fn overlay_to_def(overlay: &OverlayAst) -> Result<OverlayDef, WorldDefError> {
    let conditions = overlay
        .conditions
        .iter()
        .map(overlay_cond_to_def)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OverlayDef {
        conditions,
        text: overlay.text.clone(),
    })
}

fn overlay_cond_to_def(cond: &OverlayCondAst) -> Result<OverlayCondDef, WorldDefError> {
    Ok(match cond {
        OverlayCondAst::FlagSet(flag) => OverlayCondDef::FlagSet { flag: flag.clone() },
        OverlayCondAst::FlagUnset(flag) => OverlayCondDef::FlagUnset { flag: flag.clone() },
        OverlayCondAst::FlagComplete(flag) => OverlayCondDef::FlagComplete { flag: flag.clone() },
        OverlayCondAst::ItemPresent(item) => OverlayCondDef::ItemPresent { item: item.clone() },
        OverlayCondAst::ItemAbsent(item) => OverlayCondDef::ItemAbsent { item: item.clone() },
        OverlayCondAst::PlayerHasItem(item) => OverlayCondDef::PlayerHasItem { item: item.clone() },
        OverlayCondAst::PlayerMissingItem(item) => OverlayCondDef::PlayerMissingItem { item: item.clone() },
        OverlayCondAst::NpcPresent(npc) => OverlayCondDef::NpcPresent { npc: npc.clone() },
        OverlayCondAst::NpcAbsent(npc) => OverlayCondDef::NpcAbsent { npc: npc.clone() },
        OverlayCondAst::NpcInState { npc, state } => OverlayCondDef::NpcInState {
            npc: npc.clone(),
            state: npc_state_from_value(state)?,
        },
        OverlayCondAst::ItemInRoom { item, room } => OverlayCondDef::ItemInRoom {
            item: item.clone(),
            room: room.clone(),
        },
    })
}

fn item_to_def(item: &ItemAst) -> Result<ItemDef, WorldDefError> {
    let abilities = item
        .abilities
        .iter()
        .map(|ab| item_ability_from_ast(ab))
        .collect::<Result<Vec<_>, _>>()?;
    let interaction_requires = item
        .interaction_requires
        .iter()
        .map(|(interaction, ability)| {
            let interaction = item_interaction_from_str(interaction)?;
            let ability = item_ability_from_str(ability, None)?;
            Ok((interaction, ability))
        })
        .collect::<Result<BTreeMap<_, _>, WorldDefError>>()?;
    let consumable = match &item.consumable {
        Some(consumable) => Some(consumable_to_def(consumable)?),
        None => None,
    };

    Ok(ItemDef {
        id: item.id.clone(),
        name: item.name.clone(),
        desc: item.desc.clone(),
        movability: movability_from_ast(&item.movability),
        container_state: item.container_state.as_ref().map(container_state_from_ast),
        location: location_from_item_ast(&item.location),
        abilities,
        interaction_requires,
        text: item.text.clone(),
        consumable,
    })
}

fn consumable_to_def(consumable: &ConsumableAst) -> Result<ConsumableDef, WorldDefError> {
    let consume_on = consumable
        .consume_on
        .iter()
        .map(item_ability_from_ast)
        .collect::<Result<Vec<_>, _>>()?;
    let when_consumed = match &consumable.when_consumed {
        ConsumableWhenAst::Despawn => ConsumeTypeDef::Despawn,
        ConsumableWhenAst::ReplaceInventory { replacement } => ConsumeTypeDef::ReplaceInventory {
            replacement: replacement.clone(),
        },
        ConsumableWhenAst::ReplaceCurrentRoom { replacement } => ConsumeTypeDef::ReplaceCurrentRoom {
            replacement: replacement.clone(),
        },
    };

    Ok(ConsumableDef {
        uses_left: consumable.uses_left,
        consume_on,
        when_consumed,
    })
}

fn spinner_to_def(spinner: &SpinnerAst) -> Result<SpinnerDef, WorldDefError> {
    let wedges = spinner
        .wedges
        .iter()
        .map(spinner_wedge_to_def)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SpinnerDef {
        id: spinner.id.clone(),
        wedges,
    })
}

fn spinner_wedge_to_def(wedge: &SpinnerWedgeAst) -> Result<SpinnerWedgeDef, WorldDefError> {
    Ok(SpinnerWedgeDef {
        text: wedge.text.clone(),
        width: wedge.width,
    })
}

fn npc_to_def(npc: &NpcAst) -> Result<NpcDef, WorldDefError> {
    let mut dialogue: BTreeMap<NpcState, Vec<String>> = BTreeMap::new();
    for (state, lines) in &npc.dialogue {
        let parsed = npc_state_from_str(state)?;
        dialogue.insert(parsed, lines.clone());
    }

    Ok(NpcDef {
        id: npc.id.clone(),
        name: npc.name.clone(),
        desc: npc.desc.clone(),
        max_hp: npc.max_hp,
        location: location_from_npc_ast(&npc.location),
        state: npc_state_from_value(&npc.state)?,
        dialogue,
        movement: npc.movement.as_ref().map(npc_movement_to_def).transpose()?,
    })
}

fn npc_movement_to_def(movement: &NpcMovementAst) -> Result<NpcMovementDef, WorldDefError> {
    let movement_type = match movement.movement_type {
        NpcMovementTypeAst::Route => NpcMovementType::Route,
        NpcMovementTypeAst::Random => NpcMovementType::RandomSet,
    };
    let timing = match movement.timing.as_ref() {
        Some(raw) => Some(npc_movement_timing_from_str(raw)?),
        None => None,
    };

    Ok(NpcMovementDef {
        movement_type,
        rooms: movement.rooms.clone(),
        timing,
        active: movement.active,
        loop_route: movement.loop_route,
    })
}

fn trigger_to_def(trigger: &TriggerAst) -> Result<TriggerDef, WorldDefError> {
    Ok(TriggerDef {
        name: trigger.name.clone(),
        note: trigger.note.clone(),
        only_once: trigger.only_once,
        event: event_from_condition(&trigger.event)?,
        conditions: condition_expr_from_list(&trigger.conditions)?,
        actions: trigger
            .actions
            .iter()
            .map(action_stmt_to_def)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn action_stmt_to_def(stmt: &ActionStmt) -> Result<ActionDef, WorldDefError> {
    Ok(ActionDef {
        action: action_to_kind(&stmt.action)?,
        priority: stmt.priority,
    })
}

fn action_to_kind(action: &ActionAst) -> Result<ActionKind, WorldDefError> {
    Ok(match action {
        ActionAst::Show(text) => ActionKind::ShowMessage { text: text.clone() },
        ActionAst::AddSpinnerWedge { spinner, width, text } => ActionKind::AddSpinnerWedge {
            spinner: spinner.clone(),
            text: text.clone(),
            width: *width,
        },
        ActionAst::AddFlag(name) => ActionKind::AddFlag {
            flag: FlagDef::Simple { name: name.clone() },
        },
        ActionAst::AddSeqFlag { name, end } => ActionKind::AddFlag {
            flag: FlagDef::Sequence {
                name: name.clone(),
                end: *end,
            },
        },
        ActionAst::AwardPoints { amount, reason } => ActionKind::AwardPoints {
            amount: *amount as isize,
            reason: reason.clone(),
        },
        ActionAst::DamagePlayer { amount, turns, cause } => match turns {
            Some(turns) => ActionKind::DamagePlayerOT {
                amount: *amount,
                turns: *turns as u32,
                cause: cause.clone(),
            },
            None => ActionKind::DamagePlayer {
                amount: *amount,
                cause: cause.clone(),
            },
        },
        ActionAst::HealPlayer { amount, turns, cause } => match turns {
            Some(turns) => ActionKind::HealPlayerOT {
                amount: *amount,
                turns: *turns as u32,
                cause: cause.clone(),
            },
            None => ActionKind::HealPlayer {
                amount: *amount,
                cause: cause.clone(),
            },
        },
        ActionAst::RemovePlayerEffect { cause } => ActionKind::RemovePlayerEffect { cause: cause.clone() },
        ActionAst::DamageNpc {
            npc,
            amount,
            turns,
            cause,
        } => match turns {
            Some(turns) => ActionKind::DamageNpcOT {
                npc: npc.clone(),
                amount: *amount,
                turns: *turns as u32,
                cause: cause.clone(),
            },
            None => ActionKind::DamageNpc {
                npc: npc.clone(),
                amount: *amount,
                cause: cause.clone(),
            },
        },
        ActionAst::HealNpc {
            npc,
            amount,
            turns,
            cause,
        } => match turns {
            Some(turns) => ActionKind::HealNpcOT {
                npc: npc.clone(),
                amount: *amount,
                turns: *turns as u32,
                cause: cause.clone(),
            },
            None => ActionKind::HealNpc {
                npc: npc.clone(),
                amount: *amount,
                cause: cause.clone(),
            },
        },
        ActionAst::RemoveNpcEffect { npc, cause } => ActionKind::RemoveNpcEffect {
            npc: npc.clone(),
            cause: cause.clone(),
        },
        ActionAst::RemoveFlag(name) => ActionKind::RemoveFlag { name: name.clone() },
        ActionAst::ReplaceItem { old_sym, new_sym } => ActionKind::ReplaceItem {
            old_item: old_sym.clone(),
            new_item: new_sym.clone(),
        },
        ActionAst::ReplaceDropItem { old_sym, new_sym } => ActionKind::ReplaceDropItem {
            old_item: old_sym.clone(),
            new_item: new_sym.clone(),
        },
        ActionAst::ModifyItem { item, patch } => ActionKind::ModifyItem {
            item: item.clone(),
            patch: item_patch_to_def(patch)?,
        },
        ActionAst::ModifyRoom { room, patch } => ActionKind::ModifyRoom {
            room: room.clone(),
            patch: room_patch_to_def(patch)?,
        },
        ActionAst::ModifyNpc { npc, patch } => ActionKind::ModifyNpc {
            npc: npc.clone(),
            patch: npc_patch_to_def(patch)?,
        },
        ActionAst::SpawnItemIntoRoom { item, room } => ActionKind::SpawnItemInRoom {
            item: item.clone(),
            room: room.clone(),
        },
        ActionAst::DespawnItem(item) => ActionKind::DespawnItem { item: item.clone() },
        ActionAst::DespawnNpc(npc) => ActionKind::DespawnNpc { npc: npc.clone() },
        ActionAst::ResetFlag(name) => ActionKind::ResetFlag { name: name.clone() },
        ActionAst::AdvanceFlag(name) => ActionKind::AdvanceFlag { name: name.clone() },
        ActionAst::SetBarredMessage {
            exit_from,
            exit_to,
            msg,
        } => ActionKind::SetBarredMessage {
            exit_from: exit_from.clone(),
            exit_to: exit_to.clone(),
            msg: msg.clone(),
        },
        ActionAst::RevealExit {
            exit_from,
            exit_to,
            direction,
        } => ActionKind::RevealExit {
            exit_from: exit_from.clone(),
            exit_to: exit_to.clone(),
            direction: direction.clone(),
        },
        ActionAst::LockExit { from_room, direction } => ActionKind::LockExit {
            from_room: from_room.clone(),
            direction: direction.clone(),
        },
        ActionAst::UnlockExit { from_room, direction } => ActionKind::UnlockExit {
            from_room: from_room.clone(),
            direction: direction.clone(),
        },
        ActionAst::LockItem(item) => ActionKind::LockItem { item: item.clone() },
        ActionAst::UnlockItemAction(item) => ActionKind::UnlockItem { item: item.clone() },
        ActionAst::PushPlayerTo(room) => ActionKind::PushPlayerTo { room: room.clone() },
        ActionAst::GiveItemToPlayer { npc, item } => ActionKind::GiveItemToPlayer {
            npc: npc.clone(),
            item: item.clone(),
        },
        ActionAst::SpawnItemInInventory(item) => ActionKind::SpawnItemInInventory { item: item.clone() },
        ActionAst::SpawnItemCurrentRoom(item) => ActionKind::SpawnItemCurrentRoom { item: item.clone() },
        ActionAst::SpawnItemInContainer { item, container } => ActionKind::SpawnItemInContainer {
            item: item.clone(),
            container: container.clone(),
        },
        ActionAst::SpawnNpcIntoRoom { npc, room } => ActionKind::SpawnNpcInRoom {
            npc: npc.clone(),
            room: room.clone(),
        },
        ActionAst::SetItemDescription { item, text } => ActionKind::SetItemDescription {
            item: item.clone(),
            text: text.clone(),
        },
        ActionAst::SetItemMovability { item, movability } => ActionKind::SetItemMovability {
            item: item.clone(),
            movability: movability_from_ast(movability),
        },
        ActionAst::NpcSays { npc, quote } => ActionKind::NpcSays {
            npc: npc.clone(),
            quote: quote.clone(),
        },
        ActionAst::NpcSaysRandom { npc } => ActionKind::NpcSaysRandom { npc: npc.clone() },
        ActionAst::NpcRefuseItem { npc, reason } => ActionKind::NpcRefuseItem {
            npc: npc.clone(),
            reason: reason.clone(),
        },
        ActionAst::SetNpcActive { npc, active } => ActionKind::SetNpcActive {
            npc: npc.clone(),
            active: *active,
        },
        ActionAst::SetNpcState { npc, state } => ActionKind::SetNpcState {
            npc: npc.clone(),
            state: npc_state_from_str(state)?,
        },
        ActionAst::DenyRead(reason) => ActionKind::DenyRead { reason: reason.clone() },
        ActionAst::SetContainerState { item, state } => ActionKind::SetContainerState {
            item: item.clone(),
            state: state.as_ref().map(|s| container_state_from_str(s)).transpose()?,
        },
        ActionAst::SpinnerMessage { spinner } => ActionKind::SpinnerMessage {
            spinner: spinner.clone(),
        },
        ActionAst::ScheduleIn {
            turns_ahead,
            actions,
            note,
        } => ActionKind::ScheduleIn {
            turns_ahead: *turns_ahead,
            actions: actions_to_defs(actions)?,
            note: note.clone(),
        },
        ActionAst::ScheduleOn { on_turn, actions, note } => ActionKind::ScheduleOn {
            on_turn: *on_turn,
            actions: actions_to_defs(actions)?,
            note: note.clone(),
        },
        ActionAst::ScheduleInIf {
            turns_ahead,
            condition,
            on_false,
            actions,
            note,
        } => ActionKind::ScheduleInIf {
            turns_ahead: *turns_ahead,
            condition: condition_expr_from_ast(condition)?,
            on_false: on_false_policy_from_ast(on_false),
            actions: actions_to_defs(actions)?,
            note: note.clone(),
        },
        ActionAst::ScheduleOnIf {
            on_turn,
            condition,
            on_false,
            actions,
            note,
        } => ActionKind::ScheduleOnIf {
            on_turn: *on_turn,
            condition: condition_expr_from_ast(condition)?,
            on_false: on_false_policy_from_ast(on_false),
            actions: actions_to_defs(actions)?,
            note: note.clone(),
        },
        ActionAst::Conditional { condition, actions } => ActionKind::Conditional {
            condition: condition_expr_from_ast(condition)?,
            actions: actions_to_defs(actions)?,
        },
    })
}

fn actions_to_defs(actions: &[ActionStmt]) -> Result<Vec<ActionDef>, WorldDefError> {
    actions.iter().map(action_stmt_to_def).collect()
}

fn item_patch_to_def(patch: &crate::ItemPatchAst) -> Result<ItemPatchDef, WorldDefError> {
    Ok(ItemPatchDef {
        name: patch.name.clone(),
        desc: patch.desc.clone(),
        text: patch.text.clone(),
        movability: patch.movability.as_ref().map(movability_from_ast),
        container_state: patch.container_state.as_ref().map(container_state_from_ast),
        remove_container_state: patch.remove_container_state,
        add_abilities: patch
            .add_abilities
            .iter()
            .map(item_ability_from_ast)
            .collect::<Result<Vec<_>, _>>()?,
        remove_abilities: patch
            .remove_abilities
            .iter()
            .map(item_ability_from_ast)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn room_patch_to_def(patch: &crate::RoomPatchAst) -> Result<RoomPatchDef, WorldDefError> {
    Ok(RoomPatchDef {
        name: patch.name.clone(),
        desc: patch.desc.clone(),
        remove_exits: patch.remove_exits.clone(),
        add_exits: patch
            .add_exits
            .iter()
            .map(room_exit_patch_to_def)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn room_exit_patch_to_def(exit: &RoomExitPatchAst) -> Result<RoomExitPatchDef, WorldDefError> {
    Ok(RoomExitPatchDef {
        direction: exit.direction.clone(),
        to: exit.to.clone(),
        hidden: exit.hidden,
        locked: exit.locked,
        barred_message: exit.barred_message.clone(),
        required_flags: exit.required_flags.clone(),
        required_items: exit.required_items.clone(),
    })
}

fn npc_patch_to_def(patch: &NpcPatchAst) -> Result<NpcPatchDef, WorldDefError> {
    Ok(NpcPatchDef {
        name: patch.name.clone(),
        desc: patch.desc.clone(),
        state: patch.state.as_ref().map(npc_state_from_value).transpose()?,
        add_lines: patch
            .add_lines
            .iter()
            .map(npc_dialogue_patch_to_def)
            .collect::<Result<Vec<_>, _>>()?,
        movement: patch.movement.as_ref().map(npc_movement_patch_to_def).transpose()?,
    })
}

fn npc_dialogue_patch_to_def(patch: &crate::NpcDialoguePatchAst) -> Result<NpcDialoguePatchDef, WorldDefError> {
    Ok(NpcDialoguePatchDef {
        state: npc_state_from_value(&patch.state)?,
        line: patch.line.clone(),
    })
}

fn npc_movement_patch_to_def(patch: &crate::NpcMovementPatchAst) -> Result<NpcMovementPatchDef, WorldDefError> {
    Ok(NpcMovementPatchDef {
        route: patch.route.clone(),
        random_rooms: patch.random_rooms.clone(),
        timing: patch.timing.as_ref().map(npc_timing_patch_to_def),
        active: patch.active,
        loop_route: patch.loop_route,
    })
}

fn npc_timing_patch_to_def(timing: &NpcTimingPatchAst) -> NpcTimingPatchDef {
    match timing {
        NpcTimingPatchAst::EveryNTurns(turns) => NpcTimingPatchDef::EveryNTurns { turns: *turns },
        NpcTimingPatchAst::OnTurn(turn) => NpcTimingPatchDef::OnTurn { turn: *turn },
    }
}

fn goal_to_def(goal: &GoalAst) -> Result<GoalDef, WorldDefError> {
    Ok(GoalDef {
        id: goal.id.clone(),
        name: goal.name.clone(),
        description: goal.description.clone(),
        group: match goal.group {
            GoalGroupAst::Required => GoalGroup::Required,
            GoalGroupAst::Optional => GoalGroup::Optional,
            GoalGroupAst::StatusEffect => GoalGroup::StatusEffect,
        },
        activate_when: goal.activate_when.as_ref().map(goal_cond_to_def).transpose()?,
        failed_when: goal.failed_when.as_ref().map(goal_cond_to_def).transpose()?,
        finished_when: goal_cond_to_def(&goal.finished_when)?,
    })
}

fn goal_cond_to_def(cond: &GoalCondAst) -> Result<GoalCondition, WorldDefError> {
    Ok(match cond {
        GoalCondAst::HasFlag(flag) => GoalCondition::HasFlag { flag: flag.clone() },
        GoalCondAst::MissingFlag(flag) => GoalCondition::MissingFlag { flag: flag.clone() },
        GoalCondAst::HasItem(item) => GoalCondition::HasItem { item: item.clone() },
        GoalCondAst::ReachedRoom(room) => GoalCondition::ReachedRoom { room: room.clone() },
        GoalCondAst::GoalComplete(goal_id) => GoalCondition::GoalComplete {
            goal_id: goal_id.clone(),
        },
        GoalCondAst::FlagInProgress(flag) => GoalCondition::FlagInProgress { flag: flag.clone() },
        GoalCondAst::FlagComplete(flag) => GoalCondition::FlagComplete { flag: flag.clone() },
    })
}

fn event_from_condition(cond: &ConditionAst) -> Result<EventDef, WorldDefError> {
    Ok(match cond {
        ConditionAst::Always => EventDef::Always,
        ConditionAst::EnterRoom(room) => EventDef::EnterRoom { room: room.clone() },
        ConditionAst::LeaveRoom(room) => EventDef::LeaveRoom { room: room.clone() },
        ConditionAst::TakeItem(item) => EventDef::TakeItem { item: item.clone() },
        ConditionAst::DropItem(item) => EventDef::DropItem { item: item.clone() },
        ConditionAst::LookAtItem(item) => EventDef::LookAtItem { item: item.clone() },
        ConditionAst::OpenItem(item) => EventDef::OpenItem { item: item.clone() },
        ConditionAst::UnlockItem(item) => EventDef::UnlockItem { item: item.clone() },
        ConditionAst::TouchItem(item) => EventDef::TouchItem { item: item.clone() },
        ConditionAst::TalkToNpc(npc) => EventDef::TalkToNpc { npc: npc.clone() },
        ConditionAst::UseItem { item, ability } => EventDef::UseItem {
            item: item.clone(),
            ability: item_ability_from_str(ability, None)?,
        },
        ConditionAst::UseItemOnItem {
            tool,
            target,
            interaction,
        } => EventDef::UseItemOnItem {
            tool: tool.clone(),
            target: target.clone(),
            interaction: item_interaction_from_str(interaction)?,
        },
        ConditionAst::ActOnItem { target, action } => EventDef::ActOnItem {
            target: target.clone(),
            action: item_interaction_from_str(action)?,
        },
        ConditionAst::GiveToNpc { item, npc } => EventDef::GiveToNpc {
            item: item.clone(),
            npc: npc.clone(),
        },
        ConditionAst::TakeFromNpc { item, npc } => EventDef::TakeFromNpc {
            item: item.clone(),
            npc: npc.clone(),
        },
        ConditionAst::InsertItemInto { item, container } => EventDef::InsertItemInto {
            item: item.clone(),
            container: container.clone(),
        },
        ConditionAst::Ingest { item, mode } => EventDef::Ingest {
            item: item.clone(),
            mode: ingest_mode_from_ast(mode),
        },
        ConditionAst::PlayerDeath => EventDef::PlayerDeath,
        ConditionAst::NpcDeath(npc) => EventDef::NpcDeath { npc: npc.clone() },
        _ => {
            return Err(WorldDefError::UnsupportedAst {
                kind: "event",
                value: format!("{cond:?}"),
            });
        },
    })
}

fn condition_expr_from_list(conds: &[ConditionAst]) -> Result<ConditionExpr, WorldDefError> {
    match conds.len() {
        0 => Ok(ConditionExpr::All(Vec::new())),
        1 => condition_expr_from_ast(&conds[0]),
        _ => Ok(ConditionExpr::All(
            conds
                .iter()
                .map(condition_expr_from_ast)
                .collect::<Result<Vec<_>, _>>()?,
        )),
    }
}

fn condition_expr_from_ast(cond: &ConditionAst) -> Result<ConditionExpr, WorldDefError> {
    Ok(match cond {
        ConditionAst::All(kids) => ConditionExpr::All(
            kids.iter()
                .map(condition_expr_from_ast)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ConditionAst::Any(kids) => ConditionExpr::Any(
            kids.iter()
                .map(condition_expr_from_ast)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ConditionAst::Always => ConditionExpr::All(Vec::new()),
        ConditionAst::MissingFlag(flag) => ConditionExpr::Pred(ConditionDef::MissingFlag { flag: flag.clone() }),
        ConditionAst::HasFlag(flag) => ConditionExpr::Pred(ConditionDef::HasFlag { flag: flag.clone() }),
        ConditionAst::HasItem(item) => ConditionExpr::Pred(ConditionDef::HasItem { item: item.clone() }),
        ConditionAst::PlayerInRoom(room) => ConditionExpr::Pred(ConditionDef::PlayerInRoom { room: room.clone() }),
        ConditionAst::HasVisited(room) => ConditionExpr::Pred(ConditionDef::HasVisited { room: room.clone() }),
        ConditionAst::MissingItem(item) => ConditionExpr::Pred(ConditionDef::MissingItem { item: item.clone() }),
        ConditionAst::FlagInProgress(flag) => ConditionExpr::Pred(ConditionDef::FlagInProgress { flag: flag.clone() }),
        ConditionAst::FlagComplete(flag) => ConditionExpr::Pred(ConditionDef::FlagComplete { flag: flag.clone() }),
        ConditionAst::WithNpc(npc) => ConditionExpr::Pred(ConditionDef::WithNpc { npc: npc.clone() }),
        ConditionAst::NpcHasItem { npc, item } => ConditionExpr::Pred(ConditionDef::NpcHasItem {
            npc: npc.clone(),
            item: item.clone(),
        }),
        ConditionAst::NpcInState { npc, state } => ConditionExpr::Pred(ConditionDef::NpcInState {
            npc: npc.clone(),
            state: npc_state_from_value(state)?,
        }),
        ConditionAst::ContainerHasItem { container, item } => ConditionExpr::Pred(ConditionDef::ContainerHasItem {
            container: container.clone(),
            item: item.clone(),
        }),
        ConditionAst::Ambient { spinner, rooms } => ConditionExpr::Pred(ConditionDef::Ambient {
            spinner: spinner.clone(),
            rooms: rooms.clone(),
        }),
        ConditionAst::ChancePercent(pct) => ConditionExpr::Pred(ConditionDef::ChancePercent { percent: *pct }),
        _ => {
            return Err(WorldDefError::UnsupportedAst {
                kind: "condition",
                value: format!("{cond:?}"),
            });
        },
    })
}

fn item_ability_from_ast(ast: &ItemAbilityAst) -> Result<ItemAbility, WorldDefError> {
    item_ability_from_str(&ast.ability, ast.target.as_deref())
}

fn item_ability_from_str(value: &str, target: Option<&str>) -> Result<ItemAbility, WorldDefError> {
    let norm = normalize_token(value);
    let ability = match norm.as_str() {
        "attach" => ItemAbility::Attach,
        "clean" => ItemAbility::Clean,
        "cut" => ItemAbility::Cut,
        "cutwood" => ItemAbility::CutWood,
        "drink" => ItemAbility::Drink,
        "eat" => ItemAbility::Eat,
        "extinguish" => ItemAbility::Extinguish,
        "ignite" => ItemAbility::Ignite,
        "inhale" => ItemAbility::Inhale,
        "insulate" => ItemAbility::Insulate,
        "magnify" => ItemAbility::Magnify,
        "pluck" => ItemAbility::Pluck,
        "pry" => ItemAbility::Pry,
        "read" => ItemAbility::Read,
        "repair" => ItemAbility::Repair,
        "sharpen" => ItemAbility::Sharpen,
        "smash" => ItemAbility::Smash,
        "turnon" => ItemAbility::TurnOn,
        "turnoff" => ItemAbility::TurnOff,
        "unlock" => ItemAbility::Unlock(target.map(|t| t.to_string())),
        "use" => ItemAbility::Use,
        _ => {
            return Err(WorldDefError::UnknownItemAbility {
                value: value.to_string(),
            });
        },
    };
    Ok(ability)
}

fn item_interaction_from_str(value: &str) -> Result<ItemInteractionType, WorldDefError> {
    let norm = normalize_token(value);
    let interaction = match norm.as_str() {
        "attach" => ItemInteractionType::Attach,
        "detach" => ItemInteractionType::Detach,
        "break" => ItemInteractionType::Break,
        "burn" => ItemInteractionType::Burn,
        "extinguish" => ItemInteractionType::Extinguish,
        "clean" => ItemInteractionType::Clean,
        "cover" => ItemInteractionType::Cover,
        "cut" => ItemInteractionType::Cut,
        "handle" => ItemInteractionType::Handle,
        "move" => ItemInteractionType::Move,
        "open" => ItemInteractionType::Open,
        "repair" => ItemInteractionType::Repair,
        "sharpen" => ItemInteractionType::Sharpen,
        "turn" => ItemInteractionType::Turn,
        "unlock" => ItemInteractionType::Unlock,
        _ => {
            return Err(WorldDefError::UnknownInteraction {
                value: value.to_string(),
            });
        },
    };
    Ok(interaction)
}

fn npc_state_from_value(value: &NpcStateValue) -> Result<NpcState, WorldDefError> {
    match value {
        NpcStateValue::Named(name) => npc_state_from_str(name),
        NpcStateValue::Custom(name) => Ok(NpcState::Custom(name.clone())),
    }
}

fn npc_state_from_str(value: &str) -> Result<NpcState, WorldDefError> {
    let trimmed = value.trim();
    if trimmed.len() >= 6 && trimmed[..6].eq_ignore_ascii_case("custom") {
        let mut rest = trimmed[6..].trim_start();
        if rest.starts_with(':') {
            rest = rest[1..].trim_start();
        } else if rest.starts_with('(') {
            rest = rest[1..].trim_start();
            if let Some(idx) = rest.rfind(')') {
                rest = rest[..idx].trim_end();
            }
        }
        if !rest.is_empty() {
            return Ok(NpcState::Custom(rest.to_string()));
        }
    }
    let norm = normalize_token(trimmed);
    let state = match norm.as_str() {
        "bored" => NpcState::Bored,
        "happy" => NpcState::Happy,
        "mad" => NpcState::Mad,
        "normal" => NpcState::Normal,
        "sad" => NpcState::Sad,
        "tired" => NpcState::Tired,
        _ => {
            return Err(WorldDefError::UnknownNpcState {
                value: value.to_string(),
            });
        },
    };
    Ok(state)
}

fn npc_movement_timing_from_str(value: &str) -> Result<NpcMovementTiming, WorldDefError> {
    if let Some(turns_str) = value.strip_prefix("every_").and_then(|s| s.strip_suffix("_turns")) {
        let turns = turns_str
            .parse::<usize>()
            .map_err(|_| WorldDefError::InvalidNpcMovementTiming {
                value: value.to_string(),
            })?;
        return Ok(NpcMovementTiming::EveryNTurns { turns });
    }
    if let Some(turn_str) = value.strip_prefix("on_turn_") {
        let turn = turn_str
            .parse::<usize>()
            .map_err(|_| WorldDefError::InvalidNpcMovementTiming {
                value: value.to_string(),
            })?;
        return Ok(NpcMovementTiming::OnTurn { turn });
    }
    Err(WorldDefError::InvalidNpcMovementTiming {
        value: value.to_string(),
    })
}

fn movability_from_ast(movability: &MovabilityAst) -> Movability {
    match movability {
        MovabilityAst::Free => Movability::Free,
        MovabilityAst::Fixed { reason } => Movability::Fixed { reason: reason.clone() },
        MovabilityAst::Restricted { reason } => Movability::Restricted { reason: reason.clone() },
    }
}

fn container_state_from_ast(state: &ContainerStateAst) -> ContainerState {
    match state {
        ContainerStateAst::Open => ContainerState::Open,
        ContainerStateAst::Closed => ContainerState::Closed,
        ContainerStateAst::Locked => ContainerState::Locked,
        ContainerStateAst::TransparentClosed => ContainerState::TransparentClosed,
        ContainerStateAst::TransparentLocked => ContainerState::TransparentLocked,
    }
}

fn container_state_from_str(value: &str) -> Result<ContainerState, WorldDefError> {
    let norm = normalize_token(value);
    match norm.as_str() {
        "open" => Ok(ContainerState::Open),
        "closed" => Ok(ContainerState::Closed),
        "locked" => Ok(ContainerState::Locked),
        "transparentclosed" => Ok(ContainerState::TransparentClosed),
        "transparentlocked" => Ok(ContainerState::TransparentLocked),
        "transparentopen" => Ok(ContainerState::TransparentOpen),
        _ => Err(WorldDefError::InvalidContainerState {
            value: value.to_string(),
        }),
    }
}

fn ingest_mode_from_ast(mode: &IngestModeAst) -> IngestMode {
    match mode {
        IngestModeAst::Eat => IngestMode::Eat,
        IngestModeAst::Drink => IngestMode::Drink,
        IngestModeAst::Inhale => IngestMode::Inhale,
    }
}

fn location_from_item_ast(loc: &ItemLocationAst) -> LocationRef {
    match loc {
        ItemLocationAst::Inventory(_) => LocationRef::Inventory,
        ItemLocationAst::Room(room) => LocationRef::Room(room.clone()),
        ItemLocationAst::Npc(npc) => LocationRef::Npc(npc.clone()),
        ItemLocationAst::Chest(item) => LocationRef::Item(item.clone()),
        ItemLocationAst::Nowhere(_) => LocationRef::Nowhere,
    }
}

fn location_from_npc_ast(loc: &NpcLocationAst) -> LocationRef {
    match loc {
        NpcLocationAst::Room(room) => LocationRef::Room(room.clone()),
        NpcLocationAst::Nowhere(_) => LocationRef::Nowhere,
    }
}

fn on_false_policy_from_ast(policy: &Option<OnFalseAst>) -> OnFalsePolicy {
    match policy {
        Some(OnFalseAst::RetryAfter { turns }) => OnFalsePolicy::RetryAfter { turns: *turns },
        Some(OnFalseAst::RetryNextTurn) => OnFalsePolicy::RetryNextTurn,
        _ => OnFalsePolicy::Cancel,
    }
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect::<String>()
}
