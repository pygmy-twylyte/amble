//! Trigger action system for the Amble game engine.
//!
//! This module now acts as the public facade for grouped action submodules.
//! The public API remains stable while related handlers live in smaller files.

mod health;
mod items;
mod messaging;
mod npcs;
mod patches;
mod player;
mod rooms;
mod schedule;

pub use health::*;
pub use items::*;
pub use messaging::*;
pub use npcs::*;
pub use patches::*;
pub use player::*;
pub use rooms::*;
pub use schedule::*;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::item::{ContainerState, Movability};
use crate::npc::NpcState;
use crate::player::Flag;
use crate::scheduler::{EventCondition, OnFalsePolicy};
use crate::spinners::SpinnerType;
use crate::view::View;
use crate::world::AmbleWorld;
use crate::{ItemId, NpcId, RoomId};

use self::messaging::show_message_with_priority;
use self::player::{add_flag_with_priority, award_points_with_priority, remove_flag_with_priority};

/// Types of actions that can be fired by a `Trigger` based on a set of `TriggerConditions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Cause physical harm to an NPC once.
    DamageNpc { npc_id: NpcId, cause: String, amount: u32 },
    /// Cause physical harm to an NPC over multiple turns.
    DamageNpcOT {
        npc_id: NpcId,
        cause: String,
        amount: u32,
        turns: u32,
    },
    /// Heal an NPC once.
    HealNpc { npc_id: NpcId, cause: String, amount: u32 },
    /// Heal an NPC over multiple turns.
    HealNpcOT {
        npc_id: NpcId,
        cause: String,
        amount: u32,
        turns: u32,
    },
    /// Remove a pending health effect from an NPC by cause.
    RemoveNpcEffect { npc_id: NpcId, cause: String },
    /// Cause physical harm to the player once.
    DamagePlayer { cause: String, amount: u32 },
    /// Cause physical harm to the player over multiple turns.
    DamagePlayerOT { cause: String, amount: u32, turns: u32 },
    /// Heal the player a specified amount once.
    HealPlayer { cause: String, amount: u32 },
    /// Heal the player a specified amount for multiple turns.
    HealPlayerOT { cause: String, amount: u32, turns: u32 },
    /// Remove a pending health effect from the player by cause.
    RemovePlayerEffect { cause: String },
    /// Set the activity state of an NPC.
    SetNpcActive { npc_id: NpcId, active: bool },
    /// Set the `ContainerState` of an Item.
    SetContainerState { item_id: ItemId, state: Option<ContainerState> },
    /// Replaces an item at its current location.
    ReplaceItem { old_id: ItemId, new_id: ItemId },
    /// Replaces an item and drops it at the player's location.
    ReplaceDropItem { old_id: ItemId, new_id: ItemId },
    /// Adds a status flag to the player.
    AddFlag(Flag),
    /// Adds a weighted text option to a random text spinner.
    AddSpinnerWedge { spinner: SpinnerType, text: String, width: usize },
    /// Advances a sequence flag to the next step.
    AdvanceFlag(String),
    /// Removes a flag from the player.
    RemoveFlag(String),
    /// Awards points to the player (negative values subtract points).
    AwardPoints { amount: isize, reason: String },
    /// Sets a custom message for a blocked exit between two rooms.
    SetBarredMessage { exit_from: RoomId, exit_to: RoomId, msg: String },
    /// Prevents reading an item with a custom denial message.
    DenyRead(String),
    /// Removes an item from the world entirely.
    DespawnItem { item_id: ItemId },
    /// Removes an NPC from the world entirely.
    DespawnNpc { npc_id: NpcId },
    /// Transfers an item from an NPC to the player's inventory.
    GiveItemToPlayer { npc_id: NpcId, item_id: ItemId },
    /// Locks an exit in a specific direction from a room.
    LockExit { from_room: RoomId, direction: String },
    /// Locks a container item.
    LockItem(ItemId),
    /// Modifies multiple aspects of an `Item` at once using an `ItemPatch`.
    ModifyItem { item_id: ItemId, patch: ItemPatch },
    /// Modifies multiple aspects of a `Room` at once using a `RoomPatch`.
    ModifyRoom { room_id: RoomId, patch: RoomPatch },
    /// Modifies multiple aspects of an `Npc` at once using an `NpcPatch`.
    ModifyNpc { npc_id: NpcId, patch: NpcPatch },
    /// Makes an NPC refuse an item with a custom message.
    NpcRefuseItem { npc_id: NpcId, reason: String },
    /// Makes an NPC speak a specific line of dialogue.
    NpcSays { npc_id: NpcId, quote: String },
    /// Makes an NPC speak a random line based on their current mood.
    NpcSaysRandom { npc_id: NpcId },
    /// Instantly moves the player to a different room.
    PushPlayerTo(RoomId),
    /// Resets a sequence flag back to step 0.
    ResetFlag(String),
    /// Makes a hidden exit visible and usable.
    RevealExit {
        exit_from: RoomId,
        exit_to: RoomId,
        direction: String,
    },
    /// Changes an item's description.
    SetItemDescription { item_id: ItemId, text: String },
    /// Restricts or changes movability of an `Item`.
    SetItemMovability { item_id: ItemId, movability: Movability },
    /// Changes an NPC's behavioral state.
    SetNPCState { npc_id: NpcId, state: NpcState },
    /// Displays a message to the player.
    ShowMessage(String),
    /// Creates an item in the player's current room.
    SpawnItemCurrentRoom(ItemId),
    /// Creates an item inside a container item.
    SpawnItemInContainer { item_id: ItemId, container_id: ItemId },
    /// Creates an item in the player's inventory.
    SpawnItemInInventory(ItemId),
    /// Creates an item in a specific room.
    SpawnItemInRoom { item_id: ItemId, room_id: RoomId },
    /// Creates an NPC in a specific room.
    SpawnNpcInRoom { npc_id: NpcId, room_id: RoomId },
    /// Displays a random message from a spinner.
    SpinnerMessage { spinner: SpinnerType },
    /// Unlocks an exit in a specific direction from a room.
    UnlockExit { from_room: RoomId, direction: String },
    /// Unlocks a container item.
    UnlockItem(ItemId),
    /// Conditionally run nested actions when the condition evaluates to true.
    Conditional {
        condition: EventCondition,
        actions: Vec<ScriptedAction>,
        false_actions: Option<Vec<ScriptedAction>>,
    },
    /// Schedules a list of actions to fire after a specified number of turns.
    ScheduleIn {
        turns_ahead: usize,
        actions: Vec<ScriptedAction>,
        note: Option<String>,
    },
    /// Schedules a list of actions to fire on a specific turn.
    ScheduleOn {
        on_turn: usize,
        actions: Vec<ScriptedAction>,
        note: Option<String>,
    },
    /// Schedules actions in the future with a condition and on-false policy.
    ScheduleInIf {
        turns_ahead: usize,
        condition: EventCondition,
        on_false: OnFalsePolicy,
        actions: Vec<ScriptedAction>,
        note: Option<String>,
    },
    /// Schedules actions on a specific turn with a condition and on-false policy.
    ScheduleOnIf {
        on_turn: usize,
        condition: EventCondition,
        on_false: OnFalsePolicy,
        actions: Vec<ScriptedAction>,
        note: Option<String>,
    },
}

/// Wrapper for a trigger action with optional metadata that influences rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptedAction {
    pub action: TriggerAction,
    #[serde(default)]
    pub priority: Option<isize>,
}

impl ScriptedAction {
    pub fn new(action: TriggerAction) -> Self {
        Self { action, priority: None }
    }

    pub fn with_priority(action: TriggerAction, priority: Option<isize>) -> Self {
        Self { action, priority }
    }
}

/// Execute a single trigger action against the current world state.
///
/// # Errors
/// - Returns an error if the underlying handler cannot resolve referenced objects.
#[allow(clippy::too_many_lines)]
pub fn dispatch_action(world: &mut AmbleWorld, view: &mut View, scripted: &ScriptedAction) -> Result<()> {
    use TriggerAction::{
        AddFlag, AddSpinnerWedge, AdvanceFlag, AwardPoints, Conditional, DamageNpc, DamageNpcOT, DamagePlayer,
        DamagePlayerOT, DenyRead, DespawnItem, DespawnNpc, GiveItemToPlayer, HealNpc, HealNpcOT, HealPlayer,
        HealPlayerOT, LockExit, LockItem, ModifyItem, ModifyNpc, ModifyRoom, NpcRefuseItem, NpcSays, NpcSaysRandom,
        PushPlayerTo, RemoveFlag, RemoveNpcEffect, RemovePlayerEffect, ReplaceDropItem, ReplaceItem, ResetFlag,
        RevealExit, ScheduleIn, ScheduleInIf, ScheduleOn, ScheduleOnIf, SetBarredMessage, SetContainerState,
        SetItemDescription, SetItemMovability, SetNPCState, SetNpcActive, ShowMessage, SpawnItemCurrentRoom,
        SpawnItemInContainer, SpawnItemInInventory, SpawnItemInRoom, SpawnNpcInRoom, SpinnerMessage, UnlockExit,
        UnlockItem,
    };

    let ScriptedAction { action, priority } = scripted;
    match action {
        DamageNpc { npc_id, cause, amount } => {
            let npc = world
                .npcs
                .get_mut(npc_id)
                .with_context(|| "npc lookup for damage_NPC")?;
            damage_character(npc, cause, *amount);
        },
        DamageNpcOT {
            npc_id,
            cause,
            amount,
            turns,
        } => {
            let npc = world
                .npcs
                .get_mut(npc_id)
                .with_context(|| "npc lookup for damage_npc_ot")?;
            damage_character_ot(npc, cause, *amount, *turns);
        },
        HealNpc { npc_id, cause, amount } => {
            let npc = world.npcs.get_mut(npc_id).with_context(|| "npc lookup for heal_NPC")?;
            heal_character(npc, cause, *amount);
        },
        HealNpcOT {
            npc_id,
            cause,
            amount,
            turns,
        } => {
            let npc = world
                .npcs
                .get_mut(npc_id)
                .with_context(|| "npc lookup for heal_npc_ot")?;
            heal_character_ot(npc, cause, *amount, *turns);
        },
        RemoveNpcEffect { npc_id, cause } => {
            let npc = world
                .npcs
                .get_mut(npc_id)
                .with_context(|| "npc lookup for remove_npc_effect")?;
            remove_health_effect(npc, cause, "npc");
        },
        DamagePlayer { cause, amount } => damage_character(&mut world.player, cause, *amount),
        DamagePlayerOT { cause, amount, turns } => damage_character_ot(&mut world.player, cause, *amount, *turns),
        HealPlayer { cause, amount } => heal_character(&mut world.player, cause, *amount),
        HealPlayerOT { cause, amount, turns } => heal_character_ot(&mut world.player, cause, *amount, *turns),
        RemovePlayerEffect { cause } => remove_health_effect(&mut world.player, cause, "player"),
        ModifyItem { item_id, patch } => modify_item(world, item_id.clone(), patch)?,
        ModifyRoom { room_id, patch } => modify_room(world, room_id, patch)?,
        ModifyNpc { npc_id, patch } => modify_npc(world, npc_id, patch)?,
        SetNpcActive { npc_id, active } => set_npc_active(world, npc_id, *active)?,
        SetContainerState { item_id, state } => set_container_state(world, item_id, *state)?,
        ReplaceItem { old_id, new_id } => replace_item(world, old_id, new_id)?,
        ReplaceDropItem { old_id, new_id } => replace_drop_item(world, old_id, new_id)?,
        SetBarredMessage {
            exit_from,
            exit_to,
            msg,
        } => set_barred_message(world, exit_from, exit_to, msg)?,
        AddSpinnerWedge { spinner, text, width } => add_spinner_wedge(&mut world.spinners, spinner, text, *width)?,
        ResetFlag(flag_name) => reset_flag(&mut world.player, flag_name),
        AdvanceFlag(flag_name) => advance_flag(&mut world.player, flag_name),
        SpinnerMessage { spinner } => spinner_message(world, view, spinner, *priority)?,
        NpcRefuseItem { npc_id, reason } => npc_refuse_item(world, view, npc_id, reason, *priority)?,
        NpcSaysRandom { npc_id } => npc_says_random(world, view, npc_id, *priority)?,
        NpcSays { npc_id, quote } => npc_says(world, view, npc_id, quote, *priority)?,
        DenyRead(reason) => deny_read(view, reason),
        DespawnItem { item_id } => despawn_item(world, item_id)?,
        DespawnNpc { npc_id } => despawn_npc(world, view, npc_id)?,
        GiveItemToPlayer { npc_id, item_id } => {
            give_to_player(world, npc_id, item_id)?;
        },
        LockItem(item_id) => lock_item(world, item_id)?,
        PushPlayerTo(room_id) => push_player(world, room_id)?,
        RevealExit {
            direction,
            exit_from,
            exit_to,
        } => reveal_exit(world, direction, exit_from, exit_to)?,
        SetItemDescription { item_id, text } => set_item_description(world, item_id, text)?,
        SetItemMovability { item_id, movability } => set_item_movability(world, item_id, movability)?,
        SetNPCState { npc_id, state } => set_npc_state(world, npc_id, state)?,
        ShowMessage(text) => show_message_with_priority(view, text, *priority),
        SpawnItemInContainer { item_id, container_id } => spawn_item_in_container(world, item_id, container_id)?,
        SpawnItemInInventory(item_id) => spawn_item_in_inventory(world, item_id)?,
        SpawnItemCurrentRoom(item_id) => spawn_item_in_current_room(world, item_id)?,
        SpawnItemInRoom { item_id, room_id } => spawn_item_in_specific_room(world, item_id, room_id)?,
        SpawnNpcInRoom { npc_id, room_id } => spawn_npc_in_room(world, view, npc_id, room_id)?,
        UnlockItem(item_id) => unlock_item(world, item_id)?,
        UnlockExit { from_room, direction } => unlock_exit(world, from_room, direction)?,
        LockExit { from_room, direction } => lock_exit(world, from_room, direction)?,
        AddFlag(flag) => add_flag_with_priority(world, view, flag, *priority),
        RemoveFlag(flag) => remove_flag_with_priority(world, view, flag, *priority),
        AwardPoints { amount, reason } => award_points_with_priority(world, view, *amount, reason, *priority),
        Conditional {
            condition,
            actions,
            false_actions,
        } => {
            if condition.eval(world) {
                for nested in actions {
                    dispatch_action(world, view, nested)?;
                }
            } else if let Some(false_actions) = false_actions {
                for nested in false_actions {
                    dispatch_action(world, view, nested)?;
                }
            }
        },
        ScheduleIn {
            turns_ahead,
            actions,
            note,
        } => schedule_in(world, view, *turns_ahead, actions, note.clone())?,
        ScheduleOn { on_turn, actions, note } => schedule_on(world, view, *on_turn, actions, note.clone())?,
        ScheduleInIf {
            turns_ahead,
            condition,
            on_false,
            actions,
            note,
        } => schedule_in_if(
            world,
            view,
            *turns_ahead,
            condition,
            on_false.clone(),
            actions,
            note.clone(),
        )?,
        ScheduleOnIf {
            on_turn,
            condition,
            on_false,
            actions,
            note,
        } => schedule_on_if(
            world,
            view,
            *on_turn,
            condition,
            on_false.clone(),
            actions,
            note.clone(),
        )?,
    }
    Ok(())
}

#[cfg(test)]
mod tests;
