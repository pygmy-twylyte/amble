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
pub use parser::{parse_program, parse_trigger, AstError};

use thiserror::Error;
use toml_edit::{value, Array, ArrayOfTables, Document, InlineTable, Item, Table};

/// A minimal AST for a single trigger.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerAst {
    /// Human-readable trigger name.
    pub name: String,
    /// The event condition that triggers this (e.g., enter room, take item, talk to npc).
    pub event: ConditionAst,
    /// List of conditions (currently only missing-flag).
    pub conditions: Vec<ConditionAst>,
    /// List of actions supported in this minimal version.
    pub actions: Vec<ActionAst>,
    /// If true, the trigger should only fire once.
    pub only_once: bool,
}

/// Minimal condition variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionAst {
    /// Event: trigger has no event; conditions only.
    Always,
    /// Event: player enters a room.
    EnterRoom(String),
    /// Event: player takes an item.
    TakeItem(String),
    /// Event: player talks to an NPC.
    TalkToNpc(String),
    /// Event: player opens an item.
    OpenItem(String),
    /// Event: player leaves a room.
    LeaveRoom(String),
    /// Event: player looks at an item.
    LookAtItem(String),
    /// Event: player uses an item with an ability.
    UseItem { item: String, ability: String },
    /// Event: player gives an item to an NPC.
    GiveToNpc { item: String, npc: String },
    /// Event: player uses one item on another item with an interaction.
    UseItemOnItem { tool: String, target: String, interaction: String },
    /// Event: player performs an interaction on an item (tool-agnostic).
    ActOnItem { target: String, action: String },
    /// Event: player takes an item from an NPC.
    TakeFromNpc { item: String, npc: String },
    /// Event: player inserts an item into a container item.
    InsertItemInto { item: String, container: String },
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
    NpcHasItem { npc: String, item: String },
    NpcInState { npc: String, state: String },
    ContainerHasItem { container: String, item: String },
    Ambient { spinner: String, rooms: Option<Vec<String>> },
    /// Random chance in percent (0-100).
    ChancePercent(f64),
    /// All of the nested conditions must hold.
    All(Vec<ConditionAst>),
    /// Any of the nested conditions may hold (not yet compilable for triggers).
    Any(Vec<ConditionAst>),
}

/// Minimal action variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionAst {
    /// Show a message to the player.
    Show(String),
    /// Add a weighted wedge to a spinner
    AddSpinnerWedge { spinner: String, width: usize, text: String },
    /// Add a simple flag by name.
    AddFlag(String),
    /// Add a sequence flag by name with optional limit (end)
    AddSeqFlag { name: String, end: Option<u8> },
    /// Award points to the player's score.
    AwardPoints(i64),
    /// Remove a flag by name.
    RemoveFlag(String),
    /// Replace an item instance by symbol with another
    ReplaceItem { old_sym: String, new_sym: String },
    /// Replace an item when dropped with another symbol
    ReplaceDropItem { old_sym: String, new_sym: String },
    /// Spawn an item into a room.
    SpawnItemIntoRoom { item: String, room: String },
    /// Despawn an item.
    DespawnItem(String),
    /// Reset a sequence flag to step 0.
    ResetFlag(String),
    /// Advance a sequence flag by one step.
    AdvanceFlag(String),
    /// Set a barred message for an exit between rooms
    SetBarredMessage { exit_from: String, exit_to: String, msg: String },
    /// Reveal an exit in a room in a direction to another room.
    RevealExit { exit_from: String, exit_to: String, direction: String },
    /// Lock or unlock exits and items
    LockExit { from_room: String, direction: String },
    UnlockExit { from_room: String, direction: String },
    LockItem(String),
    UnlockItemAction(String),
    /// Push player to a room.
    PushPlayerTo(String),
    /// NPC gives an item to the player
    GiveItemToPlayer { npc: String, item: String },
    /// Spawns
    SpawnItemInInventory(String),
    SpawnItemCurrentRoom(String),
    SpawnItemInContainer { item: String, container: String },
    /// Set description for an item by symbol.
    SetItemDescription { item: String, text: String },
    NpcSays { npc: String, quote: String },
    NpcSaysRandom { npc: String },
    /// NPC refuses an item with a reason
    NpcRefuseItem { npc: String, reason: String },
    SetNpcState { npc: String, state: String },
    DenyRead(String),
    RestrictItem(String),
    /// Set container state for an item by symbol; omit state to clear
    SetContainerState { item: String, state: Option<String> },
    /// Show a random message from a spinner
    SpinnerMessage { spinner: String },
    /// Schedules without conditions
    ScheduleIn { turns_ahead: usize, actions: Vec<ActionAst>, note: Option<String> },
    ScheduleOn { on_turn: usize, actions: Vec<ActionAst>, note: Option<String> },
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
}

/// Policy to apply when a scheduled condition evaluates to false at fire time.
#[derive(Debug, Clone, PartialEq)]
pub enum OnFalseAst {
    Cancel,
    RetryAfter { turns: usize },
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

        let mut conds = Array::default();
        // event condition first (skip for Always)
        match event {
            ConditionAst::Always => { /* no event condition emitted */ }
            ConditionAst::EnterRoom(room) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("enter"));
                t.insert("room_id", toml_edit::Value::from(room.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::TakeItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("take"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::TalkToNpc(npc) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("talkToNpc"));
                t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::TakeFromNpc { item, npc } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("takeFromNpc"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::InsertItemInto { item, container } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("insert"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("container_id", toml_edit::Value::from(container.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::DropItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("drop"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::UnlockItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("unlock"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::UseItemOnItem { tool, target, interaction } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("useItemOnItem"));
                t.insert("interaction", toml_edit::Value::from(interaction.clone()));
                t.insert("target_id", toml_edit::Value::from(target.clone()));
                t.insert("tool_id", toml_edit::Value::from(tool.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::ActOnItem { target, action } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("actOnItem"));
                t.insert("target_sym", toml_edit::Value::from(target.clone()));
                t.insert("action", toml_edit::Value::from(action.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::OpenItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("open"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::LeaveRoom(room) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("leave"));
                t.insert("room_id", toml_edit::Value::from(room.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::LookAtItem(item) => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("lookAt"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::UseItem { item, ability } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("useItem"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("ability", toml_edit::Value::from(ability.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            ConditionAst::GiveToNpc { item, npc } => {
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("giveToNpc"));
                t.insert("item_id", toml_edit::Value::from(item.clone()));
                t.insert("npc_id", toml_edit::Value::from(npc.clone()));
                conds.push(toml_edit::Value::from(t));
            }
            other => {
                // shouldn't be other types here
                let mut t = InlineTable::new();
                t.insert("type", toml_edit::Value::from("unknown"));
                t.insert("text", toml_edit::Value::from(format!("{:?}", other)));
                conds.push(toml_edit::Value::from(t));
            }
        }

        // Emit flattened simple conditions
        for c in flat_conds {
            match c {
                ConditionAst::Always => { /* ignore */ }
                ConditionAst::MissingFlag(flag) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("missingFlag"));
                    t.insert("flag", toml_edit::Value::from(flag.clone()));
                    conds.push(toml_edit::Value::from(t));
                }
                ConditionAst::HasFlag(flag) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("hasFlag"));
                    t.insert("flag", toml_edit::Value::from(flag.clone()));
                    conds.push(toml_edit::Value::from(t));
                }
                ConditionAst::HasItem(item) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("hasItem"));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    conds.push(toml_edit::Value::from(t));
                }
                ConditionAst::PlayerInRoom(room_id) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("inRoom"));
                    t.insert("room_id", toml_edit::Value::from(room_id.clone()));
                    conds.push(toml_edit::Value::from(t));
                }
                ConditionAst::HasVisited(room) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("hasVisited")); t.insert("room_id", toml_edit::Value::from(room.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::MissingItem(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("missingItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::FlagInProgress(flag) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("flagInProgress")); t.insert("flag", toml_edit::Value::from(flag.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::FlagComplete(flag) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("flagComplete")); t.insert("flag", toml_edit::Value::from(flag.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::WithNpc(npc) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("withNpc")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::NpcHasItem { npc, item } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("npcHasItem")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("item_id", toml_edit::Value::from(item.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::NpcInState { npc, state } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("npcInState")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("state", toml_edit::Value::from(state.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::ContainerHasItem { container, item } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("containerHasItem")); t.insert("container_id", toml_edit::Value::from(container.clone())); t.insert("item_id", toml_edit::Value::from(item.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::Ambient { spinner, rooms } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("ambient")); if let Some(rs)=rooms { let mut arr = Array::default(); for r in rs { arr.push(toml_edit::Value::from(r.clone())); } t.insert("room_ids", toml_edit::Value::from(arr)); } t.insert("spinner", toml_edit::Value::from(spinner.clone())); conds.push(toml_edit::Value::from(t)); }
                ConditionAst::ChancePercent(pct) => {
                    let one_in = if *pct <= 0.0 { f64::INFINITY } else { 100.0 / *pct };
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("chance"));
                    t.insert("one_in", toml_edit::Value::from(one_in));
                    conds.push(toml_edit::Value::from(t));
                }
                ConditionAst::All(_kids) | ConditionAst::Any(_kids) => {
                    // Should not appear after expansion
                }
                ConditionAst::EnterRoom(_) | ConditionAst::TakeItem(_) | ConditionAst::TalkToNpc(_) | ConditionAst::OpenItem(_) | ConditionAst::LeaveRoom(_) | ConditionAst::LookAtItem(_) | ConditionAst::UseItem { .. } | ConditionAst::GiveToNpc { .. } | ConditionAst::UseItemOnItem { .. } | ConditionAst::ActOnItem { .. } | ConditionAst::TakeFromNpc { .. } | ConditionAst::InsertItemInto { .. } | ConditionAst::DropItem(_) | ConditionAst::UnlockItem(_) => {
                    // Event conditions are emitted first separately
                }
            }
        }
        trig["conditions"] = Item::Value(conds.into());

                // actions
        let mut acts = Array::default();
        for a in actions { acts.push(action_to_value(a)); }
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
                }
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
            emit_trigger(&mut aot, &ast.name, &ast.event, &[], &ast.actions, ast.only_once);
        } else {
            for flat in expanded {
                emit_trigger(&mut aot, &ast.name, &ast.event, &flat, &ast.actions, ast.only_once);
            }
        }
    }
    doc["triggers"] = Item::ArrayOfTables(aot);

    Ok(doc)
}

fn action_to_value(a: &ActionAst) -> toml_edit::Value {
    match a {
        ActionAst::Show(text) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("showMessage")); t.insert("text", toml_edit::Value::from(text.clone())); toml_edit::Value::from(t) }
        ActionAst::AddSpinnerWedge { spinner, width, text } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("addSpinnerWedge")); t.insert("spinner", toml_edit::Value::from(spinner.clone())); t.insert("width", toml_edit::Value::from(*width as i64)); t.insert("text", toml_edit::Value::from(text.clone())); toml_edit::Value::from(t) }
        ActionAst::AddFlag(name) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("addFlag")); let mut flag_tbl = InlineTable::new(); flag_tbl.insert("type", toml_edit::Value::from("simple")); flag_tbl.insert("name", toml_edit::Value::from(name.clone())); t.insert("flag", toml_edit::Value::from(flag_tbl)); toml_edit::Value::from(t) }
        ActionAst::AddSeqFlag { name, end } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("addFlag")); let mut flag_tbl = InlineTable::new(); flag_tbl.insert("type", toml_edit::Value::from("sequence")); flag_tbl.insert("name", toml_edit::Value::from(name.clone())); if let Some(e)=end { flag_tbl.insert("end", toml_edit::Value::from(*e as i64)); } t.insert("flag", toml_edit::Value::from(flag_tbl)); toml_edit::Value::from(t) }
        ActionAst::AwardPoints(amount) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("awardPoints")); t.insert("amount", toml_edit::Value::from(*amount as i64)); toml_edit::Value::from(t) }
        ActionAst::RemoveFlag(name) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("removeFlag")); t.insert("flag", toml_edit::Value::from(name.clone())); toml_edit::Value::from(t) }
        ActionAst::ReplaceItem { old_sym, new_sym } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("replaceItem")); t.insert("old_sym", toml_edit::Value::from(old_sym.clone())); t.insert("new_sym", toml_edit::Value::from(new_sym.clone())); toml_edit::Value::from(t) }
        ActionAst::ReplaceDropItem { old_sym, new_sym } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("replaceDropItem")); t.insert("old_sym", toml_edit::Value::from(old_sym.clone())); t.insert("new_sym", toml_edit::Value::from(new_sym.clone())); toml_edit::Value::from(t) }
        ActionAst::ResetFlag(name) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("resetFlag")); t.insert("flag", toml_edit::Value::from(name.clone())); toml_edit::Value::from(t) }
        ActionAst::AdvanceFlag(name) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("advanceFlag")); t.insert("flag", toml_edit::Value::from(name.clone())); toml_edit::Value::from(t) }
        ActionAst::SetBarredMessage { exit_from, exit_to, msg } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("setBarredMessage")); t.insert("exit_from", toml_edit::Value::from(exit_from.clone())); t.insert("exit_to", toml_edit::Value::from(exit_to.clone())); t.insert("msg", toml_edit::Value::from(msg.clone())); toml_edit::Value::from(t) }
        ActionAst::SpawnItemIntoRoom { item, room } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("spawnItemInRoom")); t.insert("item_id", toml_edit::Value::from(item.clone())); t.insert("room_id", toml_edit::Value::from(room.clone())); toml_edit::Value::from(t) }
        ActionAst::SpawnItemInInventory(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("spawnItemInInventory")); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::SpawnItemCurrentRoom(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("spawnItemCurrentRoom")); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::SpawnItemInContainer { item, container } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("spawnItemInContainer")); t.insert("item_id", toml_edit::Value::from(item.clone())); t.insert("container_id", toml_edit::Value::from(container.clone())); toml_edit::Value::from(t) }
        ActionAst::DespawnItem(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("despawnItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::SetItemDescription { item, text } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("setItemDescription")); t.insert("item_sym", toml_edit::Value::from(item.clone())); t.insert("text", toml_edit::Value::from(text.clone())); toml_edit::Value::from(t) }
        ActionAst::RevealExit { exit_from, exit_to, direction } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("revealExit")); t.insert("exit_from", toml_edit::Value::from(exit_from.clone())); t.insert("exit_to", toml_edit::Value::from(exit_to.clone())); t.insert("direction", toml_edit::Value::from(direction.clone())); toml_edit::Value::from(t) }
        ActionAst::LockExit { from_room, direction } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("lockExit")); t.insert("from_room", toml_edit::Value::from(from_room.clone())); t.insert("direction", toml_edit::Value::from(direction.clone())); toml_edit::Value::from(t) }
        ActionAst::UnlockExit { from_room, direction } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("unlockExit")); t.insert("from_room", toml_edit::Value::from(from_room.clone())); t.insert("direction", toml_edit::Value::from(direction.clone())); toml_edit::Value::from(t) }
        ActionAst::LockItem(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("lockItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::UnlockItemAction(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("unlockItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::PushPlayerTo(room) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("pushPlayerTo")); t.insert("room_id", toml_edit::Value::from(room.clone())); toml_edit::Value::from(t) }
        ActionAst::GiveItemToPlayer { npc, item } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("giveItemToPlayer")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::ScheduleInIf { turns_ahead, condition, on_false, actions, note } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("scheduleInIf"));
            t.insert("turns_ahead", toml_edit::Value::from(*turns_ahead as i64));
            t.insert("condition", event_condition_value(condition));
            if let Some(p) = on_false { t.insert("on_false", on_false_value(p)); } else { t.insert("on_false", on_false_value(&OnFalseAst::Cancel)); }
            let mut arr = Array::default(); for ia in actions { arr.push(action_to_value(ia)); }
            t.insert("actions", toml_edit::Value::from(arr));
            if let Some(n) = note { t.insert("note", toml_edit::Value::from(n.clone())); }
            toml_edit::Value::from(t)
        }
        ActionAst::ScheduleOnIf { on_turn, condition, on_false, actions, note } => {
            let mut t = InlineTable::new();
            t.insert("type", toml_edit::Value::from("scheduleOnIf"));
            t.insert("on_turn", toml_edit::Value::from(*on_turn as i64));
            t.insert("condition", event_condition_value(condition));
            if let Some(p) = on_false { t.insert("on_false", on_false_value(p)); } else { t.insert("on_false", on_false_value(&OnFalseAst::Cancel)); }
            let mut arr = Array::default(); for ia in actions { arr.push(action_to_value(ia)); }
            t.insert("actions", toml_edit::Value::from(arr));
            if let Some(n) = note { t.insert("note", toml_edit::Value::from(n.clone())); }
            toml_edit::Value::from(t)
        }
        ActionAst::ScheduleIn { turns_ahead, actions, note } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("scheduleIn")); t.insert("turns_ahead", toml_edit::Value::from(*turns_ahead as i64)); let mut arr = Array::default(); for ia in actions { arr.push(action_to_value(ia)); } t.insert("actions", toml_edit::Value::from(arr)); if let Some(n)=note { t.insert("note", toml_edit::Value::from(n.clone())); } toml_edit::Value::from(t) }
        ActionAst::ScheduleOn { on_turn, actions, note } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("scheduleOn")); t.insert("on_turn", toml_edit::Value::from(*on_turn as i64)); let mut arr = Array::default(); for ia in actions { arr.push(action_to_value(ia)); } t.insert("actions", toml_edit::Value::from(arr)); if let Some(n)=note { t.insert("note", toml_edit::Value::from(n.clone())); } toml_edit::Value::from(t) }
        ActionAst::NpcSays { npc, quote } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("npcSays")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("quote", toml_edit::Value::from(quote.clone())); toml_edit::Value::from(t) }
        ActionAst::NpcSaysRandom { npc } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("npcSaysRandom")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); toml_edit::Value::from(t) }
        ActionAst::NpcRefuseItem { npc, reason } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("npcRefuseItem")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("reason", toml_edit::Value::from(reason.clone())); toml_edit::Value::from(t) }
        ActionAst::SetNpcState { npc, state } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("setNpcState")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("state", toml_edit::Value::from(state.clone())); toml_edit::Value::from(t) }
        ActionAst::DenyRead(reason) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("denyRead")); t.insert("reason", toml_edit::Value::from(reason.clone())); toml_edit::Value::from(t) }
        ActionAst::RestrictItem(item) => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("restrictItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); toml_edit::Value::from(t) }
        ActionAst::SetContainerState { item, state } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("setContainerState")); t.insert("item_sym", toml_edit::Value::from(item.clone())); if let Some(s) = state { t.insert("state", toml_edit::Value::from(s.clone())); } toml_edit::Value::from(t) }
        ActionAst::SpinnerMessage { spinner } => { let mut t = InlineTable::new(); t.insert("type", toml_edit::Value::from("spinnerMessage")); t.insert("spinner", toml_edit::Value::from(spinner.clone())); toml_edit::Value::from(t) }
    }
}

fn on_false_value(p: &OnFalseAst) -> toml_edit::Value {
    let mut t = InlineTable::new();
    match p {
        OnFalseAst::Cancel => { t.insert("type", toml_edit::Value::from("cancel")); }
        OnFalseAst::RetryNextTurn => { t.insert("type", toml_edit::Value::from("retryNextTurn")); }
        OnFalseAst::RetryAfter { turns } => { t.insert("type", toml_edit::Value::from("retryAfter")); t.insert("turns", toml_edit::Value::from(*turns as i64)); }
    }
    toml_edit::Value::from(t)
}

fn event_condition_value(c: &ConditionAst) -> toml_edit::Value {
    match c {
        ConditionAst::All(kids) => { let mut t = InlineTable::new(); let mut arr = Array::default(); for k in kids { arr.push(event_condition_value(k)); } t.insert("all", toml_edit::Value::from(arr)); toml_edit::Value::from(t) }
        ConditionAst::Any(kids) => { let mut t = InlineTable::new(); let mut arr = Array::default(); for k in kids { arr.push(event_condition_value(k)); } t.insert("any", toml_edit::Value::from(arr)); toml_edit::Value::from(t) }
        leaf => toml_edit::Value::from(leaf_condition_inline(leaf)),
    }
}

fn leaf_condition_inline(c: &ConditionAst) -> InlineTable {
    let mut t = InlineTable::new();
    match c {
        ConditionAst::Always => { /* not a leaf; shouldn't be emitted */ }
        ConditionAst::MissingFlag(flag) => { t.insert("type", toml_edit::Value::from("missingFlag")); t.insert("flag", toml_edit::Value::from(flag.clone())); }
        ConditionAst::HasFlag(flag) => { t.insert("type", toml_edit::Value::from("hasFlag")); t.insert("flag", toml_edit::Value::from(flag.clone())); }
        ConditionAst::HasItem(item) => { t.insert("type", toml_edit::Value::from("hasItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::PlayerInRoom(room) => { t.insert("type", toml_edit::Value::from("inRoom")); t.insert("room_id", toml_edit::Value::from(room.clone())); }
        ConditionAst::ChancePercent(pct) => {
            let one_in = if *pct <= 0.0 { f64::INFINITY } else { 100.0 / *pct };
            t.insert("type", toml_edit::Value::from("chance"));
            t.insert("one_in", toml_edit::Value::from(one_in));
        }
        ConditionAst::HasVisited(room) => { t.insert("type", toml_edit::Value::from("hasVisited")); t.insert("room_id", toml_edit::Value::from(room.clone())); }
        ConditionAst::MissingItem(item) => { t.insert("type", toml_edit::Value::from("missingItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::FlagInProgress(flag) => { t.insert("type", toml_edit::Value::from("flagInProgress")); t.insert("flag", toml_edit::Value::from(flag.clone())); }
        ConditionAst::FlagComplete(flag) => { t.insert("type", toml_edit::Value::from("flagComplete")); t.insert("flag", toml_edit::Value::from(flag.clone())); }
        ConditionAst::WithNpc(npc) => { t.insert("type", toml_edit::Value::from("withNpc")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); }
        ConditionAst::NpcHasItem { npc, item } => { t.insert("type", toml_edit::Value::from("npcHasItem")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::NpcInState { npc, state } => { t.insert("type", toml_edit::Value::from("npcInState")); t.insert("npc_id", toml_edit::Value::from(npc.clone())); t.insert("state", toml_edit::Value::from(state.clone())); }
        ConditionAst::ContainerHasItem { container, item } => { t.insert("type", toml_edit::Value::from("containerHasItem")); t.insert("container_id", toml_edit::Value::from(container.clone())); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::Ambient { spinner, rooms } => { t.insert("type", toml_edit::Value::from("ambient")); if let Some(rs)=rooms { let mut arr = Array::default(); for r in rs { arr.push(toml_edit::Value::from(r.clone())); } t.insert("room_ids", toml_edit::Value::from(arr)); } t.insert("spinner", toml_edit::Value::from(spinner.clone())); }
        ConditionAst::EnterRoom(room) => { t.insert("type", toml_edit::Value::from("enter")); t.insert("room_id", toml_edit::Value::from(room.clone())); }
        ConditionAst::LeaveRoom(room) => { t.insert("type", toml_edit::Value::from("leave")); t.insert("room_id", toml_edit::Value::from(room.clone())); }
        ConditionAst::LookAtItem(item) => { t.insert("type", toml_edit::Value::from("lookAt")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::OpenItem(item) => { t.insert("type", toml_edit::Value::from("open")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::UseItem { item, ability } => { t.insert("type", toml_edit::Value::from("useItem")); t.insert("item_id", toml_edit::Value::from(item.clone())); t.insert("ability", toml_edit::Value::from(ability.clone())); }
        ConditionAst::GiveToNpc { item, npc } => { t.insert("type", toml_edit::Value::from("giveToNpc")); t.insert("item_id", toml_edit::Value::from(item.clone())); t.insert("npc_id", toml_edit::Value::from(npc.clone())); }
        ConditionAst::UseItemOnItem { tool, target, interaction } => { t.insert("type", toml_edit::Value::from("useItemOnItem")); t.insert("interaction", toml_edit::Value::from(interaction.clone())); t.insert("target_id", toml_edit::Value::from(target.clone())); t.insert("tool_id", toml_edit::Value::from(tool.clone())); }
        ConditionAst::ActOnItem { target, action } => { t.insert("type", toml_edit::Value::from("actOnItem")); t.insert("target_sym", toml_edit::Value::from(target.clone())); t.insert("action", toml_edit::Value::from(action.clone())); }
        ConditionAst::TakeItem(item) => { t.insert("type", toml_edit::Value::from("take")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::DropItem(item) => { t.insert("type", toml_edit::Value::from("drop")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::UnlockItem(item) => { t.insert("type", toml_edit::Value::from("unlock")); t.insert("item_id", toml_edit::Value::from(item.clone())); }
        ConditionAst::TakeFromNpc { item, npc } => { t.insert("type", toml_edit::Value::from("takeFromNpc")); t.insert("item_id", toml_edit::Value::from(item.clone())); t.insert("npc_id", toml_edit::Value::from(npc.clone())); }
        ConditionAst::InsertItemInto { item, container } => { t.insert("type", toml_edit::Value::from("insert")); t.insert("item_id", toml_edit::Value::from(item.clone())); t.insert("container_id", toml_edit::Value::from(container.clone())); }
        ConditionAst::TalkToNpc(_) => unreachable!(),
        ConditionAst::All(_) | ConditionAst::Any(_) => unreachable!(),
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(ast.conditions, vec![ConditionAst::MissingFlag("visited:high-ridge".into())]);
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
            }
            other => panic!("unexpected condition: {:?}", other),
        }
        // actions include spawn, show, add, remove, despawn, award
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::SpawnItemIntoRoom{ item, room } if item == "cake" && room == "pantry")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::Show(s) if s == "A cake materializes out of nowhere.")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::AddFlag(s) if s == "cake-spawned")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::RemoveFlag(s) if s == "got-key")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::DespawnItem(s) if s == "old-cake")));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::AwardPoints(n) if *n == 2)));

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
        assert!(matches!(asts[0].event, ConditionAst::UseItem { item: ref i, ability: ref a } if i == "portal_gun" && a == "turnOn"));
        assert!(matches!(asts[1].event, ConditionAst::GiveToNpc { item: ref i, npc: ref n } if i == "printer_paper" && n == "receptionist"));
        let toml = compile_triggers_to_toml(&asts).expect("compile ok");
        assert!(toml.contains("type = \"useItem\""));
        assert!(toml.contains("ability = \"turnOn\""));
        assert!(toml.contains("type = \"giveToNpc\""));
        assert!(toml.contains("npc_id = \"receptionist\""));
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
        assert!(matches!(asts[1].event, ConditionAst::InsertItemInto { item: ref i, container: ref c } if i == "battery" && c == "portal_gun"));
        assert!(matches!(asts[2].event, ConditionAst::UnlockItem(ref s) if s == "locker"));
        assert!(matches!(asts[3].event, ConditionAst::TakeFromNpc { item: ref i, npc: ref n } if i == "invitation" && n == "receptionist"));
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
    fn parse_without_if_block() {
        let src = r#"
trigger "burn fallen tree" when act burn on item fallen_tree {
  do show "The tree crackles and burns."
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.name, "burn fallen tree");
        assert!(matches!(ast.event, ConditionAst::ActOnItem { ref target, ref action } if target == "fallen_tree" && action == "burn"));
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
            }
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
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::NpcRefuseItem { npc, reason } if npc == "emh" && reason.starts_with("That's"))));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::SetContainerState { item, state } if item == "evidence_locker_open" && state.as_deref() == Some("locked"))));
        assert!(ast.actions.iter().any(|a| matches!(a, ActionAst::SpinnerMessage { spinner } if spinner == "ambientInterior")));

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
        // seq flag assertions
        assert!(toml.contains("type = \"addFlag\""));
        assert!(toml.contains("type = \"sequence\""));
        assert!(toml.contains("name = \"quest\""));
        assert!(toml.contains("end = 3"));
    }

}
