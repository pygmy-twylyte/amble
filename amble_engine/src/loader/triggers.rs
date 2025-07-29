//! Helpers for loading [`Trigger`] definitions from TOML.
//!
//! Triggers combine game conditions with actions. This module converts the raw
//! text representation into fully linked structures that the engine can
//! evaluate at runtime.

use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, Result, bail};
use log::info;
use serde::Deserialize;

use crate::{
    item::{ItemAbility, ItemInteractionType},
    npc::NpcState,
    player::Flag,
    spinners::SpinnerType,
    trigger::{Trigger, TriggerAction, TriggerCondition},
};

use super::SymbolTable;

#[derive(Deserialize)]
struct RawTriggerFile {
    triggers: Vec<RawTrigger>,
}

#[derive(Debug, Deserialize)]
pub struct RawTrigger {
    pub name: String,
    pub conditions: Vec<RawTriggerCondition>,
    pub actions: Vec<RawTriggerAction>,
    #[serde(default)]
    pub only_once: bool,
}
impl RawTrigger {
    /// Convert a `RawTrigger` loaded from TOML to a `Trigger`
    /// # Errors
    /// - on failed symbol lookups / failure to convert any component of the trigger
    pub fn to_trigger(&self, symbols: &SymbolTable) -> Result<Trigger> {
        let mut conditions = Vec::new();
        let mut actions = Vec::new();
        for cond in &self.conditions {
            conditions.push(cond.to_condition(symbols)?);
        }
        for act in &self.actions {
            actions.push(act.to_action(symbols)?);
        }
        Ok(Trigger {
            name: self.name.to_string(),
            conditions,
            actions,
            only_once: self.only_once,
            fired: false,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[rustfmt::skip]
pub enum RawTriggerCondition {
    Ambient { room_ids: Option<Vec<String>>, spinner: SpinnerType, },
    ContainerHasItem { container_id: String, item_id: String, },
    Drop { item_id: String, },
    Enter { room_id: String, },
    GiveToNpc { item_id: String, npc_id: String, },
    HasItem { item_id: String, },
    HasFlag { flag: String, },
    HasVisited { room_id: String, },
    InRoom { room_id: String, },
    Insert { item_id: String, container_id: String, },
    Leave { room_id: String, },
    MissingFlag { flag: String, },
    MissingItem { item_id: String, },
    NpcHasItem { npc_id: String, item_id: String, },
    NpcInState { npc_id: String, state: NpcState, },
    Open { item_id: String, },
    Take { item_id: String, },
    TakeFromNpc { item_id: String, npc_id: String, },
    TalkToNpc { npc_id: String },
    Unlock { item_id: String, },
    UseItem { item_id: String, ability: ItemAbility, },
    UseItemOnItem {
        interaction: ItemInteractionType,
        target_id: String,
        tool_id: String,
    },
    WithNpc { npc_id: String, },
}
impl RawTriggerCondition {
    fn to_condition(&self, symbols: &SymbolTable) -> Result<TriggerCondition> {
        match self {
            Self::TalkToNpc { npc_id } => cook_talk_to_npc(symbols, npc_id),
            Self::Ambient { room_ids, spinner } => cook_ambient(symbols, room_ids.as_ref(), *spinner),
            Self::ContainerHasItem { container_id, item_id } => cook_container_has_item(symbols, container_id, item_id),
            Self::MissingFlag { flag } => Ok(TriggerCondition::MissingFlag(flag.to_string())),
            Self::HasFlag { flag } => Ok(TriggerCondition::HasFlag(flag.to_string())),
            Self::UseItem { item_id, ability } => cook_use_item(symbols, item_id, ability),
            Self::TakeFromNpc { item_id, npc_id } => cook_take_from_npc(symbols, item_id, npc_id),
            Self::Take { item_id } => cook_take(symbols, item_id),
            Self::Enter { room_id } => cook_enter(symbols, room_id),
            Self::GiveToNpc { item_id, npc_id } => cook_give_to_npc(symbols, item_id, npc_id),
            Self::Leave { room_id } => cook_leave(symbols, room_id),
            Self::Drop { item_id } => cook_drop(symbols, item_id),
            Self::Insert { item_id, container_id } => cook_insert(symbols, item_id, container_id),
            Self::Unlock { item_id } => cook_unlock(symbols, item_id),
            Self::Open { item_id } => cook_open(symbols, item_id),
            Self::HasItem { item_id } => cook_has_item(symbols, item_id),
            Self::MissingItem { item_id } => cook_missing_item(symbols, item_id),
            Self::WithNpc { npc_id } => cook_with_npc(symbols, npc_id),
            Self::HasVisited { room_id } => cook_has_visited(symbols, room_id),
            Self::InRoom { room_id } => cook_in_room(symbols, room_id),
            Self::NpcHasItem { npc_id, item_id } => cook_npc_has_item(symbols, npc_id, item_id),
            Self::NpcInState { npc_id, state } => cook_npc_in_state(symbols, npc_id, state.clone()),
            Self::UseItemOnItem {
                interaction,
                target_id,
                tool_id,
            } => cook_use_item_on_item(symbols, *interaction, target_id, tool_id),
        }
    }
}

//
// "COOK" helper functions convert raw trigger components to "cooked" real instances with
// validated uuids.
//
fn cook_talk_to_npc(symbols: &SymbolTable, npc_symbol: &str) -> Result<TriggerCondition> {
    if let Some(npc_uuid) = symbols.characters.get(npc_symbol) {
        Ok(TriggerCondition::TalkToNpc(*npc_uuid))
    } else {
        bail!("converting raw condition TalkToNpc: npc symbol '{npc_symbol}' not found")
    }
}

fn cook_ambient(
    symbols: &SymbolTable,
    room_symbols: Option<&Vec<String>>,
    spinner: SpinnerType,
) -> Result<TriggerCondition> {
    let mut room_ids = HashSet::new();
    if let Some(syms) = room_symbols {
        for sym in syms {
            let uuid = symbols
                .rooms
                .get(sym)
                .with_context(|| format!("converting raw condition Ambient: room symbol '{sym}' not found"))?;
            room_ids.insert(*uuid);
        }
    }
    Ok(TriggerCondition::Ambient { room_ids, spinner })
}

fn cook_use_item_on_item(
    symbols: &SymbolTable,
    interaction: ItemInteractionType,
    target_id: &String,
    tool_id: &String,
) -> Result<TriggerCondition> {
    if let Some(target_uuid) = symbols.items.get(target_id)
        && let Some(tool_uuid) = symbols.items.get(tool_id)
    {
        Ok(TriggerCondition::UseItemOnItem {
            interaction,
            target_id: *target_uuid,
            tool_id: *tool_uuid,
        })
    } else {
        bail!("raw condition UseItemOnItem(_, {target_id}, {tool_id}): token not in symbols")
    }
}

fn cook_npc_in_state(symbols: &SymbolTable, npc_id: &String, mood: NpcState) -> Result<TriggerCondition> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerCondition::NpcInState {
            npc_id: *npc_uuid,
            mood,
        })
    } else {
        bail!("raw condition NpcInMood({npc_id}, {mood}): token not in symbols");
    }
}

fn cook_npc_has_item(symbols: &SymbolTable, npc_id: &String, item_id: &String) -> Result<TriggerCondition> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id)
        && let Some(item_uuid) = symbols.items.get(item_id)
    {
        Ok(TriggerCondition::NpcHasItem {
            npc_id: *npc_uuid,
            item_id: *item_uuid,
        })
    } else {
        bail!("raw condition NpcHasItem({npc_id},{item_id}): token not in symbols");
    }
}

fn cook_in_room(symbols: &SymbolTable, room_id: &String) -> Result<TriggerCondition> {
    if let Some(room_uuid) = symbols.rooms.get(room_id) {
        Ok(TriggerCondition::InRoom(*room_uuid))
    } else {
        bail!("raw condition InRoom({room_id}): token not in symbols");
    }
}

fn cook_has_visited(symbols: &SymbolTable, room_id: &String) -> Result<TriggerCondition> {
    if let Some(room_uuid) = symbols.rooms.get(room_id) {
        Ok(TriggerCondition::HasVisited(*room_uuid))
    } else {
        bail!("raw condition HasVisited({room_id}): token not in symbols");
    }
}

fn cook_with_npc(symbols: &SymbolTable, npc_id: &String) -> Result<TriggerCondition> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerCondition::WithNpc(*npc_uuid))
    } else {
        bail!("raw condition WithNpc({npc_id}): token not in symbols");
    }
}

fn cook_missing_item(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::MissingItem(*item_uuid))
    } else {
        bail!("raw condition MissingItem({item_id}): token not in symbols");
    }
}

fn cook_has_item(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::HasItem(*item_uuid))
    } else {
        bail!("raw condition HasItem({item_id}): token not in symbols");
    }
}

fn cook_open(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::Open(*item_uuid))
    } else {
        bail!("raw condition Open({item_id}): token not in symbols");
    }
}

fn cook_unlock(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::Unlock(*item_uuid))
    } else {
        bail!("raw condition Unlock({item_id}): token not in symbols");
    }
}

fn cook_insert(symbols: &SymbolTable, item_id: &String, container_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id)
        && let Some(container_uuid) = symbols.items.get(container_id)
    {
        Ok(TriggerCondition::Insert {
            item: *item_uuid,
            container: *container_uuid,
        })
    } else {
        bail!("raw condition Insert({item_id}, {container_id}): token not in symbols");
    }
}

fn cook_drop(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::Drop(*item_uuid))
    } else {
        bail!("raw condition Drop({item_id}): token not in symbols");
    }
}

fn cook_leave(symbols: &SymbolTable, room_id: &String) -> Result<TriggerCondition> {
    if let Some(room_uuid) = symbols.rooms.get(room_id) {
        Ok(TriggerCondition::Leave(*room_uuid))
    } else {
        bail!("raw condition Leave({room_id}): token not in symbols");
    }
}

fn cook_give_to_npc(symbols: &SymbolTable, item_id: &String, npc_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id)
        && let Some(npc_uuid) = symbols.characters.get(npc_id)
    {
        Ok(TriggerCondition::GiveToNpc {
            item_id: *item_uuid,
            npc_id: *npc_uuid,
        })
    } else {
        bail!("raw condition GiveToNpc({item_id},{npc_id}): token not in symbols");
    }
}

fn cook_enter(symbols: &SymbolTable, room_id: &String) -> Result<TriggerCondition> {
    if let Some(room_uuid) = symbols.rooms.get(room_id) {
        Ok(TriggerCondition::Enter(*room_uuid))
    } else {
        bail!("raw condition Enter({room_id}): token not in symbols");
    }
}

fn cook_take(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::Take(*item_uuid))
    } else {
        bail!("raw condition Take({item_id}): token not in symbols");
    }
}

fn cook_take_from_npc(symbols: &SymbolTable, item_id: &String, npc_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id)
        && let Some(npc_uuid) = symbols.characters.get(npc_id)
    {
        Ok(TriggerCondition::TakeFromNpc {
            item_id: *item_uuid,
            npc_id: *npc_uuid,
        })
    } else {
        bail!("raw condition TakeFromNpc({item_id}, {npc_id}): token not in symbols")
    }
}

fn cook_use_item(symbols: &SymbolTable, item_id: &String, ability: &ItemAbility) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::UseItem {
            item_id: *item_uuid,
            ability: *ability,
        })
    } else {
        bail!("raw condition UseItem({item_id}, {ability}): token not in symbols");
    }
}

fn cook_container_has_item(symbols: &SymbolTable, container_id: &String, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id)
        && let Some(container_uuid) = symbols.items.get(container_id)
    {
        Ok(TriggerCondition::ContainerHasItem {
            container_id: *container_uuid,
            item_id: *item_uuid,
        })
    } else {
        bail!("raw condition ContainerHasItem({container_id},{item_id}): item token not in symbols");
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawTriggerAction {
    AddFlag {
        flag: Flag,
    },
    AddSpinnerWedge {
        spinner: SpinnerType,
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
    LockItem {
        item_id: String,
    },
    LockExit {
        from_room: String,
        direction: String,
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
    RestrictItem {
        item_id: String,
    },
    RevealExit {
        exit_from: String,
        exit_to: String,
        direction: String,
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
        spinner: SpinnerType,
    },
    UnlockItem {
        item_id: String,
    },
    UnlockExit {
        from_room: String,
        direction: String,
    },
}
impl RawTriggerAction {
    fn to_action(&self, symbols: &SymbolTable) -> Result<TriggerAction> {
        match self {
            Self::AddSpinnerWedge { spinner, text, width } => Ok(TriggerAction::AddSpinnerWedge {
                spinner: *spinner,
                text: text.clone(),
                width: *width,
            }),
            Self::ResetFlag { flag } => Ok(TriggerAction::ResetFlag(flag.to_string())),
            Self::AdvanceFlag { flag } => Ok(TriggerAction::AdvanceFlag(flag.to_string())),
            Self::SpinnerMessage { spinner } => Ok(TriggerAction::SpinnerMessage { spinner: *spinner }),
            Self::RestrictItem { item_id } => cook_restrict_item(symbols, item_id),
            Self::NpcSaysRandom { npc_id } => cook_npc_says_random(symbols, npc_id),
            Self::NpcSays { npc_id, quote } => cook_npc_says(symbols, npc_id, quote),
            Self::AddFlag { flag } => Ok(TriggerAction::AddFlag(flag.clone())),
            Self::RemoveFlag { flag } => Ok(TriggerAction::RemoveFlag(flag.to_string())),
            Self::AwardPoints { amount } => Ok(TriggerAction::AwardPoints(*amount)),
            Self::SpawnItemCurrentRoom { item_id } => cook_spawn_item_current_room(symbols, item_id),
            Self::PushPlayerTo { room_id } => cook_push_player_to(symbols, room_id),
            Self::GiveItemToPlayer { npc_id, item_id } => cook_give_item_to_player(symbols, npc_id, item_id),
            Self::SetNpcState { npc_id, state } => cook_set_npc_state(symbols, npc_id, state.clone()),
            Self::ShowMessage { text } => Ok(TriggerAction::ShowMessage(text.to_string())),
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
            Self::DenyRead { reason } => Ok(TriggerAction::DenyRead(reason.to_string())),
        }
    }
}

/*
 * "Cook" functions below convert RawTriggerActions to TriggerActions
 */

fn cook_restrict_item(symbols: &SymbolTable, item_id: &String) -> Result<TriggerAction> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::RestrictItem(*item_uuid))
    } else {
        bail!("raw action RestrictItem({item_id}): item not found in symbols");
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
    direction: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(room_uuid) = symbols.rooms.get(from_room) {
        Ok(TriggerAction::UnlockExit {
            from_room: *room_uuid,
            direction: direction.to_string(),
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
    direction: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(room_uuid) = symbols.rooms.get(from_room) {
        Ok(TriggerAction::LockExit {
            direction: direction.to_string(),
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
    direction: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(from_id) = symbols.rooms.get(exit_from)
        && let Some(to_id) = symbols.rooms.get(exit_to)
    {
        Ok(TriggerAction::RevealExit {
            direction: direction.to_string(),
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
    quote: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerAction::NpcSays {
            npc_id: *npc_uuid,
            quote: quote.to_string(),
        })
    } else {
        bail!("raw action NpcSays({npc_id},_): token not found in symbols")
    }
}

/// Load `RawTrigger` representations from TOML
/// # Errors
/// - on failed file access or TOML parsing
pub fn load_raw_triggers(toml_path: &Path) -> Result<Vec<RawTrigger>> {
    let trigger_file =
        fs::read_to_string(toml_path).with_context(|| format!("reading triggers from \"{}\"", toml_path.display()))?;
    let wrapper: RawTriggerFile = toml::from_str(&trigger_file)?;
    info!(
        "{} raw triggers loaded from '{}'",
        wrapper.triggers.len(),
        toml_path.display(),
    );
    Ok(wrapper.triggers)
}
/// Build `Triggers` from `RawTriggers` loaded from TOML.
/// # Errors
/// - on failed conversion of any raw to real trigger
pub fn build_triggers(raw_triggers: &[RawTrigger], symbols: &SymbolTable) -> Result<Vec<Trigger>> {
    let triggers: Vec<Trigger> = raw_triggers
        .iter()
        .map(|rt| rt.to_trigger(symbols))
        .collect::<Result<_, _>>()?;
    info!("{} triggers built from raw triggers", triggers.len());
    Ok(triggers)
}
