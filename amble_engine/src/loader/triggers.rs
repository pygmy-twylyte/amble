use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use log::info;
use serde::Deserialize;

use crate::{
    item::{ItemAbility, ItemInteractionType},
    npc::NpcMood,
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
pub enum RawTriggerCondition {
    ContainerHasItem {
        container_id: String,
        item_id: String,
    },
    Drop {
        item_id: String,
    },
    Enter {
        room_id: String,
    },
    GiveToNpc {
        item_id: String,
        npc_id: String,
    },
    HasItem {
        item_id: String,
    },
    HasAchievement {
        achievement: String,
    },
    HasVisited {
        room_id: String,
    },
    InRoom {
        room_id: String,
    },
    Insert {
        item_id: String,
        container_id: String,
    },
    Leave {
        room_id: String,
    },
    MissingAchievement {
        achievement: String,
    },
    MissingItem {
        item_id: String,
    },
    NpcHasItem {
        npc_id: String,
        item_id: String,
    },
    NpcInMood {
        npc_id: String,
        mood: NpcMood,
    },
    Open {
        item_id: String,
    },
    Take {
        item_id: String,
    },
    TakeFromNpc {
        item_id: String,
        npc_id: String,
    },
    Unlock {
        item_id: String,
    },
    UseItem {
        item_id: String,
        ability: ItemAbility,
    },
    UseItemOnItem {
        interaction: ItemInteractionType,
        target_id: String,
        tool_id: String,
    },
    WithNpc {
        npc_id: String,
    },
}
impl RawTriggerCondition {
    fn to_condition(&self, symbols: &SymbolTable) -> Result<TriggerCondition> {
        match self {
            RawTriggerCondition::ContainerHasItem {
                container_id,
                item_id,
            } => cook_container_has_item(symbols, container_id, item_id),
            RawTriggerCondition::MissingAchievement { achievement } => Ok(
                TriggerCondition::MissingAchievement(achievement.to_string()),
            ),
            RawTriggerCondition::HasAchievement { achievement } => {
                Ok(TriggerCondition::HasAchievement(achievement.to_string()))
            }
            RawTriggerCondition::UseItem { item_id, ability } => {
                cook_use_item(symbols, item_id, ability)
            }
            RawTriggerCondition::TakeFromNpc { item_id, npc_id } => {
                cook_take_from_npc(symbols, item_id, npc_id)
            }
            RawTriggerCondition::Take { item_id } => cook_take(symbols, item_id),
            RawTriggerCondition::Enter { room_id } => cook_enter(symbols, room_id),
            RawTriggerCondition::GiveToNpc { item_id, npc_id } => {
                cook_give_to_npc(symbols, item_id, npc_id)
            }
            RawTriggerCondition::Leave { room_id } => cook_leave(symbols, room_id),
            RawTriggerCondition::Drop { item_id } => cook_drop(symbols, item_id),
            RawTriggerCondition::Insert {
                item_id,
                container_id,
            } => cook_insert(symbols, item_id, container_id),
            RawTriggerCondition::Unlock { item_id } => cook_unlock(symbols, item_id),
            RawTriggerCondition::Open { item_id } => cook_open(symbols, item_id),
            RawTriggerCondition::HasItem { item_id } => cook_has_item(symbols, item_id),
            RawTriggerCondition::MissingItem { item_id } => cook_missing_item(symbols, item_id),
            RawTriggerCondition::WithNpc { npc_id } => cook_with_npc(symbols, npc_id),
            RawTriggerCondition::HasVisited { room_id } => cook_has_visited(symbols, room_id),
            RawTriggerCondition::InRoom { room_id } => cook_in_room(symbols, room_id),
            RawTriggerCondition::NpcHasItem { npc_id, item_id } => {
                cook_npc_has_item(symbols, npc_id, item_id)
            }
            RawTriggerCondition::NpcInMood { npc_id, mood } => {
                cook_npc_in_mood(symbols, npc_id, *mood)
            }
            RawTriggerCondition::UseItemOnItem {
                interaction,
                target_id,
                tool_id,
            } => cook_use_item_on_item(symbols, *interaction, target_id, tool_id),
        }
    }
}

//
// "COOK" helper functions convert raw trigger components to "cooked" real instances
//

fn cook_use_item_on_item(
    symbols: &SymbolTable,
    interaction: ItemInteractionType,
    target_id: &String,
    tool_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let target_uuid = symbols
        .items
        .get(target_id)
        .with_context(|| format!("UseItemOnItem({target_id},_,_): token not in symbols"))?;
    let tool_uuid = symbols
        .items
        .get(tool_id)
        .with_context(|| format!("UseItemOnItem(_,_,{tool_id}): token not in symbols"))?;
    Ok(TriggerCondition::UseItemOnItem {
        interaction,
        target_id: *target_uuid,
        tool_id: *tool_uuid,
    })
}

fn cook_npc_in_mood(
    symbols: &SymbolTable,
    npc_id: &String,
    mood: NpcMood,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let npc_uuid = symbols
        .characters
        .get(npc_id)
        .with_context(|| format!("NpcInMood({npc_id},_): token not in symbols"))?;
    Ok(TriggerCondition::NpcInMood {
        npc_id: *npc_uuid,
        mood,
    })
}

fn cook_npc_has_item(
    symbols: &SymbolTable,
    npc_id: &String,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let npc_uuid = symbols
        .characters
        .get(npc_id)
        .with_context(|| format!("NpcHasItem({npc_id},_): token not in symbols"))?;
    let item_uuid = symbols
        .items
        .get(item_id)
        .with_context(|| format!("NpcHasItem(_,{item_id}): token not in symbols"))?;
    Ok(TriggerCondition::NpcHasItem {
        npc_id: *npc_uuid,
        item_id: *item_uuid,
    })
}

fn cook_in_room(
    symbols: &SymbolTable,
    room_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let room_uuid = symbols
        .rooms
        .get(room_id)
        .with_context(|| format!("InRoom({room_id}): token not in symbols"))?;
    Ok(TriggerCondition::InRoom(*room_uuid))
}

fn cook_has_visited(
    symbols: &SymbolTable,
    room_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let room_uuid = symbols
        .rooms
        .get(room_id)
        .with_context(|| format!("HasVisited({room_id}): token not in symbols"))?;
    Ok(TriggerCondition::HasVisited(*room_uuid))
}

fn cook_with_npc(
    symbols: &SymbolTable,
    npc_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let npc_uuid = symbols
        .characters
        .get(npc_id)
        .ok_or_else(|| anyhow!("WithNpc({}): token not in symbols", npc_id))?;
    Ok(TriggerCondition::WithNpc(*npc_uuid))
}

fn cook_missing_item(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols
        .items
        .get(item_id)
        .ok_or_else(|| anyhow!("MissingItem({}): token not in symbols", item_id))?;
    Ok(TriggerCondition::MissingItem(*item_uuid))
}

fn cook_has_item(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or_else(|| {
        anyhow!(
            "Event:HasItem({}) load error: item token not in symbols",
            item_id
        )
    })?;
    Ok(TriggerCondition::HasItem(*item_uuid))
}

fn cook_open(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
        "Event:Open({}) load error: item token not in symbols",
        item_id
    ))?;
    Ok(TriggerCondition::Open(*item_uuid))
}

fn cook_unlock(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
        "Event:Unlock({}) load error: item token not in symbols",
        item_id
    ))?;
    Ok(TriggerCondition::Unlock(*item_uuid))
}

fn cook_insert(
    symbols: &SymbolTable,
    item_id: &String,
    container_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
        "Event:Insert({},_) load error: item token not in symbols",
        item_id
    ))?;
    let container_uuid = symbols.items.get(container_id).ok_or(anyhow!(
        "Event:Insert(_,{}) load error: container token not in symbols",
        container_id
    ))?;
    Ok(TriggerCondition::Insert {
        item: *item_uuid,
        container: *container_uuid,
    })
}

fn cook_drop(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
        "Event:Drop({}) load error: item token not in symbols",
        item_id
    ))?;
    Ok(TriggerCondition::Drop(*item_uuid))
}

fn cook_leave(
    symbols: &SymbolTable,
    room_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let room_uuid = symbols.rooms.get(room_id).ok_or(anyhow!(
        "Event:Leave({}) load error: room token not in symbols",
        room_id
    ))?;
    Ok(TriggerCondition::Leave(*room_uuid))
}

fn cook_give_to_npc(
    symbols: &SymbolTable,
    item_id: &String,
    npc_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or_else(|| {
        anyhow!(
            "GiveToNpc({}, _) load error: item token not in symbols",
            item_id
        )
    })?;
    let npc_uuid = symbols
        .characters
        .get(npc_id)
        .ok_or_else(|| anyhow!("GiveToNpc(_,{}): token not in symbols", npc_id))?;
    Ok(TriggerCondition::GiveToNpc {
        item_id: *item_uuid,
        npc_id: *npc_uuid,
    })
}

fn cook_enter(
    symbols: &SymbolTable,
    room_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let room_uuid = symbols.rooms.get(room_id).ok_or(anyhow!(
        "Enter({}) load error: room token not in symbols",
        room_id
    ))?;
    Ok(TriggerCondition::Enter(*room_uuid))
}

fn cook_take(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
        "Take({}) load error: item token not in symbols",
        item_id
    ))?;
    Ok(TriggerCondition::Take(*item_uuid))
}

fn cook_take_from_npc(
    symbols: &SymbolTable,
    item_id: &String,
    npc_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).ok_or_else(|| {
        anyhow!(
            "TakeFromNpc({}, _) load error: item token not in symbols",
            item_id
        )
    })?;
    let npc_uuid = symbols
        .characters
        .get(npc_id)
        .ok_or_else(|| anyhow!("TakeFromNpc(_,{}): token not in symbols", npc_id))?;
    Ok(TriggerCondition::TakeFromNpc {
        item_id: *item_uuid,
        npc_id: *npc_uuid,
    })
}

fn cook_use_item(
    symbols: &SymbolTable,
    item_id: &String,
    ability: &ItemAbility,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols
        .items
        .get(item_id)
        .ok_or_else(|| anyhow!("UseItem({}) load error: item token not in symbols", item_id))?;
    Ok(TriggerCondition::UseItem {
        item_id: *item_uuid,
        ability: *ability,
    })
}

fn cook_container_has_item(
    symbols: &SymbolTable,
    container_id: &String,
    item_id: &String,
) -> std::result::Result<TriggerCondition, anyhow::Error> {
    let item_uuid = symbols
        .items
        .get(item_id)
        .with_context(|| format!("item {item_id} not in symbol table"))?;
    let container_uuid = symbols
        .items
        .get(container_id)
        .with_context(|| format!("container {container_id} not in symbol table"))?;
    Ok(TriggerCondition::ContainerHasItem {
        container_id: *container_uuid,
        item_id: *item_uuid,
    })
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawTriggerAction {
    AddAchievement {
        achievement: String,
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
    PushPlayerTo {
        room_id: String,
    },
    RevealExit {
        exit_from: String,
        exit_to: String,
        direction: String,
    },
    SetNpcMood {
        npc_id: String,
        mood: NpcMood,
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
            RawTriggerAction::NpcSays { npc_id, quote } => cook_npc_says(symbols, npc_id, quote),
            RawTriggerAction::AddAchievement { achievement: task } => {
                Ok(TriggerAction::AddAchievement(task.to_string()))
            }
            RawTriggerAction::AwardPoints { amount } => Ok(TriggerAction::AwardPoints(*amount)),
            RawTriggerAction::SpawnItemCurrentRoom { item_id } => {
                cook_spawn_item_current_room(symbols, item_id)
            }
            RawTriggerAction::PushPlayerTo { room_id } => cook_push_player_to(symbols, room_id),
            RawTriggerAction::GiveItemToPlayer { npc_id, item_id } => {
                cook_give_item_to_player(symbols, npc_id, item_id)
            }
            RawTriggerAction::SetNpcMood { npc_id, mood } => {
                cook_set_npc_mood(symbols, npc_id, *mood)
            }
            RawTriggerAction::ShowMessage { text } => {
                Ok(TriggerAction::ShowMessage(text.to_string()))
            }
            RawTriggerAction::UnlockItem { item_id: target } => cook_unlock_item(symbols, target),
            RawTriggerAction::RevealExit {
                exit_from,
                exit_to,
                direction,
            } => cook_reveal_exit(symbols, exit_from, exit_to, direction),
            RawTriggerAction::SpawnItemInRoom { item_id, room_id } => {
                cook_spawn_item_in_room(symbols, item_id, room_id)
            }
            RawTriggerAction::SpawnItemInContainer {
                item_id,
                container_id,
            } => cook_spawn_item_in_container(symbols, item_id, container_id),
            RawTriggerAction::DespawnItem { item_id } => cook_despawn_item(symbols, item_id),
            RawTriggerAction::LockItem { item_id } => cook_lock_item(symbols, item_id),
            RawTriggerAction::LockExit {
                from_room,
                direction,
            } => cook_lock_exit(symbols, from_room, direction),
            RawTriggerAction::SpawnItemInInventory { item_id } => {
                cook_spawn_item_in_inventory(symbols, item_id)
            }
            RawTriggerAction::UnlockExit {
                from_room,
                direction,
            } => cook_unlock_exit(symbols, from_room, direction),
            RawTriggerAction::DenyRead { reason } => {
                Ok(TriggerAction::DenyRead(reason.to_string()))
            }
        }
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
        Err(anyhow!(
            "RawTriggerAction:UnlockRoom >> unknown room ({})",
            from_room
        ))
    }
}

fn cook_spawn_item_in_inventory(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::SpawnItemInInventory(*item_uuid))
    } else {
        Err(anyhow!(
            "RawTriggerAction:SpawnItemInInventory >> unknown item ({})",
            item_id
        ))
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
        Err(anyhow!(
            "RawTriggerAction:LockRoom >> unknown room ({})",
            from_room
        ))
    }
}

fn cook_lock_item(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::LockItem(*item_uuid))
    } else {
        Err(anyhow!(
            "RawTriggerAction:LockItem >> unknown item ({})",
            item_id
        ))
    }
}

fn cook_despawn_item(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        Ok(TriggerAction::DespawnItem {
            item_id: *item_uuid,
        })
    } else {
        Err(anyhow!(
            "RawTriggerAction:DespawnItem item_id ({}) not found",
            item_id
        ))
    }
}

fn cook_spawn_item_in_container(
    symbols: &SymbolTable,
    item_id: &String,
    container_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        if let Some(room_uuid) = symbols.items.get(container_id) {
            Ok(TriggerAction::SpawnItemInContainer {
                item_id: *item_uuid,
                container_id: *room_uuid,
            })
        } else {
            Err(anyhow!(
                "container token {} not in symbols when converting raw SpawnItemInContainer",
                container_id
            ))
        }
    } else {
        Err(anyhow!(
            "item token {} not found in symbols when converting raw SpawnItemInContainer",
            item_id
        ))
    }
}

fn cook_spawn_item_in_room(
    symbols: &SymbolTable,
    item_id: &String,
    room_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(item_uuid) = symbols.items.get(item_id) {
        if let Some(room_uuid) = symbols.rooms.get(room_id) {
            Ok(TriggerAction::SpawnItemInRoom {
                item_id: *item_uuid,
                room_id: *room_uuid,
            })
        } else {
            Err(anyhow!(
                "room token {} not in symbols when converting raw SpawnItemInRoom",
                room_id
            ))
        }
    } else {
        Err(anyhow!(
            "item token {} not found in symbols when converting raw SpawnItemInRoom",
            item_id
        ))
    }
}

fn cook_reveal_exit(
    symbols: &SymbolTable,
    exit_from: &String,
    exit_to: &String,
    direction: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    let exit_from_id = symbols
        .rooms
        .get(exit_from)
        .with_context(|| format!("room symbol missing: {exit_from}"))?;
    let exit_to_id = symbols
        .rooms
        .get(exit_to)
        .with_context(|| format!("room symbol missing: {exit_to}"))?;
    Ok(TriggerAction::RevealExit {
        direction: direction.to_string(),
        exit_from: *exit_from_id,
        exit_to: *exit_to_id,
    })
}

fn cook_unlock_item(
    symbols: &SymbolTable,
    target: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    if let Some(target_id) = symbols.items.get(target) {
        Ok(TriggerAction::UnlockItem(*target_id))
    } else {
        Err(anyhow!(
            "couldn't find target ({}) in symbol table for an UnlockItem trigger",
            target
        ))
    }
}

fn cook_set_npc_mood(
    symbols: &SymbolTable,
    npc_id: &String,
    mood: NpcMood,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    let npc_uuid = symbols
        .characters
        .get(npc_id)
        .with_context(|| format!("converting RawTriggerAction({npc_id}, {mood:?})"))?;
    Ok(TriggerAction::SetNPCMood {
        npc_id: *npc_uuid,
        mood,
    })
}

fn cook_give_item_to_player(
    symbols: &SymbolTable,
    npc_id: &String,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    let npc_uuid = symbols.characters.get(npc_id).with_context(|| {
        format!("loading GiveItemToPlayer({npc_id},_): token id not in symbol table")
    })?;
    let item_uuid = symbols.items.get(item_id).with_context(|| {
        format!("loading GiveItemToPlayer(_,{item_id}): token id not in symbol table")
    })?;
    Ok(TriggerAction::GiveItemToPlayer {
        npc_id: *npc_uuid,
        item_id: *item_uuid,
    })
}

fn cook_push_player_to(
    symbols: &SymbolTable,
    room_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    let room_uuid = symbols.rooms.get(room_id).with_context(|| {
        format!("loading PushPlayerTo({room_id}): token id not in symbol table")
    })?;
    Ok(TriggerAction::PushPlayerTo(*room_uuid))
}

fn cook_spawn_item_current_room(
    symbols: &SymbolTable,
    item_id: &String,
) -> std::result::Result<TriggerAction, anyhow::Error> {
    let item_uuid = symbols.items.get(item_id).with_context(|| {
        format!("loading SpawnItemCurrentRoom({item_id}): token id not in symbol table")
    })?;
    Ok(TriggerAction::SpawnItemCurrentRoom(*item_uuid))
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
        bail!("RawTriggerAction::NpcSays({npc_id},_): token not in symbol table")
    }
}

/// Load raw trigger representations from TOML
pub fn load_raw_triggers(toml_path: &Path) -> Result<Vec<RawTrigger>> {
    let trigger_file = fs::read_to_string(toml_path)
        .with_context(|| format!("reading triggers from {toml_path:?}"))?;
    let wrapper: RawTriggerFile = toml::from_str(&trigger_file)?;
    info!(
        "{} raw triggers loaded from {:?}",
        wrapper.triggers.len(),
        toml_path
    );
    Ok(wrapper.triggers)
}
/// Build triggers from raw triggers.
pub fn build_triggers(raw_triggers: &[RawTrigger], symbols: &SymbolTable) -> Result<Vec<Trigger>> {
    let triggers: Vec<Trigger> = raw_triggers
        .iter()
        .map(|rt| rt.to_trigger(symbols))
        .collect::<Result<_, _>>()?;
    info!("{} triggers built from raw triggers", triggers.len());
    Ok(triggers)
}
