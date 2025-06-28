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
            } => {
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
            RawTriggerCondition::MissingAchievement { achievement } => Ok(
                TriggerCondition::MissingAchievement(achievement.to_string()),
            ),
            RawTriggerCondition::HasAchievement { achievement } => {
                Ok(TriggerCondition::HasAchievement(achievement.to_string()))
            }
            RawTriggerCondition::UseItem { item_id, ability } => {
                let item_uuid = symbols.items.get(item_id).ok_or_else(|| {
                    anyhow!("UseItem({}) load error: item token not in symbols", item_id)
                })?;
                Ok(TriggerCondition::UseItem {
                    item_id: *item_uuid,
                    ability: *ability,
                })
            }
            RawTriggerCondition::TakeFromNpc { item_id, npc_id } => {
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
            RawTriggerCondition::Take { item_id } => {
                let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
                    "Take({}) load error: item token not in symbols",
                    item_id
                ))?;
                Ok(TriggerCondition::Take(*item_uuid))
            }
            RawTriggerCondition::Enter { room_id } => {
                let room_uuid = symbols.rooms.get(room_id).ok_or(anyhow!(
                    "Enter({}) load error: room token not in symbols",
                    room_id
                ))?;
                Ok(TriggerCondition::Enter(*room_uuid))
            }
            RawTriggerCondition::GiveToNpc { item_id, npc_id } => {
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
            RawTriggerCondition::Leave { room_id } => {
                let room_uuid = symbols.rooms.get(room_id).ok_or(anyhow!(
                    "Event:Leave({}) load error: room token not in symbols",
                    room_id
                ))?;
                Ok(TriggerCondition::Leave(*room_uuid))
            }
            RawTriggerCondition::Drop { item_id } => {
                let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
                    "Event:Drop({}) load error: item token not in symbols",
                    item_id
                ))?;
                Ok(TriggerCondition::Drop(*item_uuid))
            }
            RawTriggerCondition::Insert {
                item_id,
                container_id,
            } => {
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
            RawTriggerCondition::Unlock { item_id } => {
                let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
                    "Event:Unlock({}) load error: item token not in symbols",
                    item_id
                ))?;
                Ok(TriggerCondition::Unlock(*item_uuid))
            }
            RawTriggerCondition::Open { item_id } => {
                let item_uuid = symbols.items.get(item_id).ok_or(anyhow!(
                    "Event:Open({}) load error: item token not in symbols",
                    item_id
                ))?;
                Ok(TriggerCondition::Open(*item_uuid))
            }
            RawTriggerCondition::HasItem { item_id } => {
                let item_uuid = symbols.items.get(item_id).ok_or_else(|| {
                    anyhow!(
                        "Event:HasItem({}) load error: item token not in symbols",
                        item_id
                    )
                })?;
                Ok(TriggerCondition::HasItem(*item_uuid))
            }
            RawTriggerCondition::MissingItem { item_id } => {
                let item_uuid = symbols
                    .items
                    .get(item_id)
                    .ok_or_else(|| anyhow!("MissingItem({}): token not in symbols", item_id))?;
                Ok(TriggerCondition::MissingItem(*item_uuid))
            }
            RawTriggerCondition::WithNpc { npc_id } => {
                let npc_uuid = symbols
                    .characters
                    .get(npc_id)
                    .ok_or_else(|| anyhow!("WithNpc({}): token not in symbols", npc_id))?;
                Ok(TriggerCondition::WithNpc(*npc_uuid))
            }
            RawTriggerCondition::HasVisited { room_id } => {
                let room_uuid = symbols
                    .rooms
                    .get(room_id)
                    .with_context(|| format!("HasVisited({room_id}): token not in symbols"))?;
                Ok(TriggerCondition::HasVisited(*room_uuid))
            }
            RawTriggerCondition::InRoom { room_id } => {
                let room_uuid = symbols
                    .rooms
                    .get(room_id)
                    .with_context(|| format!("InRoom({room_id}): token not in symbols"))?;
                Ok(TriggerCondition::InRoom(*room_uuid))
            }
            RawTriggerCondition::NpcHasItem { npc_id, item_id } => {
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
            RawTriggerCondition::NpcInMood { npc_id, mood } => {
                let npc_uuid = symbols
                    .characters
                    .get(npc_id)
                    .with_context(|| format!("NpcInMood({npc_id},_): token not in symbols"))?;
                Ok(TriggerCondition::NpcInMood {
                    npc_id: *npc_uuid,
                    mood: *mood,
                })
            }
            RawTriggerCondition::UseItemOnItem {
                interaction,
                target_id,
                tool_id,
            } => {
                let target_uuid = symbols.items.get(target_id).with_context(|| {
                    format!("UseItemOnItem({target_id},_,_): token not in symbols")
                })?;
                let tool_uuid = symbols.items.get(tool_id).with_context(|| {
                    format!("UseItemOnItem(_,_,{tool_id}): token not in symbols")
                })?;
                Ok(TriggerCondition::UseItemOnItem {
                    interaction: *interaction,
                    target_id: *target_uuid,
                    tool_id: *tool_uuid,
                })
            }
        }
    }
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
            RawTriggerAction::NpcSays { npc_id, quote } => {
                if let Some(npc_uuid) = symbols.characters.get(npc_id) {
                    Ok(TriggerAction::NpcSays {
                        npc_id: *npc_uuid,
                        quote: quote.to_string(),
                    })
                } else {
                    bail!("RawTriggerAction::NpcSays({npc_id},_): token not in symbol table")
                }
            }
            RawTriggerAction::AddAchievement { achievement: task } => {
                Ok(TriggerAction::AddAchievement(task.to_string()))
            }
            RawTriggerAction::AwardPoints { amount } => Ok(TriggerAction::AwardPoints(*amount)),
            RawTriggerAction::SpawnItemCurrentRoom { item_id } => {
                let item_uuid = symbols.items.get(item_id).with_context(|| {
                    format!("loading SpawnItemCurrentRoom({item_id}): token id not in symbol table")
                })?;
                Ok(TriggerAction::SpawnItemCurrentRoom(*item_uuid))
            }
            RawTriggerAction::PushPlayerTo { room_id } => {
                let room_uuid = symbols.rooms.get(room_id).with_context(|| {
                    format!("loading PushPlayerTo({room_id}): token id not in symbol table")
                })?;
                Ok(TriggerAction::PushPlayerTo(*room_uuid))
            }
            RawTriggerAction::GiveItemToPlayer { npc_id, item_id } => {
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
            RawTriggerAction::SetNpcMood { npc_id, mood } => {
                let npc_uuid = symbols
                    .characters
                    .get(npc_id)
                    .with_context(|| format!("converting RawTriggerAction({npc_id}, {mood:?})"))?;
                Ok(TriggerAction::SetNPCMood {
                    npc_id: *npc_uuid,
                    mood: mood.to_owned(),
                })
            }
            RawTriggerAction::ShowMessage { text } => {
                Ok(TriggerAction::ShowMessage(text.to_string()))
            }
            RawTriggerAction::UnlockItem { item_id: target } => {
                if let Some(target_id) = symbols.items.get(target) {
                    Ok(TriggerAction::UnlockItem(*target_id))
                } else {
                    Err(anyhow!(
                        "couldn't find target ({}) in symbol table for an UnlockItem trigger",
                        target
                    ))
                }
            }
            RawTriggerAction::RevealExit {
                exit_from,
                exit_to,
                direction,
            } => {
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
            RawTriggerAction::SpawnItemInRoom { item_id, room_id } => {
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
            RawTriggerAction::SpawnItemInContainer {
                item_id,
                container_id,
            } => {
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
            RawTriggerAction::DespawnItem { item_id } => {
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
            RawTriggerAction::LockItem { item_id } => {
                if let Some(item_uuid) = symbols.items.get(item_id) {
                    Ok(TriggerAction::LockItem(*item_uuid))
                } else {
                    Err(anyhow!(
                        "RawTriggerAction:LockItem >> unknown item ({})",
                        item_id
                    ))
                }
            }
            RawTriggerAction::LockExit {
                from_room,
                direction,
            } => {
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
            RawTriggerAction::SpawnItemInInventory { item_id } => {
                if let Some(item_uuid) = symbols.items.get(item_id) {
                    Ok(TriggerAction::SpawnItemInInventory(*item_uuid))
                } else {
                    Err(anyhow!(
                        "RawTriggerAction:SpawnItemInInventory >> unknown item ({})",
                        item_id
                    ))
                }
            }
            RawTriggerAction::UnlockExit {
                from_room,
                direction,
            } => {
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
            RawTriggerAction::DenyRead { reason } => {
                Ok(TriggerAction::DenyRead(reason.to_string()))
            }
        }
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
