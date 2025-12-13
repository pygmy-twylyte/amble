//! Raw trigger condition translation.
//!
//! Maps serialized condition definitions onto runtime trigger predicates,
//! resolving identifiers and validating referenced entities.

use std::collections::HashSet;

use anyhow::{Context, Ok, Result, bail};
use serde::Deserialize;

use crate::{
    command::IngestMode,
    item::{ItemAbility, ItemInteractionType},
    loader::SymbolTable,
    npc::NpcState,
    spinners::SpinnerType,
    trigger::TriggerCondition,
};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
#[rustfmt::skip]
pub enum RawTriggerCondition {
    ActOnItem { target_sym: String, action: ItemInteractionType, },
    Ambient { room_ids: Option<Vec<String>>, spinner: String, },
    Chance { one_in: f64 },
    ContainerHasItem { container_id: String, item_id: String, },
    Drop { item_id: String, },
    Enter { room_id: String, },
    FlagComplete { flag: String },
    FlagInProgress { flag: String },
    GiveToNpc { item_id: String, npc_id: String, },
    HasItem { item_id: String, },
    HasFlag { flag: String, },
    HasVisited { room_id: String, },
    InRoom { room_id: String, },
    Ingest { item_sym: String, mode: IngestMode},
    Insert { item_id: String, container_id: String, },
    Leave { room_id: String, },
    LookAt { item_id: String, },
    MissingFlag { flag: String, },
    MissingItem { item_id: String, },
    NpcDeath { npc_id: String, },
    NpcHasItem { npc_id: String, item_id: String, },
    NpcInState { npc_id: String, state: NpcState, },
    Open { item_id: String, },
    PlayerDeath,
    Take { item_id: String, },
    TakeFromNpc { item_id: String, npc_id: String, },
    TalkToNpc { npc_id: String },
    Touch { item_id: String },
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
    /// Convert this raw trigger condition into a runtime [`TriggerCondition`].
    ///
    /// # Errors
    /// Returns an error if any referenced symbols cannot be resolved while constructing the condition.
    pub fn to_condition(&self, symbols: &SymbolTable) -> Result<TriggerCondition> {
        match self {
            Self::Touch { item_id } => cook_touch_item(symbols, item_id),
            Self::Ingest { item_sym, mode } => cook_ingest(symbols, item_sym, *mode),
            Self::ActOnItem {
                target_sym: item_id,
                action,
            } => cook_act_on_item(symbols, item_id, *action),
            Self::TalkToNpc { npc_id } => cook_talk_to_npc(symbols, npc_id),
            Self::Ambient { room_ids, spinner } => cook_ambient(symbols, room_ids.as_ref(), spinner),
            Self::Chance { one_in } => Ok(TriggerCondition::Chance { one_in: *one_in }),
            Self::ContainerHasItem { container_id, item_id } => cook_container_has_item(symbols, container_id, item_id),
            Self::MissingFlag { flag } => Ok(TriggerCondition::MissingFlag(flag.clone())),
            Self::HasFlag { flag } => Ok(TriggerCondition::HasFlag(flag.clone())),
            Self::FlagComplete { flag } => Ok(TriggerCondition::FlagComplete(flag.clone())),
            Self::FlagInProgress { flag } => Ok(TriggerCondition::FlagInProgress(flag.clone())),
            Self::UseItem { item_id, ability } => cook_use_item(symbols, item_id, ability),
            Self::TakeFromNpc { item_id, npc_id } => cook_take_from_npc(symbols, item_id, npc_id),
            Self::Take { item_id } => cook_take(symbols, item_id),
            Self::Enter { room_id } => cook_enter(symbols, room_id),
            Self::GiveToNpc { item_id, npc_id } => cook_give_to_npc(symbols, item_id, npc_id),
            Self::PlayerDeath => Ok(TriggerCondition::PlayerDeath),
            Self::NpcDeath { npc_id } => cook_npc_death(symbols, npc_id),
            Self::Leave { room_id } => cook_leave(symbols, room_id),
            Self::LookAt { item_id } => cook_look_at(symbols, item_id),
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

fn cook_touch_item(symbols: &SymbolTable, item_sym: &str) -> Result<TriggerCondition> {
    if let Some(item_id) = symbols.items.get(item_sym) {
        Ok(TriggerCondition::Touch(*item_id))
    } else {
        bail!("converting raw condition Touch: item symbol '{item_sym}' not found")
    }
}

fn cook_ingest(symbols: &SymbolTable, item_sym: &str, mode: IngestMode) -> Result<TriggerCondition> {
    if let Some(item_id) = symbols.items.get(item_sym) {
        Ok(TriggerCondition::Ingest {
            item_id: *item_id,
            mode,
        })
    } else {
        bail!("converting raw condition Ingest: item symbol '{item_sym}' not found")
    }
}

fn cook_act_on_item(symbols: &SymbolTable, item_id: &str, action: ItemInteractionType) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::ActOnItem {
            target_id: *item_uuid,
            action,
        })
    } else {
        bail!("converting raw condition ActOnItem: item symbol '{item_id}' not found")
    }
}

fn cook_talk_to_npc(symbols: &SymbolTable, npc_symbol: &str) -> Result<TriggerCondition> {
    if let Some(npc_uuid) = symbols.characters.get(npc_symbol) {
        Ok(TriggerCondition::TalkToNpc(*npc_uuid))
    } else {
        bail!("converting raw condition TalkToNpc: npc symbol '{npc_symbol}' not found")
    }
}

fn cook_ambient(symbols: &SymbolTable, room_symbols: Option<&Vec<String>>, spinner: &str) -> Result<TriggerCondition> {
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
    Ok(TriggerCondition::Ambient {
        room_ids,
        spinner: SpinnerType::from_toml_key(spinner),
    })
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

fn cook_npc_death(symbols: &SymbolTable, npc_id: &str) -> Result<TriggerCondition> {
    if let Some(npc_uuid) = symbols.characters.get(npc_id) {
        Ok(TriggerCondition::NpcDeath(*npc_uuid))
    } else {
        bail!("raw condition NpcDeath({npc_id}): token not in symbols");
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

fn cook_look_at(symbols: &SymbolTable, item_id: &String) -> Result<TriggerCondition> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerCondition::LookAt(*item_uuid))
    } else {
        bail!("raw condition LookAt({item_id}): token not in symbols");
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
