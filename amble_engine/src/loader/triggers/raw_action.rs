//! Raw trigger action translation.
//!
//! Converts deserialized trigger actions from TOML into strongly typed
//! engine actions, performing symbol to UUID resolution and validation.

use anyhow::{Result, bail};
use serde::Deserialize;
use std::collections::HashSet;

use super::raw_condition::RawTriggerCondition;
use crate::item::Movability;
use crate::loader::items::RawItemAbility;
use crate::scheduler::{EventCondition, OnFalsePolicy};
use crate::trigger::{
    ItemPatch, NpcDialoguePatch, NpcMovementPatch, NpcPatch, RoomExitPatch, RoomPatch, ScriptedAction, TriggerAction,
};
use crate::{
    item::ContainerState,
    loader::SymbolTable,
    npc::{MovementTiming, NpcState},
    player::Flag,
    spinners::SpinnerType,
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawEventCondition {
    Trigger(RawTriggerCondition),
    All { all: Vec<RawEventCondition> },
    Any { any: Vec<RawEventCondition> },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawOnFalsePolicy {
    Cancel,
    RetryAfter { turns: usize },
    RetryNextTurn,
}

#[derive(Debug, Deserialize)]
pub struct RawItemPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub text: Option<String>,
    pub movability: Option<Movability>,
    pub container_state: Option<ContainerState>,
    #[serde(default)]
    pub remove_container_state: bool,
    #[serde(default)]
    pub add_abilities: Vec<RawItemAbility>,
    #[serde(default)]
    pub remove_abilities: Vec<RawItemAbility>,
}
impl RawItemPatch {
    /// Convert this raw item patch into an engine [`ItemPatch`].
    ///
    /// # Errors
    /// Returns an error if any referenced item abilities cannot be resolved in the symbol table.
    pub fn to_patch(&self, symbols: &SymbolTable) -> Result<ItemPatch> {
        Ok(ItemPatch {
            name: self.name.clone(),
            desc: self.desc.clone(),
            text: self.text.clone(),
            container_state: self.container_state,
            movability: self.movability.clone(),
            remove_container_state: self.remove_container_state,
            add_abilities: self
                .add_abilities
                .iter()
                .map(|ab| ab.to_ability(symbols))
                .collect::<Result<Vec<_>, _>>()?,
            remove_abilities: self
                .remove_abilities
                .iter()
                .map(|ab| ab.to_ability(symbols))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RawRoomPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub remove_exits: Vec<String>,
    #[serde(default)]
    pub add_exits: Vec<RawPatchedExit>,
}

#[derive(Debug, Deserialize)]
pub struct RawPatchedExit {
    pub direction: String,
    pub to: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub required_flags: Vec<Flag>,
    #[serde(default)]
    pub required_items: Vec<String>,
    pub barred_message: Option<String>,
}

impl RawRoomPatch {
    /// Convert this raw room patch into an engine [`RoomPatch`].
    ///
    /// # Errors
    /// Returns an error if referenced rooms or items cannot be found in the symbol table.
    pub fn to_patch(&self, symbols: &SymbolTable) -> Result<RoomPatch> {
        let mut remove_exits = Vec::new();
        for dest_sym in &self.remove_exits {
            if let Some(room_id) = symbols.rooms.get(dest_sym) {
                remove_exits.push(*room_id);
            } else {
                bail!("loading TriggerAction:ModifyRoom: remove exit room symbol ({dest_sym}) not in table");
            }
        }

        let mut add_exits = Vec::new();
        for raw in &self.add_exits {
            let to_id = if let Some(id) = symbols.rooms.get(&raw.to) {
                *id
            } else {
                bail!(
                    "loading TriggerAction:ModifyRoom: add exit destination room symbol ({}) not in table",
                    raw.to
                );
            };

            let mut required_items = HashSet::new();
            for item_sym in &raw.required_items {
                if let Some(item_id) = symbols.items.get(item_sym) {
                    required_items.insert(*item_id);
                } else {
                    bail!("loading TriggerAction:ModifyRoom: required item symbol ({item_sym}) not in table");
                }
            }

            let required_flags: HashSet<Flag> = raw.required_flags.iter().cloned().collect();

            add_exits.push(RoomExitPatch {
                direction: raw.direction.clone(),
                to: to_id,
                hidden: raw.hidden,
                locked: raw.locked,
                required_flags,
                required_items,
                barred_message: raw.barred_message.clone(),
            });
        }

        Ok(RoomPatch {
            name: self.name.clone(),
            desc: self.desc.clone(),
            remove_exits,
            add_exits,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RawNpcDialoguePatch {
    pub state: NpcState,
    pub line: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct RawNpcMovementPatch {
    pub route: Option<Vec<String>>,
    pub random_rooms: Option<Vec<String>>,
    pub timing: Option<RawMovementTimingPatch>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawMovementTimingPatch {
    EveryNTurns { turns: usize },
    OnTurn { turn: usize },
}
impl RawNpcMovementPatch {
    /// Convert this raw NPC movement patch into an engine [`NpcMovementPatch`].
    ///
    /// # Errors
    /// Returns an error if referenced rooms in the route or random pool cannot be resolved.
    pub fn to_patch(&self, symbols: &SymbolTable) -> Result<NpcMovementPatch> {
        let route = if let Some(route_syms) = &self.route {
            let mut resolved = Vec::new();
            for room_sym in route_syms {
                if let Some(room_id) = symbols.rooms.get(room_sym) {
                    resolved.push(*room_id);
                } else {
                    bail!("loading TriggerAction:ModifyNpc: route room symbol ({room_sym}) not in table");
                }
            }
            Some(resolved)
        } else {
            None
        };

        let random_rooms = if let Some(random_syms) = &self.random_rooms {
            let mut resolved = HashSet::new();
            for room_sym in random_syms {
                if let Some(room_id) = symbols.rooms.get(room_sym) {
                    resolved.insert(*room_id);
                } else {
                    bail!("loading TriggerAction:ModifyNpc: random room symbol ({room_sym}) not in table");
                }
            }
            Some(resolved)
        } else {
            None
        };

        if let Some(route) = &route
            && route.is_empty()
        {
            bail!("loading TriggerAction:ModifyNpc: movement route must include at least one room");
        }
        if let Some(random) = &random_rooms
            && random.is_empty()
        {
            bail!("loading TriggerAction:ModifyNpc: random room set must include at least one room");
        }
        if route.is_some() && random_rooms.is_some() {
            bail!("loading TriggerAction:ModifyNpc: cannot set both route and random rooms");
        }

        let timing = self.timing.as_ref().map(|timing_str| match timing_str {
            RawMovementTimingPatch::EveryNTurns { turns } => MovementTiming::EveryNTurns { turns: *turns },
            RawMovementTimingPatch::OnTurn { turn } => MovementTiming::OnTurn { turn: *turn },
        });

        Ok(NpcMovementPatch {
            route,
            random_rooms,
            timing,
            active: self.active,
            loop_route: self.loop_route,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RawNpcPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub state: Option<NpcState>,
    #[serde(default)]
    pub add_lines: Vec<RawNpcDialoguePatch>,
    pub movement: Option<RawNpcMovementPatch>,
}
impl RawNpcPatch {
    /// Convert this raw NPC patch into an engine [`NpcPatch`].
    ///
    /// # Errors
    /// Returns an error if the nested movement patch references symbols that cannot be resolved.
    pub fn to_patch(&self, symbols: &SymbolTable) -> Result<NpcPatch> {
        let add_lines = self
            .add_lines
            .iter()
            .map(|raw| NpcDialoguePatch {
                state: raw.state.clone(),
                line: raw.line.clone(),
            })
            .collect();

        let movement = if let Some(raw_mvmt) = &self.movement {
            let mvmt_patch = raw_mvmt.to_patch(symbols)?;
            if mvmt_patch.has_updates() {
                Some(mvmt_patch)
            } else {
                None
            }
        } else {
            None
        };

        Ok(NpcPatch {
            name: self.name.clone(),
            desc: self.desc.clone(),
            state: self.state.clone(),
            add_lines,
            movement,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RawActionStmt {
    #[serde(default)]
    pub priority: Option<isize>,
    #[serde(flatten)]
    pub action: RawTriggerAction,
}
impl RawActionStmt {
    pub fn to_action(&self, symbols: &SymbolTable) -> Result<ScriptedAction> {
        let cooked = self.action.to_action(symbols)?;
        Ok(ScriptedAction::with_priority(cooked, self.priority))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawTriggerAction {
    ModifyItem {
        item_sym: String,
        patch: RawItemPatch,
    },
    ModifyRoom {
        room_sym: String,
        patch: RawRoomPatch,
    },
    ModifyNpc {
        npc_sym: String,
        patch: RawNpcPatch,
    },
    SpawnNpcIntoRoom {
        npc_sym: String,
        room_sym: String,
    },
    DespawnNpc {
        npc_sym: String,
    },
    SetNpcActive {
        npc_sym: String,
        active: bool,
    },
    SetContainerState {
        item_sym: String,
        state: Option<ContainerState>,
    },
    ReplaceItem {
        old_sym: String,
        new_sym: String,
    },
    ReplaceDropItem {
        old_sym: String,
        new_sym: String,
    },
    AddFlag {
        flag: Flag,
    },
    AddSpinnerWedge {
        spinner: String,
        text: String,
        width: usize,
    },
    AdvanceFlag {
        flag: String,
    },
    RemoveFlag {
        flag: String,
    },
    AwardPoints {
        amount: isize,
        reason: String,
    },
    SetBarredMessage {
        exit_from: String,
        exit_to: String,
        msg: String,
    },
    DenyRead {
        reason: String,
    },
    DespawnItem {
        item_id: String,
    },
    GiveItemToPlayer {
        npc_id: String,
        item_id: String,
    },
    DamageNpc {
        npc_id: String,
        cause: String,
        amount: u32,
    },
    DamageNpcOT {
        npc_id: String,
        cause: String,
        amount: u32,
        turns: u32,
    },
    HealNpc {
        npc_id: String,
        cause: String,
        amount: u32,
    },
    HealNpcOT {
        npc_id: String,
        cause: String,
        amount: u32,
        turns: u32,
    },
    RemoveNpcEffect {
        npc_id: String,
        cause: String,
    },
    DamagePlayer {
        cause: String,
        amount: u32,
    },
    DamagePlayerOT {
        cause: String,
        amount: u32,
        turns: u32,
    },
    HealPlayer {
        cause: String,
        amount: u32,
    },
    HealPlayerOT {
        cause: String,
        amount: u32,
        turns: u32,
    },
    RemovePlayerEffect {
        cause: String,
    },
    LockItem {
        item_id: String,
    },
    LockExit {
        from_room: String,
        direction: String,
    },
    NpcRefuseItem {
        npc_id: String,
        reason: String,
    },
    NpcSays {
        npc_id: String,
        quote: String,
    },
    NpcSaysRandom {
        npc_id: String,
    },
    PushPlayerTo {
        room_id: String,
    },
    ResetFlag {
        flag: String,
    },
    RevealExit {
        exit_from: String,
        exit_to: String,
        direction: String,
    },
    SetItemDescription {
        item_sym: String,
        text: String,
    },
    SetItemMovability {
        item_sym: String,
        movability: Movability,
    },
    SetNpcState {
        npc_id: String,
        state: NpcState,
    },
    ShowMessage {
        text: String,
    },
    SpawnItemCurrentRoom {
        item_id: String,
    },
    SpawnItemInContainer {
        item_id: String,
        container_id: String,
    },
    SpawnItemInInventory {
        item_id: String,
    },
    SpawnItemInRoom {
        item_id: String,
        room_id: String,
    },
    SpinnerMessage {
        spinner: String,
    },
    UnlockItem {
        item_id: String,
    },
    UnlockExit {
        from_room: String,
        direction: String,
    },
    Conditional {
        condition: RawEventCondition,
        actions: Vec<RawActionStmt>,
    },
    ScheduleIn {
        turns_ahead: usize,
        actions: Vec<RawActionStmt>,
        note: Option<String>,
    },
    ScheduleOn {
        on_turn: usize,
        actions: Vec<RawActionStmt>,
        note: Option<String>,
    },
    ScheduleInIf {
        turns_ahead: usize,
        condition: RawEventCondition,
        on_false: RawOnFalsePolicy,
        actions: Vec<RawActionStmt>,
        note: Option<String>,
    },
    ScheduleOnIf {
        on_turn: usize,
        condition: RawEventCondition,
        on_false: RawOnFalsePolicy,
        actions: Vec<RawActionStmt>,
        note: Option<String>,
    },
}
impl RawTriggerAction {
    /// Convert the TOML representation of this action to a fully realized `TriggerAction`.
    ///
    /// # Errors
    /// Returns an error if referenced symbols cannot be resolved or if nested actions fail to convert.
    pub fn to_action(&self, symbols: &SymbolTable) -> Result<TriggerAction> {
        match self {
            Self::ModifyItem { item_sym, patch } => cook_modify_item(symbols, item_sym, patch),
            Self::ModifyRoom { room_sym, patch } => cook_modify_room(symbols, room_sym, patch),
            Self::ModifyNpc { npc_sym, patch } => cook_modify_npc(symbols, npc_sym, patch),
            Self::SpawnNpcIntoRoom { npc_sym, room_sym } => cook_spawn_npc_into_room(symbols, npc_sym, room_sym),
            Self::DespawnNpc { npc_sym } => cook_despawn_npc(symbols, npc_sym),
            Self::SetNpcActive { npc_sym, active } => cook_set_npc_active(symbols, npc_sym, *active),
            Self::SetContainerState { item_sym, state } => cook_set_container_state(symbols, item_sym, *state),
            Self::ReplaceItem { old_sym, new_sym } => cook_replace_item(symbols, old_sym, new_sym),
            Self::ReplaceDropItem { old_sym, new_sym } => cook_replace_drop_item(symbols, old_sym, new_sym),
            Self::SetItemDescription { item_sym, text } => cook_set_item_description(symbols, item_sym, text),
            Self::SetItemMovability { item_sym, movability } => cook_set_item_movability(symbols, item_sym, movability),
            Self::SetBarredMessage {
                msg,
                exit_from,
                exit_to,
            } => cook_barred_message(symbols, msg, exit_from, exit_to),
            Self::AddSpinnerWedge { spinner, text, width } => Ok(TriggerAction::AddSpinnerWedge {
                spinner: SpinnerType::from_toml_key(spinner),
                text: text.clone(),
                width: *width,
            }),
            Self::ResetFlag { flag } => Ok(TriggerAction::ResetFlag(flag.clone())),
            Self::AdvanceFlag { flag } => Ok(TriggerAction::AdvanceFlag(flag.clone())),
            Self::SpinnerMessage { spinner } => Ok(TriggerAction::SpinnerMessage {
                spinner: SpinnerType::from_toml_key(spinner),
            }),
            Self::DamageNpc { npc_id, cause, amount } => cook_damage_npc(symbols, npc_id, cause, *amount),
            Self::DamageNpcOT {
                npc_id,
                cause,
                amount,
                turns,
            } => cook_damage_npc_ot(symbols, npc_id, cause, *amount, *turns),
            Self::HealNpc { npc_id, cause, amount } => cook_heal_npc(symbols, npc_id, cause, *amount),
            Self::HealNpcOT {
                npc_id,
                cause,
                amount,
                turns,
            } => cook_heal_npc_ot(symbols, npc_id, cause, *amount, *turns),
            Self::RemoveNpcEffect { npc_id, cause } => cook_remove_npc_effect(symbols, npc_id, cause),
            Self::DamagePlayer { cause, amount } => Ok(TriggerAction::DamagePlayer {
                cause: cause.clone(),
                amount: *amount,
            }),
            Self::DamagePlayerOT { cause, amount, turns } => Ok(TriggerAction::DamagePlayerOT {
                cause: cause.clone(),
                amount: *amount,
                turns: *turns,
            }),
            Self::HealPlayer { cause, amount } => Ok(TriggerAction::HealPlayer {
                cause: cause.clone(),
                amount: *amount,
            }),
            Self::HealPlayerOT { cause, amount, turns } => Ok(TriggerAction::HealPlayerOT {
                cause: cause.clone(),
                amount: *amount,
                turns: *turns,
            }),
            Self::RemovePlayerEffect { cause } => Ok(TriggerAction::RemovePlayerEffect { cause: cause.clone() }),
            Self::NpcRefuseItem { npc_id, reason } => cook_npc_refuse_item(symbols, npc_id, reason),
            Self::NpcSaysRandom { npc_id } => cook_npc_says_random(symbols, npc_id),
            Self::NpcSays { npc_id, quote } => cook_npc_says(symbols, npc_id, quote),
            Self::AddFlag { flag } => Ok(TriggerAction::AddFlag(flag.clone())),
            Self::RemoveFlag { flag } => Ok(TriggerAction::RemoveFlag(flag.clone())),
            Self::AwardPoints { amount, reason } => Ok(TriggerAction::AwardPoints {
                amount: *amount,
                reason: reason.clone(),
            }),
            Self::SpawnItemCurrentRoom { item_id } => cook_spawn_item_current_room(symbols, item_id),
            Self::PushPlayerTo { room_id } => cook_push_player_to(symbols, room_id),
            Self::GiveItemToPlayer { npc_id, item_id } => cook_give_item_to_player(symbols, npc_id, item_id),
            Self::SetNpcState { npc_id, state } => cook_set_npc_state(symbols, npc_id, state.clone()),
            Self::ShowMessage { text } => Ok(TriggerAction::ShowMessage(text.clone())),
            Self::RevealExit {
                exit_from,
                exit_to,
                direction,
            } => cook_reveal_exit(symbols, exit_from, exit_to, direction),
            Self::SpawnItemInRoom { item_id, room_id } => cook_spawn_item_in_room(symbols, item_id, room_id),
            Self::SpawnItemInContainer { item_id, container_id } => {
                cook_spawn_item_in_container(symbols, item_id, container_id)
            },
            Self::DespawnItem { item_id } => cook_despawn_item(symbols, item_id),
            Self::LockItem { item_id } => cook_lock_item(symbols, item_id),
            Self::UnlockItem { item_id } => cook_unlock_item(symbols, item_id),
            Self::LockExit { from_room, direction } => cook_lock_exit(symbols, from_room, direction),
            Self::SpawnItemInInventory { item_id } => cook_spawn_item_in_inventory(symbols, item_id),
            Self::UnlockExit { from_room, direction } => cook_unlock_exit(symbols, from_room, direction),
            Self::DenyRead { reason } => Ok(TriggerAction::DenyRead(reason.clone())),
            Self::Conditional { condition, actions } => cook_conditional(symbols, condition, actions),
            Self::ScheduleIn {
                turns_ahead,
                actions,
                note,
            } => cook_schedule_in(symbols, *turns_ahead, actions, note.clone()),
            Self::ScheduleOn { on_turn, actions, note } => cook_schedule_on(symbols, *on_turn, actions, note.clone()),
            Self::ScheduleInIf {
                turns_ahead,
                condition,
                on_false,
                actions,
                note,
            } => cook_schedule_in_if(symbols, *turns_ahead, condition, on_false, actions, note.clone()),
            Self::ScheduleOnIf {
                on_turn,
                condition,
                on_false,
                actions,
                note,
            } => cook_schedule_on_if(symbols, *on_turn, condition, on_false, actions, note.clone()),
        }
    }
}

/*
 * "Cook" functions below convert RawTriggerActions to "fully cooked" TriggerActions
 */
fn cook_set_item_movability(symbols: &SymbolTable, item_sym: &str, movability: &Movability) -> Result<TriggerAction> {
    if let Some(&item_id) = symbols.items.get(item_sym) {
        Ok(TriggerAction::SetItemMovability {
            item_id,
            movability: movability.clone(),
        })
    } else {
        bail!("loading TriggerAction::SetItemMovability: symbol {item_sym} not found in table");
    }
}

fn cook_modify_item(symbols: &SymbolTable, item_sym: &str, raw_patch: &RawItemPatch) -> Result<TriggerAction> {
    if let Some(&item_id) = symbols.items.get(item_sym) {
        Ok(TriggerAction::ModifyItem {
            item_id,
            patch: raw_patch.to_patch(symbols)?,
        })
    } else {
        bail!("loading TriggerAction:ModifyItem: item symbol ({item_sym}) not in table");
    }
}

fn cook_modify_room(symbols: &SymbolTable, room_sym: &str, patch: &RawRoomPatch) -> Result<TriggerAction> {
    if let Some(room_id) = symbols.rooms.get(room_sym) {
        Ok(TriggerAction::ModifyRoom {
            room_id: *room_id,
            patch: patch.to_patch(symbols)?,
        })
    } else {
        bail!("loading TriggerAction:ModifyRoom: room symbol ({room_sym}) not in table");
    }
}

fn cook_modify_npc(symbols: &SymbolTable, npc_sym: &str, patch: &RawNpcPatch) -> Result<TriggerAction> {
    if let Some(npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::ModifyNpc {
            npc_id: *npc_id,
            patch: patch.to_patch(symbols)?,
        })
    } else {
        bail!("loading TriggerAction:ModifyNpc: npc symbol ({npc_sym}) not in table");
    }
}

fn cook_spawn_npc_into_room(symbols: &SymbolTable, npc_sym: &str, room_sym: &str) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym)
        && let Some(&room_id) = symbols.rooms.get(room_sym)
    {
        Ok(TriggerAction::SpawnNpcInRoom { npc_id, room_id })
    } else {
        bail!("loading SpawnNpcIntoRoom action: npc ({npc_sym}) or room ({room_sym}) not found in symbol table");
    }
}

fn cook_despawn_npc(symbols: &SymbolTable, npc_sym: &str) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::DespawnNpc { npc_id })
    } else {
        bail!("loading DespawnNpc action: npc ({npc_sym}) not found in symbol table");
    }
}

fn cook_set_npc_active(symbols: &SymbolTable, npc_sym: &str, active: bool) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::SetNpcActive { npc_id, active })
    } else {
        bail!("npc symbol '{npc_sym}' not found when loading raw SetNpcActive trigger action");
    }
}

fn cook_damage_npc(symbols: &SymbolTable, npc_sym: &str, cause: &str, amount: u32) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::DamageNpc {
            npc_id,
            cause: cause.to_string(),
            amount,
        })
    } else {
        bail!("npc symbol '{npc_sym}' not found when loading raw DamageNpc action");
    }
}

fn cook_damage_npc_ot(
    symbols: &SymbolTable,
    npc_sym: &str,
    cause: &str,
    amount: u32,
    turns: u32,
) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::DamageNpcOT {
            npc_id,
            cause: cause.to_string(),
            amount,
            turns,
        })
    } else {
        bail!("npc symbol '{npc_sym}' not found when loading raw DamageNpcOT action");
    }
}

fn cook_heal_npc(symbols: &SymbolTable, npc_sym: &str, cause: &str, amount: u32) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::HealNpc {
            npc_id,
            cause: cause.to_string(),
            amount,
        })
    } else {
        bail!("npc symbol '{npc_sym}' not found when loading raw HealNpc action");
    }
}

fn cook_heal_npc_ot(
    symbols: &SymbolTable,
    npc_sym: &str,
    cause: &str,
    amount: u32,
    turns: u32,
) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::HealNpcOT {
            npc_id,
            cause: cause.to_string(),
            amount,
            turns,
        })
    } else {
        bail!("npc symbol '{npc_sym}' not found when loading raw HealNpcOT action");
    }
}

fn cook_remove_npc_effect(symbols: &SymbolTable, npc_sym: &str, cause: &str) -> Result<TriggerAction> {
    if let Some(&npc_id) = symbols.characters.get(npc_sym) {
        Ok(TriggerAction::RemoveNpcEffect {
            npc_id,
            cause: cause.to_string(),
        })
    } else {
        bail!("npc symbol '{npc_sym}' not found when loading raw RemoveNpcEffect action");
    }
}

fn cook_set_container_state(
    symbols: &SymbolTable,
    item_sym: &str,
    state: Option<ContainerState>,
) -> Result<TriggerAction> {
    if let Some(&item_id) = symbols.items.get(item_sym) {
        Ok(TriggerAction::SetContainerState { item_id, state })
    } else {
        bail!("item symbol '{item_sym}' not found when loading raw SetContainerState trigger action");
    }
}

fn cook_replace_item(symbols: &SymbolTable, old_sym: &str, new_sym: &str) -> Result<TriggerAction> {
    if let Some(old_id) = symbols.items.get(old_sym)
        && let Some(new_id) = symbols.items.get(new_sym)
    {
        Ok(TriggerAction::ReplaceItem {
            old_id: *old_id,
            new_id: *new_id,
        })
    } else {
        bail!("item symbol '{old_sym}' or '{new_sym}' not found when loading raw ReplaceItem trigger action");
    }
}

fn cook_replace_drop_item(symbols: &SymbolTable, old_sym: &str, new_sym: &str) -> Result<TriggerAction> {
    if let Some(old_uuid) = symbols.items.get(old_sym)
        && let Some(new_uuid) = symbols.items.get(new_sym)
    {
        Ok(TriggerAction::ReplaceDropItem {
            old_id: *old_uuid,
            new_id: *new_uuid,
        })
    } else {
        bail!("item symbol '{old_sym}' or '{new_sym}' not found when loading raw ReplaceDropItem trigger action");
    }
}

fn cook_set_item_description(symbols: &SymbolTable, item_sym: &str, text: &str) -> Result<TriggerAction> {
    if let Some(item_id) = symbols.items.get(item_sym) {
        Ok(TriggerAction::SetItemDescription {
            item_id: *item_id,
            text: text.to_string(),
        })
    } else {
        bail!("item symbol '{item_sym}' not found when loading raw SetItemDescription trigger action");
    }
}

fn cook_barred_message(symbols: &SymbolTable, msg: &str, exit_from: &str, exit_to: &str) -> Result<TriggerAction> {
    if let Some(from_id) = symbols.rooms.get(exit_from)
        && let Some(to_id) = symbols.rooms.get(exit_to)
    {
        Ok(TriggerAction::SetBarredMessage {
            exit_from: *from_id,
            exit_to: *to_id,
            msg: msg.to_string(),
        })
    } else {
        bail!("failed room {exit_from} or destination {exit_to} lookup setting barred message {msg}");
    }
}

fn cook_npc_refuse_item(symbols: &SymbolTable, npc_symbol: &String, reason: &str) -> Result<TriggerAction> {
    if let Some(npc_id) = symbols.characters.get(npc_symbol) {
        Ok(TriggerAction::NpcRefuseItem {
            npc_id: *npc_id,
            reason: reason.to_owned(),
        })
    } else {
        bail!("raw action NpcRefuseItem({npc_symbol}, _): npc not found in symbols");
    }
}

fn cook_npc_says_random(symbols: &SymbolTable, npc_id: &String) -> Result<TriggerAction> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerAction::NpcSaysRandom { npc_id: *npc_uuid })
    } else {
        bail!("raw action NpcSaysRandom({npc_id}): token not in symbol table");
    }
}

fn cook_unlock_exit(
    symbols: &SymbolTable,
    from_room: &String,
    direction: &str,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(room_uuid) = symbols.rooms.get(from_room) {
        Ok(TriggerAction::UnlockExit {
            from_room: *room_uuid,
            direction: direction.to_owned(),
        })
    } else {
        bail!("raw action UnlockExit({from_room}): token not in symbol table");
    }
}

fn cook_spawn_item_in_inventory(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::SpawnItemInInventory(*item_uuid))
    } else {
        bail!("raw action SpawnItemInInventory({item_id}): token not in symbol table");
    }
}

fn cook_lock_exit(
    symbols: &SymbolTable,
    from_room: &String,
    direction: &str,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(room_uuid) = symbols.rooms.get(from_room) {
        Ok(TriggerAction::LockExit {
            direction: direction.to_owned(),
            from_room: *room_uuid,
        })
    } else {
        bail!("raw action LockExit({from_room}): token not in symbol table");
    }
}

fn cook_lock_item(symbols: &SymbolTable, item_id: &String) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::LockItem(*item_uuid))
    } else {
        bail!("raw action LockItem({item_id}): token not in symbol table");
    }
}

fn cook_despawn_item(symbols: &SymbolTable, item_id: &String) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::DespawnItem { item_id: *item_uuid })
    } else {
        bail!("raw action DespawnItem({item_id}): token not in symbol table");
    }
}

fn cook_spawn_item_in_container(
    symbols: &SymbolTable,
    item_id: &String,
    container_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id)
        && let Some(room_uuid) = symbols.items.get(container_id)
    {
        Ok(TriggerAction::SpawnItemInContainer {
            item_id: *item_uuid,
            container_id: *room_uuid,
        })
    } else {
        bail!("raw action SpawnItemInContainer({item_id},{container_id}): token not in symbol table");
    }
}

fn cook_spawn_item_in_room(
    symbols: &SymbolTable,
    item_id: &String,
    room_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id)
        && let Some(room_uuid) = symbols.rooms.get(room_id)
    {
        Ok(TriggerAction::SpawnItemInRoom {
            item_id: *item_uuid,
            room_id: *room_uuid,
        })
    } else {
        bail!("raw action SpawnItemInRoom({item_id},{room_id}): token not in symbol table");
    }
}

fn cook_reveal_exit(
    symbols: &SymbolTable,
    exit_from: &String,
    exit_to: &String,
    direction: &str,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(from_id) = symbols.rooms.get(exit_from)
        && let Some(to_id) = symbols.rooms.get(exit_to)
    {
        Ok(TriggerAction::RevealExit {
            direction: direction.to_owned(),
            exit_from: *from_id,
            exit_to: *to_id,
        })
    } else {
        bail!("raw action RevealExit({exit_from}, {exit_to}): token not in symbols");
    }
}

fn cook_unlock_item(symbols: &SymbolTable, target: &String) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(target_id) = symbols.items.get(target) {
        Ok(TriggerAction::UnlockItem(*target_id))
    } else {
        bail!("raw action UnlockItem({target}): token not found in symbols");
    }
}

fn cook_set_npc_state(
    symbols: &SymbolTable,
    npc_id: &String,
    state: NpcState,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerAction::SetNPCState {
            npc_id: *npc_uuid,
            state,
        })
    } else {
        bail!("raw action SetNpcMood({npc_id}, {state}): token not found in symbols");
    }
}

fn cook_give_item_to_player(
    symbols: &SymbolTable,
    npc_id: &String,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id)
        && let Some(item_uuid) = symbols.items.get(item_id)
    {
        Ok(TriggerAction::GiveItemToPlayer {
            npc_id: *npc_uuid,
            item_id: *item_uuid,
        })
    } else {
        bail!("raw action GiveItemToPlayer({npc_id},{item_id}): token not found in symbols");
    }
}

fn cook_push_player_to(symbols: &SymbolTable, room_id: &String) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(room_uuid) = symbols.rooms.get(room_id) {
        Ok(TriggerAction::PushPlayerTo(*room_uuid))
    } else {
        bail!("raw action PushPlayerTo({room_id}): token not found in symbols");
    }
}

fn cook_spawn_item_current_room(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::SpawnItemCurrentRoom(*item_uuid))
    } else {
        bail!("raw action SpawnItemCurrentRoom({item_id}): token not found in symbols");
    }
}

fn cook_npc_says(
    symbols: &SymbolTable,
    npc_id: &String,
    quote: &str,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerAction::NpcSays {
            npc_id: *npc_uuid,
            quote: quote.to_owned(),
        })
    } else {
        bail!("raw action NpcSays({npc_id},_): token not found in symbols")
    }
}

fn cook_scripted_actions(symbols: &SymbolTable, raw_actions: &[RawActionStmt]) -> Result<Vec<ScriptedAction>> {
    raw_actions
        .iter()
        .map(|raw_action| raw_action.to_action(symbols))
        .collect()
}

fn cook_schedule_in(
    symbols: &SymbolTable,
    turns_ahead: usize,
    raw_actions: &[RawActionStmt],
    note: Option<String>,
) -> Result<TriggerAction> {
    let cooked_actions = cook_scripted_actions(symbols, raw_actions)?;
    Ok(TriggerAction::ScheduleIn {
        turns_ahead,
        actions: cooked_actions,
        note,
    })
}

fn cook_schedule_on(
    symbols: &SymbolTable,
    on_turn: usize,
    raw_actions: &[RawActionStmt],
    note: Option<String>,
) -> Result<TriggerAction> {
    let cooked_actions = cook_scripted_actions(symbols, raw_actions)?;
    Ok(TriggerAction::ScheduleOn {
        on_turn,
        actions: cooked_actions,
        note,
    })
}

fn raw_event_condition_to_event_condition(symbols: &SymbolTable, rec: &RawEventCondition) -> Result<EventCondition> {
    Ok(match rec {
        RawEventCondition::Trigger(raw) => EventCondition::Trigger(raw.to_condition(symbols)?),
        RawEventCondition::All { all } => {
            let mut cooked = Vec::new();
            for c in all {
                cooked.push(raw_event_condition_to_event_condition(symbols, c)?);
            }
            EventCondition::All(cooked)
        },
        RawEventCondition::Any { any } => {
            let mut cooked = Vec::new();
            for c in any {
                cooked.push(raw_event_condition_to_event_condition(symbols, c)?);
            }
            EventCondition::Any(cooked)
        },
    })
}

fn raw_on_false_to_policy(raw: &RawOnFalsePolicy) -> OnFalsePolicy {
    match raw {
        RawOnFalsePolicy::Cancel => OnFalsePolicy::Cancel,
        RawOnFalsePolicy::RetryAfter { turns } => OnFalsePolicy::RetryAfter(*turns),
        RawOnFalsePolicy::RetryNextTurn => OnFalsePolicy::RetryNextTurn,
    }
}

fn cook_conditional(
    symbols: &SymbolTable,
    condition: &RawEventCondition,
    raw_actions: &[RawActionStmt],
) -> Result<TriggerAction> {
    let cooked_actions = cook_scripted_actions(symbols, raw_actions)?;
    let event_condition = raw_event_condition_to_event_condition(symbols, condition)?;
    Ok(TriggerAction::Conditional {
        condition: event_condition,
        actions: cooked_actions,
    })
}

fn cook_schedule_in_if(
    symbols: &SymbolTable,
    turns_ahead: usize,
    condition: &RawEventCondition,
    on_false: &RawOnFalsePolicy,
    raw_actions: &[RawActionStmt],
    note: Option<String>,
) -> Result<TriggerAction> {
    let cooked_actions = cook_scripted_actions(symbols, raw_actions)?;
    let ec = raw_event_condition_to_event_condition(symbols, condition)?;
    let policy = raw_on_false_to_policy(on_false);
    Ok(TriggerAction::ScheduleInIf {
        turns_ahead,
        condition: ec,
        on_false: policy,
        actions: cooked_actions,
        note,
    })
}

fn cook_schedule_on_if(
    symbols: &SymbolTable,
    on_turn: usize,
    condition: &RawEventCondition,
    on_false: &RawOnFalsePolicy,
    raw_actions: &[RawActionStmt],
    note: Option<String>,
) -> Result<TriggerAction> {
    let cooked_actions = cook_scripted_actions(symbols, raw_actions)?;
    let ec = raw_event_condition_to_event_condition(symbols, condition)?;
    let policy = raw_on_false_to_policy(on_false);
    Ok(TriggerAction::ScheduleOnIf {
        on_turn,
        condition: ec,
        on_false: policy,
        actions: cooked_actions,
        note,
    })
}
