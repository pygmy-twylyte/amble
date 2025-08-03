//! Trigger loader submodule for RawTriggerActions

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::{loader::SymbolTable, npc::NpcState, player::Flag, spinners::SpinnerType, trigger::TriggerAction};

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
    /// Convert the TOML representation of this action to a fully realized TriggerAction.
    pub fn to_action(&self, symbols: &SymbolTable) -> Result<TriggerAction> {
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
            Self::NpcRefuseItem { npc_id, reason } => cook_npc_refuse_item(symbols, npc_id, reason),
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
fn cook_npc_refuse_item(symbols: &SymbolTable, npc_symbol: &String, reason: &String) -> Result<TriggerAction> {
    if let Some(npc_id) = symbols.characters.get(npc_symbol) {
        Ok(TriggerAction::NpcRefuseItem {
            npc_id: *npc_id,
            reason: reason.to_string(),
        })
    } else {
        bail!("raw action NpcRefuseItem({npc_symbol}, _): npc not found in symbols");
    }
}

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
