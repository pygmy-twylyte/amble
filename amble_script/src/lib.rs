//! amble_script: Minimal DSL and compiler for Amble triggers
//!
//! This first iteration supports a single trigger shape:
//! - `trigger "name" when enter room <ident> {`
//! - `  if missing flag <ident> {`
//! - `    do show "..."`
//! - `    do add flag <ident>`
//! - `    do award points <number>`
//! - `  }`
//! - `}`
//!
//! The compiler produces a TOML representation matching the Amble engine's
//! expected RawTrigger schema and prints it to stdout.

mod parser;
pub use parser::{AstError, parse_program, parse_trigger};
pub use parser::{parse_goals, parse_items, parse_npcs, parse_program_full, parse_rooms, parse_spinners};

use thiserror::Error;
use toml_edit::{Array, ArrayOfTables, Document, InlineTable, Item, Table, value};

/// A minimal AST for a single trigger.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerAst {
    /// Human-readable trigger name.
    pub name: String,
    /// Optional developer note for this trigger.
    pub note: Option<String>,
    /// 1-based line number in the source file where this trigger starts.
    pub src_line: usize,
    /// The event condition that triggers this (e.g., enter room, take item, talk to npc).
    pub event: ConditionAst,
    /// List of conditions (currently only missing-flag).
    pub conditions: Vec<ConditionAst>,
    /// List of actions supported in this minimal version.
    pub actions: Vec<ActionAst>,
    /// If true, the trigger should only fire once.
    pub only_once: bool,
}

/// Trigger condition variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionAst {
    /// Event: trigger has no event; conditions only.
    Always,
    /// Event: player enters a room.
    EnterRoom(String),
    /// Event: player takes an item.
    TakeItem(String),
    /// Event: player touches / presses an item
    TouchItem(String),
    /// Event: player talks to an NPC.
    TalkToNpc(String),
    /// Event: player opens an item.
    OpenItem(String),
    /// Event: player leaves a room.
    LeaveRoom(String),
    /// Event: player looks at an item.
    LookAtItem(String),
    /// Event: player uses an item with an ability.
    UseItem {
        item: String,
        ability: String,
    },
    /// Event: player gives an item to an NPC.
    GiveToNpc {
        item: String,
        npc: String,
    },
    /// Event: player uses one item on another item with an interaction.
    UseItemOnItem {
        tool: String,
        target: String,
        interaction: String,
    },
    /// Event: player ingests an item using a specific mode (eat, drink, inhale).
    Ingest {
        item: String,
        mode: IngestModeAst,
    },
    /// Event: player performs an interaction on an item (tool-agnostic).
    ActOnItem {
        target: String,
        action: String,
    },
    /// Event: player takes an item from an NPC.
    TakeFromNpc {
        item: String,
        npc: String,
    },
    /// Event: player inserts an item into a container item.
    InsertItemInto {
        item: String,
        container: String,
    },
    /// Event: player drops an item.
    DropItem(String),
    /// Event: player unlocks an item.
    UnlockItem(String),
    /// Require that a flag is missing (by name).
    MissingFlag(String),
    /// Require that a flag is present (by name).
    HasFlag(String),
    /// Require that the player has an item (by symbol id).
    HasItem(String),
    /// Require that the player is currently in a room (by symbol id).
    PlayerInRoom(String),
    HasVisited(String),
    MissingItem(String),
    FlagInProgress(String),
    FlagComplete(String),
    WithNpc(String),
    NpcHasItem {
        npc: String,
        item: String,
    },
    NpcInState {
        npc: String,
        state: String,
    },
    ContainerHasItem {
        container: String,
        item: String,
    },
    Ambient {
        spinner: String,
        rooms: Option<Vec<String>>,
    },
    /// Random chance in percent (0-100).
    ChancePercent(f64),
    /// All of the nested conditions must hold.
    All(Vec<ConditionAst>),
    /// Any of the nested conditions may hold (not yet compilable for triggers).
    Any(Vec<ConditionAst>),
}

/// Ingestion modes supported by the DSL; mirrors engine `IngestMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestModeAst {
    Eat,
    Drink,
    Inhale,
}
impl IngestModeAst {
    fn as_str(&self) -> &'static str {
        match self {
            IngestModeAst::Eat => "eat",
            IngestModeAst::Drink => "drink",
            IngestModeAst::Inhale => "inhale",
        }
    }
}

/// Minimal action variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionAst {
    /// Show a message to the player.
    Show(String),
    /// Add a weighted wedge to a spinner
    AddSpinnerWedge {
        spinner: String,
        width: usize,
        text: String,
    },
    /// Add a simple flag by name.
    AddFlag(String),
    /// Add a sequence flag by name with optional limit (end)
    AddSeqFlag {
        name: String,
        end: Option<u8>,
    },
    /// Award points to the player's score.
    AwardPoints(i64),
    /// Remove a flag by name.
    RemoveFlag(String),
    /// Replace an item instance by symbol with another
    ReplaceItem {
        old_sym: String,
        new_sym: String,
    },
    /// Replace an item when dropped with another symbol
    ReplaceDropItem {
        old_sym: String,
        new_sym: String,
    },
    /// Apply an item patch to mutate fields atomically.
    ModifyItem {
        item: String,
        patch: ItemPatchAst,
    },
    /// Apply a room patch to mutate room fields atomically.
    ModifyRoom {
        room: String,
        patch: RoomPatchAst,
    },
    /// Apply an NPC patch to mutate npc fields atomically.
    ModifyNpc {
        npc: String,
        patch: NpcPatchAst,
    },
    /// Spawn an item into a room.
    SpawnItemIntoRoom {
        item: String,
        room: String,
    },
    /// Despawn an item.
    DespawnItem(String),
    /// Despawn an NPC
    DespawnNpc(String),
    /// Reset a sequence flag to step 0.
    ResetFlag(String),
    /// Advance a sequence flag by one step.
    AdvanceFlag(String),
    /// Set a barred message for an exit between rooms
    SetBarredMessage {
        exit_from: String,
        exit_to: String,
        msg: String,
    },
    /// Reveal an exit in a room in a direction to another room.
    RevealExit {
        exit_from: String,
        exit_to: String,
        direction: String,
    },
    /// Lock or unlock exits and items
    LockExit {
        from_room: String,
        direction: String,
    },
    UnlockExit {
        from_room: String,
        direction: String,
    },
    LockItem(String),
    UnlockItemAction(String),
    /// Push player to a room.
    PushPlayerTo(String),
    /// NPC gives an item to the player
    GiveItemToPlayer {
        npc: String,
        item: String,
    },
    /// Spawns
    SpawnItemInInventory(String),
    SpawnItemCurrentRoom(String),
    SpawnItemInContainer {
        item: String,
        container: String,
    },
    SpawnNpcIntoRoom {
        npc: String,
        room: String,
    },
    /// Set description for an item by symbol.
    SetItemDescription {
        item: String,
        text: String,
    },
    NpcSays {
        npc: String,
        quote: String,
    },
    NpcSaysRandom {
        npc: String,
    },
    /// NPC refuses an item with a reason
    NpcRefuseItem {
        npc: String,
        reason: String,
    },
    /// Set NPC active/inactive for movement
    SetNpcActive {
        npc: String,
        active: bool,
    },
    SetNpcState {
        npc: String,
        state: String,
    },
    DenyRead(String),
    RestrictItem(String),
    /// Set container state for an item by symbol; omit state to clear
    SetContainerState {
        item: String,
        state: Option<String>,
    },
    /// Show a random message from a spinner
    SpinnerMessage {
        spinner: String,
    },
    /// Schedules without conditions
    ScheduleIn {
        turns_ahead: usize,
        actions: Vec<ActionAst>,
        note: Option<String>,
    },
    ScheduleOn {
        on_turn: usize,
        actions: Vec<ActionAst>,
        note: Option<String>,
    },
    /// Schedule actions at turns ahead if a condition holds.
    ScheduleInIf {
        turns_ahead: usize,
        condition: Box<ConditionAst>,
        on_false: Option<OnFalseAst>,
        actions: Vec<ActionAst>,
        note: Option<String>,
    },
    /// Schedule actions on an absolute turn if a condition holds.
    ScheduleOnIf {
        on_turn: usize,
        condition: Box<ConditionAst>,
        on_false: Option<OnFalseAst>,
        actions: Vec<ActionAst>,
        note: Option<String>,
    },
    /// Conditionally execute nested actions when the condition evaluates true at runtime.
    Conditional {
        condition: Box<ConditionAst>,
        actions: Vec<ActionAst>,
    },
}

/// Data patch applied to an item when executing a `modify item` action.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemPatchAst {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub text: Option<String>,
    pub portable: Option<bool>,
    pub restricted: Option<bool>,
    pub container_state: Option<ContainerStateAst>,
    pub remove_container_state: bool,
    pub add_abilities: Vec<ItemAbilityAst>,
    pub remove_abilities: Vec<ItemAbilityAst>,
}

/// Data patch applied to a room when executing a `modify room` action.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoomPatchAst {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub remove_exits: Vec<String>,
    pub add_exits: Vec<RoomExitPatchAst>,
}

/// Exit data emitted inside a `modify room` action patch.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoomExitPatchAst {
    pub direction: String,
    pub to: String,
    pub hidden: bool,
    pub locked: bool,
    pub barred_message: Option<String>,
    pub required_flags: Vec<String>,
    pub required_items: Vec<String>,
}

/// NPC dialogue line update used inside a `modify npc` action.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcDialoguePatchAst {
    pub state: NpcStateValue,
    pub line: String,
}

/// Movement timing update for an NPC.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcTimingPatchAst {
    /// NPC moves every _n_ turns.
    EveryNTurns(usize),
    /// NPC moves on the specified absolute turn.
    OnTurn(usize),
}

/// Movement configuration updates for an NPC.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NpcMovementPatchAst {
    pub route: Option<Vec<String>>,
    pub random_rooms: Option<Vec<String>>,
    pub timing: Option<NpcTimingPatchAst>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

/// Data patch applied to an NPC when executing a `modify npc` action.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NpcPatchAst {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub state: Option<NpcStateValue>,
    pub add_lines: Vec<NpcDialoguePatchAst>,
    pub movement: Option<NpcMovementPatchAst>,
}

/// Policy to apply when a scheduled condition evaluates to false at fire time.
#[derive(Debug, Clone, PartialEq)]
pub enum OnFalseAst {
    /// Drop the scheduled event entirely.
    Cancel,
    /// Reschedule the event the specified number of turns ahead.
    RetryAfter { turns: usize },
    /// Reschedule the event for the very next turn.
    RetryNextTurn,
}

/// Errors that can occur while compiling the AST to TOML.
#[derive(Debug, Error)]
pub enum CompileError {
    /// The AST was missing expected components.
    #[error("invalid AST: {0}")]
    InvalidAst(String),
    /// Unsupported Any-group in trigger conditions (engine triggers only AND conditions).
    #[error("any(...) groups are not supported for triggers yet")]
    UnsupportedAnyGroup,
}

/// Compile a `TriggerAst` into a TOML string representing one RawTrigger.
///
/// The output matches the structure of `amble_engine/data/triggers.toml`, using
/// a single `[[triggers]]` table with `name`, `conditions`, and `actions`.
/// The `enter room` and `missingFlag` are emitted as conditions; actions map to
/// `showMessage`, `addFlag`, and `awardPoints` respectively.
///
/// Returns a pretty TOML string.
pub fn compile_trigger_to_toml(ast: &TriggerAst) -> Result<String, CompileError> {
    let doc = compile_triggers_to_doc(&[ast.clone()])?;
    Ok(doc.to_string())
}

/// Compile multiple triggers into a single TOML string with one `triggers` array.
pub fn compile_triggers_to_toml(asts: &[TriggerAst]) -> Result<String, CompileError> {
    let doc = compile_triggers_to_doc(asts)?;
    Ok(doc.to_string())
}

fn compile_triggers_to_doc(asts: &[TriggerAst]) -> Result<Document, CompileError> {
    for ast in asts {
        if ast.name.trim().is_empty() {
            return Err(CompileError::InvalidAst("trigger name is empty".into()));
        }
        // event presence validated in compile_triggers_to_doc
    }

    let mut doc = Document::new();

    // Helper: emit a single trigger to the given ArrayOfTables with the provided flat conditions
    fn emit_trigger(
        aot: &mut ArrayOfTables,
        name: &str,
        note: &Option<String>,
        src_line: usize,
        event: &ConditionAst,
        flat_conds: &[ConditionAst],
        actions: &[ActionAst],
        only_once: bool,
    ) {
        let mut trig = Table::new();
        trig["name"] = value(name.to_string());
        if only_once {
            trig["only_once"] = value(true);
        }
        if let Some(n) = note {
            trig["note"] = value(n.clone());
        }
        // Per-entry prefix comment pointing to source line
        if src_line > 0 {
            trig.decor_mut()
                .set_prefix(format!("# trigger {name} (source line {src_line})\n"));
        } else {
            trig.decor_mut().set_prefix(format!("# trigger {name}\n"));
        }

        let mut conds = Array::default();
        // event condition first (skip for Always)
        match event {
            ConditionAst::Always => { /* no event condition emitted */ },
            ConditionAst::EnterRoom(room) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("enter"));
                t.insert("room_id", toml_edit::Value::from(room.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::TakeItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("take"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::TouchItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("touch"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::TalkToNpc(npc) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("talkToNpc"));
                t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::TakeFromNpc { item, npc } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("takeFromNpc"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::InsertItemInto { item, container } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("insert"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("container_id", toml_edit::Value::from(container.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::DropItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("drop"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::UnlockItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("unlock"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::UseItemOnItem {
                tool,
                target,
                interaction,
            } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("useItemOnItem"));
                t.insert("interaction", toml_edit::Value::from(interaction.clone()));
                t.insert("target_id", toml_edit::Value::from(target.clone()));
                t.insert("tool_id", toml_edit::Value::from(tool.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::Ingest { item, mode } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("ingest"));
                t.insert("item_sym", toml_edit::Value::from(item.clone()));
                t.insert("mode", toml_edit::Value::from(mode.as_str()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::ActOnItem { target, action } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("actOnItem"));
                t.insert("target_sym", toml_edit::Value::from(target.clone()));
                t.insert("action", toml_edit::Value::from(action.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::OpenItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("open"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::LeaveRoom(room) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("leave"));
                t.insert("room_id", toml_edit::Value::from(room.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::LookAtItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("lookAt"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::UseItem { item, ability } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("useItem"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("ability", toml_edit::Value::from(ability.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            ConditionAst::GiveToNpc { item, npc } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("giveToNpc"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                conds.push(toml_edit::Value::from(t));
            },
            other => {
                // shouldn't be other types here
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("unknown"));
                t.insert("text", toml_edit::Value::from(format!("{:?}", other)));
                conds.push(toml_edit::Value::from(t));
            },
        }

        // Emit flattened simple conditions
        for c in flat_conds {
            match c {
                ConditionAst::Always => { /* ignore */ },
                ConditionAst::MissingFlag(flag) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("missingFlag"));
                    t.insert("flag", toml_edit::Value::from(flag.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::HasFlag(flag) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("hasFlag"));
                    t.insert("flag", toml_edit::Value::from(flag.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::HasItem(item) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("hasItem"));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::PlayerInRoom(room_id) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("inRoom"));
                    t.insert("room_id", toml_edit::Value::from(room_id.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::HasVisited(room) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("hasVisited"));
                    t.insert("room_id", toml_edit::Value::from(room.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::MissingItem(item) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("missingItem"));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::FlagInProgress(flag) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("flagInProgress"));
                    t.insert("flag", toml_edit::Value::from(flag.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::FlagComplete(flag) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("flagComplete"));
                    t.insert("flag", toml_edit::Value::from(flag.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::WithNpc(npc) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("withNpc"));
                    t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::NpcHasItem { npc, item } => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("npcHasItem"));
                    t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::NpcInState { npc, state } => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("npcInState"));
                    t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                    t.insert("state", toml_edit::Value::from(state.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::ContainerHasItem { container, item } => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("containerHasItem"));
                    t.insert("container_id", toml_edit::Value::from(container.clone()));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::Ambient { spinner, rooms } => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("ambient"));
                    if let Some(rs) = rooms {
                        let mut arr = Array::default();
                        for r in rs {
                            arr.push(toml_edit::Value::from(r.clone()));
                        }
                        t.insert("room_ids", toml_edit::Value::from(arr));
                    }
                    t.insert("spinner", toml_edit::Value::from(spinner.clone()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::Ingest { item, mode } => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("ingest"));
                    t.insert("item_sym", toml_edit::Value::from(item.clone()));
                    t.insert("mode", toml_edit::Value::from(mode.as_str()));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::ChancePercent(pct) => {
                    let one_in = if *pct <= 0.0 { f64::INFINITY } else { 100.0 / *pct };
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("chance"));
                    t.insert("one_in", toml_edit::Value::from(one_in));
                    conds.push(toml_edit::Value::from(t));
                },
                ConditionAst::All(_kids) | ConditionAst::Any(_kids) => {
                    // Should not appear after expansion
                },
                ConditionAst::EnterRoom(_)
                | ConditionAst::TakeItem(_)
                | ConditionAst::TouchItem(_)
                | ConditionAst::TalkToNpc(_)
                | ConditionAst::OpenItem(_)
                | ConditionAst::LeaveRoom(_)
                | ConditionAst::LookAtItem(_)
                | ConditionAst::UseItem { .. }
                | ConditionAst::GiveToNpc { .. }
                | ConditionAst::UseItemOnItem { .. }
                | ConditionAst::ActOnItem { .. }
                | ConditionAst::TakeFromNpc { .. }
                | ConditionAst::InsertItemInto { .. }
                | ConditionAst::DropItem(_)
                | ConditionAst::UnlockItem(_) => {
                    // Event conditions are emitted first separately
                },
            }
        }
        trig["conditions"] = Item::Value(conds.into());

        // actions
        let mut acts = Array::default();
        for a in actions {
            acts.push(action_to_value(a));
        }
        trig["actions"] = Item::Value(acts.into());

        trig.set_implicit(true);
        aot.push(trig);
    }

    // Expand conditions to handle All/Any into multiple flat variants
    fn expand_conditions(list: &[ConditionAst]) -> Vec<Vec<ConditionAst>> {
        // Start with an empty combination
        let mut combos: Vec<Vec<ConditionAst>> = vec![Vec::new()];
        for c in list {
            let alts: Vec<Vec<ConditionAst>> = match c {
                ConditionAst::All(kids) => expand_conditions(kids),
                ConditionAst::Any(kids) => {
                    let mut acc = Vec::new();
                    for kid in kids {
                        for v in expand_conditions(std::slice::from_ref(kid)) {
                            acc.push(v);
                        }
                    }
                    if acc.is_empty() { vec![Vec::new()] } else { acc }
                },
                simple => vec![vec![simple.clone()]],
            };
            // combine
            let mut next = Vec::new();
            for prefix in &combos {
                for alt in &alts {
                    let mut merged = prefix.clone();
                    merged.extend(alt.clone());
                    next.push(merged);
                }
            }
            combos = next;
        }
        combos
    }

    let mut aot = ArrayOfTables::new();
    for ast in asts {
        let expanded = expand_conditions(&ast.conditions);
        if expanded.is_empty() {
            emit_trigger(
                &mut aot,
                &ast.name,
                &ast.note,
                ast.src_line,
                &ast.event,
                &[],
                &ast.actions,
                ast.only_once,
            );
        } else {
            for flat in expanded {
                emit_trigger(
                    &mut aot,
                    &ast.name,
                    &ast.note,
                    ast.src_line,
                    &ast.event,
                    &flat,
                    &ast.actions,
                    ast.only_once,
                );
            }
        }
    }
    doc["triggers"] = Item::ArrayOfTables(aot);

    Ok(doc)
}

// -----------------
// Rooms (minimal)
// -----------------

/// Minimal AST for a room definition.
/// AST node describing a compiled room definition.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomAst {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub visited: bool, // defaults to false when omitted in DSL
    pub exits: Vec<(String, ExitAst)>,
    pub overlays: Vec<OverlayAst>,
    pub src_line: usize,
}

/// Connection between rooms emitted within a room AST.
#[derive(Debug, Clone, PartialEq)]
pub struct ExitAst {
    pub to: String,
    pub hidden: bool,
    pub locked: bool,
    pub barred_message: Option<String>,
    pub required_flags: Vec<String>,
    pub required_items: Vec<String>,
}

/// Conditional overlay text applied to a room.
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayAst {
    pub conditions: Vec<OverlayCondAst>,
    pub text: String,
}

/// Overlay predicate used when computing room description variants.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayCondAst {
    FlagSet(String),
    FlagUnset(String),
    FlagComplete(String),
    ItemPresent(String),
    ItemAbsent(String),
    PlayerHasItem(String),
    PlayerMissingItem(String),
    NpcPresent(String),
    NpcAbsent(String),
    NpcInState { npc: String, state: NpcStateValue },
    ItemInRoom { item: String, room: String },
}

/// NPC state reference used in overlays and patches.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcStateValue {
    Named(String),
    Custom(String),
}

/// AST node describing an item definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemAst {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub portable: bool,
    pub location: ItemLocationAst,
    pub container_state: Option<ContainerStateAst>,
    pub restricted: bool,
    pub abilities: Vec<ItemAbilityAst>,
    pub text: Option<String>,
    pub interaction_requires: Vec<(String, String)>,
    pub consumable: Option<ConsumableAst>,
    pub src_line: usize,
}

/// Possible item locations in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemLocationAst {
    Inventory(String),
    Room(String),
    Npc(String),
    Chest(String),
    Nowhere(String),
}

/// Container states expressible in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStateAst {
    Open,
    Closed,
    Locked,
    TransparentClosed,
    TransparentLocked,
}
impl ContainerStateAst {
    fn as_str(&self) -> &'static str {
        match self {
            ContainerStateAst::Open => "open",
            ContainerStateAst::Closed => "closed",
            ContainerStateAst::Locked => "locked",
            ContainerStateAst::TransparentClosed => "transparentClosed",
            ContainerStateAst::TransparentLocked => "transparentLocked",
        }
    }
}

/// Single item ability entry declared within an item.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemAbilityAst {
    pub ability: String,
    pub target: Option<String>,
}

/// Consumable configuration attached to an item.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumableAst {
    pub uses_left: usize,
    pub consume_on: Vec<ItemAbilityAst>,
    pub when_consumed: ConsumableWhenAst,
}

/// Behavior when a consumable item is depleted.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsumableWhenAst {
    Despawn,
    ReplaceInventory { replacement: String },
    ReplaceCurrentRoom { replacement: String },
}

// -----------------
// Spinners
// -----------------

/// Spinner definition containing weighted text wedges.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinnerAst {
    pub id: String,
    pub wedges: Vec<SpinnerWedgeAst>,
    pub src_line: usize,
}

/// Individual wedge (value + weight) inside a spinner.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinnerWedgeAst {
    pub text: String,
    pub width: usize,
}

// -----------------
// NPCs
// -----------------

/// Movement types supported for NPC definitions.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcMovementTypeAst {
    Route,
    Random,
}

/// Movement configuration emitted for NPCs.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcMovementAst {
    pub movement_type: NpcMovementTypeAst,
    pub rooms: Vec<String>,
    pub timing: Option<String>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

/// AST node describing an NPC definition.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcAst {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub location: NpcLocationAst,
    pub state: NpcStateValue,
    pub movement: Option<NpcMovementAst>,
    pub dialogue: Vec<(String, Vec<String>)>,
    pub src_line: usize,
}

/// Location specifier used for NPC placement.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcLocationAst {
    Room(String),
    Nowhere(String),
}

/// Compile rooms into TOML string matching amble_engine/data/rooms.toml structure.
pub fn compile_rooms_to_toml(rooms: &[RoomAst]) -> Result<String, CompileError> {
    let mut doc = Document::new();
    let mut aot = ArrayOfTables::new();
    for r in rooms {
        if r.id.trim().is_empty() || r.name.trim().is_empty() {
            return Err(CompileError::InvalidAst("room id/name missing".into()));
        }
        let mut t = Table::new();
        t["id"] = value(r.id.clone());
        t["name"] = value(r.name.clone());
        t["base_description"] = value(r.desc.clone());
        t["location"] = value("Nowhere");
        if r.visited {
            t["visited"] = value(true);
        }
        if !r.exits.is_empty() {
            let exits_item = t.entry("exits").or_insert(Item::Table(Table::new()));
            let exits_tbl = exits_item.as_table_mut().expect("exits should be table");
            // Mark the parent table implicit so no separate [rooms.exits] header is emitted
            exits_tbl.set_implicit(true);
            for (dir, ex) in &r.exits {
                let mut et = Table::new();
                et["to"] = value(ex.to.clone());
                if ex.hidden {
                    et["hidden"] = value(true);
                }
                if ex.locked {
                    et["locked"] = value(true);
                }
                if let Some(msg) = &ex.barred_message {
                    et["barred_message"] = value(msg.clone());
                }
                if !ex.required_items.is_empty() {
                    let mut arr = Array::default();
                    for it in &ex.required_items {
                        arr.push(it.clone());
                    }
                    et["required_items"] = Item::Value(arr.into());
                }
                if !ex.required_flags.is_empty() {
                    let mut arr = Array::default();
                    for name in &ex.required_flags {
                        let mut itab = InlineTable::new();
                        itab.insert("type", toml_edit::Value::from("simple"));
                        itab.insert("name", toml_edit::Value::from(name.clone()));
                        arr.push(toml_edit::Value::from(itab));
                    }
                    et["required_flags"] = Item::Value(arr.into());
                }
                exits_tbl.insert(dir.as_str(), Item::Table(et));
            }
        }
        if !r.overlays.is_empty() {
            let mut ov_aot = ArrayOfTables::new();
            for ov in &r.overlays {
                let mut ot = Table::new();
                // conditions
                let mut conds = Array::default();
                for c in &ov.conditions {
                    let mut itab = InlineTable::new();
                    match c {
                        OverlayCondAst::FlagSet(flag) => {
                            itab.insert("type", toml_edit::Value::from("flagSet"));
                            itab.insert("flag", toml_edit::Value::from(flag.clone()));
                        },
                        OverlayCondAst::FlagUnset(flag) => {
                            itab.insert("type", toml_edit::Value::from("flagUnset"));
                            itab.insert("flag", toml_edit::Value::from(flag.clone()));
                        },
                        OverlayCondAst::FlagComplete(flag) => {
                            itab.insert("type", toml_edit::Value::from("flagComplete"));
                            itab.insert("flag", toml_edit::Value::from(flag.clone()));
                        },
                        OverlayCondAst::ItemPresent(item) => {
                            itab.insert("type", toml_edit::Value::from("itemPresent"));
                            itab.insert("item_id", toml_edit::Value::from(item.clone()));
                        },
                        OverlayCondAst::ItemAbsent(item) => {
                            itab.insert("type", toml_edit::Value::from("itemAbsent"));
                            itab.insert("item_id", toml_edit::Value::from(item.clone()));
                        },
                        OverlayCondAst::PlayerHasItem(item) => {
                            itab.insert("type", toml_edit::Value::from("playerHasItem"));
                            itab.insert("item_id", toml_edit::Value::from(item.clone()));
                        },
                        OverlayCondAst::PlayerMissingItem(item) => {
                            itab.insert("type", toml_edit::Value::from("playerMissingItem"));
                            itab.insert("item_id", toml_edit::Value::from(item.clone()));
                        },
                        OverlayCondAst::NpcPresent(npc) => {
                            itab.insert("type", toml_edit::Value::from("npcPresent"));
                            itab.insert("npc_id", toml_edit::Value::from(npc.clone()));
                        },
                        OverlayCondAst::NpcAbsent(npc) => {
                            itab.insert("type", toml_edit::Value::from("npcAbsent"));
                            itab.insert("npc_id", toml_edit::Value::from(npc.clone()));
                        },
                        OverlayCondAst::NpcInState { npc, state } => {
                            itab.insert("type", toml_edit::Value::from("npcInState"));
                            itab.insert("npc_id", toml_edit::Value::from(npc.clone()));
                            match state {
                                NpcStateValue::Named(s) => {
                                    itab.insert("state", toml_edit::Value::from(s.clone()));
                                },
                                NpcStateValue::Custom(s) => {
                                    let mut st = InlineTable::new();
                                    st.insert("custom", toml_edit::Value::from(s.clone()));
                                    itab.insert("state", toml_edit::Value::from(st));
                                },
                            }
                        },
                        OverlayCondAst::ItemInRoom { item, room } => {
                            itab.insert("type", toml_edit::Value::from("itemInRoom"));
                            itab.insert("item_id", toml_edit::Value::from(item.clone()));
                            itab.insert("room_id", toml_edit::Value::from(room.clone()));
                        },
                    }
                    conds.push(toml_edit::Value::from(itab));
                }
                ot["conditions"] = Item::Value(conds.into());
                ot["text"] = value(ov.text.clone());
                ov_aot.push(ot);
            }
            t["overlays"] = Item::ArrayOfTables(ov_aot);
        }
        // add a prefix comment pointing to source line for this room (filename provided in CLI header)
        if r.src_line > 0 {
            t.decor_mut()
                .set_prefix(format!("# room {} (source line {})\n", r.id, r.src_line));
        } else {
            t.decor_mut().set_prefix(format!("# room {}\n", r.id));
        }
        aot.push(t);
    }
    doc["rooms"] = Item::ArrayOfTables(aot);
    Ok(doc.to_string())
}

/// Compile items into TOML string matching amble_engine/data/items.toml structure.
pub fn compile_items_to_toml(items: &[ItemAst]) -> Result<String, CompileError> {
    let mut doc = Document::new();
    let mut aot = ArrayOfTables::new();
    for it in items {
        if it.id.trim().is_empty() || it.name.trim().is_empty() {
            return Err(CompileError::InvalidAst("item id/name missing".into()));
        }
        let mut t = Table::new();
        t["id"] = value(it.id.clone());
        t["name"] = value(it.name.clone());
        t["description"] = value(it.desc.clone());
        t["portable"] = value(it.portable);
        let mut loc = InlineTable::new();
        match &it.location {
            ItemLocationAst::Inventory(owner) => {
                loc.insert("Inventory", toml_edit::Value::from(owner.clone()));
            },
            ItemLocationAst::Room(room) => {
                loc.insert("Room", toml_edit::Value::from(room.clone()));
            },
            ItemLocationAst::Npc(npc) => {
                loc.insert("Npc", toml_edit::Value::from(npc.clone()));
            },
            ItemLocationAst::Chest(chest) => {
                loc.insert("Chest", toml_edit::Value::from(chest.clone()));
            },
            ItemLocationAst::Nowhere(note) => {
                loc.insert("Nowhere", toml_edit::Value::from(note.clone()));
            },
        }
        t["location"] = Item::Value(loc.into());
        if let Some(cs) = &it.container_state {
            t["container_state"] = value(cs.as_str());
        }
        if it.restricted {
            t["restricted"] = value(true);
        }
        if let Some(txt) = &it.text {
            t["text"] = value(txt.clone());
        }
        if !it.interaction_requires.is_empty() {
            // nested table: interaction_requires.<interaction> = "ability"
            let inner = t.entry("interaction_requires").or_insert(Item::Table(Table::new()));
            let tbl = inner.as_table_mut().expect("table");
            for (interaction, ability) in &it.interaction_requires {
                tbl[interaction.as_str()] = value(ability.clone());
            }
        }
        if !it.abilities.is_empty() {
            let mut abil_aot = ArrayOfTables::new();
            for ab in &it.abilities {
                let mut at = Table::new();
                at["type"] = value(ab.ability.clone());
                if let Some(target) = &ab.target {
                    at["target"] = value(target.clone());
                }
                abil_aot.push(at);
            }
            t["abilities"] = Item::ArrayOfTables(abil_aot);
        }
        if let Some(consumable) = &it.consumable {
            let mut cons_table = Table::new();
            cons_table["uses_left"] = value(consumable.uses_left as i64);

            if !consumable.consume_on.is_empty() {
                let mut consume_aot = ArrayOfTables::new();
                for ab in &consumable.consume_on {
                    let mut at = Table::new();
                    at["type"] = value(ab.ability.clone());
                    if let Some(target) = &ab.target {
                        at["target"] = value(target.clone());
                    }
                    consume_aot.push(at);
                }
                cons_table["consume_on"] = Item::ArrayOfTables(consume_aot);
            }

            let mut when_tbl = Table::new();
            match &consumable.when_consumed {
                ConsumableWhenAst::Despawn => {
                    when_tbl["type"] = value("despawn");
                },
                ConsumableWhenAst::ReplaceInventory { replacement } => {
                    when_tbl["type"] = value("replaceInventory");
                    when_tbl["replacement"] = value(replacement.clone());
                },
                ConsumableWhenAst::ReplaceCurrentRoom { replacement } => {
                    when_tbl["type"] = value("replaceCurrentRoom");
                    when_tbl["replacement"] = value(replacement.clone());
                },
            }
            cons_table["when_consumed"] = Item::Table(when_tbl);

            t["consumable"] = Item::Table(cons_table);
        }
        if it.src_line > 0 {
            t.decor_mut()
                .set_prefix(format!("# item {} (source line {})\n", it.id, it.src_line));
        } else {
            t.decor_mut().set_prefix(format!("# item {}\n", it.id));
        }
        aot.push(t);
    }
    doc["items"] = Item::ArrayOfTables(aot);
    Ok(doc.to_string())
}

/// Compile spinners into TOML string matching amble_engine/data/spinners.toml structure.
pub fn compile_spinners_to_toml(spinners: &[SpinnerAst]) -> Result<String, CompileError> {
    let mut doc = Document::new();
    let mut aot = ArrayOfTables::new();
    for sp in spinners {
        if sp.id.trim().is_empty() {
            return Err(CompileError::InvalidAst("spinner id missing".into()));
        }
        let mut t = Table::new();
        t["spinnerType"] = value(sp.id.clone());
        let mut vals = Array::default();
        let mut widths = Array::default();
        for w in &sp.wedges {
            vals.push(w.text.clone());
            widths.push(w.width as i64);
        }
        vals.set_trailing_comma(true);
        widths.set_trailing_comma(true);
        t["values"] = Item::Value(vals.into());
        if sp.wedges.iter().any(|w| w.width != 1) {
            t["widths"] = Item::Value(widths.into());
        }
        if sp.src_line > 0 {
            t.decor_mut()
                .set_prefix(format!("# spinner {} (source line {})\n", sp.id, sp.src_line));
        } else {
            t.decor_mut().set_prefix(format!("# spinner {}\n", sp.id));
        }
        aot.push(t);
    }
    doc["spinners"] = Item::ArrayOfTables(aot);
    Ok(doc.to_string())
}

/// Compile NPCs into TOML string matching amble_engine/data/npcs.toml structure.
pub fn compile_npcs_to_toml(npcs: &[NpcAst]) -> Result<String, CompileError> {
    let mut doc = Document::new();
    let mut aot = ArrayOfTables::new();
    for n in npcs {
        if n.id.trim().is_empty() || n.name.trim().is_empty() {
            return Err(CompileError::InvalidAst("npc id/name missing".into()));
        }
        let mut t = Table::new();
        t["id"] = value(n.id.clone());
        t["name"] = value(n.name.clone());
        t["description"] = value(n.desc.clone());
        // state
        match &n.state {
            NpcStateValue::Named(s) => {
                t["state"] = value(s.clone());
            },
            NpcStateValue::Custom(s) => {
                let mut st = InlineTable::new();
                st.insert("custom", toml_edit::Value::from(s.clone()));
                t["state"] = Item::Value(st.into());
            },
        }
        // location
        let mut loc = InlineTable::new();
        match &n.location {
            NpcLocationAst::Room(room) => {
                loc.insert("Room", toml_edit::Value::from(room.clone()));
            },
            NpcLocationAst::Nowhere(note) => {
                loc.insert("Nowhere", toml_edit::Value::from(note.clone()));
            },
        }
        t["location"] = Item::Value(loc.into());
        // movement (optional)
        if let Some(mv) = &n.movement {
            let mut mt = Table::new();
            let mtype = match mv.movement_type {
                NpcMovementTypeAst::Route => "route",
                NpcMovementTypeAst::Random => "random",
            };
            mt["movement_type"] = value(mtype);
            let mut arr = Array::default();
            for r in &mv.rooms {
                arr.push(r.clone());
            }
            mt["rooms"] = Item::Value(arr.into());
            if let Some(ti) = &mv.timing {
                mt["timing"] = value(ti.clone());
            }
            if let Some(a) = mv.active {
                mt["active"] = value(a);
            }
            if let Some(loop_route) = mv.loop_route {
                mt["loop_route"] = value(loop_route);
            }
            t["movement"] = Item::Table(mt);
        }
        // dialogue
        if !n.dialogue.is_empty() {
            let mut dt = Table::new();
            for (k, lines) in &n.dialogue {
                let mut arr = Array::default();
                for line in lines {
                    arr.push(line.clone());
                }
                arr.set_trailing_comma(true);
                dt[k.as_str()] = Item::Value(arr.into());
            }
            t["dialogue"] = Item::Table(dt);
        }
        if n.src_line > 0 {
            t.decor_mut()
                .set_prefix(format!("# npc {} (source line {})\n", n.id, n.src_line));
        } else {
            t.decor_mut().set_prefix(format!("# npc {}\n", n.id));
        }
        aot.push(t);
    }
    doc["npcs"] = Item::ArrayOfTables(aot);
    Ok(doc.to_string())
}

// -----------------
// Goals
// -----------------

/// Logical grouping for goals used when rendering score breakdowns.
#[derive(Debug, Clone, PartialEq)]
pub enum GoalGroupAst {
    /// Mandatory goals that count toward completion.
    Required,
    /// Optional side objectives.
    Optional,
    /// Status effects or temporary conditions.
    StatusEffect,
}

/// Conditions that can activate, complete, or fail a goal.
#[derive(Debug, Clone, PartialEq)]
pub enum GoalCondAst {
    /// Goal requires the player to have a flag.
    HasFlag(String),
    /// Goal requires the player to be missing a flag.
    MissingFlag(String),
    /// Goal requires the player to possess an item.
    HasItem(String),
    /// Goal requires the player to reach a room.
    ReachedRoom(String),
    /// Goal requires another goal to be complete.
    GoalComplete(String),
    /// Goal depends on a sequence flag progressing but not finishing.
    FlagInProgress(String),
    /// Goal requires a sequence flag to reach its terminal step.
    FlagComplete(String),
}

/// High-level representation of a single goal definition in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub struct GoalAst {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group: GoalGroupAst,
    pub activate_when: Option<GoalCondAst>,
    pub failed_when: Option<GoalCondAst>,
    pub finished_when: GoalCondAst,
    pub src_line: usize,
}

pub fn compile_goals_to_toml(goals: &[GoalAst]) -> Result<String, CompileError> {
    let mut doc = Document::new();
    let mut aot = ArrayOfTables::new();
    for g in goals {
        if g.id.trim().is_empty() || g.name.trim().is_empty() {
            return Err(CompileError::InvalidAst("goal id/name missing".into()));
        }
        let mut t = Table::new();
        t["id"] = value(g.id.clone());
        t["name"] = value(g.name.clone());
        t["description"] = value(g.description.clone());
        let mut grp = InlineTable::new();
        grp.insert(
            "type",
            toml_edit::Value::from(match g.group {
                GoalGroupAst::Required => "required",
                GoalGroupAst::Optional => "optional",
                GoalGroupAst::StatusEffect => "status-effect",
            }),
        );
        t["group"] = Item::Value(grp.into());
        let cond_to_val = |c: &GoalCondAst| {
            let mut it = InlineTable::new();
            match c {
                GoalCondAst::HasFlag(f) => {
                    it.insert("type", toml_edit::Value::from("hasFlag"));
                    it.insert("flag", toml_edit::Value::from(f.clone()));
                },
                GoalCondAst::MissingFlag(f) => {
                    it.insert("type", toml_edit::Value::from("missingFlag"));
                    it.insert("flag", toml_edit::Value::from(f.clone()));
                },
                GoalCondAst::HasItem(i) => {
                    it.insert("type", toml_edit::Value::from("hasItem"));
                    it.insert("item_sym", toml_edit::Value::from(i.clone()));
                },
                GoalCondAst::ReachedRoom(r) => {
                    it.insert("type", toml_edit::Value::from("reachedRoom"));
                    it.insert("room_sym", toml_edit::Value::from(r.clone()));
                },
                GoalCondAst::GoalComplete(gid) => {
                    it.insert("type", toml_edit::Value::from("goalComplete"));
                    it.insert("goal_id", toml_edit::Value::from(gid.clone()));
                },
                GoalCondAst::FlagInProgress(f) => {
                    it.insert("type", toml_edit::Value::from("flagInProgress"));
                    it.insert("flag", toml_edit::Value::from(f.clone()));
                },
                GoalCondAst::FlagComplete(f) => {
                    it.insert("type", toml_edit::Value::from("flagComplete"));
                    it.insert("flag", toml_edit::Value::from(f.clone()));
                },
            }
            toml_edit::Value::from(it)
        };
        if let Some(cond) = &g.activate_when {
            if matches!(cond, GoalCondAst::HasFlag(s) if s.is_empty()) {
                // sentinel for immediate activation – skip emitting
            } else {
                t["activate_when"] = Item::Value(cond_to_val(cond));
            }
        }
        t["finished_when"] = Item::Value(cond_to_val(&g.finished_when));
        if let Some(cond) = &g.failed_when {
            t["failed_when"] = Item::Value(cond_to_val(cond));
        }
        if g.src_line > 0 {
            t.decor_mut()
                .set_prefix(format!("# goal {} (source line {})\n", g.id, g.src_line));
        } else {
            t.decor_mut().set_prefix(format!("# goal {}\n", g.id));
        }
        aot.push(t);
    }
    doc["goals"] = Item::ArrayOfTables(aot);
    Ok(doc.to_string())
}

fn action_to_value(a: &ActionAst) -> toml_edit::Value {
    match a {
        ActionAst::Show(text) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("showMessage"));
            t.insert("text", toml_edit::Value::from(text.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::AddSpinnerWedge { spinner, width, text } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("addSpinnerWedge"));
            t.insert("spinner", toml_edit::Value::from(spinner.clone()));
            t.insert("width", toml_edit::Value::from(*width as i64));
            t.insert("text", toml_edit::Value::from(text.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::AddFlag(name) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("addFlag"));
            let mut flag_tbl = InlineTable::new();
            flag_tbl.insert("type", toml_edit::Value::from("simple"));
            flag_tbl.insert("name", toml_edit::Value::from(name.clone()));
            t.insert("flag", toml_edit::Value::from(flag_tbl));
            toml_edit::Value::from(t)
        },
        ActionAst::AddSeqFlag { name, end } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("addFlag"));
            let mut flag_tbl = InlineTable::new();
            flag_tbl.insert("type", toml_edit::Value::from("sequence"));
            flag_tbl.insert("name", toml_edit::Value::from(name.clone()));
            if let Some(e) = end {
                flag_tbl.insert("end", toml_edit::Value::from(*e as i64));
            }
            t.insert("flag", toml_edit::Value::from(flag_tbl));
            toml_edit::Value::from(t)
        },
        ActionAst::AwardPoints(amount) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("awardPoints"));
            t.insert("amount", toml_edit::Value::from(*amount as i64));
            toml_edit::Value::from(t)
        },
        ActionAst::RemoveFlag(name) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("removeFlag"));
            t.insert("flag", toml_edit::Value::from(name.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::ReplaceItem { old_sym, new_sym } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("replaceItem"));
            t.insert("old_sym", toml_edit::Value::from(old_sym.clone()));
            t.insert("new_sym", toml_edit::Value::from(new_sym.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::ReplaceDropItem { old_sym, new_sym } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("replaceDropItem"));
            t.insert("old_sym", toml_edit::Value::from(old_sym.clone()));
            t.insert("new_sym", toml_edit::Value::from(new_sym.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::ModifyItem { item, patch } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("modifyItem"));
            t.insert("item_sym", toml_edit::Value::from(item.clone()));
            let patch_tbl = item_patch_to_inline_table(patch);
            t.insert("patch", toml_edit::Value::from(patch_tbl));
            toml_edit::Value::from(t)
        },
        ActionAst::ModifyRoom { room, patch } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("modifyRoom"));
            t.insert("room_sym", toml_edit::Value::from(room.clone()));
            let patch_tbl = room_patch_to_inline_table(patch);
            t.insert("patch", toml_edit::Value::from(patch_tbl));
            toml_edit::Value::from(t)
        },
        ActionAst::ModifyNpc { npc, patch } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("modifyNpc"));
            t.insert("npc_sym", toml_edit::Value::from(npc.clone()));
            let patch_tbl = npc_patch_to_inline_table(patch);
            t.insert("patch", toml_edit::Value::from(patch_tbl));
            toml_edit::Value::from(t)
        },
        ActionAst::ResetFlag(name) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("resetFlag"));
            t.insert("flag", toml_edit::Value::from(name.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::AdvanceFlag(name) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("advanceFlag"));
            t.insert("flag", toml_edit::Value::from(name.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SetBarredMessage {
            exit_from,
            exit_to,
            msg,
        } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("setBarredMessage"));
            t.insert("exit_from", toml_edit::Value::from(exit_from.clone()));
            t.insert("exit_to", toml_edit::Value::from(exit_to.clone()));
            t.insert("msg", toml_edit::Value::from(msg.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SpawnItemIntoRoom { item, room } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("spawnItemInRoom"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            t.insert("room_id", toml_edit::Value::from(room.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SpawnItemInInventory(item) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("spawnItemInInventory"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SpawnItemCurrentRoom(item) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("spawnItemCurrentRoom"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SpawnItemInContainer { item, container } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("spawnItemInContainer"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            t.insert("container_id", toml_edit::Value::from(container.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SpawnNpcIntoRoom { npc, room } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("spawnNpcIntoRoom"));
            t.insert("npc_sym", toml_edit::Value::from(npc.clone()));
            t.insert("room_sym", toml_edit::Value::from(room.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::DespawnNpc(npc) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("despawnNpc"));
            t.insert("npc_sym", toml_edit::Value::from(npc.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::DespawnItem(item) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("despawnItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SetItemDescription { item, text } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("setItemDescription"));
            t.insert("item_sym", toml_edit::Value::from(item.clone()));
            t.insert("text", toml_edit::Value::from(text.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::RevealExit {
            exit_from,
            exit_to,
            direction,
        } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("revealExit"));
            t.insert("exit_from", toml_edit::Value::from(exit_from.clone()));
            t.insert("exit_to", toml_edit::Value::from(exit_to.clone()));
            t.insert("direction", toml_edit::Value::from(direction.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::LockExit { from_room, direction } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("lockExit"));
            t.insert("from_room", toml_edit::Value::from(from_room.clone()));
            t.insert("direction", toml_edit::Value::from(direction.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::UnlockExit { from_room, direction } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("unlockExit"));
            t.insert("from_room", toml_edit::Value::from(from_room.clone()));
            t.insert("direction", toml_edit::Value::from(direction.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::LockItem(item) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("lockItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::UnlockItemAction(item) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("unlockItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::PushPlayerTo(room) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("pushPlayerTo"));
            t.insert("room_id", toml_edit::Value::from(room.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::GiveItemToPlayer { npc, item } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("giveItemToPlayer"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::ScheduleInIf {
            turns_ahead,
            condition,
            on_false,
            actions,
            note,
        } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("scheduleInIf"));
            t.insert("turns_ahead", toml_edit::Value::from(*turns_ahead as i64));
            t.insert("condition", event_condition_value(condition));
            if let Some(p) = on_false {
                t.insert("on_false", on_false_value(p));
            } else {
                t.insert("on_false", on_false_value(&OnFalseAst::Cancel));
            }
            let mut arr = Array::default();
            for ia in actions {
                arr.push(action_to_value(ia));
            }
            t.insert("actions", toml_edit::Value::from(arr));
            if let Some(n) = note {
                t.insert("note", toml_edit::Value::from(n.clone()));
            }
            toml_edit::Value::from(t)
        },
        ActionAst::ScheduleOnIf {
            on_turn,
            condition,
            on_false,
            actions,
            note,
        } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("scheduleOnIf"));
            t.insert("on_turn", toml_edit::Value::from(*on_turn as i64));
            t.insert("condition", event_condition_value(condition));
            if let Some(p) = on_false {
                t.insert("on_false", on_false_value(p));
            } else {
                t.insert("on_false", on_false_value(&OnFalseAst::Cancel));
            }
            let mut arr = Array::default();
            for ia in actions {
                arr.push(action_to_value(ia));
            }
            t.insert("actions", toml_edit::Value::from(arr));
            if let Some(n) = note {
                t.insert("note", toml_edit::Value::from(n.clone()));
            }
            toml_edit::Value::from(t)
        },
        ActionAst::ScheduleIn {
            turns_ahead,
            actions,
            note,
        } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("scheduleIn"));
            t.insert("turns_ahead", toml_edit::Value::from(*turns_ahead as i64));
            let mut arr = Array::default();
            for ia in actions {
                arr.push(action_to_value(ia));
            }
            t.insert("actions", toml_edit::Value::from(arr));
            if let Some(n) = note {
                t.insert("note", toml_edit::Value::from(n.clone()));
            }
            toml_edit::Value::from(t)
        },
        ActionAst::ScheduleOn { on_turn, actions, note } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("scheduleOn"));
            t.insert("on_turn", toml_edit::Value::from(*on_turn as i64));
            let mut arr = Array::default();
            for ia in actions {
                arr.push(action_to_value(ia));
            }
            t.insert("actions", toml_edit::Value::from(arr));
            if let Some(n) = note {
                t.insert("note", toml_edit::Value::from(n.clone()));
            }
            toml_edit::Value::from(t)
        },
        ActionAst::Conditional { condition, actions } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("conditional"));
            t.insert("condition", event_condition_value(condition));
            let mut arr = Array::default();
            for ia in actions {
                arr.push(action_to_value(ia));
            }
            t.insert("actions", toml_edit::Value::from(arr));
            toml_edit::Value::from(t)
        },
        ActionAst::NpcSays { npc, quote } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("npcSays"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            t.insert("quote", toml_edit::Value::from(quote.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::NpcSaysRandom { npc } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("npcSaysRandom"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::NpcRefuseItem { npc, reason } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("npcRefuseItem"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            t.insert("reason", toml_edit::Value::from(reason.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SetNpcActive { npc, active } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("setNpcActive"));
            t.insert("npc_sym", toml_edit::Value::from(npc.clone()));
            t.insert("active", toml_edit::Value::from(*active));
            toml_edit::Value::from(t)
        },
        ActionAst::SetNpcState { npc, state } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("setNpcState"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            // Support custom states via "custom:<name>" shorthand in DSL
            if let Some(rest) = state.strip_prefix("custom:") {
                let mut st = InlineTable::new();
                st.insert("custom", toml_edit::Value::from(rest.to_string()));
                t.insert("state", toml_edit::Value::from(st));
            } else {
                t.insert("state", toml_edit::Value::from(state.clone()));
            }
            toml_edit::Value::from(t)
        },
        ActionAst::DenyRead(reason) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("denyRead"));
            t.insert("reason", toml_edit::Value::from(reason.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::RestrictItem(item) => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("restrictItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            toml_edit::Value::from(t)
        },
        ActionAst::SetContainerState { item, state } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("setContainerState"));
            t.insert("item_sym", toml_edit::Value::from(item.clone()));
            if let Some(s) = state {
                t.insert("state", toml_edit::Value::from(s.clone()));
            }
            toml_edit::Value::from(t)
        },
        ActionAst::SpinnerMessage { spinner } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("spinnerMessage"));
            t.insert("spinner", toml_edit::Value::from(spinner.clone()));
            toml_edit::Value::from(t)
        },
    }
}

fn item_patch_to_inline_table(patch: &ItemPatchAst) -> InlineTable {
    let mut tbl = InlineTable::new();
    if let Some(name) = &patch.name {
        tbl.insert("name", toml_edit::Value::from(name.clone()));
    }
    if let Some(desc) = &patch.desc {
        tbl.insert("desc", toml_edit::Value::from(desc.clone()));
    }
    if let Some(text) = &patch.text {
        tbl.insert("text", toml_edit::Value::from(text.clone()));
    }
    if let Some(portable) = patch.portable {
        tbl.insert("portable", toml_edit::Value::from(portable));
    }
    if let Some(restricted) = patch.restricted {
        tbl.insert("restricted", toml_edit::Value::from(restricted));
    }
    if let Some(cs) = &patch.container_state {
        tbl.insert("container_state", toml_edit::Value::from(cs.as_str()));
    }
    if patch.remove_container_state {
        tbl.insert("remove_container_state", toml_edit::Value::from(true));
    }
    if !patch.add_abilities.is_empty() {
        let mut arr = Array::default();
        for ability in &patch.add_abilities {
            arr.push(item_ability_to_value(ability));
        }
        tbl.insert("add_abilities", toml_edit::Value::from(arr));
    }
    if !patch.remove_abilities.is_empty() {
        let mut arr = Array::default();
        for ability in &patch.remove_abilities {
            arr.push(item_ability_to_value(ability));
        }
        tbl.insert("remove_abilities", toml_edit::Value::from(arr));
    }
    tbl
}

fn room_patch_to_inline_table(patch: &RoomPatchAst) -> InlineTable {
    let mut tbl = InlineTable::new();
    if let Some(name) = &patch.name {
        tbl.insert("name", toml_edit::Value::from(name.clone()));
    }
    if let Some(desc) = &patch.desc {
        tbl.insert("desc", toml_edit::Value::from(desc.clone()));
    }
    if !patch.remove_exits.is_empty() {
        let mut arr = Array::default();
        for exit_sym in &patch.remove_exits {
            arr.push(exit_sym.clone());
        }
        tbl.insert("remove_exits", toml_edit::Value::from(arr));
    }
    if !patch.add_exits.is_empty() {
        let mut arr = Array::default();
        for exit in &patch.add_exits {
            let mut et = InlineTable::new();
            et.insert("direction", toml_edit::Value::from(exit.direction.clone()));
            et.insert("to", toml_edit::Value::from(exit.to.clone()));
            if exit.hidden {
                et.insert("hidden", toml_edit::Value::from(true));
            }
            if exit.locked {
                et.insert("locked", toml_edit::Value::from(true));
            }
            if let Some(msg) = &exit.barred_message {
                et.insert("barred_message", toml_edit::Value::from(msg.clone()));
            }
            if !exit.required_items.is_empty() {
                let mut items = Array::default();
                for item in &exit.required_items {
                    items.push(item.clone());
                }
                et.insert("required_items", toml_edit::Value::from(items));
            }
            if !exit.required_flags.is_empty() {
                let mut flags = Array::default();
                for flag in &exit.required_flags {
                    let mut itab = InlineTable::new();
                    itab.insert("type", toml_edit::Value::from("simple"));
                    itab.insert("name", toml_edit::Value::from(flag.clone()));
                    flags.push(toml_edit::Value::from(itab));
                }
                et.insert("required_flags", toml_edit::Value::from(flags));
            }
            arr.push(toml_edit::Value::from(et));
        }
        tbl.insert("add_exits", toml_edit::Value::from(arr));
    }
    tbl
}

fn npc_patch_to_inline_table(patch: &NpcPatchAst) -> InlineTable {
    let mut tbl = InlineTable::new();
    if let Some(name) = &patch.name {
        tbl.insert("name", toml_edit::Value::from(name.clone()));
    }
    if let Some(desc) = &patch.desc {
        tbl.insert("desc", toml_edit::Value::from(desc.clone()));
    }
    if let Some(state) = &patch.state {
        tbl.insert("state", npc_state_value_to_value(state));
    }
    if !patch.add_lines.is_empty() {
        let mut arr = Array::default();
        for entry in &patch.add_lines {
            let mut lt = InlineTable::new();
            lt.insert("line", toml_edit::Value::from(entry.line.clone()));
            lt.insert("state", npc_state_value_to_value(&entry.state));
            arr.push(toml_edit::Value::from(lt));
        }
        tbl.insert("add_lines", toml_edit::Value::from(arr));
    }
    if let Some(movement) = &patch.movement {
        let mut mt = InlineTable::new();
        if let Some(route) = &movement.route {
            if !route.is_empty() {
                let mut arr = Array::default();
                for room in route {
                    arr.push(room.clone());
                }
                mt.insert("route", toml_edit::Value::from(arr));
            }
        }
        if let Some(random_rooms) = &movement.random_rooms {
            if !random_rooms.is_empty() {
                let mut arr = Array::default();
                for room in random_rooms {
                    arr.push(room.clone());
                }
                mt.insert("random_rooms", toml_edit::Value::from(arr));
            }
        }
        if let Some(timing) = &movement.timing {
            let mut tt = InlineTable::new();
            match timing {
                NpcTimingPatchAst::EveryNTurns(turns) => {
                    tt.insert("type", toml_edit::Value::from("everyNTurns"));
                    tt.insert("turns", toml_edit::Value::from(*turns as i64));
                },
                NpcTimingPatchAst::OnTurn(turn) => {
                    tt.insert("type", toml_edit::Value::from("onTurn"));
                    tt.insert("turn", toml_edit::Value::from(*turn as i64));
                },
            }
            mt.insert("timing", toml_edit::Value::from(tt));
        }
        if let Some(active) = movement.active {
            mt.insert("active", toml_edit::Value::from(active));
        }
        if let Some(loop_route) = movement.loop_route {
            mt.insert("loop_route", toml_edit::Value::from(loop_route));
        }
        if !mt.is_empty() {
            tbl.insert("movement", toml_edit::Value::from(mt));
        }
    }
    tbl
}

fn npc_state_value_to_value(state: &NpcStateValue) -> toml_edit::Value {
    match state {
        NpcStateValue::Named(s) => toml_edit::Value::from(s.clone()),
        NpcStateValue::Custom(s) => {
            let mut st = InlineTable::new();
            st.insert("custom", toml_edit::Value::from(s.clone()));
            toml_edit::Value::from(st)
        },
    }
}

fn item_ability_to_value(ability: &ItemAbilityAst) -> toml_edit::Value {
    let mut entry = InlineTable::new();
    entry.insert("type", toml_edit::Value::from(ability.ability.clone()));
    if let Some(target) = &ability.target {
        entry.insert("target", toml_edit::Value::from(target.clone()));
    }
    toml_edit::Value::from(entry)
}

fn on_false_value(p: &OnFalseAst) -> toml_edit::Value {
    let mut t = InlineTable::new();
    match p {
        OnFalseAst::Cancel => {
            t.insert("type", toml_edit::Value::from("cancel"));
        },
        OnFalseAst::RetryNextTurn => {
            t.insert("type", toml_edit::Value::from("retryNextTurn"));
        },
        OnFalseAst::RetryAfter { turns } => {
            t.insert("type", toml_edit::Value::from("retryAfter"));
            t.insert("turns", toml_edit::Value::from(*turns as i64));
        },
    }
    toml_edit::Value::from(t)
}

fn event_condition_value(c: &ConditionAst) -> toml_edit::Value {
    match c {
        ConditionAst::All(kids) => {
            let mut t = InlineTable::new();
            let mut arr = Array::default();
            for k in kids {
                arr.push(event_condition_value(k));
            }
            t.insert("all", toml_edit::Value::from(arr));
            toml_edit::Value::from(t)
        },
        ConditionAst::Any(kids) => {
            let mut t = InlineTable::new();
            let mut arr = Array::default();
            for k in kids {
                arr.push(event_condition_value(k));
            }
            t.insert("any", toml_edit::Value::from(arr));
            toml_edit::Value::from(t)
        },
        leaf => toml_edit::Value::from(leaf_condition_inline(leaf)),
    }
}

fn leaf_condition_inline(c: &ConditionAst) -> InlineTable {
    let mut t = InlineTable::new();
    match c {
        ConditionAst::Always => { /* not a leaf; shouldn't be emitted */ },
        ConditionAst::MissingFlag(flag) => {
            t.insert("type", toml_edit::Value::from("missingFlag"));
            t.insert("flag", toml_edit::Value::from(flag.clone()));
        },
        ConditionAst::HasFlag(flag) => {
            t.insert("type", toml_edit::Value::from("hasFlag"));
            t.insert("flag", toml_edit::Value::from(flag.clone()));
        },
        ConditionAst::HasItem(item) => {
            t.insert("type", toml_edit::Value::from("hasItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::PlayerInRoom(room) => {
            t.insert("type", toml_edit::Value::from("inRoom"));
            t.insert("room_id", toml_edit::Value::from(room.clone()));
        },
        ConditionAst::ChancePercent(pct) => {
            let one_in = if *pct <= 0.0 { f64::INFINITY } else { 100.0 / *pct };
            t.insert("type", toml_edit::Value::from("chance"));
            t.insert("one_in", toml_edit::Value::from(one_in));
        },
        ConditionAst::HasVisited(room) => {
            t.insert("type", toml_edit::Value::from("hasVisited"));
            t.insert("room_id", toml_edit::Value::from(room.clone()));
        },
        ConditionAst::MissingItem(item) => {
            t.insert("type", toml_edit::Value::from("missingItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::FlagInProgress(flag) => {
            t.insert("type", toml_edit::Value::from("flagInProgress"));
            t.insert("flag", toml_edit::Value::from(flag.clone()));
        },
        ConditionAst::FlagComplete(flag) => {
            t.insert("type", toml_edit::Value::from("flagComplete"));
            t.insert("flag", toml_edit::Value::from(flag.clone()));
        },
        ConditionAst::WithNpc(npc) => {
            t.insert("type", toml_edit::Value::from("withNpc"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
        },
        ConditionAst::NpcHasItem { npc, item } => {
            t.insert("type", toml_edit::Value::from("npcHasItem"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::NpcInState { npc, state } => {
            t.insert("type", toml_edit::Value::from("npcInState"));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
            t.insert("state", toml_edit::Value::from(state.clone()));
        },
        ConditionAst::ContainerHasItem { container, item } => {
            t.insert("type", toml_edit::Value::from("containerHasItem"));
            t.insert("container_id", toml_edit::Value::from(container.clone()));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::Ambient { spinner, rooms } => {
            t.insert("type", toml_edit::Value::from("ambient"));
            if let Some(rs) = rooms {
                let mut arr = Array::default();
                for r in rs {
                    arr.push(toml_edit::Value::from(r.clone()));
                }
                t.insert("room_ids", toml_edit::Value::from(arr));
            }
            t.insert("spinner", toml_edit::Value::from(spinner.clone()));
        },
        ConditionAst::Ingest { item, mode } => {
            t.insert("type", toml_edit::Value::from("ingest"));
            t.insert("item_sym", toml_edit::Value::from(item.clone()));
            t.insert("mode", toml_edit::Value::from(mode.as_str()));
        },
        ConditionAst::EnterRoom(room) => {
            t.insert("type", toml_edit::Value::from("enter"));
            t.insert("room_id", toml_edit::Value::from(room.clone()));
        },
        ConditionAst::LeaveRoom(room) => {
            t.insert("type", toml_edit::Value::from("leave"));
            t.insert("room_id", toml_edit::Value::from(room.clone()));
        },
        ConditionAst::LookAtItem(item) => {
            t.insert("type", toml_edit::Value::from("lookAt"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::OpenItem(item) => {
            t.insert("type", toml_edit::Value::from("open"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::UseItem { item, ability } => {
            t.insert("type", toml_edit::Value::from("useItem"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            t.insert("ability", toml_edit::Value::from(ability.clone()));
        },
        ConditionAst::GiveToNpc { item, npc } => {
            t.insert("type", toml_edit::Value::from("giveToNpc"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
        },
        ConditionAst::UseItemOnItem {
            tool,
            target,
            interaction,
        } => {
            t.insert("type", toml_edit::Value::from("useItemOnItem"));
            t.insert("interaction", toml_edit::Value::from(interaction.clone()));
            t.insert("target_id", toml_edit::Value::from(target.clone()));
            t.insert("tool_id", toml_edit::Value::from(tool.clone()));
        },
        ConditionAst::ActOnItem { target, action } => {
            t.insert("type", toml_edit::Value::from("actOnItem"));
            t.insert("target_sym", toml_edit::Value::from(target.clone()));
            t.insert("action", toml_edit::Value::from(action.clone()));
        },
        ConditionAst::TakeItem(item) => {
            t.insert("type", toml_edit::Value::from("take"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::TouchItem(item) => {
            t.insert("type", toml_edit::Value::from("touch"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::DropItem(item) => {
            t.insert("type", toml_edit::Value::from("drop"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::UnlockItem(item) => {
            t.insert("type", toml_edit::Value::from("unlock"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
        },
        ConditionAst::TakeFromNpc { item, npc } => {
            t.insert("type", toml_edit::Value::from("takeFromNpc"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            t.insert("npc_id", toml_edit::Value::from(npc.clone()));
        },
        ConditionAst::InsertItemInto { item, container } => {
            t.insert("type", toml_edit::Value::from("insert"));
            t.insert("item_id", toml_edit::Value::from(item.clone()));
            t.insert("container_id", toml_edit::Value::from(container.clone()));
        },
        ConditionAst::TalkToNpc(_) => unreachable!(),
        ConditionAst::All(_) | ConditionAst::Any(_) => unreachable!(),
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_room_and_compile() {
        let src = r#"
room high-ridge {
  name "High Isolated Ridge"
  desc "A small, flat ridge."
  exit up -> parish-landing
  exit west -> snowfield
  overlay if flag set cleaned-plaque-1 {
    text "Shimmering plaque."
  }
}
"#;
        let rooms = crate::parse_rooms(src).expect("parse rooms ok");
        assert_eq!(rooms.len(), 1);
        let r = &rooms[0];
        assert_eq!(r.id, "high-ridge");
        assert_eq!(r.name, "High Isolated Ridge");
        assert_eq!(r.desc, "A small, flat ridge.");
        assert_eq!(r.visited, false);
        assert_eq!(r.exits.len(), 2);
        assert!(r.exits.iter().any(|(d, e)| d == "up" && e.to == "parish-landing"));
        assert!(r.exits.iter().any(|(d, e)| d == "west" && e.to == "snowfield"));

        let toml = crate::compile_rooms_to_toml(&rooms).expect("compile ok");
        assert!(toml.contains("[[rooms]]"));
        assert!(toml.contains("id = \"high-ridge\""));
        assert!(toml.contains("name = \"High Isolated Ridge\""));
        assert!(toml.contains("base_description = \"A small, flat ridge.\""));
        assert!(toml.contains("location = \"Nowhere\""));
        assert!(!toml.contains("visited = false"));
        assert!(toml.contains("[rooms.exits.up]"));
        assert!(toml.contains("to = \"parish-landing\""));
        assert!(toml.contains("[rooms.exits.west]"));
        assert!(toml.contains("to = \"snowfield\""));
        assert!(toml.contains("[[rooms.overlays]]"));
        assert!(toml.contains("type = \"flagSet\""));
        assert!(toml.contains("flag = \"cleaned-plaque-1\""));
        assert!(toml.contains("text = \"Shimmering plaque.\""));
    }

    #[test]
    fn parse_room_with_visited_true() {
        let src = r#"
room start {
  name "Start"
  desc "First room"
  visited true
  exit up -> guard-post {
    required_flags(simple cleared-fallen-tree),
    barred "You'll need to clear the tree from the path first."
  }
  overlay if npc present cmot_dibbler, npc in state cmot_dibbler happy {
    text "Dibbler is here and happy."
  }
  overlay if npc in state emh custom "want-emitter" {
    text "EMH wants an emitter."
  }
}
"#;
        let rooms = crate::parse_rooms(src).expect("parse rooms ok");
        assert_eq!(rooms[0].visited, true);
        let toml = crate::compile_rooms_to_toml(&rooms).expect("compile ok");
        assert!(toml.contains("visited = true"));
        assert!(toml.contains("[rooms.exits.up]"));
        assert!(toml.contains("to = \"guard-post\""));
        assert!(toml.contains("required_flags"));
        assert!(toml.contains("type = \"simple\""));
        assert!(toml.contains("name = \"cleared-fallen-tree\""));
        assert!(toml.contains("barred_message = \"You'll need to clear the tree from the path first.\""));
        assert!(toml.contains("[[rooms.overlays]]"));
        assert!(toml.contains("type = \"npcPresent\""));
        assert!(toml.contains("npc_id = \"cmot_dibbler\""));
        assert!(toml.contains("type = \"npcInState\""));
        assert!(toml.contains("state = \"happy\""));
        assert!(toml.contains("state = { custom = \"want-emitter\" }"));
    }

    #[test]
    fn room_exit_allows_quoted_direction_and_emits_quoted_key() {
        let src = r#"
room shoreline {
  name "Shoreline"
  desc "A long, pebbled shore."
  exit "along the shore" -> dunes
}
"#;
        let rooms = crate::parse_rooms(src).expect("parse rooms ok");
        assert_eq!(rooms.len(), 1);
        let r = &rooms[0];
        assert_eq!(r.id, "shoreline");
        assert!(r.exits.iter().any(|(d, e)| d == "along the shore" && e.to == "dunes"));

        let toml = crate::compile_rooms_to_toml(&rooms).expect("compile ok");
        // Expect a quoted TOML key for the exit direction containing spaces
        assert!(toml.contains("[rooms.exits.\"along the shore\"]"));
        assert!(toml.contains("to = \"dunes\""));
    }

    #[test]
    fn parse_minimal_trigger_ast() {
        let src = r#"
trigger "first visit high-ridge" when enter room high-ridge {
  if missing flag visited:high-ridge {
    do show "You take in the view."
    do add flag visited:high-ridge
    do award points 1
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.name, "first visit high-ridge");
        assert!(matches!(ast.event, ConditionAst::EnterRoom(ref s) if s == "high-ridge"));
        assert_eq!(
            ast.conditions,
            vec![ConditionAst::MissingFlag("visited:high-ridge".into())]
        );
        assert_eq!(
            ast.actions,
            vec![
                ActionAst::Show("You take in the view.".into()),
                ActionAst::AddFlag("visited:high-ridge".into()),
                ActionAst::AwardPoints(1),
            ]
        );

        // Ensure TOML is generated and contains expected keys
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("[[triggers]]"));
        assert!(toml.contains("name = \"first visit high-ridge\""));
        assert!(toml.contains("type = \"enter\""));
        assert!(toml.contains("room_id = \"high-ridge\""));
        assert!(toml.contains("type = \"missingFlag\""));
        assert!(toml.contains("flag = \"visited:high-ridge\""));
        assert!(toml.contains("type = \"showMessage\""));
        assert!(toml.contains("You take in the view."));
        assert!(toml.contains("type = \"addFlag\""));
        assert!(toml.contains("name = \"visited:high-ridge\""));
        assert!(toml.contains("type = \"awardPoints\""));
        assert!(toml.contains("amount = 1"));
    }

    #[test]
    fn parse_reveal_exit_with_quoted_direction() {
        let src = r#"
trigger "reveal special exit" when always {
  do reveal exit from hallway to armory direction "along the shore"
}
"#;
        let asts = parse_program(src).expect("parse ok");
        assert_eq!(asts.len(), 1);
        let ast = &asts[0];
        assert_eq!(ast.name, "reveal special exit");
        assert!(matches!(ast.event, ConditionAst::Always));
        assert!(asts[0]
            .actions
            .iter()
            .any(|a| matches!(a, ActionAst::RevealExit { exit_from, exit_to, direction } if exit_from == "hallway" && exit_to == "armory" && direction == "along the shore")));
    }

    #[test]
    fn parse_all_group_and_new_actions() {
        let src = r#"
trigger "example spawn" when enter room pantry {
  if all(has flag got-key, missing flag door-open) {
    do spawn item cake into room pantry
    do show "A cake materializes out of nowhere."
    do add flag cake-spawned
    do remove flag got-key
    do despawn item old-cake
    do award points 2
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.name, "example spawn");
        assert!(matches!(ast.event, ConditionAst::EnterRoom(ref s) if s == "pantry"));
        // conditions: All([...])
        match &ast.conditions[0] {
            ConditionAst::All(kids) => {
                assert!(matches!(kids[0], ConditionAst::HasFlag(ref s) if s == "got-key"));
                assert!(matches!(kids[1], ConditionAst::MissingFlag(ref s) if s == "door-open"));
            },
            other => panic!("unexpected condition: {:?}", other),
        }
        // actions include spawn, show, add, remove, despawn, award
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::SpawnItemIntoRoom{ item, room } if item == "cake" && room == "pantry"))
        );
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::Show(s) if s == "A cake materializes out of nowhere."))
        );
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::AddFlag(s) if s == "cake-spawned"))
        );
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::RemoveFlag(s) if s == "got-key"))
        );
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::DespawnItem(s) if s == "old-cake"))
        );
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::AwardPoints(n) if *n == 2))
        );

        // compile to TOML and spot-check keys
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("[[triggers]]"));
        assert!(toml.contains("type = \"enter\""));
        assert!(toml.contains("room_id = \"pantry\""));
        assert!(toml.contains("type = \"hasFlag\""));
        assert!(toml.contains("flag = \"got-key\""));
        assert!(toml.contains("type = \"missingFlag\""));
        assert!(toml.contains("flag = \"door-open\""));
        assert!(toml.contains("type = \"spawnItemInRoom\""));
        assert!(toml.contains("item_id = \"cake\""));
        assert!(toml.contains("room_id = \"pantry\""));
        assert!(toml.contains("type = \"removeFlag\""));
        assert!(toml.contains("type = \"despawnItem\""));
    }

    #[test]
    fn parse_chance_condition() {
        let src = r#"
trigger "example chance" when enter room forest {
  if chance 30% {
    do show "A branch nearly falls on your head."
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        match &ast.conditions[0] {
            ConditionAst::ChancePercent(p) => assert!((*p - 30.0).abs() < 0.01),
            _ => panic!("expected ChancePercent"),
        }
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"chance\""));
        assert!(toml.contains("one_in"));
    }

    #[test]
    fn parse_has_item_and_player_in_room_and_any_lowering() {
        let src = r#"
trigger "combo test" when enter room lab {
  if all(player in room lab, any(has item wrench, has item screwdriver)) {
    do show "tools present"
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        // Compile and expect two triggers (any(...) lowered)
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        // two [[triggers]] markers
        let count = toml.match_indices("[[triggers]]").count();
        assert_eq!(count, 2, "expected 2 triggers due to any() lowering:\n{}", toml);
        assert!(toml.contains("item_id = \"wrench\""));
        assert!(toml.contains("item_id = \"screwdriver\""));
        assert!(toml.contains("type = \"inRoom\""));
        assert!(toml.contains("room_id = \"lab\""));
    }

    #[test]
    fn parse_when_take_and_talk() {
        let src = r#"
trigger "take test" when take item wrench {
  if has flag ready {
    do show "took wrench"
  }
}

trigger "talk test" when talk to npc gonk {
  if missing flag shy {
    do show "hello gonk"
  }
}
"#;
        let asts = super::parser::parse_program(src).expect("parse ok");
        assert_eq!(asts.len(), 2);
        assert!(matches!(asts[0].event, ConditionAst::TakeItem(ref s) if s == "wrench"));
        assert!(matches!(asts[1].event, ConditionAst::TalkToNpc(ref s) if s == "gonk"));
    }

    #[test]
    fn parse_when_open_leave_look() {
        let src = r#"
trigger "open test" when open item box {
  if has flag ready {
    do show "box opened"
  }
}

trigger "leave test" when leave room hallway {
  if missing flag blocked {
    do show "you leave"
  }
}

trigger "look test" when look at item statue {
  if has flag curious {
    do show "you gaze at the statue"
  }
}
"#;
        let asts = super::parser::parse_program(src).expect("parse ok");
        assert_eq!(asts.len(), 3);
        assert!(matches!(asts[0].event, ConditionAst::OpenItem(ref s) if s == "box"));
        assert!(matches!(asts[1].event, ConditionAst::LeaveRoom(ref s) if s == "hallway"));
        assert!(matches!(asts[2].event, ConditionAst::LookAtItem(ref s) if s == "statue"));
    }

    #[test]
    fn parse_multiple_if_blocks_and_unconditional() {
        let src = r#"
trigger "multi-if" when enter room lab {
  if has flag a { do show "A" }
  if chance 50% { do show "B" }
  do show "C"
}
"#;
        let asts = super::parser::parse_program(src).expect("parse ok");
        // Expect three lowered triggers
        assert_eq!(asts.len(), 3);
        let toml = crate::compile_triggers_to_toml(&asts).expect("compile ok");
        // Count [[triggers]] blocks
        let count = toml.match_indices("[[triggers]]").count();
        assert_eq!(count, 3, "expected 3 triggers after lowering:\n{}", toml);
        // Contains actions A, B, C
        assert!(toml.contains("text = \"A\""));
        assert!(toml.contains("text = \"B\""));
        assert!(toml.contains("text = \"C\""));
        // Has a hasFlag and chance condition
        assert!(toml.contains("type = \"hasFlag\""));
        assert!(toml.contains("type = \"chance\""));
    }

    #[test]
    fn parse_when_use_and_give() {
        let src = r#"
trigger "use test" when use item portal_gun ability turnOn {
  if has flag portal-gun-powered {
    do show "gun used"
  }
}

trigger "give test" when give item printer_paper to npc receptionist {
  if missing flag refused {
    do show "paper given"
  }
}
"#;
        let asts = super::parser::parse_program(src).expect("parse ok");
        assert_eq!(asts.len(), 2);
        assert!(
            matches!(asts[0].event, ConditionAst::UseItem { item: ref i, ability: ref a } if i == "portal_gun" && a == "turnOn")
        );
        assert!(
            matches!(asts[1].event, ConditionAst::GiveToNpc { item: ref i, npc: ref n } if i == "printer_paper" && n == "receptionist")
        );
        let toml = compile_triggers_to_toml(&asts).expect("compile ok");
        assert!(toml.contains("type = \"useItem\""));
        assert!(toml.contains("ability = \"turnOn\""));
        assert!(toml.contains("type = \"giveToNpc\""));
        assert!(toml.contains("npc_id = \"receptionist\""));
    }

    #[test]
    fn parse_when_ingest_modes() {
        let src = r#"
trigger "eat test" when eat item apple {
  do show "tasty"
}

trigger "drink test" when drink item tonic_water {
  if has flag thirsty {
    do show "Refreshing."
  }
}

trigger "inhale test" when inhale item fumes {
  do show "You cough."
}
"#;
        let asts = super::parser::parse_program(src).expect("parse ok");
        assert_eq!(asts.len(), 3);
        assert!(matches!(
            asts[0].event,
            ConditionAst::Ingest { ref item, mode: IngestModeAst::Eat } if item == "apple"
        ));
        assert!(matches!(
            asts[1].event,
            ConditionAst::Ingest { ref item, mode: IngestModeAst::Drink } if item == "tonic_water"
        ));
        assert!(matches!(
            asts[2].event,
            ConditionAst::Ingest { ref item, mode: IngestModeAst::Inhale } if item == "fumes"
        ));

        let toml = compile_triggers_to_toml(&asts).expect("compile ok");
        assert!(toml.contains("type = \"ingest\""));
        assert!(toml.contains("item_sym = \"apple\""));
        assert!(toml.contains("mode = \"eat\""));
        assert!(toml.contains("mode = \"drink\""));
        assert!(toml.contains("mode = \"inhale\""));
    }

    #[test]
    fn parse_flag_in_progress_and_push_player() {
        let src = r#"
trigger "Aperture-Lab: Can't Enter While On Fire" when enter room aperture-lab {
  if flag in progress foam-fire-in-lab {
    do show "You try to get back into the lab, but the flames and oily smoke from the foam fire drive you back through the portal within seconds."
    do push player to portal-room
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert!(matches!(ast.event, ConditionAst::EnterRoom(ref s) if s == "aperture-lab"));
        assert!(matches!(ast.conditions[0], ConditionAst::FlagInProgress(ref s) if s == "foam-fire-in-lab"));
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::PushPlayerTo(ref r) if r == "portal-room"))
        );

        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"enter\""));
        assert!(toml.contains("room_id = \"aperture-lab\""));
        assert!(toml.contains("type = \"flagInProgress\""));
        assert!(toml.contains("flag = \"foam-fire-in-lab\""));
        assert!(toml.contains("type = \"pushPlayerTo\""));
        assert!(toml.contains("room_id = \"portal-room\""));
    }

    #[test]
    fn parse_all_with_npc_and_flag_in_progress() {
        let src = r#"
trigger "npc and progress" when always {
  if all(with npc emh, flag in progress hal-reboot) {
    do show "ok"
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        match &ast.conditions[0] {
            ConditionAst::All(kids) => {
                assert!(matches!(kids[0], ConditionAst::WithNpc(ref s) if s == "emh"));
                assert!(matches!(kids[1], ConditionAst::FlagInProgress(ref s) if s == "hal-reboot"));
            },
            other => panic!("unexpected condition: {:?}", other),
        }
    }

    #[test]
    fn parse_when_misc_events() {
        let src = r#"
trigger "drop test" when drop item towel {
  if has flag ready { do show "drop" }
}

trigger "insert test" when insert item battery into item portal_gun {
  if has flag ready { do show "insert" }
}

trigger "unlock test" when unlock item locker {
  if has flag key { do show "unlock" }
}

trigger "take-from test" when take item invitation from npc receptionist {
  if has flag ready { do show "take-from" }
}
"#;
        let asts = super::parser::parse_program(src).expect("parse ok");
        assert_eq!(asts.len(), 4);
        assert!(matches!(asts[0].event, ConditionAst::DropItem(ref s) if s == "towel"));
        assert!(
            matches!(asts[1].event, ConditionAst::InsertItemInto { item: ref i, container: ref c } if i == "battery" && c == "portal_gun")
        );
        assert!(matches!(asts[2].event, ConditionAst::UnlockItem(ref s) if s == "locker"));
        assert!(
            matches!(asts[3].event, ConditionAst::TakeFromNpc { item: ref i, npc: ref n } if i == "invitation" && n == "receptionist")
        );
        let toml = compile_triggers_to_toml(&asts).expect("compile ok");
        assert!(toml.contains("type = \"drop\""));
        assert!(toml.contains("type = \"insert\""));
        assert!(toml.contains("container_id = \"portal_gun\""));
        assert!(toml.contains("type = \"unlock\""));
        assert!(toml.contains("type = \"takeFromNpc\""));
    }

    #[test]
    fn parse_schedule_in_if_and_on_if() {
        let src = r#"
trigger "schedule demo" when enter room lab {
  if has flag ready {
    do schedule in 2 if all(has flag a, has flag b) onFalse retryAfter 3 note "demo" {
      do show "ready soon"
      do add flag scheduled
    }
    do schedule on 20 if missing flag late onFalse cancel {
      do award points 1
    }
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 2);
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"scheduleInIf\""));
        assert!(toml.contains("turns_ahead = 2"));
        assert!(toml.contains("all = ["));
        assert!(toml.contains("type = \"retryAfter\""));
        assert!(toml.contains("turns = 3"));
        assert!(toml.contains("note = \"demo\""));
        assert!(toml.contains("type = \"showMessage\""));
        assert!(toml.contains("type = \"addFlag\""));
        assert!(toml.contains("type = \"scheduleOnIf\""));
        assert!(toml.contains("on_turn = 20"));
        assert!(toml.contains("type = \"cancel\""));
        assert!(toml.contains("type = \"awardPoints\""));
    }

    #[test]
    fn parse_schedule_with_inner_if_blocks_emits_conditional_actions() {
        let src = r#"
trigger "gnat punctuation" when leave room high-ridge {
  do schedule in 2 if in rooms woods onFalse retryNextTurn note "gnats" {
    if missing flag read-scrawled-note {
      do show "first"
    }
    if has flag read-scrawled-note {
      do show "second"
    }
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        let conditional_count = toml.match_indices("type = \"conditional\"").count();
        assert_eq!(conditional_count, 2, "expected two conditional actions:\n{}", toml);
        assert!(toml.contains("type = \"missingFlag\""));
        assert!(toml.contains("type = \"hasFlag\""));
        assert!(toml.contains("text = \"first\""));
        assert!(toml.contains("text = \"second\""));
    }

    #[test]
    fn parse_schedule_in_and_on_unconditional() {
        let src = r#"
trigger "schedule simple" when enter room lab {
  do schedule in 3 {
    do show "soon"
  }
  do schedule on 42 note "absolute" {
    do award points 5
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 2);
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"scheduleIn\""));
        assert!(toml.contains("turns_ahead = 3"));
        assert!(toml.contains("type = \"scheduleOn\""));
        assert!(toml.contains("on_turn = 42"));
        assert!(toml.contains("note = \"absolute\""));
    }

    #[test]
    fn parse_without_if_block() {
        let src = r#"
trigger "burn fallen tree" when act burn on item fallen_tree {
  do show "The tree crackles and burns."
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.name, "burn fallen tree");
        assert!(
            matches!(ast.event, ConditionAst::ActOnItem { ref target, ref action } if target == "fallen_tree" && action == "burn")
        );
        assert!(ast.conditions.is_empty());
        assert!(matches!(ast.actions[0], ActionAst::Show(_)));
    }

    #[test]
    fn parse_container_has_item_condition() {
        let src = r#"
trigger "container check" when open item toolbox {
  if container toolbox has item wrench {
    do show "A trusty wrench sits inside."
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        // event
        assert!(matches!(ast.event, ConditionAst::OpenItem(ref i) if i == "toolbox"));
        // condition
        match &ast.conditions[0] {
            ConditionAst::ContainerHasItem { container, item } => {
                assert_eq!(container, "toolbox");
                assert_eq!(item, "wrench");
            },
            other => panic!("unexpected condition: {:?}", other),
        }
        // toml
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"containerHasItem\""));
        assert!(toml.contains("container_id = \"toolbox\""));
        assert!(toml.contains("item_id = \"wrench\""));
    }

    #[test]
    fn parse_when_always_with_ambient_and_chance() {
        let src = r#"
let set outside_house = (front-lawn, side-yard, back-yard)

trigger "Ambient: test" when always {
  if all(chance 50%, ambient ambientInterior in rooms outside_house,lobby) {
    do spinner message ambientInterior
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert!(matches!(ast.event, ConditionAst::Always));
        // compile; ensure no event-type emitted, but ambient + chance present
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"ambient\""));
        assert!(toml.contains("type = \"chance\""));
        assert!(!toml.contains("type = \"enter\""));
        assert!(!toml.contains("type = \"leave\""));
        assert!(toml.contains("type = \"spinnerMessage\""));
        assert!(toml.contains("front-lawn"));
        assert!(toml.contains("back-yard"));
    }

    #[test]
    fn parse_when_always_with_in_rooms_and_chance() {
        let src = r#"
let set outside_house = (front-lawn, side-yard, back-yard)

trigger "Ambient: preferred syntax" when always {
  if all(chance 20%, in rooms outside_house,lobby) {
    do spinner message ambientInterior
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert!(matches!(ast.event, ConditionAst::Always));
        // compile; ensure chance + inRoom emitted, not ambient
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        // any(...) lowering duplicates triggers; we can at least verify inRoom shows up
        assert!(toml.contains("type = \"inRoom\""));
        assert!(toml.contains("room_id = \"lobby\""));
        assert!(toml.contains("type = \"chance\""));
        assert!(toml.contains("type = \"spinnerMessage\""));
        assert!(!toml.contains("type = \"ambient\""));
    }
    #[test]
    fn parse_and_compile_misc_actions() {
        let src = r#"
trigger "misc actions" when enter room lab {
  if has flag ready {
    do add wedge "Chime" width 1 spinner ambientInterior
    do add seq flag quest limit 3
    do add seq flag status:nauseated
    do replace item dull_longsword with keen_longsword
    do replace drop item schrodingers_sandwich with schrodingers_sandwich
    do set barred message from hallway to armory "You can't go that way."
    do give item printer_paper to player from npc receptionist
    do npc refuse item emh "That's mine."
    do set container state evidence_locker_open locked
    do set npc state emh custom:want-emitter
    do spinner message ambientInterior
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::AddSpinnerWedge { spinner, width, text } if spinner == "ambientInterior" && *width == 1 && text == "Chime")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::ReplaceItem { old_sym, new_sym } if old_sym == "dull_longsword" && new_sym == "keen_longsword")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::ReplaceDropItem { old_sym, new_sym } if old_sym == "schrodingers_sandwich" && new_sym == "schrodingers_sandwich")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::SetBarredMessage { exit_from, exit_to, msg } if exit_from == "hallway" && exit_to == "armory" && msg.starts_with("You can't"))));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::GiveItemToPlayer { npc, item } if npc == "receptionist" && item == "printer_paper")));
        assert!(ast.actions.iter().any(
            |a| matches!(a, ActionAst::NpcRefuseItem { npc, reason } if npc == "emh" && reason.starts_with("That's"))
        ));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::SetContainerState { item, state } if item == "evidence_locker_open" && state.as_deref() == Some("locked"))));
        assert!(ast.actions.iter().any(
            |a| matches!(a, ActionAst::SetNpcState { npc, state } if npc == "emh" && state == "custom:want-emitter")
        ));
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::SpinnerMessage { spinner } if spinner == "ambientInterior"))
        );

        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        // spot check TOML output
        assert!(toml.contains("type = \"addSpinnerWedge\""));
        assert!(toml.contains("spinner = \"ambientInterior\""));
        assert!(toml.contains("type = \"replaceItem\""));
        assert!(toml.contains("type = \"replaceDropItem\""));
        assert!(toml.contains("type = \"setBarredMessage\""));
        assert!(toml.contains("type = \"giveItemToPlayer\""));
        assert!(toml.contains("type = \"npcRefuseItem\""));
        assert!(toml.contains("type = \"setContainerState\""));
        assert!(toml.contains("type = \"spinnerMessage\""));
        // custom NPC state should emit inline table
        assert!(toml.contains("type = \"setNpcState\""));
        assert!(toml.contains("npc_id = \"emh\""));
        assert!(toml.contains("custom = \"want-emitter\""));
        // seq flag assertions
        assert!(toml.contains("type = \"addFlag\""));
        assert!(toml.contains("type = \"sequence\""));
        assert!(toml.contains("name = \"quest\""));
        assert!(toml.contains("end = 3"));
    }

    #[test]
    fn add_spinner_wedge_width_optional_in_action() {
        let src = r#"
trigger "spinner add" when always {
  do add wedge "Ding" spinner ambientInterior
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        // ensure action parsed with default width 1
        assert!(ast
            .actions
            .iter()
            .any(|a| matches!(a, ActionAst::AddSpinnerWedge { spinner, width, text } if spinner == "ambientInterior" && *width == 1 && text == "Ding")));
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"addSpinnerWedge\""));
        assert!(toml.contains("spinner = \"ambientInterior\""));
        assert!(toml.contains("width = 1"));
    }

    #[test]
    fn spinner_wedge_width_optional_in_def() {
        let src = r#"
spinner ambientTest {
  wedge "Chime"
  wedge "Clack" width 2
}
"#;
        let spinners = parse_spinners(src).expect("parse ok");
        assert_eq!(spinners.len(), 1);
        assert_eq!(spinners[0].wedges.len(), 2);
        assert_eq!(spinners[0].wedges[0].text, "Chime");
        assert_eq!(spinners[0].wedges[0].width, 1);
        assert_eq!(spinners[0].wedges[1].text, "Clack");
        assert_eq!(spinners[0].wedges[1].width, 2);
        let toml = compile_spinners_to_toml(&spinners).expect("compile ok");
        assert!(toml.contains("spinnerType = \"ambientTest\""));
        assert!(toml.contains("values = [\"Chime\", \"Clack\",") || toml.contains("values = [\"Chime\", \"Clack\"]"));
        // widths should be emitted because there is a width != 1
        assert!(toml.contains("widths"));
    }

    #[test]
    fn parse_spawn_in_container_alias() {
        let src = r#"
trigger "spawn in container" when always {
  do spawn item broken_emitter in container lost_and_found_box
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert!(matches!(ast.event, ConditionAst::Always));
        assert!(ast
            .actions
            .iter()
            .any(|a| matches!(a, ActionAst::SpawnItemInContainer { item, container } if item == "broken_emitter" && container == "lost_and_found_box")));
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"spawnItemInContainer\""));
        assert!(toml.contains("item_id = \"broken_emitter\""));
        assert!(toml.contains("container_id = \"lost_and_found_box\""));
    }

    #[test]
    fn parse_goal_block_any_order() {
        let src = r#"
goal demo-goal {
  group optional
  done when has flag finished
  name "Demo Goal"
  start when missing flag prereq
  desc "Sequence can vary"
}
"#;
        let goals = crate::parse_goals(src).expect("parse goals ok");
        assert_eq!(goals.len(), 1);
        let g = &goals[0];
        assert_eq!(g.id, "demo-goal");
        assert_eq!(g.name, "Demo Goal");
        assert_eq!(g.description, "Sequence can vary");
        assert_eq!(g.group, GoalGroupAst::Optional);
        assert!(matches!(
            g.activate_when,
            Some(GoalCondAst::MissingFlag(ref cond)) if cond == "prereq"
        ));
        assert!(matches!(&g.finished_when, GoalCondAst::HasFlag(cond) if cond == "finished"));
    }

    #[test]
    fn parse_goal_with_fail_when() {
        let src = r#"
goal fail-goal {
  name "Failure State"
  desc "Demonstrates fail when support"
  group required
  done when has flag success
  fail when has flag abort
}
"#;
        let goals = crate::parse_goals(src).expect("parse goals ok");
        assert_eq!(goals.len(), 1);
        let g = &goals[0];
        assert!(g.activate_when.is_none());
        assert!(matches!(g.failed_when, Some(GoalCondAst::HasFlag(ref f)) if f == "abort"));

        let toml = crate::compile_goals_to_toml(&goals).expect("compile ok");
        assert!(toml.contains("failed_when"));
        assert!(toml.contains("type = \"hasFlag\""));
        assert!(toml.contains("flag = \"abort\""));
    }

    #[test]
    fn parse_goal_block_missing_required_fields_errors() {
        let src = r#"
goal incomplete-goal {
  name "Incomplete"
  group required
  done when has flag finished
}
"#;
        let err = crate::parse_goals(src).expect_err("missing desc should error");
        match err {
            crate::AstError::Shape(msg) => assert_eq!(msg, "goal missing desc"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn parse_and_compile_set_npc_active() {
        let src = r#"
trigger "test set npc active" when always {
  do set npc active robot true
  do set npc active guard false
}
"#;
        let ast = parse_trigger(src).expect("parse ok");

        // Verify AST contains the correct actions
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::SetNpcActive { npc, active } if npc == "robot" && *active == true))
        );
        assert!(
            ast.actions
                .iter()
                .any(|a| matches!(a, ActionAst::SetNpcActive { npc, active } if npc == "guard" && *active == false))
        );

        // Compile to TOML and verify output
        let toml = compile_trigger_to_toml(&ast).expect("compile ok");

        // Check that the TOML contains the correct structure
        assert!(toml.contains("type = \"setNpcActive\""));
        assert!(toml.contains("npc_sym = \"robot\""));
        assert!(toml.contains("active = true"));
        assert!(toml.contains("npc_sym = \"guard\""));
        assert!(toml.contains("active = false"));
    }

    #[test]
    fn parse_set_npc_active_error_cases() {
        // Test invalid boolean value
        let src_invalid_bool = r#"
trigger "test invalid bool" when always {
  do set npc active robot maybe
}
"#;
        let result = parse_trigger(src_invalid_bool);
        assert!(result.is_err(), "Should fail with invalid boolean value");

        // Test missing boolean value
        let src_missing_bool = r#"
trigger "test missing bool" when always {
  do set npc active robot
}
"#;
        let result = parse_trigger(src_missing_bool);
        assert!(result.is_err(), "Should fail with missing boolean value");

        // Test missing npc name
        let src_missing_npc = r#"
trigger "test missing npc" when always {
  do set npc active true
}
"#;
        let result = parse_trigger(src_missing_npc);
        assert!(result.is_err(), "Should fail with missing npc name");

        // Test completely malformed syntax
        let src_malformed = r#"
trigger "test malformed" when always {
  do set npc active
}
"#;
        let result = parse_trigger(src_malformed);
        assert!(result.is_err(), "Should fail with malformed syntax");
    }

    // TODO: Add NPC golden test once DSL stabilizes
}
