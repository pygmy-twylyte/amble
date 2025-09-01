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
}

/// Minimal condition variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionAst {
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
    /// Add a simple flag by name.
    AddFlag(String),
    /// Award points to the player's score.
    AwardPoints(i64),
    /// Remove a flag by name.
    RemoveFlag(String),
    /// Spawn an item into a room.
    SpawnItemIntoRoom { item: String, room: String },
    /// Despawn an item.
    DespawnItem(String),
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
    ) {
        let mut trig = Table::new();
        trig["name"] = value(name.to_string());

        let mut conds = Array::default();
        // event condition first
        match event {
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
        for a in actions {
            match a {
                ActionAst::Show(text) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("showMessage"));
                    t.insert("text", toml_edit::Value::from(text.clone()));
                    acts.push(toml_edit::Value::from(t));
                }
                ActionAst::AddFlag(name) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("addFlag"));
                    let mut flag_tbl = InlineTable::new();
                    flag_tbl.insert("type", toml_edit::Value::from("simple"));
                    flag_tbl.insert("name", toml_edit::Value::from(name.clone()));
                    t.insert("flag", toml_edit::Value::from(flag_tbl));
                    acts.push(toml_edit::Value::from(t));
                }
                ActionAst::AwardPoints(amount) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("awardPoints"));
                    t.insert("amount", toml_edit::Value::from(*amount as i64));
                    acts.push(toml_edit::Value::from(t));
                }
                ActionAst::RemoveFlag(name) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("removeFlag"));
                    t.insert("flag", toml_edit::Value::from(name.clone()));
                    acts.push(toml_edit::Value::from(t));
                }
                ActionAst::SpawnItemIntoRoom { item, room } => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("spawnItemInRoom"));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    t.insert("room_id", toml_edit::Value::from(room.clone()));
                    acts.push(toml_edit::Value::from(t));
                }
                ActionAst::DespawnItem(item) => {
                    let mut t = InlineTable::new();
                    t.insert("type", toml_edit::Value::from("despawnItem"));
                    t.insert("item_id", toml_edit::Value::from(item.clone()));
                    acts.push(toml_edit::Value::from(t));
                }
            }
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
            emit_trigger(&mut aot, &ast.name, &ast.event, &[], &ast.actions);
        } else {
            for flat in expanded {
                emit_trigger(&mut aot, &ast.name, &ast.event, &flat, &ast.actions);
            }
        }
    }
    doc["triggers"] = Item::ArrayOfTables(aot);

    Ok(doc)
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

}
