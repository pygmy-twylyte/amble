//! Trigger action system for the Amble game engine.
//!
//! This module contains the implementation of all actions that can be triggered
//! by game events. Actions are the "effects" part of the trigger system - they
//! modify world state, update the user interface, or cause other changes when
//! specific conditions are met.
//!
//! # Overview
//!
//! The trigger system works in two phases:
//! 1. **Conditions** - Check if certain game states are met (handled elsewhere)
//! 2. **Actions** - Execute changes to world state (implemented in this module)
//!
//! Actions are defined as enum variants in [`TriggerAction`] and implemented
//! as individual functions that take the necessary parameters to modify the
//! game world.
//!
//! # Action Categories
//!
//! ## Player State Management
//! Actions that modify the player's status, inventory, or progression:
//! - Flag management (add, remove, advance, reset)
//! - Score changes (award/subtract points)
//! - Inventory manipulation (spawn items, transfers)
//! - Player movement (teleportation)
//!
//! ## World State Changes
//! Actions that modify the game world itself:
//! - Room connections (lock/unlock exits, reveal passages)
//! - Item states (lock/unlock containers, restrict items, change description)
//! - Item placement (spawn in rooms, containers, inventory)
//! - Item removal (despawn from world)
//!
//! ## NPC Interactions
//! Actions that control non-player characters:
//! - Dialogue (scripted or random responses)
//! - State changes (modify NPC behavior)
//! - Item transfers (give items to player)
//! - Response control (refuse item transfers)
//!
//! ## User Interface
//! Actions that affect what the player sees:
//! - Message display (triggered events, ambient messages)
//! - Reading restrictions (conditional text access)
//! - Random text generation (spinner messages)
//!
//! # Usage Pattern
//!
//! Actions are typically not called directly. Instead, they are:
//! 1. Defined in TOML configuration files as part of trigger definitions
//! 2. Deserialized into [`TriggerAction`] enum variants
//! 3. Executed by [`dispatch_action`] when trigger conditions are met
//!
//! # Error Handling
//!
//! Most action functions return `Result<()>` and can fail if:
//! - Required world objects (items, rooms, NPCs) don't exist
//! - Invalid UUIDs are provided
//! - World state is inconsistent
//!
//! Errors are typically logged and may be displayed to the player depending
//! on the context in which the action was triggered.
//!
//! # Logging
//!
//! All actions log their execution with a consistent format:
//! ```text
//! └─ action: ActionName(parameters)
//! ```
//!
//! This provides an audit trail of all world state changes and helps with
//! debugging game logic.

use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result, anyhow, bail};
use gametools::{Spinner, Wedge};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Item;
use crate::helpers::{symbol_from_id, symbol_or_unknown};
use crate::item::{ContainerState, ItemAbility, ItemHolder};
use crate::npc::{MovementTiming, MovementType, Npc, NpcMovement, NpcState, move_npc};
use crate::player::{Flag, Player};
use crate::room::Exit;
use crate::scheduler::{EventCondition, OnFalsePolicy};
use crate::spinners::{CoreSpinnerType, SpinnerType};
use crate::style::GameStyle;
use crate::trigger::TriggerCondition;
use crate::view::{StatusAction, View, ViewItem};
use crate::world::{AmbleWorld, Location, WorldObject};

/// Types of actions that can be fired by a `Trigger` based on a set of `TriggerConditions`.
/// Represents all possible actions that can be triggered by game events.
///
/// Each variant corresponds to a specific action function that modifies world state,
/// updates the user interface, or triggers some other game behavior. Actions are
/// executed when their associated trigger conditions are met.
///
/// # Action Types
///
/// ## Flag Management
/// - [`AddFlag`] - Adds a status flag to the player
/// - [`AdvanceFlag`] - Advances a sequence flag to the next step
/// - [`RemoveFlag`] - Removes a flag from the player
/// - [`ResetFlag`] - Resets a sequence flag to step 0
///
/// ## Item Manipulation
/// - [`SetContainerState`] - Monkey wrench for containers: open/close, lock/unlock, set transparency.
/// - [`ReplaceDropItem`] - Drops item at current location AND replaces it with another.
/// - [`DespawnItem`] - Removes an item from the world entirely
/// - [`ReplaceItem`] - Replaces one item with another, in the same location.
/// - [`LockItem`] - Locks a container item
/// - [`RestrictItem`] - Makes an item non-transferable
/// - [`SpawnItemCurrentRoom`] - Creates an item in the player's current room
/// - [`SpawnItemInContainer`] - Creates an item inside a container
/// - [`SpawnItemInInventory`] - Creates an item in the player's inventory
/// - [`SpawnItemInRoom`] - Creates an item in a specific room
/// - [`UnlockItem`] - Unlocks a container item
/// - [`SetItemDescription`] - Sets a new description for an item
///
/// ## Player Actions
/// - [`AwardPoints`] - Adds or subtracts points from the player's score
/// - [`GiveItemToPlayer`] - Transfers an item from an NPC to the player
/// - [`PushPlayerTo`] - Instantly moves the player to a different room
///
/// ## NPC Interactions
/// - [`NpcRefuseItem`] - Makes an NPC refuse an item with a custom message
/// - [`NpcSays`] - Makes an NPC speak a specific line
/// - [`NpcSaysRandom`] - Makes an NPC speak a random line based on their mood
/// - [`SetNPCState`] - Changes an NPC's behavioral state
///
/// ## World Modification
/// - [`LockExit`] - Locks a room exit, preventing passage
/// - [`RevealExit`] - Makes a hidden exit visible and usable
/// - [`SetBarredMessage`] - Sets a custom message for blocked exits
/// - [`UnlockExit`] - Unlocks a previously locked exit
///
/// ## UI/UX Actions
/// - [`AddSpinnerWedge`] - Adds a new random text option to a spinner
/// - [`DenyRead`] - Prevents reading an item with a custom message
/// - [`ShowMessage`] - Displays a message to the player
/// - [`SpinnerMessage`] - Displays a random message from a spinner
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Set the activity state of an NPC
    SetNpcActive { npc_id: Uuid, active: bool },
    /// Set the ContainerState of an Item
    SetContainerState { item_id: Uuid, state: Option<ContainerState> },
    /// Replaces an item at its current location
    ReplaceItem { old_id: Uuid, new_id: Uuid },
    /// Replaces an item and drops it at the player's location
    ReplaceDropItem { old_id: Uuid, new_id: Uuid },
    /// Adds a status flag to the player
    AddFlag(Flag),
    /// Adds a weighted text option to a random text spinner
    AddSpinnerWedge { spinner: SpinnerType, text: String, width: usize },
    /// Advances a sequence flag to the next step
    AdvanceFlag(String),
    /// Removes a flag from the player
    RemoveFlag(String),
    /// Awards points to the player (negative values subtract points)
    AwardPoints(isize),
    /// Sets a custom message for a blocked exit between two rooms
    SetBarredMessage { exit_from: Uuid, exit_to: Uuid, msg: String },
    /// Prevents reading an item with a custom denial message
    DenyRead(String),
    /// Removes an item from the world entirely
    DespawnItem { item_id: Uuid },
    /// Removes an NPC from the world entirely
    DespawnNpc { npc_id: Uuid },
    /// Transfers an item from an NPC to the player's inventory
    GiveItemToPlayer { npc_id: Uuid, item_id: Uuid },
    /// Locks an exit in a specific direction from a room
    LockExit { from_room: Uuid, direction: String },
    /// Locks a container item
    LockItem(Uuid),
    /// Modifies multiple aspects of an `Item` at once using an `ItemPatch`
    ModifyItem { item_id: Uuid, patch: ItemPatch },
    /// Modifies multiple aspects of a `Room` at once using a `RoomPatch`
    ModifyRoom { room_id: Uuid, patch: RoomPatch },
    /// Modifies multiple aspects of an `Npc` at once using an `NpcPatch`
    ModifyNpc { npc_id: Uuid, patch: NpcPatch },
    /// Makes an NPC refuse an item with a custom message
    NpcRefuseItem { npc_id: Uuid, reason: String },
    /// Makes an NPC speak a specific line of dialogue
    NpcSays { npc_id: Uuid, quote: String },
    /// Makes an NPC speak a random line based on their current mood
    NpcSaysRandom { npc_id: Uuid },
    /// Instantly moves the player to a different room
    PushPlayerTo(Uuid),
    /// Resets a sequence flag back to step 0
    ResetFlag(String),
    /// Makes an item non-transferable (cannot be taken or dropped)
    RestrictItem(Uuid),
    /// Makes a hidden exit visible and usable
    RevealExit { exit_from: Uuid, exit_to: Uuid, direction: String },
    /// Changes an item's description
    SetItemDescription { item_id: Uuid, text: String },
    /// Changes an NPC's behavioral state
    SetNPCState { npc_id: Uuid, state: NpcState },
    /// Displays a message to the player
    ShowMessage(String),
    /// Creates an item in the player's current room
    SpawnItemCurrentRoom(Uuid),
    /// Creates an item inside a container item
    SpawnItemInContainer { item_id: Uuid, container_id: Uuid },
    /// Creates an item in the player's inventory
    SpawnItemInInventory(Uuid),
    /// Creates an item in a specific room
    SpawnItemInRoom { item_id: Uuid, room_id: Uuid },
    /// Creates an NPC in a specific room
    SpawnNpcInRoom { npc_id: Uuid, room_id: Uuid },
    /// Displays a random message from a spinner
    SpinnerMessage { spinner: SpinnerType },
    /// Unlocks an exit in a specific direction from a room
    UnlockExit { from_room: Uuid, direction: String },
    /// Unlocks a container item
    UnlockItem(Uuid),
    /// Conditionally run nested actions when the condition evaluates to true.
    Conditional {
        condition: EventCondition,
        actions: Vec<TriggerAction>,
    },
    /// Schedules a list of actions to fire after a specified number of turns
    ScheduleIn {
        turns_ahead: usize,
        actions: Vec<TriggerAction>,
        note: Option<String>,
    },
    /// Schedules a list of actions to fire on a specific turn
    ScheduleOn {
        on_turn: usize,
        actions: Vec<TriggerAction>,
        note: Option<String>,
    },
    /// Schedules actions in the future with a condition and on-false policy
    ScheduleInIf {
        turns_ahead: usize,
        condition: EventCondition,
        on_false: OnFalsePolicy,
        actions: Vec<TriggerAction>,
        note: Option<String>,
    },
    /// Schedules actions on a specific turn with a condition and on-false policy
    ScheduleOnIf {
        on_turn: usize,
        condition: EventCondition,
        on_false: OnFalsePolicy,
        actions: Vec<TriggerAction>,
        note: Option<String>,
    },
}

/// Fires the matching trigger action by calling its handler function
///
/// # Errors
/// - on failed triggered actions due to bad uuids
pub fn dispatch_action(world: &mut AmbleWorld, view: &mut View, action: &TriggerAction) -> Result<()> {
    use TriggerAction::*;
    match action {
        ModifyItem { item_id, patch } => modify_item(world, *item_id, patch)?,
        ModifyRoom { room_id, patch } => modify_room(world, *room_id, patch)?,
        ModifyNpc { npc_id, patch } => modify_npc(world, *npc_id, patch)?,
        SetNpcActive { npc_id, active } => set_npc_active(world, npc_id, *active)?,
        SetContainerState { item_id, state } => set_container_state(world, *item_id, *state)?,
        ReplaceItem { old_id, new_id } => replace_item(world, old_id, new_id)?,
        ReplaceDropItem { old_id, new_id } => replace_drop_item(world, old_id, new_id)?,
        SetBarredMessage {
            exit_from,
            exit_to,
            msg,
        } => set_barred_message(world, exit_from, exit_to, msg)?,
        AddSpinnerWedge { spinner, text, width } => {
            add_spinner_wedge(&mut world.spinners, spinner, text, *width)?;
        },
        ResetFlag(flag_name) => reset_flag(&mut world.player, flag_name),
        AdvanceFlag(flag_name) => advance_flag(&mut world.player, flag_name),
        SpinnerMessage { spinner } => spinner_message(world, view, spinner)?,
        RestrictItem(item_id) => restrict_item(world, item_id)?,
        NpcRefuseItem { npc_id, reason } => npc_refuse_item(world, view, *npc_id, reason)?,
        NpcSaysRandom { npc_id } => npc_says_random(world, view, npc_id)?,
        NpcSays { npc_id, quote } => npc_says(world, view, npc_id, quote)?,
        DenyRead(reason) => deny_read(view, reason),
        DespawnItem { item_id } => despawn_item(world, item_id)?,
        DespawnNpc { npc_id } => despawn_npc(world, view, *npc_id)?,
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
        SetNPCState { npc_id, state } => set_npc_state(world, npc_id, state)?,
        ShowMessage(text) => show_message(view, text),
        SpawnItemInContainer { item_id, container_id } => {
            spawn_item_in_container(world, item_id, container_id)?;
        },
        SpawnItemInInventory(item_id) => spawn_item_in_inventory(world, item_id)?,
        SpawnItemCurrentRoom(item_id) => spawn_item_in_current_room(world, item_id)?,
        SpawnItemInRoom { item_id, room_id } => {
            spawn_item_in_specific_room(world, item_id, room_id)?;
        },
        SpawnNpcInRoom { npc_id, room_id } => spawn_npc_in_room(world, view, *npc_id, *room_id)?,
        UnlockItem(item_id) => unlock_item(world, item_id)?,
        UnlockExit { from_room, direction } => unlock_exit(world, from_room, direction)?,
        LockExit { from_room, direction } => lock_exit(world, from_room, direction)?,
        AddFlag(flag) => add_flag(world, view, flag),
        RemoveFlag(flag) => remove_flag(world, view, flag),
        AwardPoints(amount) => award_points(world, view, *amount),
        Conditional { condition, actions } => {
            if condition.eval(world) {
                for nested in actions {
                    dispatch_action(world, view, nested)?;
                }
            }
        },
        ScheduleIn {
            turns_ahead,
            actions,
            note,
        } => {
            schedule_in(world, view, *turns_ahead, actions, note.clone())?;
        },
        ScheduleOn { on_turn, actions, note } => {
            schedule_on(world, view, *on_turn, actions, note.clone())?;
        },
        ScheduleInIf {
            turns_ahead,
            condition,
            on_false,
            actions,
            note,
        } => {
            schedule_in_if(
                world,
                view,
                *turns_ahead,
                condition,
                on_false.clone(),
                actions,
                note.clone(),
            )?;
        },
        ScheduleOnIf {
            on_turn,
            condition,
            on_false,
            actions,
            note,
        } => {
            schedule_on_if(
                world,
                view,
                *on_turn,
                condition,
                on_false.clone(),
                actions,
                note.clone(),
            )?;
        },
    }
    Ok(())
}

/// Patch that can be applied to modify multiple properties of an `Item` at once
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ItemPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub text: Option<String>,
    pub portable: Option<bool>,
    pub restricted: Option<bool>,
    pub container_state: Option<ContainerState>,
    #[serde(default)]
    pub remove_container_state: bool,
    #[serde(default)]
    pub add_abilities: Vec<ItemAbility>,
    #[serde(default)]
    pub remove_abilities: Vec<ItemAbility>,
}

/// Patch that can be applied to modify multiple properties of a `Room` at once.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RoomPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub remove_exits: Vec<Uuid>,
    #[serde(default)]
    pub add_exits: Vec<RoomExitPatch>,
}

/// Exit data used when adding an exit via `RoomPatch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomExitPatch {
    pub direction: String,
    pub to: Uuid,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub required_flags: HashSet<Flag>,
    #[serde(default)]
    pub required_items: HashSet<Uuid>,
    #[serde(default)]
    pub barred_message: Option<String>,
}

impl Default for RoomExitPatch {
    fn default() -> Self {
        Self {
            direction: String::new(),
            to: Uuid::nil(),
            hidden: false,
            locked: false,
            required_flags: HashSet::new(),
            required_items: HashSet::new(),
            barred_message: None,
        }
    }
}

/// Represents a line of dialogue to be appended to a specific NPC state when applying an `NpcPatch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcDialoguePatch {
    pub state: NpcState,
    pub line: String,
}

/// Movement updates that may accompany an `NpcPatch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NpcMovementPatch {
    #[serde(default)]
    pub route: Option<Vec<Uuid>>,
    #[serde(default)]
    pub random_rooms: Option<HashSet<Uuid>>,
    pub timing: Option<MovementTiming>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}
impl NpcMovementPatch {
    pub fn has_updates(&self) -> bool {
        self.route.is_some()
            || self.random_rooms.is_some()
            || self.timing.is_some()
            || self.active.is_some()
            || self.loop_route.is_some()
    }

    fn wants_new_instance(&self) -> bool {
        self.route.is_some() || self.random_rooms.is_some()
    }
}

/// Patch that can be applied to modify multiple properties of an `Npc` at once.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NpcPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub state: Option<NpcState>,
    #[serde(default)]
    pub add_lines: Vec<NpcDialoguePatch>,
    pub movement: Option<NpcMovementPatch>,
}

/*
 *
 * ACTION HANDLERS
 *
 */

/// Modifies multiple properties of an `Item` at once by applying an `ItemPatch`
pub fn modify_item(world: &mut AmbleWorld, item_id: Uuid, patch: &ItemPatch) -> Result<()> {
    info!(
        "└─ action: modifying item {} using patch: {:?}",
        symbol_or_unknown(&world.items, item_id),
        patch
    );
    let patched = apply_item_patch(world, item_id, patch)?;
    world.items.insert(item_id, patched);
    Ok(())
}

/// Modifies multiple properties of a `Room` at once by applying a `RoomPatch`.
pub fn modify_room(world: &mut AmbleWorld, room_id: Uuid, patch: &RoomPatch) -> Result<()> {
    info!(
        "└─ action: modifying room {} using patch: {:?}",
        symbol_or_unknown(&world.rooms, room_id),
        patch
    );
    apply_room_patch(world, room_id, patch)?;
    Ok(())
}

/// Modifies multiple properties of an `Npc` at once by applying an `NpcPatch`.
pub fn modify_npc(world: &mut AmbleWorld, npc_id: Uuid, patch: &NpcPatch) -> Result<()> {
    info!(
        "└─ action: modifying npc {} using patch: {:?}",
        symbol_or_unknown(&world.npcs, npc_id),
        patch
    );
    apply_npc_patch(world, npc_id, patch)?;
    Ok(())
}

/// Applies a `RoomPatch` to the targeted `Room`, mutating it in place.
fn apply_room_patch(world: &mut AmbleWorld, room_id: Uuid, patch: &RoomPatch) -> Result<()> {
    let mut removal_plan: Vec<(String, Uuid)> = Vec::new();
    {
        let room_ref = world
            .rooms
            .get(&room_id)
            .with_context(|| format!("patching a room: Uuid ({room_id}) not found in world room map"))?;
        for target_room_id in &patch.remove_exits {
            if let Some(direction) = room_ref
                .exits
                .iter()
                .find_map(|(dir, exit)| (exit.to == *target_room_id).then_some(dir.clone()))
            {
                removal_plan.push((direction, *target_room_id));
            } else {
                let target_sym = symbol_or_unknown(&world.rooms, *target_room_id);
                warn!(
                    "modifyRoom patch attempted to remove exit to '{}' but room '{}' has no such exit",
                    target_sym, room_ref.symbol
                );
            }
        }
    }

    let room = world.rooms.get_mut(&room_id).expect("room existence validated above");

    if let Some(ref new_name) = patch.name {
        room.name = new_name.clone();
    }

    if let Some(ref new_desc) = patch.desc {
        room.base_description = new_desc.clone();
    }

    for (direction, _) in &removal_plan {
        room.exits.remove(direction);
    }

    for addition in &patch.add_exits {
        let mut exit = Exit::new(addition.to);
        exit.hidden = addition.hidden;
        exit.locked = addition.locked;
        exit.barred_message = addition.barred_message.clone();
        exit.required_flags = addition.required_flags.clone();
        exit.required_items = addition.required_items.clone();
        room.exits.insert(addition.direction.clone(), exit);
    }

    Ok(())
}

/// Applies an `NpcPatch` to the targeted `Npc`, mutating it in place.
fn apply_npc_patch(world: &mut AmbleWorld, npc_id: Uuid, patch: &NpcPatch) -> Result<()> {
    let npc = world
        .npcs
        .get_mut(&npc_id)
        .with_context(|| format!("patching an npc: Uuid ({npc_id}) not found in world npc map"))?;

    if let Some(ref new_name) = patch.name {
        npc.name = new_name.clone();
    }
    if let Some(ref new_desc) = patch.desc {
        npc.description = new_desc.clone();
    }
    if let Some(ref new_state) = patch.state {
        npc.state = new_state.clone();
    }

    if !patch.add_lines.is_empty() {
        for addition in &patch.add_lines {
            npc.dialogue
                .entry(addition.state.clone())
                .or_default()
                .push(addition.line.clone());
        }
    }

    if let Some(movement_patch) = &patch.movement {
        if movement_patch.has_updates() {
            let current_turn = world.turn_count;
            apply_npc_movement_patch(current_turn, npc, movement_patch)?;
        }
    }

    Ok(())
}

fn apply_npc_movement_patch(current_turn: usize, npc: &mut Npc, patch: &NpcMovementPatch) -> Result<()> {
    if patch.route.is_some() && patch.random_rooms.is_some() {
        bail!(
            "modifyNpc patch for '{}' cannot set both a route and random movement set",
            npc.symbol()
        );
    }

    if let Some(route) = &patch.route {
        if route.is_empty() {
            bail!(
                "modifyNpc patch for '{}' requires at least one room in a movement route",
                npc.symbol()
            );
        }
    }
    if let Some(random) = &patch.random_rooms {
        if random.is_empty() {
            bail!(
                "modifyNpc patch for '{}' requires at least one room in a random movement set",
                npc.symbol()
            );
        }
    }

    let npc_symbol = npc.symbol().to_string();

    if npc.movement.is_none() && patch.wants_new_instance() {
        npc.movement = Some(NpcMovement {
            movement_type: if let Some(route) = &patch.route {
                MovementType::Route {
                    rooms: route.clone(),
                    current_idx: 0,
                    loop_route: patch.loop_route.unwrap_or(true),
                }
            } else if let Some(random) = &patch.random_rooms {
                MovementType::RandomSet { rooms: random.clone() }
            } else {
                // shouldn't happen due to wants_new_instance check, but guard anyway
                warn!(
                    "modifyNpc patch for '{}' requested new movement without route or random rooms",
                    npc_symbol
                );
                return Ok(());
            },
            timing: patch
                .timing
                .clone()
                .unwrap_or_else(|| MovementTiming::EveryNTurns { turns: 1 }),
            active: patch.active.unwrap_or(true),
            last_moved_turn: current_turn,
            paused_until: None,
        });
    }

    if let Some(movement) = npc.movement.as_mut() {
        if let Some(route) = &patch.route {
            let loop_setting = patch.loop_route.unwrap_or_else(|| match &movement.movement_type {
                MovementType::Route { loop_route, .. } => *loop_route,
                MovementType::RandomSet { .. } => true,
            });
            movement.movement_type = MovementType::Route {
                rooms: route.clone(),
                current_idx: 0,
                loop_route: loop_setting,
            };
        } else if let Some(random) = &patch.random_rooms {
            movement.movement_type = MovementType::RandomSet { rooms: random.clone() };
        }

        if let Some(loop_setting) = patch.loop_route {
            if let MovementType::Route { loop_route, .. } = &mut movement.movement_type {
                *loop_route = loop_setting;
            } else {
                warn!(
                    "modifyNpc patch attempted to set loop on a non-route movement for '{}'",
                    npc_symbol
                );
            }
        }

        if let Some(timing) = &patch.timing {
            movement.timing = timing.clone();
            movement.last_moved_turn = current_turn;
        }

        if let Some(active) = patch.active {
            movement.active = active;
        }
    } else {
        warn!(
            "modifyNpc patch for '{}' requested movement updates but NPC has no movement configured",
            npc_symbol
        );
    }

    Ok(())
}

/// Clones an `Item`, modifies contents, and returns the updated `Item`
fn apply_item_patch(world: &mut AmbleWorld, item_id: Uuid, patch: &ItemPatch) -> Result<Item> {
    let mut patched = if let Some(old_item) = world.items.get(&item_id) {
        old_item.clone()
    } else {
        bail!("patching an item: Uuid ({item_id}) not found in world item map");
    };

    if let Some(ref new_name) = patch.name {
        patched.name = new_name.clone();
    }

    if let Some(ref new_desc) = patch.desc {
        patched.description = new_desc.clone();
    }

    if patch.text.is_some() {
        patched.text = patch.text.clone();
    }

    if let Some(new_portable) = patch.portable {
        patched.portable = new_portable;
    }

    if let Some(new_restricted) = patch.restricted {
        patched.restricted = new_restricted;
    }

    if patch.remove_container_state {
        if patch.container_state.is_some() {
            error!("modifyItem patch cannot set and remove container state simultaneously");
        } else if !patched.contents.is_empty() {
            error!(
                "modifyItem cannot remove container state from '{}' while it still holds items",
                patched.symbol
            );
        } else {
            patched.container_state = None;
        }
    } else if let Some(new_state) = patch.container_state {
        patched.container_state = Some(new_state);
    }

    // add / remove abilities
    for removal in &patch.remove_abilities {
        patched.abilities.remove(removal);
    }
    for addition in &patch.add_abilities {
        patched.abilities.insert(*addition);
    }

    Ok(patched)
}

/// Spawn an NPC in a Room. If the NPC is already in the world, it will be moved and a warning logged.
pub fn spawn_npc_in_room(world: &mut AmbleWorld, view: &mut View, npc_id: Uuid, room_id: Uuid) -> Result<()> {
    info!(
        "└─ action: spawning NPC '{}' in Room '{}'",
        symbol_or_unknown(&world.npcs, npc_id),
        symbol_or_unknown(&world.rooms, room_id)
    );
    if let Some(npc) = world.npcs.get_mut(&npc_id) {
        if npc.location.is_not_nowhere() {
            let current_loc = symbol_from_id(&world.rooms, npc.location.room_id()?).unwrap_or("<unknown room>");
            warn!(
                "spawn called on NPC {} who was already in-game -- MOVING from {current_loc}",
                npc.symbol(),
            );
        }
    }
    move_npc(world, view, npc_id, Location::Room(room_id))?;
    Ok(())
}

/// Remove an NPC from the world.
pub fn despawn_npc(world: &mut AmbleWorld, view: &mut View, npc_id: Uuid) -> Result<()> {
    info!("└─ action: despawning NPC '{}'", symbol_or_unknown(&world.npcs, npc_id));
    move_npc(world, view, npc_id, Location::Nowhere)?;
    Ok(())
}

pub fn set_npc_active(world: &mut AmbleWorld, npc_id: &Uuid, active: bool) -> Result<()> {
    if let Some(npc) = world.npcs.get_mut(&npc_id) {
        if let Some(ref mut mvmt) = npc.movement {
            mvmt.active = active;
        }
    } else {
        bail!("error: npc_id {npc_id} not found in world.npcs")
    }
    Ok(())
}

pub fn set_container_state(world: &mut AmbleWorld, item_id: Uuid, state: Option<ContainerState>) -> Result<()> {
    if let Some(item) = world.items.get_mut(&item_id) {
        item.container_state = state;
    } else {
        bail!("setting container state for item {item_id}: item not found");
    }
    info!("└─ action: setting container state for item {item_id}: {state:?}");
    Ok(())
}

/// Replaces an item with another, wherever it's located
pub fn replace_item(world: &mut AmbleWorld, old_id: &Uuid, new_id: &Uuid) -> Result<()> {
    // record old item's location and symbol
    let (location, old_sym) = if let Some(old_item) = world.items.get(&old_id) {
        (old_item.location, old_item.symbol.clone())
    } else {
        bail!("replacing item {old_id}: item not found");
    };

    // despawn old item
    despawn_item(world, &old_id)?;

    // update location of new item in world.items
    if let Some(new_item) = world.get_item_mut(*new_id) {
        new_item.location = location;
    }

    // update location itself to contain the new item, if needed; process depends
    // on which type of location the old item came from...
    match &location {
        Location::Item(container_uuid) => {
            if let Some(container) = world.get_item_mut(*container_uuid) {
                container.add_item(*new_id);
            }
        },
        Location::Inventory => world.player.add_item(*new_id),
        Location::Nowhere => warn!("replace_item called on an unspawned item ({old_sym})"),
        Location::Npc(npc_id) => {
            if let Some(npc) = world.npcs.get_mut(npc_id) {
                npc.add_item(*new_id);
            }
        },
        Location::Room(room_id) => {
            if let Some(room) = world.rooms.get_mut(room_id) {
                room.add_item(*new_id);
            }
        },
    }
    info!(
        "└─ action: ReplaceItem({}, {}) [Location = {location:?}",
        old_sym,
        symbol_or_unknown(&world.items, *new_id)
    );
    Ok(())
}

/// Drops an item in the current room and replaces it with the new version.
pub fn replace_drop_item(world: &mut AmbleWorld, old_id: &Uuid, new_id: &Uuid) -> Result<()> {
    despawn_item(world, old_id)?;
    spawn_item_in_current_room(world, new_id)?;
    Ok(())
}

/// Sets a new description for an `Item`
pub fn set_item_description(world: &mut AmbleWorld, item_id: &Uuid, text: &str) -> Result<()> {
    let item = world
        .get_item_mut(*item_id)
        .with_context(|| format!("changing item '{item_id} description"))?;
    item.description = text.to_string();
    // text is truncated below at max 50 chars for the log
    info!(
        "└─ action: SetItemDescription({}, \"{}\")",
        symbol_or_unknown(&world.items, *item_id),
        &text[..std::cmp::min(text.len(), 50)]
    );
    Ok(())
}

/// Sets a custom message that will be displayed when a player tries to use a blocked exit.
///
/// This function finds the exit from one room to another and sets a custom denial message
/// that will be shown instead of the generic "you can't go that way" message.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `exit_from` - UUID of the room containing the exit
/// * `exit_to` - UUID of the destination room (used to identify the specific exit)
/// * `msg` - The custom message to display when the exit is blocked
///
/// # Returns
///
/// Returns `Ok(())` if the message was set successfully, or an error if the exit
/// couldn't be found.
///
/// # Errors
///
/// - If the source room doesn't exist
/// - If no exit from the source room leads to the destination room
pub fn set_barred_message(world: &mut AmbleWorld, exit_from: &Uuid, exit_to: &Uuid, msg: &str) -> Result<()> {
    let room = world
        .rooms
        .get_mut(exit_from)
        .with_context(|| format!("trigger setting barred message: room_id {exit_from} not found"))?;
    let exit = room.exits.iter().find(|exit| exit.1.to == *exit_to);
    if let Some(exit) = exit {
        let (direction, mut exit) = (exit.0.clone(), exit.1.clone());
        exit.set_barred_msg(Some(msg.to_string()));
        room.exits.insert(direction, exit);
    }
    info!(
        "└─ action: SetBarredMessage({} -> {}, '{msg}')",
        symbol_or_unknown(&world.rooms, *exit_from),
        symbol_or_unknown(&world.rooms, *exit_to)
    );
    Ok(())
}

/// Make NPC refuse a specific item for a specific reason.
/// # Errors
///
pub fn npc_refuse_item(world: &mut AmbleWorld, view: &mut View, npc_id: Uuid, reason: &str) -> Result<()> {
    npc_says(world, view, &npc_id, reason)?;
    let npc_name = world
        .npcs
        .get(&npc_id)
        .with_context(|| "looking up NPC {npc_id} during item refusal")?
        .name();
    view.push(ViewItem::TriggeredEvent(format!(
        "{} returns it to you.",
        npc_name.npc_style()
    )));
    info!("└─ action: NpcRefuseItem({npc_name}, \"{reason}\"");
    Ok(())
}

/// Adds a weighted text option ("wedge") to a random text spinner.
///
/// Spinners are used throughout the game to provide randomized text for various
/// situations like NPC dialogue, ambient messages, and flavor text. Each wedge
/// has a weight that affects how likely it is to be selected.
///
/// # Parameters
///
/// * `spinners` - Mutable reference to the world's spinner collection
/// * `spin_type` - The type of spinner to modify (e.g., `NpcIgnore`, `TakeVerb`)
/// * `text` - The text content to add to the spinner
/// * `width` - The weight of this option (higher values make it more likely to be selected)
///
/// # Returns
///
/// Returns `Ok(())` if the wedge was added successfully.
///
/// # Errors
///
/// - If the specified spinner type doesn't exist in the world's spinner collection
///
/// # Examples
///
/// Adding a new take verb with higher probability:
/// ```ignore
/// add_spinner_wedge(spinners, SpinnerType::Core(CoreSpinnerType::TakeVerb), "snatch", 3)?;
/// ```
pub fn add_spinner_wedge(
    spinners: &mut HashMap<SpinnerType, Spinner<String>>,
    spin_type: &SpinnerType,
    text: &str,
    width: usize,
) -> Result<()> {
    let wedge = Wedge::new_weighted(text.to_string(), width);
    let spinref = spinners
        .get_mut(spin_type)
        .with_context(|| format!("add_spinner_wedge(_, {spin_type:?}, _, _): spinner not found"))?;
    *spinref = spinref.add_wedge(wedge);
    info!("└─ action: AddSpinnerWedge({spin_type:?}, \"{text}\"");
    Ok(())
}

/// Resets a sequence flag back to its initial step (0).
///
/// Sequence flags are used to track multi-step progress through game events,
/// puzzles, or storylines. This action resets the flag back to the beginning,
/// effectively undoing any progress made on that sequence.
///
/// # Parameters
///
/// * `player` - Mutable reference to the player whose flag will be reset
/// * `flag_name` - Name of the sequence flag to reset
///
/// # Behavior
///
/// - If the flag exists and is a sequence flag, it's reset to step 0
/// - If the flag doesn't exist, no action is taken
/// - Simple (non-sequence) flags are unaffected by this action
pub fn reset_flag(player: &mut Player, flag_name: &str) {
    info!("└─ action: ResetFlag(\"{flag_name}\")");
    player.reset_flag(flag_name);
}

/// Advances a sequence flag to the next step in the sequence.
///
/// Sequence flags track multi-step progress and can have optional limits.
/// This action moves the flag forward by one step, unless it has already
/// reached its maximum limit.
///
/// # Parameters
///
/// * `player` - Mutable reference to the player whose flag will be advanced
/// * `flag_name` - Name of the sequence flag to advance
///
/// # Behavior
///
/// - If the flag exists and hasn't reached its limit, it advances by one step
/// - If the flag is at its limit, no advancement occurs
/// - If the flag doesn't exist, no action is taken
/// - Simple (non-sequence) flags are unaffected by this action
pub fn advance_flag(player: &mut Player, flag_name: &str) {
    info!("└─ action: AdvanceFlag(\"{flag_name}\")");
    player.advance_flag(flag_name);
}

/// Displays a random message from a world spinner.
///
/// Spinners provide randomized ambient messages, flavor text, or contextual responses
/// throughout the game. This action selects a random message from the specified spinner
/// and displays it to the player as an ambient event.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the spinners
/// * `view` - Mutable reference to the player's view for displaying the message
/// * `spinner_type` - The type of spinner to draw a message from
///
/// # Returns
///
/// Returns `Ok(())` if a message was successfully selected and displayed.
///
/// # Errors
///
/// - If the requested spinner type doesn't exist in the world
///
/// # Behavior
///
/// - Selects a random weighted message from the spinner
/// - If the spinner returns an empty message, nothing is displayed
/// - Messages are styled as ambient events for visual distinction
pub fn spinner_message(world: &mut AmbleWorld, view: &mut View, spinner_type: &SpinnerType) -> Result<()> {
    if let Some(spinner) = world.spinners.get(spinner_type) {
        let msg = spinner.spin().unwrap_or_default();
        if !msg.is_empty() {
            view.push(ViewItem::AmbientEvent(format!("{}", msg.ambient_trig_style())));
        }
        info!("└─ action: SpinnerMessage(\"{msg}\")");
        Ok(())
    } else {
        bail!("action SpinnerMessage({spinner_type:?}): no spinner found for type");
    }
}

/// Removes a flag from the player.
///
/// This action removes a status flag from the player, effectively undoing
/// whatever condition or state the flag represented. This is useful for
/// temporary conditions, completed objectives, or state transitions.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the game's view system for notification
/// * `flag` - Name of the flag to remove
///
/// # Behavior
///
/// - If the flag exists, it's removed from the player's flag collection
/// - If exists and is a status flag (prefix = "status:"), push status change to view
/// - If the flag doesn't exist, a warning is logged but no error occurs
/// - Both simple and sequence flags can be removed with this action
pub fn remove_flag(world: &mut AmbleWorld, view: &mut View, flag: &str) {
    let target = Flag::simple(flag, 0);
    if world.player.flags.remove(&target) {
        info!("└─ action: RemoveFlag(\"{flag}\")");
        if let Some(status) = flag.strip_prefix("status:") {
            view.push(ViewItem::StatusChange {
                action: StatusAction::Remove,
                status: status.to_string(),
            });
        }
    } else {
        warn!("└─ action: RemoveFlag(\"{flag}\") - flag was not set");
    }
}
/// Makes an item non-transferable by marking it as restricted.
///
/// Restricted items cannot be taken from their current location or dropped
/// if they're already in inventory. This is useful for items that should
/// remain fixed in place once certain conditions are met, or for preventing
/// players from transferring key items at inappropriate times.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the item
/// * `item_id` - UUID of the item to restrict
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully restricted.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world
pub fn restrict_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        item.restricted = true;
        info!("└─ action: RestrictItem({}) \"{}\"", item.symbol(), item.name());
        Ok(())
    } else {
        bail!("action RestrictItem({item_id}): item not found");
    }
}
/// Makes an NPC speak a random line of dialogue based on their current mood.
///
/// This action uses the NPC's mood and the world's `NpcIgnore` spinner to
/// generate contextually appropriate dialogue. Different NPC states may
/// result in different types of responses.
///
/// # Parameters
///
/// * `world` - Reference to the game world containing the NPC and spinners
/// * `view` - Mutable reference to the player's view for displaying the dialogue
/// * `npc_id` - UUID of the NPC who should speak
///
/// # Returns
///
/// Returns `Ok(())` if dialogue was successfully generated and displayed.
///
/// # Errors
///
/// - If the specified NPC doesn't exist in the world
/// - If the `NpcIgnore` spinner required for dialogue generation is missing
pub fn npc_says_random(world: &AmbleWorld, view: &mut View, npc_id: &Uuid) -> Result<()> {
    let npc = world
        .npcs
        .get(npc_id)
        .with_context(|| format!("action NpcSaysRandom({npc_id}): npc not found"))?;
    let ignore_spinner = world
        .spinners
        .get(&SpinnerType::Core(CoreSpinnerType::NpcIgnore))
        .with_context(|| "failed lookup of NpcIgnore spinner".to_string())?;
    let line = npc.random_dialogue(ignore_spinner);
    view.push(ViewItem::NpcSpeech {
        speaker: npc.name().to_string(),
        quote: line.clone(),
    });
    info!("└─ action: NpcSays({}, \"{line}\")", npc.symbol());
    Ok(())
}

/// Makes an NPC speak a specific line of dialogue.
///
/// This action bypasses the NPC's mood system and makes them speak an exact
/// line of text. This is useful for scripted dialogue, important story moments,
/// or specific responses to player actions.
///
/// # Parameters
///
/// * `world` - Reference to the game world containing the NPC
/// * `view` - Mutable reference to the player's view for displaying the dialogue
/// * `npc_id` - UUID of the NPC who should speak
/// * `quote` - The exact text the NPC should say
///
/// # Returns
///
/// Returns `Ok(())` if the dialogue was successfully displayed.
///
/// # Errors
///
/// - If the specified NPC doesn't exist in the world
pub fn npc_says(world: &AmbleWorld, view: &mut View, npc_id: &Uuid, quote: &str) -> Result<()> {
    let npc_name = world
        .npcs
        .get(npc_id)
        .with_context(|| format!("action NpcSays({npc_id},_): npc not found"))?
        .name();
    view.push(ViewItem::NpcSpeech {
        speaker: npc_name.to_string(),
        quote: quote.to_string(),
    });
    info!("└─ action: NpcSays({npc_name}, \"{quote}\")");
    Ok(())
}

/// Awards points to the player or penalizes them if the amount is negative.
///
/// This action modifies the player's score and displays a message indicating
/// the point change. Points are typically awarded for solving puzzles,
/// discovering new areas, or completing objectives.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for displaying the point award
/// * `amount` - Number of points to award (negative values subtract points)
///
/// # Behavior
///
/// - Player's score is updated using saturating arithmetic to prevent overflow/underflow
/// - A message is displayed to the player showing the point change
/// - The action is logged for audit purposes
pub fn award_points(world: &mut AmbleWorld, view: &mut View, amount: isize) {
    world.player.score = world.player.score.saturating_add_signed(amount);
    info!("└─ action: AwardPoints({amount})");
    view.push(ViewItem::PointsAwarded(amount));
}

/// Adds a status flag to the player.
///
/// Flags are used to track player progress, unlock new content, and control
/// game flow. They can be simple boolean flags or sequence flags that track
/// multi-step progress through events or puzzles.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the game view for notification
/// * `flag` - The flag to add to the player's flag collection
///
/// # Behavior
///
/// - The flag is added to the player's flag collection
/// - If a flag with the same name already exists, it's replaced
/// - Both simple and sequence flags can be added
pub fn add_flag(world: &mut AmbleWorld, view: &mut View, flag: &Flag) {
    if let Some(status) = flag.name().strip_prefix("status:") {
        view.push(ViewItem::StatusChange {
            action: StatusAction::Apply,
            status: status.to_string(),
        });
    }
    world.player.flags.insert(flag.clone());
    info!("└─ action: AddFlag(\"{flag}\")");
}

/// Locks an exit in a specific direction from a room, preventing player movement.
///
/// Locked exits cannot be used by the player until they are unlocked again.
/// This is useful for creating barriers that require keys, solving puzzles,
/// or meeting other conditions before areas become accessible.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the rooms
/// * `from_room` - UUID of the room containing the exit to lock
/// * `direction` - Direction name of the exit to lock (e.g., "north", "up")
///
/// # Returns
///
/// Returns `Ok(())` if the exit was successfully locked.
///
/// # Errors
///
/// - If the specified room doesn't exist in the world
/// - If no exit in the specified direction exists from the given room
pub fn lock_exit(world: &mut AmbleWorld, from_room: &Uuid, direction: &String) -> Result<()> {
    if let Some(exit) = world
        .rooms
        .get_mut(from_room)
        .and_then(|rm| rm.exits.get_mut(direction))
    {
        exit.locked = true;
        info!(
            "└─ action: LockExit({direction}, from [{}]",
            symbol_or_unknown(&world.rooms, *from_room)
        );
        Ok(())
    } else {
        bail!("LockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// Unlocks an exit in a specific direction from a room, allowing player movement.
///
/// This action removes the lock from an exit, making it passable again.
/// Unlocked exits can be traversed by the player without restriction
/// (subject to any other requirements like flags or items).
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the rooms
/// * `from_room` - UUID of the room containing the exit to unlock
/// * `direction` - Direction name of the exit to unlock (e.g., "north", "up")
///
/// # Returns
///
/// Returns `Ok(())` if the exit was successfully unlocked.
///
/// # Errors
///
/// - If the specified room doesn't exist in the world
/// - If no exit in the specified direction exists from the given room
pub fn unlock_exit(world: &mut AmbleWorld, from_room: &Uuid, direction: &String) -> Result<()> {
    if let Some(exit) = world.rooms.get_mut(from_room).and_then(|r| r.exits.get_mut(direction)) {
        exit.locked = false;
        info!(
            "└─ action: UnlockExit({direction}, from [{}])",
            symbol_or_unknown(&world.rooms, *from_room)
        );
        Ok(())
    } else {
        bail!("UnlockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// Unlock an item
/// # Errors
/// - on invalid item uuid
pub fn unlock_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        match item.container_state {
            Some(ContainerState::Locked) => {
                item.container_state = Some(ContainerState::Open);
                info!("└─ action: UnlockItem({}) '{}'", item.symbol(), item.name());
            },
            Some(ContainerState::TransparentLocked) => {
                item.container_state = Some(ContainerState::Open);
                info!(
                    "└─ action: UnlockItem({}) '{}' (was transparent locked)",
                    item.symbol(),
                    item.name()
                );
            },
            Some(_) => warn!(
                "action UnlockItem({}): item wasn't locked",
                symbol_or_unknown(&world.items, *item_id)
            ),
            None => warn!("action UnlockItem({item_id}): item '{}' isn't a container", item.name()),
        }
        Ok(())
    } else {
        bail!("UnlockItem({item_id}): item id not found")
    }
}

/// Creates an item in a specific room.
///
/// This action places an item in a designated room, making it available for
/// player interaction. If the item already exists elsewhere in the world,
/// it will be moved to the new location (with a warning logged).
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `item_id` - UUID of the item to spawn
/// * `room_id` - UUID of the room where the item should appear
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully spawned in the room.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world's item collection
/// - If the specified room doesn't exist in the world
///
/// # Behavior
///
/// - If the item is already located somewhere, it's moved rather than duplicated
/// - The item's location is updated and it's added to the room's contents
/// - A warning is logged if an existing item had to be moved
pub fn spawn_item_in_specific_room(world: &mut AmbleWorld, item_id: &Uuid, room_id: &Uuid) -> Result<()> {
    // warn and remove item from world if it's already somewhere to avoid dups
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemRoom({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    // spawn in specified room as intended
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;
    info!(
        "└─ action: SpawnItemInRoom({}, {})",
        item.symbol(),
        symbol_or_unknown(&world.rooms, *room_id)
    );
    item.set_location_room(*room_id);
    world
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| anyhow!("room {} missing", room_id))?
        .add_item(*item_id);
    Ok(())
}

/// Creates an item in the player's current room.
///
/// This action places an item in whatever room the player is currently in,
/// making it immediately available for interaction. This is useful for
/// dynamically creating items based on player actions or story events.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `item_id` - UUID of the item to spawn
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully spawned in the current room.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world's item collection
/// - If the player is not currently in a room (e.g., in an invalid state)
/// - If the player's current room doesn't exist in the world
///
/// # Behavior
///
/// - If the item already exists elsewhere, it's moved rather than duplicated
/// - The item appears in whatever room the player currently occupies
/// - A warning is logged if an existing item had to be moved
pub fn spawn_item_in_current_room(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    // warn and remove item from world if it's already somewhere to avoid dups
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemCurrentRoom({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    // then spawn at current location as intended
    let room_id = world
        .player
        .location
        .room_ref()
        .with_context(|| "SpawnItemCurrentRoom: player not in a room".to_string())?;
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;

    info!("└─ action: SpawnItemCurrentRoom({})", item.symbol());
    item.set_location_room(*room_id);
    world
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| anyhow!("room {} missing", room_id))?
        .add_item(*item_id);
    Ok(())
}

/// Creates an item directly in the player's inventory.
///
/// This action places an item directly into the player's inventory, making it
/// immediately available for use. This is useful for rewards, quest items,
/// or other objects that should go straight to the player without appearing
/// in the world first.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `item_id` - UUID of the item to spawn in inventory
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully added to inventory.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world's item collection
///
/// # Behavior
///
/// - If the item already exists elsewhere, it's moved rather than duplicated
/// - The item's location is updated to inventory and added to player's items
/// - A warning is logged if an existing item had to be moved
pub fn spawn_item_in_inventory(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    // warn and remove item from world if it's already somewhere to avoid dups
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemInInventory({}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.symbol(),
            item.name()
        );
        despawn_item(world, item_id)?;
    }
    // add item to player inventory as intended
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;
    info!("└─ action: SpawnItemInInventory({})", item.symbol());
    item.set_location_inventory();
    world.player.add_item(*item_id);
    Ok(())
}

/// Creates an item inside a container item.
///
/// This action places an item within another item that has container capabilities.
/// The item becomes part of the container's contents and can be accessed when
/// the player examines or interacts with the container.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `item_id` - UUID of the item to spawn inside the container
/// * `container_id` - UUID of the container item that will hold the new item
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully placed in the container.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world's item collection
/// - If the specified container doesn't exist in the world
///
/// # Behavior
///
/// - If the item already exists elsewhere, it's moved rather than duplicated
/// - The item's location is updated to reference the container
/// - The container's contents are updated to include the item
/// - A warning is logged if an existing item had to be moved
pub fn spawn_item_in_container(world: &mut AmbleWorld, item_id: &Uuid, container_id: &Uuid) -> Result<()> {
    // if item is already in-world, warn and remove it to avoid duplications / inconsistent state
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemInContainer({item_id},_): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    // need to grab this here to avoid trouble with the borrow checker below
    let container_sym = symbol_or_unknown(&world.items, *container_id);

    // then spawn again in the desired location
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;
    info!("└─ action: SpawnItemInContainer({}, {})", item.symbol(), container_sym);
    item.set_location_item(*container_id);
    world
        .items
        .get_mut(container_id)
        .ok_or_else(|| anyhow!("container {} missing", container_id))?
        .add_item(*item_id);
    Ok(())
}

/// Displays a message to the player as a triggered event.
///
/// This action shows text to the player with special styling to indicate
/// it was triggered by a game event rather than being part of normal
/// room descriptions or dialogue.
///
/// # Parameters
///
/// * `view` - Mutable reference to the player's view for displaying the message
/// * `text` - The message text to display to the player
///
/// # Behavior
///
/// - The message is styled as a triggered event for visual distinction
/// - Long messages are truncated in the log for readability (first 50 characters)
pub fn show_message(view: &mut View, text: &String) {
    view.push(ViewItem::TriggeredEvent(format!("{}", text.triggered_style())));
    info!(
        "└─ action: ShowMessage(\"{}...\")",
        &text[..std::cmp::min(text.len(), 50)]
    );
}

/// Changes an NPC's behavioral state.
///
/// NPC states control how NPCs behave, what dialogue they use, and how they
/// respond to player interactions. Changing an NPC's state can dramatically
/// alter their behavior and available interactions.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the NPC
/// * `npc_id` - UUID of the NPC whose state should be changed
/// * `state` - The new state to assign to the NPC
///
/// # Returns
///
/// Returns `Ok(())` if the NPC's state was successfully changed.
///
/// # Errors
///
/// - If the specified NPC doesn't exist in the world
///
/// # Behavior
///
/// - If the NPC is already in the target state, no action is taken
/// - State changes are logged for debugging and audit purposes
pub fn set_npc_state(world: &mut AmbleWorld, npc_id: &Uuid, state: &NpcState) -> Result<()> {
    if let Some(npc) = world.npcs.get_mut(npc_id) {
        if npc.state == *state {
            return Ok(()); // no-op if NPC already in state
        }
        npc.state = state.clone();
        info!("└─ action: SetNpcState({}, {state:?})", npc.symbol());
        Ok(())
    } else {
        bail!("SetNpcState({npc_id},_): unknown NPC id");
    }
}

/// Makes a hidden exit visible and usable, or creates a new exit if none exists.
///
/// This action can either reveal a previously hidden exit or create an entirely
/// new passage between rooms. Hidden exits are useful for secret passages that
/// become available after solving puzzles or meeting certain conditions.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the rooms
/// * `direction` - Direction name for the exit (e.g., "north", "secret passage")
/// * `exit_from` - UUID of the room where the exit should appear
/// * `exit_to` - UUID of the destination room the exit leads to
///
/// # Returns
///
/// Returns `Ok(())` if the exit was successfully revealed or created.
///
/// # Errors
///
/// - If the source room doesn't exist in the world
///
/// # Behavior
///
/// - If an exit already exists in that direction, it's unhidden
/// - If no exit exists, a new one is created leading to the destination
/// - The exit becomes immediately usable by the player
pub fn reveal_exit(world: &mut AmbleWorld, direction: &String, exit_from: &Uuid, exit_to: &Uuid) -> Result<()> {
    let exit = world
        .rooms
        .get_mut(exit_from)
        .ok_or_else(|| anyhow!("invalid exit_from room {}", exit_from))?
        .exits
        .entry(direction.clone())
        .or_insert_with(|| Exit::new(*exit_to));
    exit.hidden = false;
    info!(
        "└─ action: RevealExit({direction}, from '{}', to '{}')",
        symbol_or_unknown(&world.rooms, *exit_from),
        symbol_or_unknown(&world.rooms, *exit_to)
    );
    Ok(())
}

/// Instantly moves the player to a different room.
///
/// This action bypasses normal movement mechanics and teleports the player
/// directly to a new location. This is useful for story events, traps,
/// magical transportation, or other situations where instant relocation is needed.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player and rooms
/// * `room_id` - UUID of the destination room
///
/// # Returns
///
/// Returns `Ok(())` if the player was successfully moved.
///
/// # Errors
///
/// - If the destination room doesn't exist in the world
///
/// # Behavior
///
/// - Player's location is immediately updated to the new room
/// - No movement restrictions or exit requirements are checked
/// - The move is logged for audit purposes
pub fn push_player(world: &mut AmbleWorld, room_id: &Uuid) -> Result<()> {
    if world.rooms.contains_key(room_id) {
        world.player.location = Location::Room(*room_id);
        info!("└─ action: PushPlayerTo({})", symbol_or_unknown(&world.rooms, *room_id));
        Ok(())
    } else {
        bail!("tried to push player to unknown room ({room_id})");
    }
}

/// Locks a container item, preventing access to its contents.
///
/// This action changes a container's state to locked, preventing players from
/// opening it or accessing its contents until it's unlocked. Only items with
/// container capabilities can be locked.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the item
/// * `item_id` - UUID of the container item to lock
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully locked.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world
/// - If the item is not a container (doesn't have a `container_state`)
///
/// # Behavior
///
/// - The item's container state is set to locked
/// - Players cannot access the container's contents until it's unlocked
/// - Items that are already locked remain locked (no state change)
pub fn lock_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        if item.container_state.is_some() {
            item.container_state = Some(ContainerState::Locked);
            info!("└─ action: LockItem({})", item.symbol());
        } else {
            warn!(
                "action LockItem({}): '{}' is not a container",
                item.symbol(),
                item.name()
            );
        }
        Ok(())
    } else {
        bail!("item ({item_id}) not found in world.items");
    }
}

/// Transfers an item from an NPC's inventory to the player's inventory.
///
/// This action handles the complete transfer of an item from an NPC to the player,
/// updating all necessary world state including item location, NPC inventory,
/// and player inventory.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `npc_id` - UUID of the NPC who currently has the item
/// * `item_id` - UUID of the item to transfer
///
/// # Returns
///
/// Returns `Ok(())` if the transfer was successful.
///
/// # Errors
///
/// - If the specified NPC doesn't exist in the world
/// - If the specified item doesn't exist in the world
/// - If the NPC doesn't actually have the item in their inventory
///
/// # Behavior
///
/// - Item's location is updated to inventory
/// - Item is removed from NPC's inventory and added to player's inventory
/// - The transfer is logged for audit purposes
pub fn give_to_player(world: &mut AmbleWorld, npc_id: &Uuid, item_id: &Uuid) -> Result<()> {
    let npc = world
        .npcs
        .get_mut(npc_id)
        .with_context(|| format!("NPC {npc_id} not found"))?;
    if npc.contains_item(*item_id) {
        let item = world
            .items
            .get_mut(item_id)
            .with_context(|| format!("item {item_id} in NPC inventory but missing from world.items"))?;
        item.set_location_inventory();
        npc.remove_item(*item_id);
        world.player.add_item(*item_id);
        info!("└─ action: GiveItemToPlayer({}, {})", npc.symbol(), item.symbol());
        Ok(())
    } else {
        bail!("item {} not found in NPC {} inventory", item_id, npc_id);
    }
}

/// Completely removes an item from the world.
///
/// This action removes an item from all world collections and sets its location
/// to "Nowhere", effectively making it disappear from the game. This is useful
/// for consumable items, temporary objects, or cleanup operations.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `item_id` - UUID of the item to remove from the world
///
/// # Returns
///
/// Returns `Ok(())` if the item was successfully removed.
///
/// # Errors
///
/// - If the specified item doesn't exist in the world
///
/// # Behavior
///
/// - Item is removed from its current location (room, inventory, container, or NPC)
/// - Item's location is set to `Location::Nowhere`
/// - All references to the item are cleaned up to maintain world consistency
/// - The removal is logged for audit purposes
pub fn despawn_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("unknown item {}", item_id))?;
    let prev_loc = std::mem::replace(&mut item.location, Location::Nowhere);
    info!("└─ action: DespawnItem({})", item.symbol());
    match prev_loc {
        Location::Room(id) => {
            if let Some(r) = world.rooms.get_mut(&id) {
                r.remove_item(*item_id);
            }
        },
        Location::Item(id) => {
            if let Some(c) = world.items.get_mut(&id) {
                c.remove_item(*item_id);
            }
        },
        Location::Npc(id) => {
            if let Some(n) = world.npcs.get_mut(&id) {
                n.remove_item(*item_id);
            }
        },
        Location::Inventory => {
            world.player.remove_item(*item_id);
        },
        Location::Nowhere => {},
    }
    Ok(())
}

/// Prevents a player from reading an item and displays a custom denial message.
///
/// This action is used to conditionally block reading of items based on
/// game state, missing tools, or story requirements. For example, text
/// might be too small to read without a magnifying glass, or a document
/// might be in a language the player doesn't understand yet.
///
/// # Parameters
///
/// * `view` - Mutable reference to the player's view for displaying the denial message
/// * `reason` - Custom message explaining why the item cannot be read
///
/// # Behavior
///
/// - The denial message is displayed to the player with special styling
/// - The read attempt is logged for debugging purposes
/// - No changes are made to world state (the item remains readable for future attempts)
///
/// # Common Use Cases
///
/// - Items requiring special tools (magnifying glass, translator, etc.)
/// - Text that becomes readable after learning languages or skills
/// - Documents that require specific lighting or conditions
/// - Story-gated content that shouldn't be accessible yet
pub fn deny_read(view: &mut View, reason: &String) {
    view.push(ViewItem::ActionFailure(format!("{}", reason.denied_style())));
    info!("└─ action: DenyRead(\"{reason}\")");
}

/// Schedules a list of actions to fire after a specified number of turns.
///
/// This action uses the world's scheduler to queue up future events. The actions
/// will be executed automatically when the specified number of turns have passed.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the scheduler
/// * `view` - Mutable reference to the view (used for potential error messages)
/// * `turns_ahead` - Number of turns to wait before firing the actions
/// * `actions` - List of actions to execute when the time comes
/// * `note` - Optional description of the event for logging purposes
///
/// # Returns
///
/// Returns `Ok(())` if the actions were successfully scheduled.
///
/// # Behavior
///
/// - Actions are queued in the world's scheduler priority queue
/// - The event will fire automatically during the REPL turn processing
/// - Multiple events can be scheduled for the same future turn
/// - Scheduled events persist across game saves/loads
pub fn schedule_in(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleIn({turns_ahead} turns, {} actions): \"{log_note}\"",
        actions.len()
    );

    world
        .scheduler
        .schedule_in(world.turn_count, turns_ahead, actions.to_vec(), note);
    Ok(())
}

/// Schedules a list of actions to fire on a specific turn.
///
/// This action uses the world's scheduler to queue up future events for an
/// exact turn number. The actions will be executed automatically when that
/// turn is reached.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the scheduler
/// * `view` - Mutable reference to the view (used for potential error messages)
/// * `on_turn` - Exact turn number when the actions should fire
/// * `actions` - List of actions to execute when the time comes
/// * `note` - Optional description of the event for logging purposes
///
/// # Returns
///
/// Returns `Ok(())` if the actions were successfully scheduled.
///
/// # Behavior
///
/// - Actions are queued in the world's scheduler priority queue
/// - If the target turn has already passed, the event may fire immediately
/// - Multiple events can be scheduled for the same turn
/// - Events fire in FIFO order when scheduled for the same turn
/// - Scheduled events persist across game saves/loads
pub fn schedule_on(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleOn(turn {on_turn}, {} actions): \"{log_note}\"",
        actions.len()
    );

    world.scheduler.schedule_on(on_turn, actions.to_vec(), note);
    Ok(())
}

/// Schedule actions to fire in the future, gated by a condition.
pub fn schedule_in_if(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    condition: &EventCondition,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleInIf({turns_ahead} turns, {} actions, on_false={on_false:?}): \"{log_note}\"",
        actions.len()
    );
    world.scheduler.schedule_in_if(
        world.turn_count,
        turns_ahead,
        Some(condition.clone()),
        on_false,
        actions.to_vec(),
        note,
    );
    Ok(())
}

/// Schedule actions to fire on a specific turn, gated by a condition.
pub fn schedule_on_if(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    condition: &EventCondition,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let log_note = note.as_deref().unwrap_or("<no note>");
    info!(
        "└─ action: ScheduleOnIf(turn {on_turn}, {} actions, on_false={on_false:?}): \"{log_note}\"",
        actions.len()
    );
    world
        .scheduler
        .schedule_on_if(on_turn, Some(condition.clone()), on_false, actions.to_vec(), note);
    Ok(())
}

/// Convenience: schedule with condition that player is in any of the supplied rooms.
pub fn schedule_if_player_in_any(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    room_ids: impl IntoIterator<Item = uuid::Uuid>,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let mut conds = Vec::new();
    for r in room_ids {
        conds.push(EventCondition::Trigger(TriggerCondition::InRoom(r)));
    }
    let condition = if conds.len() == 1 {
        conds.remove(0)
    } else {
        EventCondition::Any(conds)
    };
    schedule_in_if(world, _view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on an absolute turn with condition that player is in any of the supplied rooms.
pub fn schedule_on_if_player_in_any(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    room_ids: impl IntoIterator<Item = uuid::Uuid>,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let mut conds = Vec::new();
    for r in room_ids {
        conds.push(EventCondition::Trigger(TriggerCondition::InRoom(r)));
    }
    let condition = if conds.len() == 1 {
        conds.remove(0)
    } else {
        EventCondition::Any(conds)
    };
    schedule_on_if(world, _view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if the player has a specific item.
pub fn schedule_in_if_player_has_item(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    item_id: uuid::Uuid,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasItem(item_id));
    schedule_in_if(world, _view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if the player has a specific item.
pub fn schedule_on_if_player_has_item(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    item_id: uuid::Uuid,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasItem(item_id));
    schedule_on_if(world, _view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if the player is missing a specific item.
pub fn schedule_in_if_player_missing_item(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    item_id: uuid::Uuid,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingItem(item_id));
    schedule_in_if(world, _view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if the player is missing a specific item.
pub fn schedule_on_if_player_missing_item(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    item_id: uuid::Uuid,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingItem(item_id));
    schedule_on_if(world, _view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if a flag is set.
pub fn schedule_in_if_flag_set(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasFlag(flag.to_string()));
    schedule_in_if(world, _view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if a flag is set.
pub fn schedule_on_if_flag_set(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::HasFlag(flag.to_string()));
    schedule_on_if(world, _view, on_turn, &condition, on_false, actions, note)
}

/// Convenience: schedule in N turns if a flag is missing.
pub fn schedule_in_if_flag_missing(
    world: &mut AmbleWorld,
    _view: &mut View,
    turns_ahead: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingFlag(flag.to_string()));
    schedule_in_if(world, _view, turns_ahead, &condition, on_false, actions, note)
}

/// Convenience: schedule on a specific turn if a flag is missing.
pub fn schedule_on_if_flag_missing(
    world: &mut AmbleWorld,
    _view: &mut View,
    on_turn: usize,
    flag: &str,
    on_false: OnFalsePolicy,
    actions: &[TriggerAction],
    note: Option<String>,
) -> Result<()> {
    let condition = EventCondition::Trigger(TriggerCondition::MissingFlag(flag.to_string()));
    schedule_on_if(world, _view, on_turn, &condition, on_false, actions, note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item},
        npc::{MovementTiming, MovementType, Npc, NpcMovement, NpcState},
        player::Flag,
        room::Room,
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn build_test_world() -> (AmbleWorld, Uuid, Uuid) {
        let mut world = AmbleWorld::new_empty();
        let room1_id = Uuid::new_v4();
        let room2_id = Uuid::new_v4();

        let room1 = Room {
            id: room1_id,
            symbol: "r1".into(),
            name: "Room1".into(),
            base_description: "Room1".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        let room2 = Room {
            id: room2_id,
            symbol: "r2".into(),
            name: "Room2".into(),
            base_description: "Room2".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room1_id, room1);
        world.rooms.insert(room2_id, room2);
        world.player.location = Location::Room(room1_id);
        (world, room1_id, room2_id)
    }

    #[test]
    fn modify_room_updates_name_desc_and_exits() {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let dest_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        world.rooms.insert(
            room_id,
            Room {
                id: room_id,
                symbol: "lab".into(),
                name: "Lab".into(),
                base_description: "Original description".into(),
                overlays: Vec::new(),
                location: Location::Nowhere,
                visited: false,
                exits: HashMap::from([("north".into(), Exit::new(dest_id))]),
                contents: Default::default(),
                npcs: Default::default(),
            },
        );
        world.rooms.insert(
            dest_id,
            Room {
                id: dest_id,
                symbol: "hall".into(),
                name: "Hall".into(),
                base_description: "Hall".into(),
                overlays: Vec::new(),
                location: Location::Nowhere,
                visited: false,
                exits: HashMap::new(),
                contents: Default::default(),
                npcs: Default::default(),
            },
        );
        world.rooms.insert(
            target_id,
            Room {
                id: target_id,
                symbol: "vault".into(),
                name: "Vault".into(),
                base_description: "Vault".into(),
                overlays: Vec::new(),
                location: Location::Nowhere,
                visited: false,
                exits: HashMap::new(),
                contents: Default::default(),
                npcs: Default::default(),
            },
        );

        let patch = RoomPatch {
            name: Some("Ruined Lab".into()),
            desc: Some("Destroyed lab".into()),
            remove_exits: vec![dest_id],
            add_exits: vec![RoomExitPatch {
                direction: "through the vault door".into(),
                to: target_id,
                hidden: false,
                locked: true,
                required_flags: HashSet::from([Flag::simple("opened-vault", 0)]),
                required_items: HashSet::new(),
                barred_message: Some("You can't go that way yet.".into()),
            }],
        };

        modify_room(&mut world, room_id, &patch).expect("modify room");
        let room = world.rooms.get(&room_id).expect("room present");
        assert_eq!(room.name, "Ruined Lab");
        assert_eq!(room.base_description, "Destroyed lab");
        assert!(room.exits.get("north").is_none());
        let exit = room.exits.get("through the vault door").expect("new exit present");
        assert_eq!(exit.to, target_id);
        assert!(exit.locked);
        assert_eq!(exit.barred_message.as_deref(), Some("You can't go that way yet."));
        assert!(exit.required_flags.contains(&Flag::simple("opened-vault", 0)));
    }

    #[test]
    fn modify_room_missing_exit_ok() {
        // attempt to remove a non-existent exit should fail silently (with a log warning),
        // not panic
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        world.rooms.insert(
            room_id,
            Room {
                id: room_id,
                symbol: "lab".into(),
                name: "Lab".into(),
                base_description: "Original description".into(),
                overlays: Vec::new(),
                location: Location::Nowhere,
                visited: false,
                exits: HashMap::new(),
                contents: Default::default(),
                npcs: Default::default(),
            },
        );
        let missing_exit_target = Uuid::new_v4();
        let patch = RoomPatch {
            remove_exits: vec![missing_exit_target],
            ..Default::default()
        };
        assert!(modify_room(&mut world, room_id, &patch).is_ok());
    }

    #[test]
    fn modify_npc_updates_identity_and_dialogue() {
        let (mut world, room_id, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        let mut npc = make_npc(npc_id, Location::Room(room_id), NpcState::Normal);
        npc.dialogue.insert(NpcState::Normal, vec!["Hello there.".into()]);
        world.npcs.insert(npc_id, npc);

        let patch = NpcPatch {
            name: Some("Professor Whistles".into()),
            desc: Some("An eccentric inventor with wild hair.".into()),
            state: Some(NpcState::Happy),
            add_lines: vec![NpcDialoguePatch {
                state: NpcState::Happy,
                line: "Have you met my clockwork ferret?".into(),
            }],
            movement: None,
        };

        modify_npc(&mut world, npc_id, &patch).expect("modify npc succeeds");

        let npc = world.npcs.get(&npc_id).expect("npc present");
        assert_eq!(npc.name, "Professor Whistles");
        assert_eq!(npc.description, "An eccentric inventor with wild hair.");
        assert!(matches!(npc.state, NpcState::Happy));
        let happy_lines = npc.dialogue.get(&NpcState::Happy).expect("happy dialogue");
        assert!(
            happy_lines
                .iter()
                .any(|line| line == "Have you met my clockwork ferret?")
        );
    }

    #[test]
    fn modify_npc_updates_movement_and_flags() {
        let (mut world, room_a, room_b) = build_test_world();
        let npc_id = Uuid::new_v4();
        let mut npc = make_npc(npc_id, Location::Room(room_a), NpcState::Normal);
        npc.movement = Some(NpcMovement {
            movement_type: MovementType::Route {
                rooms: vec![room_a],
                current_idx: 0,
                loop_route: true,
            },
            timing: MovementTiming::EveryNTurns { turns: 2 },
            active: true,
            last_moved_turn: 0,
            paused_until: None,
        });
        world.npcs.insert(npc_id, npc);

        let patch = NpcPatch {
            movement: Some(NpcMovementPatch {
                route: Some(vec![room_a, room_b]),
                random_rooms: None,
                timing: Some(MovementTiming::EveryNTurns { turns: 5 }),
                active: Some(false),
                loop_route: Some(false),
            }),
            ..Default::default()
        };

        modify_npc(&mut world, npc_id, &patch).expect("modify npc succeeds");

        let npc = world.npcs.get(&npc_id).expect("npc present");
        let movement = npc.movement.as_ref().expect("movement present");
        match &movement.movement_type {
            MovementType::Route {
                rooms,
                current_idx,
                loop_route,
            } => {
                assert_eq!(rooms, &vec![room_a, room_b]);
                assert_eq!(*current_idx, 0);
                assert!(!loop_route);
            },
            other => panic!("expected route movement, got {other:?}"),
        }
        assert!(matches!(movement.timing, MovementTiming::EveryNTurns { turns: 5 }));
        assert!(!movement.active);
        assert_eq!(movement.last_moved_turn, world.turn_count);
    }

    #[test]
    fn modify_npc_creates_movement_if_missing() {
        let (mut world, room_id, other_room) = build_test_world();
        let npc_id = Uuid::new_v4();
        let npc = make_npc(npc_id, Location::Room(room_id), NpcState::Normal);
        world.npcs.insert(npc_id, npc);

        let patch = NpcPatch {
            movement: Some(NpcMovementPatch {
                route: Some(vec![room_id, other_room]),
                random_rooms: None,
                timing: Some(MovementTiming::OnTurn { turn: 7 }),
                active: Some(true),
                loop_route: Some(true),
            }),
            ..Default::default()
        };

        modify_npc(&mut world, npc_id, &patch).expect("modify npc succeeds");

        let npc = world.npcs.get(&npc_id).expect("npc present");
        let movement = npc.movement.as_ref().expect("movement present");
        assert!(matches!(movement.movement_type, MovementType::Route { .. }));
        assert!(matches!(movement.timing, MovementTiming::OnTurn { turn: 7 }));
        assert!(movement.active);
    }

    fn make_item(id: Uuid, location: Location, container_state: Option<ContainerState>) -> Item {
        Item {
            id,
            symbol: "it".into(),
            name: "Item".into(),
            description: "".into(),
            location,
            portable: true,
            container_state,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        }
    }

    fn make_npc(id: Uuid, location: Location, state: NpcState) -> Npc {
        Npc {
            id,
            symbol: "n".into(),
            name: "Npc".into(),
            description: "".into(),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state,
            movement: None,
        }
    }

    #[test]
    fn push_player_moves_player_to_room() {
        let (mut world, _start, dest) = build_test_world();
        assert!(push_player(&mut world, &dest).is_ok());
        assert_eq!(world.player.location, Location::Room(dest));
    }

    #[test]
    fn push_player_errors_with_invalid_room() {
        let (mut world, _, _) = build_test_world();
        let bad_room = Uuid::new_v4();
        assert!(push_player(&mut world, &bad_room).is_err());
    }

    #[test]
    fn add_and_remove_flag_updates_player_flags() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let flag = Flag::simple("test", 0);
        add_flag(&mut world, &mut view, &flag);
        assert!(world.player.flags.contains(&flag));
        remove_flag(&mut world, &mut view, "test");
        assert!(!world.player.flags.contains(&flag));
    }

    #[test]
    fn reset_and_advance_flag_modifies_sequence() {
        let (mut world, _, _) = build_test_world();
        let flag = Flag::sequence("quest", Some(2), 0);
        world.player.flags.insert(flag);
        advance_flag(&mut world.player, "quest");
        assert!(
            world
                .player
                .flags
                .iter()
                .any(|f| matches!(f, Flag::Sequence { name, step, .. } if name == "quest" && *step == 1))
        );
        reset_flag(&mut world.player, "quest");
        assert!(
            world
                .player
                .flags
                .iter()
                .any(|f| matches!(f, Flag::Sequence { name, step, .. } if name == "quest" && *step == 0))
        );
    }

    #[test]
    fn award_points_modifies_player_score() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        award_points(&mut world, &mut view, 5);
        assert_eq!(world.player.score, 6);
        award_points(&mut world, &mut view, -3);
        assert_eq!(world.player.score, 3);
    }

    #[test]
    fn restrict_item_sets_restricted_flag() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Room(room_id), None);
        world.items.insert(item_id, item);
        restrict_item(&mut world, &item_id).unwrap();
        assert!(world.items.get(&item_id).unwrap().restricted);
    }

    #[test]
    fn lock_and_unlock_item_changes_state() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Room(room_id), Some(ContainerState::Open));
        world.items.insert(item_id, item);
        lock_item(&mut world, &item_id).unwrap();
        assert_eq!(
            world.items.get(&item_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
        unlock_item(&mut world, &item_id).unwrap();
        assert_eq!(
            world.items.get(&item_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn lock_and_unlock_exit_changes_state() {
        let (mut world, room1_id, room2_id) = build_test_world();
        world
            .rooms
            .get_mut(&room1_id)
            .unwrap()
            .exits
            .insert("north".into(), Exit::new(room2_id));
        lock_exit(&mut world, &room1_id, &"north".into()).unwrap();
        assert!(world.rooms[&room1_id].exits["north"].locked);
        unlock_exit(&mut world, &room1_id, &"north".into()).unwrap();
        assert!(!world.rooms[&room1_id].exits["north"].locked);
    }

    #[test]
    fn modify_item_updates_scalar_fields() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let mut item = make_item(item_id, Location::Room(room_id), Some(ContainerState::Closed));
        item.text = Some("old text".to_string());
        world.items.insert(item_id, item);

        let patch = ItemPatch {
            name: Some("Renamed Widget".to_string()),
            desc: Some("Updated description".to_string()),
            text: Some("new dynamic text".to_string()),
            portable: Some(false),
            restricted: Some(true),
            container_state: Some(ContainerState::Open),
            ..Default::default()
        };

        modify_item(&mut world, item_id, &patch).unwrap();

        let updated = world.items.get(&item_id).unwrap();
        assert_eq!(updated.name, "Renamed Widget");
        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.text.as_deref(), Some("new dynamic text"));
        assert!(!updated.portable);
        assert!(updated.restricted);
        assert_eq!(updated.container_state, Some(ContainerState::Open));
    }

    #[test]
    fn modify_item_leaves_container_state_when_unset() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Room(room_id), Some(ContainerState::Locked));
        world.items.insert(item_id, item);

        let patch = ItemPatch {
            name: Some("Still Locked".to_string()),
            ..Default::default()
        };

        modify_item(&mut world, item_id, &patch).unwrap();

        let updated = world.items.get(&item_id).unwrap();
        assert_eq!(updated.container_state, Some(ContainerState::Locked));
    }

    #[test]
    fn modify_item_updates_abilities() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let mut item = make_item(item_id, Location::Room(room_id), None);
        item.abilities.insert(ItemAbility::Ignite);
        world.items.insert(item_id, item);

        let patch = ItemPatch {
            add_abilities: vec![ItemAbility::Read],
            remove_abilities: vec![ItemAbility::Ignite],
            ..Default::default()
        };

        modify_item(&mut world, item_id, &patch).unwrap();

        let updated = world.items.get(&item_id).unwrap();
        assert!(updated.abilities.contains(&ItemAbility::Read));
        assert!(!updated.abilities.contains(&ItemAbility::Ignite));
    }

    #[test]
    fn modify_item_removes_container_state_when_empty() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let container = make_item(item_id, Location::Room(room_id), Some(ContainerState::Open));
        world.items.insert(item_id, container);

        let patch = ItemPatch {
            remove_container_state: true,
            ..Default::default()
        };

        modify_item(&mut world, item_id, &patch).unwrap();

        let updated = world.items.get(&item_id).unwrap();
        assert_eq!(updated.container_state, None);
    }

    #[test]
    fn modify_item_errors_when_removing_container_state_with_contents() {
        let (mut world, room_id, _) = build_test_world();
        let container_id = Uuid::new_v4();
        let container = make_item(container_id, Location::Room(room_id), Some(ContainerState::Open));
        world.items.insert(container_id, container);

        let child_id = Uuid::new_v4();
        let child_item = make_item(child_id, Location::Item(container_id), None);
        world.items.insert(child_id, child_item);
        world.items.get_mut(&container_id).unwrap().contents.insert(child_id);

        let patch = ItemPatch {
            remove_container_state: true,
            ..Default::default()
        };

        let result = modify_item(&mut world, container_id, &patch);
        // should emit an error to the log but continue with other patch aspects and return OK
        // to allow execution to continue
        assert!(result.is_ok());
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn spawn_item_in_specific_room_places_item() {
        let (mut world, _room1, room2) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Nowhere, None);
        world.items.insert(item_id, item);
        spawn_item_in_specific_room(&mut world, &item_id, &room2).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Room(room2));
        assert!(world.rooms[&room2].contents.contains(&item_id));
    }

    #[test]
    fn spawn_item_in_current_room_places_item() {
        let (mut world, room1, _room2) = build_test_world();
        let item_id = Uuid::new_v4();
        world.items.insert(item_id, make_item(item_id, Location::Nowhere, None));
        spawn_item_in_current_room(&mut world, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Room(room1));
        assert!(world.rooms[&room1].contents.contains(&item_id));
    }

    #[test]
    fn spawn_item_in_inventory_adds_to_player() {
        let (mut world, _, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let mut item = make_item(item_id, Location::Nowhere, None);
        item.restricted = true;
        world.items.insert(item_id, item);
        spawn_item_in_inventory(&mut world, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Inventory);
        assert!(world.player.inventory.contains(&item_id));
        assert!(!world.items[&item_id].restricted);
    }

    #[test]
    fn spawn_item_in_container_places_item_inside() {
        let (mut world, room1, _) = build_test_world();
        let container_id = Uuid::new_v4();
        let container = make_item(container_id, Location::Room(room1), Some(ContainerState::Open));
        world.items.insert(container_id, container);
        world.rooms.get_mut(&room1).unwrap().contents.insert(container_id);
        let item_id = Uuid::new_v4();
        world.items.insert(item_id, make_item(item_id, Location::Nowhere, None));
        spawn_item_in_container(&mut world, &item_id, &container_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Item(container_id));
        assert!(world.items[&container_id].contents.contains(&item_id));
    }

    #[test]
    fn despawn_item_removes_item_from_world() {
        let (mut world, room1, _) = build_test_world();
        let item_id = Uuid::new_v4();
        world
            .items
            .insert(item_id, make_item(item_id, Location::Room(room1), None));
        world.rooms.get_mut(&room1).unwrap().contents.insert(item_id);
        despawn_item(&mut world, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Nowhere);
        assert!(!world.rooms[&room1].contents.contains(&item_id));
    }

    #[test]
    fn give_to_player_transfers_item_from_npc() {
        let (mut world, room1, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        let npc = make_npc(npc_id, Location::Room(room1), NpcState::Normal);
        world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id);
        world.npcs.insert(npc_id, npc);
        let item_id = Uuid::new_v4();
        world
            .items
            .insert(item_id, make_item(item_id, Location::Npc(npc_id), None));
        world.npcs.get_mut(&npc_id).unwrap().inventory.insert(item_id);
        give_to_player(&mut world, &npc_id, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Inventory);
        assert!(world.player.inventory.contains(&item_id));
        assert!(!world.npcs[&npc_id].inventory.contains(&item_id));
    }

    #[test]
    fn schedule_in_adds_event_to_scheduler() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let actions = vec![TriggerAction::ShowMessage("Test message".to_string())];

        schedule_in(&mut world, &mut view, 5, &actions, Some("Test event".to_string())).unwrap();

        assert_eq!(world.scheduler.events.len(), 1);
        assert_eq!(world.scheduler.heap.len(), 1);

        let event = &world.scheduler.events[0];
        assert_eq!(event.on_turn, world.turn_count + 5); // Should be current turn + 5
        assert_eq!(event.actions.len(), 1);
        assert_eq!(event.note, Some("Test event".to_string()));
    }

    #[test]
    fn schedule_on_adds_event_to_scheduler() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let actions = vec![TriggerAction::ShowMessage("Test message".to_string())];

        schedule_on(
            &mut world,
            &mut view,
            42,
            &actions,
            Some("Exact turn event".to_string()),
        )
        .unwrap();

        assert_eq!(world.scheduler.events.len(), 1);
        assert_eq!(world.scheduler.heap.len(), 1);

        let event = &world.scheduler.events[0];
        assert_eq!(event.on_turn, 42);
        assert_eq!(event.actions.len(), 1);
        assert_eq!(event.note, Some("Exact turn event".to_string()));
    }

    #[test]
    fn schedule_in_with_multiple_actions() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let actions = vec![
            TriggerAction::ShowMessage("First message".to_string()),
            TriggerAction::AwardPoints(10),
            TriggerAction::ShowMessage("Second message".to_string()),
        ];

        schedule_in(&mut world, &mut view, 3, &actions, None).unwrap();

        let event = &world.scheduler.events[0];
        assert_eq!(event.actions.len(), 3);
        assert_eq!(event.note, None);
    }

    #[test]
    fn schedule_on_with_no_note() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let actions = vec![TriggerAction::AwardPoints(5)];

        schedule_on(&mut world, &mut view, 100, &actions, None).unwrap();

        let event = &world.scheduler.events[0];
        assert_eq!(event.note, None);
        assert_eq!(event.on_turn, 100);
    }

    #[test]
    fn dispatch_action_schedule_in_works() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();

        let nested_actions = vec![TriggerAction::ShowMessage("Delayed message".to_string())];
        let action = TriggerAction::ScheduleIn {
            turns_ahead: 7,
            actions: nested_actions,
            note: Some("Integration test".to_string()),
        };

        dispatch_action(&mut world, &mut view, &action).unwrap();

        assert_eq!(world.scheduler.events.len(), 1);
        let event = &world.scheduler.events[0];
        assert_eq!(event.on_turn, world.turn_count + 7);
        assert_eq!(event.note, Some("Integration test".to_string()));
    }

    #[test]
    fn dispatch_action_schedule_on_works() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();

        let nested_actions = vec![
            TriggerAction::AwardPoints(25),
            TriggerAction::ShowMessage("Exact timing!".to_string()),
        ];
        let action = TriggerAction::ScheduleOn {
            on_turn: 50,
            actions: nested_actions,
            note: None,
        };

        dispatch_action(&mut world, &mut view, &action).unwrap();

        assert_eq!(world.scheduler.events.len(), 1);
        let event = &world.scheduler.events[0];
        assert_eq!(event.on_turn, 50);
        assert_eq!(event.actions.len(), 2);
        assert_eq!(event.note, None);
    }

    #[test]
    fn dispatch_action_conditional_executes_nested_actions_when_condition_true() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();

        let action = TriggerAction::Conditional {
            condition: EventCondition::Trigger(TriggerCondition::MissingFlag("hint".into())),
            actions: vec![TriggerAction::ShowMessage("Conditional fired".into())],
        };

        dispatch_action(&mut world, &mut view, &action).unwrap();

        assert!(
            view.items
                .iter()
                .any(|vi| matches!(vi, ViewItem::TriggeredEvent(msg) if msg.contains("Conditional fired")))
        );
    }

    #[test]
    fn dispatch_action_conditional_skips_actions_when_condition_false() {
        let (mut world, _, _) = build_test_world();
        world.player.flags.insert(Flag::simple("hint".into(), world.turn_count));
        let mut view = View::new();

        let action = TriggerAction::Conditional {
            condition: EventCondition::Trigger(TriggerCondition::MissingFlag("hint".into())),
            actions: vec![TriggerAction::ShowMessage("Should not appear".into())],
        };

        dispatch_action(&mut world, &mut view, &action).unwrap();

        assert!(
            view.items
                .iter()
                .all(|vi| !matches!(vi, ViewItem::TriggeredEvent(msg) if msg.contains("Should not appear")))
        );
    }

    #[test]
    fn replace_item_swaps_items_preserving_location() {
        let (mut world, room1, _) = build_test_world();
        let old_id = Uuid::new_v4();
        let new_id = Uuid::new_v4();
        world
            .items
            .insert(old_id, make_item(old_id, Location::Room(room1), None));
        world.rooms.get_mut(&room1).unwrap().contents.insert(old_id);
        world.items.insert(new_id, make_item(new_id, Location::Nowhere, None));

        replace_item(&mut world, &old_id, &new_id).unwrap();

        assert_eq!(world.items[&old_id].location, Location::Nowhere);
        assert_eq!(world.items[&new_id].location, Location::Room(room1));
        assert!(world.rooms[&room1].contents.contains(&new_id));
        assert!(!world.rooms[&room1].contents.contains(&old_id));
    }

    #[test]
    fn set_barred_message_updates_exit() {
        let (mut world, room1, room2) = build_test_world();
        world
            .rooms
            .get_mut(&room1)
            .unwrap()
            .exits
            .insert("north".into(), Exit::new(room2));

        set_barred_message(&mut world, &room1, &room2, "No entry").unwrap();

        let exit = world.rooms[&room1].exits.get("north").unwrap();
        assert_eq!(exit.barred_message, Some("No entry".to_string()));
    }

    #[test]
    fn npc_says_adds_dialogue_to_view() {
        let (mut world, room1, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        let npc = make_npc(npc_id, Location::Room(room1), NpcState::Normal);
        world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id);
        world.npcs.insert(npc_id, npc);

        let mut view = View::new();
        npc_says(&world, &mut view, &npc_id, "Hello there").unwrap();

        assert!(matches!(
            view.items.last(),
            Some(ViewItem::NpcSpeech { quote, .. }) if quote == "Hello there"
        ));
    }

    #[test]
    fn npc_says_random_uses_npc_dialogue() {
        let (mut world, room1, _) = build_test_world();
        world.spinners.insert(
            SpinnerType::Core(CoreSpinnerType::NpcIgnore),
            Spinner::new(vec![Wedge::new("Ignores you.".into())]),
        );
        let npc_id = Uuid::new_v4();
        let mut npc = make_npc(npc_id, Location::Room(room1), NpcState::Normal);
        npc.dialogue.insert(NpcState::Normal, vec!["Howdy".to_string()]);
        world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id);
        world.npcs.insert(npc_id, npc);

        let mut view = View::new();
        npc_says_random(&world, &mut view, &npc_id).unwrap();

        assert!(matches!(
            view.items.last(),
            Some(ViewItem::NpcSpeech { quote, .. }) if quote == "Howdy"
        ));
    }

    #[test]
    fn set_npc_state_changes_state() {
        let (mut world, room1, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        let npc = make_npc(npc_id, Location::Room(room1), NpcState::Normal);
        world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id);
        world.npcs.insert(npc_id, npc);

        set_npc_state(&mut world, &npc_id, &NpcState::Mad).unwrap();

        assert_eq!(world.npcs[&npc_id].state, NpcState::Mad);
    }
}
