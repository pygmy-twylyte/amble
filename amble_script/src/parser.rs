//! Parser and AST builders for the Amble DSL.
//!
//! Wraps the Pest-generated grammar with helpers that construct the
//! compiler's abstract syntax tree for triggers, rooms, items, and more.

use pest::Parser;
use pest_derive::Parser as PestParser;

use crate::{
    ActionAst, ConditionAst, ConsumableAst, ConsumableWhenAst, ContainerStateAst, GoalAst, GoalCondAst, GoalGroupAst,
    IngestModeAst, ItemAbilityAst, ItemAst, ItemLocationAst, ItemPatchAst, NpcAst, NpcDialoguePatchAst, NpcMovementAst,
    NpcMovementPatchAst, NpcMovementTypeAst, NpcPatchAst, NpcStateValue, NpcTimingPatchAst, OnFalseAst, RoomAst,
    RoomExitPatchAst, RoomPatchAst, SpinnerAst, SpinnerWedgeAst, TriggerAst,
};
use std::collections::HashMap;

#[derive(PestParser)]
#[grammar = "src/grammar.pest"]
struct DslParser;

/// Errors that can happen when parsing the DSL input.
#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("parse error: {0}")]
    Pest(String),
    #[error("unexpected grammar shape: {0}")]
    Shape(&'static str),
    #[error("unexpected grammar shape: {msg} ({context})")]
    ShapeAt { msg: &'static str, context: String },
}

/// Parse a single trigger source string; returns the first trigger found.
///
/// # Errors
/// Returns an error if the source cannot be parsed or if no trigger is found.
pub fn parse_trigger(source: &str) -> Result<TriggerAst, AstError> {
    let v = parse_program(source)?;
    v.into_iter().next().ok_or(AstError::Shape("no trigger found"))
}

/// Parse multiple triggers from a full source file (triggers only view).
///
/// # Errors
/// Returns an error if the source cannot be parsed.
pub fn parse_program(source: &str) -> Result<Vec<TriggerAst>, AstError> {
    let (triggers, ..) = parse_program_full(source)?;
    Ok(triggers)
}

/// Parse a full program returning triggers, rooms, items, and spinners.
///
/// # Errors
/// Returns an error when parsing fails or when the grammar encounters an
/// unexpected shape.
pub fn parse_program_full(source: &str) -> Result<ProgramAstBundle, AstError> {
    let mut pairs = DslParser::parse(Rule::program, source).map_err(|e| AstError::Pest(e.to_string()))?;
    let pair = pairs.next().ok_or(AstError::Shape("expected program"))?;
    let smap = SourceMap::new(source);
    let mut sets: HashMap<String, Vec<String>> = HashMap::new();
    let mut trigger_pairs = Vec::new();
    let mut room_pairs = Vec::new();
    let mut item_pairs = Vec::new();
    let mut spinner_pairs = Vec::new();
    let mut npc_pairs = Vec::new();
    let mut goal_pairs = Vec::new();
    for item in pair.clone().into_inner() {
        match item.as_rule() {
            Rule::set_decl => {
                let mut it = item.into_inner();
                let name = it.next().expect("set name").as_str().to_string();
                let list_pair = it.next().expect("set list");
                let mut vals = Vec::new();
                for p in list_pair.into_inner() {
                    if p.as_rule() == Rule::ident {
                        vals.push(p.as_str().to_string());
                    }
                }
                sets.insert(name, vals);
            },
            Rule::trigger => {
                trigger_pairs.push(item);
            },
            Rule::room_def => {
                room_pairs.push(item);
            },
            Rule::item_def => {
                item_pairs.push(item);
            },
            Rule::spinner_def => {
                spinner_pairs.push(item);
            },
            Rule::npc_def => {
                npc_pairs.push(item);
            },
            Rule::goal_def => {
                goal_pairs.push(item);
            },
            _ => {},
        }
    }
    let mut out = Vec::new();
    for trig in trigger_pairs {
        let mut ts = parse_trigger_pair(trig, source, &smap, &sets)?;
        out.append(&mut ts);
    }
    let mut rooms = Vec::new();
    for rp in room_pairs {
        let r = parse_room_pair(rp, source)?;
        rooms.push(r);
    }
    let mut items = Vec::new();
    for ip in item_pairs {
        let it = parse_item_pair(ip, source)?;
        items.push(it);
    }
    let mut spinners = Vec::new();
    for sp in spinner_pairs {
        let s = parse_spinner_pair(sp, source)?;
        spinners.push(s);
    }
    let mut npcs = Vec::new();
    for np in npc_pairs {
        let n = parse_npc_pair(np, source)?;
        npcs.push(n);
    }
    let mut goals = Vec::new();
    for gp in goal_pairs {
        let g = parse_goal_pair(gp, source)?;
        goals.push(g);
    }
    Ok((out, rooms, items, spinners, npcs, goals))
}

fn parse_trigger_pair(
    trig: pest::iterators::Pair<Rule>,
    source: &str,
    smap: &SourceMap,
    sets: &HashMap<String, Vec<String>>,
) -> Result<Vec<TriggerAst>, AstError> {
    let src_line = trig.as_span().start_pos().line_col().0;
    let mut it = trig.into_inner();

    // trigger -> "trigger" ~ string ~ (only once|note)* ~ "when" ~ when_cond ~ block
    let q = it.next().ok_or(AstError::Shape("expected trigger name"))?;
    if q.as_rule() != Rule::string {
        return Err(AstError::Shape("expected string trigger name"));
    }
    let name = unquote(q.as_str());

    // optional modifiers: only once and/or note in any order
    let mut only_once = false;
    let mut trig_note: Option<String> = None;
    let mut next_pair = it.next().ok_or(AstError::Shape("expected when/only once/note"))?;
    loop {
        match next_pair.as_rule() {
            Rule::only_once_kw => {
                only_once = true;
            },
            Rule::note_kw => {
                let mut inner = next_pair.into_inner();
                let s = inner.next().ok_or(AstError::Shape("missing note string"))?;
                trig_note = Some(unquote(s.as_str()));
            },
            _ => break,
        }
        next_pair = it.next().ok_or(AstError::Shape("expected when or more modifiers"))?;
    }
    let mut when = next_pair;
    if when.as_rule() == Rule::when_cond {
        when = when.into_inner().next().ok_or(AstError::Shape("empty when_cond"))?;
    }
    let event = match when.as_rule() {
        Rule::always_event => ConditionAst::Always,
        Rule::enter_room => {
            let mut i = when.into_inner();
            let ident = i
                .next()
                .ok_or(AstError::Shape("enter room ident"))?
                .as_str()
                .to_string();
            ConditionAst::EnterRoom(ident)
        },
        Rule::take_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("take item ident"))?.as_str().to_string();
            ConditionAst::TakeItem(ident)
        },
        Rule::touch_item => {
            let mut i = when.into_inner();
            let ident = i
                .next()
                .ok_or(AstError::Shape("touch item ident"))?
                .as_str()
                .to_string();
            ConditionAst::TouchItem(ident)
        },
        Rule::talk_to_npc => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("talk npc ident"))?.as_str().to_string();
            ConditionAst::TalkToNpc(ident)
        },
        Rule::open_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("open item ident"))?.as_str().to_string();
            ConditionAst::OpenItem(ident)
        },
        Rule::leave_room => {
            let mut i = when.into_inner();
            let ident = i
                .next()
                .ok_or(AstError::Shape("leave room ident"))?
                .as_str()
                .to_string();
            ConditionAst::LeaveRoom(ident)
        },
        Rule::look_at_item => {
            let mut i = when.into_inner();
            let ident = i
                .next()
                .ok_or(AstError::Shape("look at item ident"))?
                .as_str()
                .to_string();
            ConditionAst::LookAtItem(ident)
        },
        Rule::use_item => {
            let mut i = when.into_inner();
            let item = i.next().ok_or(AstError::Shape("use item ident"))?.as_str().to_string();
            let ability = i
                .next()
                .ok_or(AstError::Shape("use item ability"))?
                .as_str()
                .to_string();
            ConditionAst::UseItem { item, ability }
        },
        Rule::give_to_npc => {
            let mut i = when.into_inner();
            let item = i.next().ok_or(AstError::Shape("give item ident"))?.as_str().to_string();
            let npc = i
                .next()
                .ok_or(AstError::Shape("give to npc ident"))?
                .as_str()
                .to_string();
            ConditionAst::GiveToNpc { item, npc }
        },
        Rule::use_item_on_item => {
            let mut i = when.into_inner();
            let tool = i.next().ok_or(AstError::Shape("use tool ident"))?.as_str().to_string();
            let target = i
                .next()
                .ok_or(AstError::Shape("use target ident"))?
                .as_str()
                .to_string();
            let interaction = i
                .next()
                .ok_or(AstError::Shape("use interaction ident"))?
                .as_str()
                .to_string();
            ConditionAst::UseItemOnItem {
                tool,
                target,
                interaction,
            }
        },
        Rule::ingest_item => {
            let mut i = when.into_inner();
            let mode_pair = i.next().ok_or(AstError::Shape("ingest mode"))?;
            let mode = match mode_pair.as_str() {
                "eat" => IngestModeAst::Eat,
                "drink" => IngestModeAst::Drink,
                "inhale" => IngestModeAst::Inhale,
                other => {
                    return Err(AstError::ShapeAt {
                        msg: "unsupported ingest mode",
                        context: other.to_string(),
                    });
                },
            };
            let item = i
                .next()
                .ok_or(AstError::Shape("ingest item ident"))?
                .as_str()
                .to_string();
            ConditionAst::Ingest { item, mode }
        },
        Rule::act_on_item => {
            let mut i = when.into_inner();
            let action = i
                .next()
                .ok_or(AstError::Shape("act interaction ident"))?
                .as_str()
                .to_string();
            let target = i
                .next()
                .ok_or(AstError::Shape("act target ident"))?
                .as_str()
                .to_string();
            ConditionAst::ActOnItem { target, action }
        },
        Rule::take_from_npc => {
            let mut i = when.into_inner();
            let item = i
                .next()
                .ok_or(AstError::Shape("take-from item ident"))?
                .as_str()
                .to_string();
            let npc = i
                .next()
                .ok_or(AstError::Shape("take-from npc ident"))?
                .as_str()
                .to_string();
            ConditionAst::TakeFromNpc { item, npc }
        },
        Rule::insert_item_into => {
            let mut i = when.into_inner();
            let item = i
                .next()
                .ok_or(AstError::Shape("insert item ident"))?
                .as_str()
                .to_string();
            let container = i
                .next()
                .ok_or(AstError::Shape("insert into container ident"))?
                .as_str()
                .to_string();
            ConditionAst::InsertItemInto { item, container }
        },
        Rule::drop_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("drop item ident"))?.as_str().to_string();
            ConditionAst::DropItem(ident)
        },
        Rule::unlock_item => {
            let mut i = when.into_inner();
            let ident = i
                .next()
                .ok_or(AstError::Shape("unlock item ident"))?
                .as_str()
                .to_string();
            ConditionAst::UnlockItem(ident)
        },
        _ => return Err(AstError::Shape("unknown when condition")),
    };

    let block = it.next().ok_or(AstError::Shape("expected block"))?;
    if block.as_rule() != Rule::block {
        return Err(AstError::Shape("expected block"));
    }

    // Parse the trigger body and lower into multiple TriggerAst entries:
    // - Each top-level `if { ... }` becomes its own trigger with those actions.
    // - Top-level `do ...` lines (not inside any if) become an unconditional trigger (if any).
    let inner = extract_body(block.as_str())?;
    let mut unconditional_actions: Vec<ActionAst> = Vec::new();
    let mut lowered: Vec<TriggerAst> = Vec::new();
    let bytes = inner.as_bytes();
    let mut i = 0usize;
    while i < inner.len() {
        // Skip whitespace
        while i < inner.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= inner.len() {
            break;
        }
        // Skip comments
        if bytes[i] as char == '#' {
            while i < inner.len() && (bytes[i] as char) != '\n' {
                i += 1;
            }
            continue;
        }
        // If-block
        if inner[i..].starts_with("if ") {
            let if_pos = i;
            // Find opening brace
            let rest = &inner[if_pos + 3..];
            let brace_rel = rest.find('{').ok_or(AstError::Shape("missing '{' after if"))?;
            let cond_text = &rest[..brace_rel].trim();
            let cond = match parse_condition_text(cond_text, sets) {
                Ok(c) => c,
                Err(AstError::Shape(m)) => {
                    let base_offset = str_offset(source, inner);
                    let cond_abs = base_offset + (cond_text.as_ptr() as usize - inner.as_ptr() as usize);
                    let (line, col) = smap.line_col(cond_abs);
                    let snippet = smap.line_snippet(line);
                    return Err(AstError::ShapeAt {
                        msg: m,
                        context: format!(
                            "line {line}, col {col}: {snippet}\n{}^",
                            " ".repeat(col.saturating_sub(1))
                        ),
                    });
                },
                Err(e) => return Err(e),
            };
            // Extract the block body after this '{' balancing braces
            let block_after = &rest[brace_rel..]; // starts with '{'
            let body = extract_body(block_after)?;
            let actions = parse_actions_from_body(body, source, smap, sets)?;
            lowered.push(TriggerAst {
                name: name.clone(),
                note: None,
                src_line,
                event: event.clone(),
                conditions: vec![cond],
                actions,
                only_once,
            });
            // Advance i to after the block we just consumed
            let consumed = brace_rel + 1 + body.len() + 1; // '{' + body + '}'
            i = if_pos + 3 + consumed;
            continue;
        }
        let remainder = &inner[i..];
        match parse_modify_item_action(remainder) {
            Ok((action, used)) => {
                unconditional_actions.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a modify item action")) => {},
            Err(AstError::Shape(m)) => {
                let base = str_offset(source, inner);
                let abs = base + i;
                let (line_no, col) = smap.line_col(abs);
                let snippet = smap.line_snippet(line_no);
                return Err(AstError::ShapeAt {
                    msg: m,
                    context: format!(
                        "line {line_no}, col {col}: {snippet}\n{}^",
                        " ".repeat(col.saturating_sub(1))
                    ),
                });
            },
            Err(e) => return Err(e),
        }
        match parse_modify_room_action(remainder) {
            Ok((action, used)) => {
                unconditional_actions.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a modify room action")) => {},
            Err(AstError::Shape(m)) => {
                let base = str_offset(source, inner);
                let abs = base + i;
                let (line_no, col) = smap.line_col(abs);
                let snippet = smap.line_snippet(line_no);
                return Err(AstError::ShapeAt {
                    msg: m,
                    context: format!(
                        "line {line_no}, col {col}: {snippet}\n{}^",
                        " ".repeat(col.saturating_sub(1))
                    ),
                });
            },
            Err(e) => return Err(e),
        }

        match parse_modify_npc_action(remainder) {
            Ok((action, used)) => {
                unconditional_actions.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a modify npc action")) => {},
            Err(AstError::Shape(m)) => {
                let base = str_offset(source, inner);
                let abs = base + i;
                let (line_no, col) = smap.line_col(abs);
                let snippet = smap.line_snippet(line_no);
                return Err(AstError::ShapeAt {
                    msg: m,
                    context: format!(
                        "line {line_no}, col {col}: {snippet}\n{}^",
                        " ".repeat(col.saturating_sub(1))
                    ),
                });
            },
            Err(e) => return Err(e),
        }
        // Top-level do schedule ... or do ... line
        if remainder.starts_with("do schedule in ") || remainder.starts_with("do schedule on ") {
            let (action, used) = parse_schedule_action(remainder, source, smap, sets)?;
            unconditional_actions.push(action);
            i += used;
            continue;
        }
        if remainder.starts_with("do ") {
            // Consume a single line
            let mut j = i;
            while j < inner.len() && (bytes[j] as char) != '\n' {
                j += 1;
            }
            let line = inner[i..j].trim_end();
            match parse_action_from_str(line) {
                Ok(a) => unconditional_actions.push(a),
                Err(AstError::Shape(m)) => {
                    let base = str_offset(source, inner);
                    let abs = base + i;
                    let (line_no, col) = smap.line_col(abs);
                    let snippet = smap.line_snippet(line_no);
                    return Err(AstError::ShapeAt {
                        msg: m,
                        context: format!(
                            "line {line_no}, col {col}: {snippet}\n{}^",
                            " ".repeat(col.saturating_sub(1))
                        ),
                    });
                },
                Err(e) => return Err(e),
            }
            i = j;
            continue;
        }
        // Unknown token on this line, skip to newline
        while i < inner.len() && (bytes[i] as char) != '\n' {
            i += 1;
        }
    }
    if !unconditional_actions.is_empty() {
        lowered.push(TriggerAst {
            name,
            note: trig_note.clone(),
            src_line,
            event,
            conditions: Vec::new(),
            actions: unconditional_actions,
            only_once,
        });
    }
    // Inject note into previously lowered triggers
    for t in &mut lowered {
        if t.note.is_none() {
            t.note = trig_note.clone();
        }
    }
    Ok(lowered)
}

fn parse_room_pair(room: pest::iterators::Pair<Rule>, _source: &str) -> Result<RoomAst, AstError> {
    // room_def = "room" ~ ident ~ room_block
    let (src_line, _src_col) = room.as_span().start_pos().line_col();
    let mut it = room.into_inner();
    // capture source line from the outer pair's span; .line_col() is 1-based
    // Note: this is the start of the room keyword; good enough for a reference
    let id = it
        .next()
        .ok_or(AstError::Shape("expected room ident"))?
        .as_str()
        .to_string();
    let block = it.next().ok_or(AstError::Shape("expected room block"))?;
    if block.as_rule() != Rule::room_block {
        return Err(AstError::Shape("expected room block"));
    }
    let mut name: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut visited: Option<bool> = None;
    let mut exits: Vec<(String, crate::ExitAst)> = Vec::new();
    let mut overlays: Vec<crate::OverlayAst> = Vec::new();
    for stmt in block.into_inner() {
        // room_block yields Rule::room_stmt nodes; unwrap to the concrete inner rule
        let inner_stmt = {
            let mut it = stmt.clone().into_inner();
            if let Some(p) = it.next() { p } else { stmt.clone() }
        };
        match inner_stmt.as_rule() {
            Rule::room_name => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing room name string"))?;
                name = Some(unquote(s.as_str()));
            },
            Rule::room_desc => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing room desc string"))?;
                desc = Some(unquote(s.as_str()));
            },
            Rule::room_visited => {
                let tok = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing visited token"))?;
                let val = match tok.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(AstError::Shape("visited must be true or false")),
                };
                visited = Some(val);
            },
            Rule::exit_stmt => {
                let mut it = inner_stmt.into_inner();
                let dir_tok = it.next().ok_or(AstError::Shape("exit direction"))?;
                let dir = if dir_tok.as_rule() == Rule::string {
                    unquote(dir_tok.as_str())
                } else {
                    dir_tok.as_str().to_string()
                };
                let to = it
                    .next()
                    .ok_or(AstError::Shape("exit destination"))?
                    .as_str()
                    .to_string();
                // Defaults
                let mut hidden = false;
                let mut locked = false;
                let mut barred_message: Option<String> = None;
                let mut required_items: Vec<String> = Vec::new();
                let mut required_flags: Vec<String> = Vec::new();
                if let Some(next) = it.next() {
                    if next.as_rule() == Rule::exit_opts {
                        for opt in next.into_inner() {
                            // Simplest detection by textual head, then use children for values
                            let opt_text = opt.as_str().trim();
                            if opt_text == "hidden" {
                                hidden = true;
                                continue;
                            }
                            if opt_text == "locked" {
                                locked = true;
                                continue;
                            }

                            // pull children
                            let children: Vec<_> = opt.clone().into_inner().collect();
                            // barred <string>
                            if let Some(s) = children.iter().find(|p| p.as_rule() == Rule::string) {
                                barred_message = Some(unquote(s.as_str()));
                                continue;
                            }
                            // required_items(...): list of idents only
                            if children.iter().all(|p| p.as_rule() == Rule::ident)
                                && opt_text.starts_with("required_items")
                            {
                                for idp in children {
                                    required_items.push(idp.as_str().to_string());
                                }
                                continue;
                            }
                            // required_flags(...): list of idents or flag_req; we normalize to base name
                            if opt_text.starts_with("required_flags") {
                                for frp in opt.into_inner() {
                                    match frp.as_rule() {
                                        Rule::ident => {
                                            required_flags.push(frp.as_str().to_string());
                                        },
                                        Rule::flag_req => {
                                            // Extract ident child and keep only base name (ignore step/end since equality is by name)
                                            let mut itf = frp.into_inner();
                                            let ident =
                                                itf.next().ok_or(AstError::Shape("flag ident"))?.as_str().to_string();
                                            let base = ident.split('#').next().unwrap_or(&ident).to_string();
                                            required_flags.push(base);
                                        },
                                        _ => {},
                                    }
                                }
                                continue;
                            }
                        }
                    }
                }
                exits.push((
                    dir,
                    crate::ExitAst {
                        to,
                        hidden,
                        locked,
                        barred_message,
                        required_flags,
                        required_items,
                    },
                ));
            },
            Rule::overlay_stmt => {
                // overlay if <cond_list> { text "..." }
                let mut it = inner_stmt.into_inner();
                // First group: overlay_cond_list
                let conds_pair = it.next().ok_or(AstError::Shape("overlay cond list"))?;
                let mut conds = Vec::new();
                for cp in conds_pair.into_inner() {
                    if cp.as_rule() != Rule::overlay_cond {
                        continue;
                    }
                    let text = cp.as_str().trim();
                    let mut kids = cp.clone().into_inner();
                    if let Some(stripped) = text.strip_prefix("flag set ") {
                        let name = kids.next().ok_or(AstError::Shape("flag name"))?.as_str().to_string();
                        debug_assert_eq!(stripped, name);
                        conds.push(crate::OverlayCondAst::FlagSet(name));
                        continue;
                    }
                    if let Some(stripped) = text.strip_prefix("flag unset ") {
                        let name = kids.next().ok_or(AstError::Shape("flag name"))?.as_str().to_string();
                        debug_assert_eq!(stripped, name);
                        conds.push(crate::OverlayCondAst::FlagUnset(name));
                        continue;
                    }
                    if let Some(stripped) = text.strip_prefix("flag complete ") {
                        let name = kids.next().ok_or(AstError::Shape("flag name"))?.as_str().to_string();
                        debug_assert_eq!(stripped, name);
                        conds.push(crate::OverlayCondAst::FlagComplete(name));
                        continue;
                    }
                    if let Some(stripped) = text.strip_prefix("item present ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        debug_assert_eq!(stripped, item);
                        conds.push(crate::OverlayCondAst::ItemPresent(item));
                        continue;
                    }
                    if let Some(stripped) = text.strip_prefix("item absent ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        debug_assert_eq!(stripped, item);
                        conds.push(crate::OverlayCondAst::ItemAbsent(item));
                        continue;
                    }
                    if text.starts_with("player has item ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::PlayerHasItem(item));
                        continue;
                    }
                    if text.starts_with("player missing item ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::PlayerMissingItem(item));
                        continue;
                    }
                    if text.starts_with("npc present ") {
                        let npc = kids.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::NpcPresent(npc));
                        continue;
                    }
                    if text.starts_with("npc absent ") {
                        let npc = kids.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::NpcAbsent(npc));
                        continue;
                    }
                    if text.starts_with("npc in state ") {
                        let npc = kids.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                        let nxt = kids.next().ok_or(AstError::Shape("state token"))?;
                        let oc = match nxt.as_rule() {
                            Rule::ident => crate::OverlayCondAst::NpcInState {
                                npc,
                                state: crate::NpcStateValue::Named(nxt.as_str().to_string()),
                            },
                            Rule::string => crate::OverlayCondAst::NpcInState {
                                npc,
                                state: crate::NpcStateValue::Custom(unquote(nxt.as_str())),
                            },
                            _ => {
                                let mut sub = nxt.into_inner();
                                let sval = sub.next().ok_or(AstError::Shape("custom string"))?;
                                crate::OverlayCondAst::NpcInState {
                                    npc,
                                    state: crate::NpcStateValue::Custom(unquote(sval.as_str())),
                                }
                            },
                        };
                        conds.push(oc);
                        continue;
                    }
                    if text.starts_with("item in room ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        let room = kids.next().ok_or(AstError::Shape("room id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::ItemInRoom { item, room });
                        continue;
                    }
                    // Unknown overlay condition; ignore silently per current behavior
                }
                // Ensure at least one condition was parsed (catch typos early)
                if conds.is_empty() {
                    return Err(AstError::Shape("overlay requires at least one condition"));
                }

                // Then block with text
                let block = it.next().ok_or(AstError::Shape("overlay block"))?;
                let mut txt = String::new();
                for p in block.into_inner() {
                    if p.as_rule() == Rule::string {
                        txt = unquote(p.as_str());
                        break;
                    }
                }
                overlays.push(crate::OverlayAst {
                    conditions: conds,
                    text: txt,
                });
            },
            Rule::overlay_flag_pair_stmt => {
                // overlay if flag <id> { set "..." unset "..." }
                let mut it = inner_stmt.into_inner();
                let flag = it.next().ok_or(AstError::Shape("flag name"))?.as_str().to_string();
                let block = it.next().ok_or(AstError::Shape("flag pair block"))?;
                let mut bi = block.into_inner();
                let set_txt = unquote(bi.next().ok_or(AstError::Shape("set text"))?.as_str());
                let unset_txt = unquote(bi.next().ok_or(AstError::Shape("unset text"))?.as_str());
                overlays.push(crate::OverlayAst {
                    conditions: vec![crate::OverlayCondAst::FlagSet(flag.clone())],
                    text: set_txt,
                });
                overlays.push(crate::OverlayAst {
                    conditions: vec![crate::OverlayCondAst::FlagUnset(flag)],
                    text: unset_txt,
                });
            },
            Rule::overlay_item_pair_stmt => {
                // overlay if item <id> { present "..." absent "..." }
                let mut it = inner_stmt.into_inner();
                let item = it.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                let block = it.next().ok_or(AstError::Shape("item pair block"))?;
                let mut bi = block.into_inner();
                let present_txt = unquote(bi.next().ok_or(AstError::Shape("present text"))?.as_str());
                let absent_txt = unquote(bi.next().ok_or(AstError::Shape("absent text"))?.as_str());
                overlays.push(crate::OverlayAst {
                    conditions: vec![crate::OverlayCondAst::ItemPresent(item.clone())],
                    text: present_txt,
                });
                overlays.push(crate::OverlayAst {
                    conditions: vec![crate::OverlayCondAst::ItemAbsent(item)],
                    text: absent_txt,
                });
            },
            Rule::overlay_npc_pair_stmt => {
                // overlay if npc <id> { present "..." absent "..." }
                let mut it = inner_stmt.into_inner();
                let npc = it.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                let block = it.next().ok_or(AstError::Shape("npc pair block"))?;
                let mut bi = block.into_inner();
                let present_txt = unquote(bi.next().ok_or(AstError::Shape("present text"))?.as_str());
                let absent_txt = unquote(bi.next().ok_or(AstError::Shape("absent text"))?.as_str());
                overlays.push(crate::OverlayAst {
                    conditions: vec![crate::OverlayCondAst::NpcPresent(npc.clone())],
                    text: present_txt,
                });
                overlays.push(crate::OverlayAst {
                    conditions: vec![crate::OverlayCondAst::NpcAbsent(npc)],
                    text: absent_txt,
                });
            },
            Rule::overlay_npc_states_stmt => {
                // overlay if npc <id> here { <state> "..." | custom(<id>) "..." }+
                let mut it = inner_stmt.into_inner();
                let npc = it.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                let block = it.next().ok_or(AstError::Shape("npc states block"))?;
                for line in block.into_inner() {
                    let mut kids = line.clone().into_inner();
                    let is_custom = line.as_str().trim_start().starts_with("custom(");
                    if is_custom {
                        let mut state_ident: Option<String> = None;
                        let mut text = None;
                        for p in kids {
                            match p.as_rule() {
                                Rule::ident => state_ident = Some(p.as_str().to_string()),
                                Rule::string => text = Some(unquote(p.as_str())),
                                _ => {},
                            }
                        }
                        let s = state_ident.ok_or(AstError::Shape("custom(state) requires ident"))?;
                        let txt = text.ok_or(AstError::Shape("custom(state) requires text"))?;
                        overlays.push(crate::OverlayAst {
                            conditions: vec![
                                crate::OverlayCondAst::NpcPresent(npc.clone()),
                                crate::OverlayCondAst::NpcInState {
                                    npc: npc.clone(),
                                    state: crate::NpcStateValue::Custom(s),
                                },
                            ],
                            text: txt,
                        });
                    } else {
                        // named state
                        let state_tok = kids.next().ok_or(AstError::Shape("npc state name"))?;
                        let state_name = state_tok.as_str().to_string();
                        let txt_pair = kids.next().ok_or(AstError::Shape("npc state text"))?;
                        let text = unquote(txt_pair.as_str());
                        overlays.push(crate::OverlayAst {
                            conditions: vec![
                                crate::OverlayCondAst::NpcPresent(npc.clone()),
                                crate::OverlayCondAst::NpcInState {
                                    npc: npc.clone(),
                                    state: crate::NpcStateValue::Named(state_name),
                                },
                            ],
                            text,
                        });
                    }
                }
            },
            _ => {},
        }
    }
    let name = name.ok_or(AstError::Shape("room missing name"))?;
    let desc = desc.ok_or(AstError::Shape("room missing desc"))?;
    Ok(RoomAst {
        id,
        name,
        desc,
        visited: visited.unwrap_or(false),
        exits,
        overlays,
        src_line,
    })
}

fn parse_item_pair(item: pest::iterators::Pair<Rule>, _source: &str) -> Result<ItemAst, AstError> {
    let (src_line, _src_col) = item.as_span().start_pos().line_col();
    let mut it = item.into_inner();
    let id = it
        .next()
        .ok_or(AstError::Shape("expected item ident"))?
        .as_str()
        .to_string();
    let block = it.next().ok_or(AstError::Shape("expected item block"))?;
    let mut name: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut portable: Option<bool> = None;
    let mut location: Option<ItemLocationAst> = None;
    let mut container_state: Option<ContainerStateAst> = None;
    let mut restricted: Option<bool> = None;
    let mut abilities: Vec<ItemAbilityAst> = Vec::new();
    let mut text: Option<String> = None;
    let mut requires: Vec<(String, String)> = Vec::new();
    let mut consumable: Option<ConsumableAst> = None;
    for stmt in block.into_inner() {
        match stmt.as_rule() {
            Rule::item_name => {
                let s = stmt.into_inner().next().ok_or(AstError::Shape("missing item name"))?;
                name = Some(unquote(s.as_str()));
            },
            Rule::item_desc => {
                let s = stmt.into_inner().next().ok_or(AstError::Shape("missing item desc"))?;
                desc = Some(unquote(s.as_str()));
            },
            Rule::item_portable => {
                let tok = stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing portable token"))?;
                portable = Some(tok.as_str() == "true");
            },
            Rule::item_restricted => {
                let tok = stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing restricted token"))?;
                restricted = Some(tok.as_str() == "true");
            },
            Rule::item_location => {
                let mut li = stmt.into_inner();
                let branch = li.next().ok_or(AstError::Shape("location kind"))?;
                let loc = match branch.as_rule() {
                    Rule::inventory_loc => {
                        let owner = branch
                            .into_inner()
                            .next()
                            .ok_or(AstError::Shape("inventory id"))?
                            .as_str()
                            .to_string();
                        ItemLocationAst::Inventory(owner)
                    },
                    Rule::room_loc => {
                        let room = branch
                            .into_inner()
                            .next()
                            .ok_or(AstError::Shape("room id"))?
                            .as_str()
                            .to_string();
                        ItemLocationAst::Room(room)
                    },
                    Rule::npc_loc => {
                        let npc = branch
                            .into_inner()
                            .next()
                            .ok_or(AstError::Shape("npc id"))?
                            .as_str()
                            .to_string();
                        ItemLocationAst::Npc(npc)
                    },
                    Rule::chest_loc => {
                        let chest = branch
                            .into_inner()
                            .next()
                            .ok_or(AstError::Shape("chest id"))?
                            .as_str()
                            .to_string();
                        ItemLocationAst::Chest(chest)
                    },
                    Rule::nowhere_loc => {
                        let note = branch
                            .into_inner()
                            .next()
                            .ok_or(AstError::Shape("nowhere note"))?
                            .as_str();
                        ItemLocationAst::Nowhere(unquote(note))
                    },
                    _ => return Err(AstError::Shape("unknown location kind")),
                };
                location = Some(loc);
            },
            Rule::item_container_state => {
                let val = stmt
                    .as_str()
                    .split_whitespace()
                    .last()
                    .ok_or(AstError::Shape("container state"))?;
                container_state = match val {
                    "open" => Some(ContainerStateAst::Open),
                    "closed" => Some(ContainerStateAst::Closed),
                    "locked" => Some(ContainerStateAst::Locked),
                    "transparentClosed" => Some(ContainerStateAst::TransparentClosed),
                    "transparentLocked" => Some(ContainerStateAst::TransparentLocked),
                    "none" => None,
                    _ => None,
                };
            },
            Rule::item_ability => {
                let mut ai = stmt.into_inner();
                let ability = ai.next().ok_or(AstError::Shape("ability name"))?.as_str().to_string();
                let target = ai.next().map(|p| p.as_str().to_string());
                abilities.push(ItemAbilityAst { ability, target });
            },
            Rule::item_text => {
                let s = stmt.into_inner().next().ok_or(AstError::Shape("missing text"))?;
                text = Some(unquote(s.as_str()));
            },
            Rule::item_requires => {
                let mut ri = stmt.into_inner();
                // New order: ability first, then interaction
                let ability = ri
                    .next()
                    .ok_or(AstError::Shape("requires ability"))?
                    .as_str()
                    .to_string();
                let interaction = ri
                    .next()
                    .ok_or(AstError::Shape("requires interaction"))?
                    .as_str()
                    .to_string();
                // Store as (interaction, ability) to match TOML mapping
                requires.push((interaction, ability));
            },
            Rule::item_consumable => {
                let mut uses_left: Option<usize> = None;
                let mut consume_on: Vec<ItemAbilityAst> = Vec::new();
                let mut when_consumed: Option<ConsumableWhenAst> = None;
                let mut stmt_iter = stmt.into_inner();
                let block = stmt_iter.next().ok_or(AstError::Shape("consumable block"))?;
                for cons_stmt in block.into_inner() {
                    let mut cons = cons_stmt.into_inner();
                    let Some(inner) = cons.next() else { continue };
                    match inner.as_rule() {
                        Rule::consumable_uses => {
                            let num_pair = inner.into_inner().next().ok_or(AstError::Shape("consumable uses"))?;
                            let raw = num_pair.as_str();
                            let val: i64 = raw
                                .parse()
                                .map_err(|_| AstError::Shape("consumable uses must be a number"))?;
                            if val < 0 {
                                return Err(AstError::Shape("consumable uses must be >= 0"));
                            }
                            uses_left = Some(val as usize);
                        },
                        Rule::consumable_consume_on => {
                            let mut ci = inner.into_inner();
                            let ability = ci
                                .next()
                                .ok_or(AstError::Shape("consume_on ability"))?
                                .as_str()
                                .to_string();
                            let target = ci.next().map(|p| p.as_str().to_string());
                            consume_on.push(ItemAbilityAst { ability, target });
                        },
                        Rule::consumable_when_consumed => {
                            let mut wi = inner.into_inner();
                            let variant = wi.next().ok_or(AstError::Shape("when_consumed value"))?;
                            when_consumed = Some(match variant.as_rule() {
                                Rule::consume_despawn => ConsumableWhenAst::Despawn,
                                Rule::consume_replace_inventory => {
                                    let replacement = variant
                                        .into_inner()
                                        .next()
                                        .ok_or(AstError::Shape("when_consumed replacement"))?
                                        .as_str()
                                        .to_string();
                                    ConsumableWhenAst::ReplaceInventory { replacement }
                                },
                                Rule::consume_replace_current_room => {
                                    let replacement = variant
                                        .into_inner()
                                        .next()
                                        .ok_or(AstError::Shape("when_consumed replacement"))?
                                        .as_str()
                                        .to_string();
                                    ConsumableWhenAst::ReplaceCurrentRoom { replacement }
                                },
                                _ => return Err(AstError::Shape("unknown when_consumed variant")),
                            });
                        },
                        _ => {},
                    }
                }
                let uses_left = uses_left.ok_or(AstError::Shape("consumable missing uses_left"))?;
                let when_consumed = when_consumed.ok_or(AstError::Shape("consumable missing when_consumed"))?;
                consumable = Some(ConsumableAst {
                    uses_left,
                    consume_on,
                    when_consumed,
                });
            },
            _ => {},
        }
    }
    let name = name.ok_or(AstError::Shape("item missing name"))?;
    let desc = desc.ok_or(AstError::Shape("item missing desc"))?;
    let portable = portable.ok_or(AstError::Shape("item missing portable"))?;
    let location = location.ok_or(AstError::Shape("item missing location"))?;
    Ok(ItemAst {
        id,
        name,
        desc,
        portable,
        location,
        container_state,
        restricted: restricted.unwrap_or(false),
        abilities,
        text,
        interaction_requires: requires,
        consumable,
        src_line,
    })
}

fn parse_spinner_pair(sp: pest::iterators::Pair<Rule>, _source: &str) -> Result<SpinnerAst, AstError> {
    let (src_line, _src_col) = sp.as_span().start_pos().line_col();
    let mut it = sp.into_inner();
    let id = it
        .next()
        .ok_or(AstError::Shape("expected spinner ident"))?
        .as_str()
        .to_string();
    let block = it.next().ok_or(AstError::Shape("expected spinner block"))?;
    let mut wedges = Vec::new();
    for w in block.into_inner() {
        let mut wi = w.into_inner();
        let text_pair = wi.next().ok_or(AstError::Shape("wedge text"))?;
        let text = unquote(text_pair.as_str());
        // width is optional; default to 1
        let width: usize = if let Some(width_pair) = wi.next() {
            width_pair
                .as_str()
                .parse()
                .map_err(|_| AstError::Shape("invalid wedge width"))?
        } else {
            1
        };
        wedges.push(SpinnerWedgeAst { text, width });
    }
    Ok(SpinnerAst { id, wedges, src_line })
}

fn parse_goal_pair(goal: pest::iterators::Pair<Rule>, _source: &str) -> Result<GoalAst, AstError> {
    let (src_line, _src_col) = goal.as_span().start_pos().line_col();
    let mut it = goal.into_inner();
    let id = it.next().ok_or(AstError::Shape("goal id"))?.as_str().to_string();
    let block = it.next().ok_or(AstError::Shape("goal block"))?;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut group: Option<GoalGroupAst> = None;
    let mut activate_when: Option<GoalCondAst> = None;
    let mut finished_when: Option<GoalCondAst> = None;
    let mut failed_when: Option<GoalCondAst> = None;
    for p in block.into_inner() {
        match p.as_rule() {
            Rule::goal_name => {
                let s = p.into_inner().next().ok_or(AstError::Shape("goal name text"))?.as_str();
                name = Some(unquote(s));
            },
            Rule::goal_desc => {
                let s = p.into_inner().next().ok_or(AstError::Shape("desc text"))?.as_str();
                description = Some(unquote(s));
            },
            Rule::goal_group => {
                let val = p.as_str().split_whitespace().last().unwrap_or("");
                group = Some(match val {
                    "required" => GoalGroupAst::Required,
                    "optional" => GoalGroupAst::Optional,
                    "status-effect" => GoalGroupAst::StatusEffect,
                    _ => GoalGroupAst::Required,
                });
            },
            Rule::goal_start => {
                let cond = p.into_inner().next().ok_or(AstError::Shape("start cond"))?;
                activate_when = Some(parse_goal_cond_pair(cond));
            },
            Rule::goal_done => {
                let cond = p.into_inner().next().ok_or(AstError::Shape("done cond"))?;
                finished_when = Some(parse_goal_cond_pair(cond));
            },
            Rule::goal_fail => {
                let cond = p.into_inner().next().ok_or(AstError::Shape("fail cond"))?;
                failed_when = Some(parse_goal_cond_pair(cond));
            },
            _ => {},
        }
    }
    let name = name.ok_or(AstError::Shape("goal missing name"))?;
    let description = description.ok_or(AstError::Shape("goal missing desc"))?;
    let group = group.ok_or(AstError::Shape("goal missing group"))?;
    let finished_when = finished_when.ok_or(AstError::Shape("goal missing done"))?;
    Ok(GoalAst {
        id,
        name,
        description,
        group,
        activate_when,
        failed_when,
        finished_when,
        src_line,
    })
}

fn parse_goal_cond_pair(p: pest::iterators::Pair<Rule>) -> GoalCondAst {
    let s = p.as_str().trim();
    if let Some(rest) = s.strip_prefix("has flag ") {
        return GoalCondAst::HasFlag(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("missing flag ") {
        return GoalCondAst::MissingFlag(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("has item ") {
        return GoalCondAst::HasItem(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("reached room ") {
        return GoalCondAst::ReachedRoom(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("goal complete ") {
        return GoalCondAst::GoalComplete(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("flag in progress ") {
        return GoalCondAst::FlagInProgress(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("flag complete ") {
        return GoalCondAst::FlagComplete(rest.trim().to_string());
    }
    GoalCondAst::HasFlag(s.to_string())
}
fn parse_npc_pair(npc: pest::iterators::Pair<Rule>, _source: &str) -> Result<NpcAst, AstError> {
    let (src_line, _src_col) = npc.as_span().start_pos().line_col();
    let mut it = npc.into_inner();
    let id = it
        .next()
        .ok_or(AstError::Shape("expected npc ident"))?
        .as_str()
        .to_string();
    let block = it.next().ok_or(AstError::Shape("expected npc block"))?;
    let mut name: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut location: Option<crate::NpcLocationAst> = None;
    let mut state: Option<NpcStateValue> = None;
    let mut movement: Option<NpcMovementAst> = None;
    let mut dialogue: Vec<(String, Vec<String>)> = Vec::new();
    for stmt in block.into_inner() {
        match stmt.as_rule() {
            Rule::npc_name => {
                let s = stmt.into_inner().next().ok_or(AstError::Shape("missing npc name"))?;
                name = Some(unquote(s.as_str()));
            },
            Rule::npc_desc => {
                let s = stmt.into_inner().next().ok_or(AstError::Shape("missing npc desc"))?;
                desc = Some(unquote(s.as_str()));
            },
            Rule::npc_location => {
                let mut li = stmt.into_inner();
                let tok = li.next().ok_or(AstError::Shape("location value"))?;
                let loc = match tok.as_rule() {
                    Rule::ident => crate::NpcLocationAst::Room(tok.as_str().to_string()),
                    Rule::string => crate::NpcLocationAst::Nowhere(unquote(tok.as_str())),
                    _ => return Err(AstError::Shape("npc location")),
                };
                location = Some(loc);
            },
            Rule::npc_state => {
                let mut si = stmt.into_inner();
                // First token: either ident or 'custom'
                let first = si.next().ok_or(AstError::Shape("state token"))?;
                let st = if first.as_rule() == Rule::ident {
                    NpcStateValue::Named(first.as_str().to_string())
                } else {
                    // custom ident
                    let v = si
                        .next()
                        .ok_or(AstError::Shape("custom state ident"))?
                        .as_str()
                        .to_string();
                    NpcStateValue::Custom(v)
                };
                state = Some(st);
            },
            Rule::npc_movement => {
                // movement <random|route> rooms (<ids>) [timing <ident>] [active <bool>]
                let s = stmt.as_str();
                let mtype = if s.contains(" movement random ") || s.trim_start().starts_with("movement random ") {
                    NpcMovementTypeAst::Random
                } else {
                    NpcMovementTypeAst::Route
                };
                // rooms list inside (...)
                let mut rooms: Vec<String> = Vec::new();
                if let Some(open) = s.find('(') {
                    if let Some(close_rel) = s[open + 1..].find(')') {
                        let inner = &s[open + 1..open + 1 + close_rel];
                        for tok in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                            rooms.push(tok.to_string());
                        }
                    }
                }
                let timing = s
                    .find(" timing ")
                    .map(|idx| s[idx + 8..].split_whitespace().next().unwrap_or("").to_string());
                let active = if let Some(idx) = s.find(" active ") {
                    let rest = &s[idx + 8..];
                    if rest.trim_start().starts_with("true") {
                        Some(true)
                    } else if rest.trim_start().starts_with("false") {
                        Some(false)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let loop_route = if let Some(idx) = s.find(" loop ") {
                    let rest = &s[idx + 6..];
                    if rest.trim_start().starts_with("true") {
                        Some(true)
                    } else if rest.trim_start().starts_with("false") {
                        Some(false)
                    } else {
                        None
                    }
                } else {
                    None
                };
                movement = Some(NpcMovementAst {
                    movement_type: mtype,
                    rooms,
                    timing,
                    active,
                    loop_route,
                });
            },
            Rule::npc_dialogue_block => {
                // dialogue <state|custom ident> { "..."+ }
                let mut di = stmt.into_inner();
                let first = di.next().ok_or(AstError::Shape("dialogue state"))?;
                let key = if first.as_rule() == Rule::ident {
                    first.as_str().to_string()
                } else {
                    let id = di
                        .next()
                        .ok_or(AstError::Shape("custom dialogue state ident"))?
                        .as_str()
                        .to_string();
                    format!("custom:{id}")
                };
                let mut lines: Vec<String> = Vec::new();
                for p in di {
                    if p.as_rule() == Rule::string {
                        lines.push(unquote(p.as_str()));
                    }
                }
                dialogue.push((key, lines));
            },
            _ => {
                // Fallback: simple text-based parsing for robustness
                let txt = stmt.as_str().trim_start();
                if let Some(rest) = txt.strip_prefix("name ") {
                    let (nm, _used) =
                        parse_string_at(rest).map_err(|_| AstError::Shape("npc name invalid quoted text"))?;
                    name = Some(nm);
                    continue;
                }
                if let Some(rest) = txt.strip_prefix("desc ") {
                    // or description
                    let (ds, _used) =
                        parse_string_at(rest).map_err(|_| AstError::Shape("npc desc invalid quoted text"))?;
                    desc = Some(ds);
                    continue;
                }
                if let Some(rest) = txt.strip_prefix("location room ") {
                    location = Some(crate::NpcLocationAst::Room(rest.trim().to_string()));
                    continue;
                }
                if let Some(rest) = txt.strip_prefix("location nowhere ") {
                    let (note, _used) = parse_string_at(rest)
                        .map_err(|_| AstError::Shape("npc location nowhere invalid quoted text"))?;
                    location = Some(crate::NpcLocationAst::Nowhere(note));
                    continue;
                }
                if let Some(rest) = txt.strip_prefix("state ") {
                    let rest = rest.trim();
                    if let Some(val) = rest.strip_prefix("custom ") {
                        state = Some(NpcStateValue::Custom(val.trim().to_string()));
                    } else {
                        // take first token as named state
                        let token = rest.split_whitespace().next().unwrap_or("");
                        if !token.is_empty() {
                            state = Some(NpcStateValue::Named(token.to_string()));
                        }
                    }
                    continue;
                }
                if let Some(rest) = txt.strip_prefix("movement ") {
                    let mut mtype = NpcMovementTypeAst::Route;
                    if rest.trim_start().starts_with("random ") {
                        mtype = NpcMovementTypeAst::Random;
                    }
                    let mut rooms: Vec<String> = Vec::new();
                    if let Some(open) = txt.find('(') {
                        if let Some(close_rel) = txt[open + 1..].find(')') {
                            let inner = &txt[open + 1..open + 1 + close_rel];
                            for tok in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                                rooms.push(tok.to_string());
                            }
                        }
                    }

                    let timing = txt
                        .find(" timing ")
                        .map(|idx| txt[idx + 8..].split_whitespace().next().unwrap_or("").to_string());

                    let active = if let Some(idx) = txt.find(" active ") {
                        let rest = &txt[idx + 8..];
                        if rest.trim_start().starts_with("true") {
                            Some(true)
                        } else if rest.trim_start().starts_with("false") {
                            Some(false)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let loop_route = if let Some(idx) = txt.find(" loop ") {
                        let rest = &txt[idx + 6..];
                        if rest.trim_start().starts_with("true") {
                            Some(true)
                        } else if rest.trim_start().starts_with("false") {
                            Some(false)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    movement = Some(NpcMovementAst {
                        movement_type: mtype,
                        rooms,
                        timing,
                        active,
                        loop_route,
                    });
                    continue;
                }
                if let Some(rest) = txt.strip_prefix("dialogue ") {
                    // dialogue <state|custom id> { "..."+ }
                    let rest = rest.trim_start();
                    let (key, after_key) = if let Some(val) = rest.strip_prefix("custom ") {
                        let mut parts = val.splitn(2, char::is_whitespace);
                        let id = parts.next().unwrap_or("").to_string();
                        (format!("custom:{id}"), parts.next().unwrap_or("").to_string())
                    } else {
                        let mut parts = rest.splitn(2, char::is_whitespace);
                        let id = parts.next().unwrap_or("").to_string();
                        (id, parts.next().unwrap_or("").to_string())
                    };
                    if let Some(open_idx) = after_key.find('{') {
                        if let Some(close_rel) = after_key[open_idx + 1..].rfind('}') {
                            let mut inner = &after_key[open_idx + 1..open_idx + 1 + close_rel];
                            let mut lines: Vec<String> = Vec::new();
                            loop {
                                inner = inner.trim_start();
                                if inner.is_empty() {
                                    break;
                                }
                                if inner.starts_with('"') || inner.starts_with('r') || inner.starts_with('\'') {
                                    if let Ok((val, used)) = parse_string_at(inner) {
                                        lines.push(val);
                                        inner = &inner[used..];
                                        continue;
                                    } else {
                                        break;
                                    }
                                } else {
                                    // consume until next quote or end
                                    if let Some(pos) = inner.find('"') {
                                        inner = &inner[pos..];
                                    } else {
                                        break;
                                    }
                                }
                            }
                            if !lines.is_empty() {
                                dialogue.push((key, lines));
                                continue;
                            }
                        }
                    }
                }
            },
        }
    }
    let name = name.ok_or(AstError::Shape("npc missing name"))?;
    let desc = desc.ok_or(AstError::Shape("npc missing desc"))?;
    let location = location.ok_or(AstError::Shape("npc missing location"))?;
    let state = state.unwrap_or(NpcStateValue::Named("normal".to_string()));
    Ok(NpcAst {
        id,
        name,
        desc,
        location,
        state,
        movement,
        dialogue,
        src_line,
    })
}

/// Parse only rooms from a source (helper/testing).
/// Parse only room definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into rooms.
pub fn parse_rooms(source: &str) -> Result<Vec<RoomAst>, AstError> {
    let (_, rooms, _, _, _, _) = parse_program_full(source)?;
    Ok(rooms)
}

/// Parse only items from a source (helper/testing).
/// Parse only item definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into items.
pub fn parse_items(source: &str) -> Result<Vec<ItemAst>, AstError> {
    let (_, _, items, _, _, _) = parse_program_full(source)?;
    Ok(items)
}

/// Parse only spinners from a source (helper/testing).
/// Parse only spinner definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into spinners.
pub fn parse_spinners(source: &str) -> Result<Vec<SpinnerAst>, AstError> {
    let (_, _, _, spinners, _, _) = parse_program_full(source)?;
    Ok(spinners)
}

/// Parse only npcs from a source (helper/testing).
/// Parse only NPC definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into NPCs.
pub fn parse_npcs(source: &str) -> Result<Vec<NpcAst>, AstError> {
    let (_, _, _, _, npcs, _) = parse_program_full(source)?;
    Ok(npcs)
}

/// Parse only goal definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into goals.
pub fn parse_goals(source: &str) -> Result<Vec<GoalAst>, AstError> {
    let (_, _, _, _, _, goals) = parse_program_full(source)?;
    Ok(goals)
}

fn unquote(s: &str) -> String {
    parse_string(s).unwrap_or_else(|_| s.to_string())
}

/// Parse a string literal (supports "...", r"...", and """..."""). Returns the decoded value.
fn parse_string(s: &str) -> Result<String, AstError> {
    let (v, _u) = parse_string_at(s)?;
    Ok(v)
}

/// Parse a string literal starting at the beginning of `s`. Returns (decoded, bytes_consumed).
fn parse_string_at(s: &str) -> Result<(String, usize), AstError> {
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(AstError::Shape("empty string"));
    }
    // Triple-quoted
    if s.starts_with("\"\"\"") {
        let mut out = String::new();
        let mut i = 3usize; // after opening """
        let mut escape = false;
        while i < s.len() {
            if !escape && s[i..].starts_with("\"\"\"") {
                return Ok((out, i + 3));
            }
            let ch = s[i..].chars().next().unwrap();
            i += ch.len_utf8();
            if escape {
                match ch {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    },
                }
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
            } else {
                out.push(ch);
            }
        }
        return Err(AstError::Shape("missing closing triple quote"));
    }
    // Raw r"..." and hashed raw r#"..."#
    if s.starts_with('r') {
        // Count hashes after r
        let mut i = 1usize;
        while i < s.len() && s.as_bytes()[i] as char == '#' {
            i += 1;
        }
        if i < s.len() && s.as_bytes()[i] as char == '"' {
            let num_hashes = i - 1;
            let close_seq = {
                let mut seq = String::from("\"");
                for _ in 0..num_hashes {
                    seq.push('#');
                }
                seq
            };
            let content_start = i + 1;
            let rest = &s[content_start..];
            if let Some(pos) = rest.find(&close_seq) {
                let val = &rest[..pos];
                return Ok((val.to_string(), content_start + pos + close_seq.len()));
            } else {
                return Err(AstError::Shape("missing closing raw quote"));
            }
        }
    }
    // Single-quoted
    if b[0] as char == '\'' {
        let mut out = String::new();
        let mut i = 1usize; // skip opening '
        let mut escape = false;
        while i < s.len() {
            let ch = s[i..].chars().next().unwrap();
            i += ch.len_utf8();
            if escape {
                match ch {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\'' => out.push('\''),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    },
                }
                escape = false;
                continue;
            }
            match ch {
                '\\' => {
                    escape = true;
                },
                '\'' => return Ok((out, i)),
                _ => out.push(ch),
            }
        }
        return Err(AstError::Shape("missing closing single quote"));
    }
    // Regular quoted
    if b[0] as char != '"' {
        return Err(AstError::Shape("missing opening quote"));
    }
    let mut out = String::new();
    let mut i = 1usize; // skip opening quote
    let mut escape = false;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        i += ch.len_utf8();
        if escape {
            match ch {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                other => {
                    out.push('\\');
                    out.push(other);
                },
            }
            escape = false;
            continue;
        }
        match ch {
            '\\' => {
                escape = true;
            },
            '"' => return Ok((out, i)),
            _ => out.push(ch),
        }
    }
    Err(AstError::Shape("missing closing quote"))
}

fn extract_body(src: &str) -> Result<&str, AstError> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut start = None;
    let mut end = None;
    let mut i = 0usize;
    let mut in_str = false;
    let mut escape = false;
    let mut in_comment = false;
    let mut at_line_start = true;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_comment {
            if c == '\n' {
                in_comment = false;
                at_line_start = true;
            }
            i += 1;
            continue;
        }
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }

            // inside string, line starts don't apply
            i += 1;
            continue;
        }
        match c {
            '\n' => {
                at_line_start = true;
            },
            ' ' | '\t' | '\r' => {
                // keep at_line_start as-is
            },
            '"' => {
                in_str = true;
                at_line_start = false;
            },
            '#' => {
                // Treat '#' as a comment only if it begins the line (ignoring leading spaces)
                if at_line_start {
                    in_comment = true;
                }
                at_line_start = false;
            },
            '{' => {
                if depth == 0 {
                    start = Some(i + 1);
                }
                depth += 1;
                at_line_start = false;
            },
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
                at_line_start = false;
            },
            _ => {
                at_line_start = false;
            },
        }
        i += 1;
    }
    let s = start.ok_or(AstError::Shape("missing '{' body start"))?;
    let e = end.ok_or(AstError::Shape("missing '}' body end"))?;
    Ok(&src[s..e])
}

fn parse_condition_text(text: &str, sets: &HashMap<String, Vec<String>>) -> Result<ConditionAst, AstError> {
    let t = text.trim();
    if let Some(inner) = t.strip_prefix("all(") {
        let inner = inner.strip_suffix(')').ok_or(AstError::Shape("all() close"))?;
        let parts = split_top_level_commas(inner);
        let mut kids = Vec::new();
        for p in parts {
            kids.push(parse_condition_text(p, sets)?);
        }
        return Ok(ConditionAst::All(kids));
    }
    if let Some(inner) = t.strip_prefix("any(") {
        let inner = inner.strip_suffix(')').ok_or(AstError::Shape("any() close"))?;
        let parts = split_top_level_commas(inner);
        let mut kids = Vec::new();
        for p in parts {
            kids.push(parse_condition_text(p, sets)?);
        }
        return Ok(ConditionAst::Any(kids));
    }
    if let Some(rest) = t.strip_prefix("has flag ") {
        return Ok(ConditionAst::HasFlag(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("missing flag ") {
        return Ok(ConditionAst::MissingFlag(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("has item ") {
        return Ok(ConditionAst::HasItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("has visited room ") {
        return Ok(ConditionAst::HasVisited(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("missing item ") {
        return Ok(ConditionAst::MissingItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("flag in progress ") {
        return Ok(ConditionAst::FlagInProgress(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("flag complete ") {
        return Ok(ConditionAst::FlagComplete(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("with npc ") {
        return Ok(ConditionAst::WithNpc(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("npc has item ") {
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let item = rest[space + 1..].trim();
            return Ok(ConditionAst::NpcHasItem {
                npc: npc.to_string(),
                item: item.to_string(),
            });
        }
        return Err(AstError::Shape("npc has item syntax"));
    }
    if let Some(rest) = t.strip_prefix("npc in state ") {
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let state = rest[space + 1..].trim();
            return Ok(ConditionAst::NpcInState {
                npc: npc.to_string(),
                state: state.to_string(),
            });
        }
        return Err(AstError::Shape("npc in state syntax"));
    }
    // Preferred: container <container> has item <item>
    if let Some(rest) = t.strip_prefix("container ") {
        let rest = rest.trim();
        if let Some(idx) = rest.find(" has item ") {
            let container = &rest[..idx];
            let item = &rest[idx + " has item ".len()..];
            return Ok(ConditionAst::ContainerHasItem {
                container: container.trim().to_string(),
                item: item.trim().to_string(),
            });
        }
    }
    if let Some(rest) = t.strip_prefix("container has item ") {
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let container = &rest[..space];
            let item = rest[space + 1..].trim();
            return Ok(ConditionAst::ContainerHasItem {
                container: container.to_string(),
                item: item.to_string(),
            });
        }
        return Err(AstError::Shape("container has item syntax"));
    }
    if let Some(rest) = t.strip_prefix("ambient ") {
        let rest = rest.trim();
        if let Some(idx) = rest.find(" in rooms ") {
            let spinner = rest[..idx].trim().to_string();
            let list = rest[idx + 10..].trim();
            let mut rooms: Vec<String> = Vec::new();
            for tok in list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if let Some(v) = sets.get(tok) {
                    rooms.extend(v.clone());
                } else {
                    rooms.push(tok.to_string());
                }
            }
            return Ok(ConditionAst::Ambient {
                spinner,
                rooms: Some(rooms),
            });
        } else {
            return Ok(ConditionAst::Ambient {
                spinner: rest.to_string(),
                rooms: None,
            });
        }
    }
    // Preferred shorthand: "in rooms <r1,r2,...>" expands to any(player in room r1, player in room r2, ...)
    if let Some(rest) = t.strip_prefix("in rooms ") {
        let list = rest.trim();
        let mut rooms: Vec<String> = Vec::new();
        for tok in list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if let Some(v) = sets.get(tok) {
                rooms.extend(v.clone());
            } else {
                rooms.push(tok.to_string());
            }
        }
        // If only one room, return simple PlayerInRoom; else return Any of PlayerInRoom
        if rooms.len() == 1 {
            return Ok(ConditionAst::PlayerInRoom(rooms.remove(0)));
        } else if !rooms.is_empty() {
            let kids = rooms.into_iter().map(ConditionAst::PlayerInRoom).collect();
            return Ok(ConditionAst::Any(kids));
        } else {
            return Err(AstError::Shape("in rooms requires at least one room"));
        }
    }
    if let Some(rest) = t.strip_prefix("player in room ") {
        return Ok(ConditionAst::PlayerInRoom(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("chance ") {
        let rest = rest.trim();
        let num = rest.strip_suffix('%').ok_or(AstError::Shape("chance percent %"))?;
        let pct: f64 = num
            .trim()
            .parse()
            .map_err(|_| AstError::Shape("invalid chance percent"))?;
        return Ok(ConditionAst::ChancePercent(pct));
    }
    Err(AstError::Shape("unknown condition"))
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b as char {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                // Only split at commas that separate conditions, not those inside
                // lists like "ambient ... in rooms a,b,c".
                let mut j = i + 1;
                while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                    j += 1;
                }
                // capture next word
                let mut k = j;
                while k < bytes.len() {
                    let ch = bytes[k] as char;
                    if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                        k += 1;
                    } else {
                        break;
                    }
                }
                let next_word = if j < k {
                    s[j..k].to_ascii_lowercase()
                } else {
                    String::new()
                };
                let is_sep = next_word.is_empty()
                    || matches!(
                        next_word.as_str(),
                        "has"
                            | "missing"
                            | "player"
                            | "container"
                            | "ambient"
                            | "in"
                            | "npc"
                            | "with"
                            | "flag"
                            | "chance"
                            | "all"
                            | "any"
                    );
                if is_sep {
                    out.push(s[start..i].trim());
                    start = i + 1;
                }
            },
            _ => {},
        }
    }
    if start < s.len() {
        out.push(s[start..].trim());
    }
    out
}

fn parse_action_from_str(text: &str) -> Result<ActionAst, AstError> {
    let t = text.trim();
    if t.starts_with("do modify item ") {
        let (action, _) = parse_modify_item_action(t)?;
        return Ok(action);
    }
    if t.starts_with("do modify room ") {
        let (action, _) = parse_modify_room_action(t)?;
        return Ok(action);
    }
    if let Some(rest) = t.strip_prefix("do show ") {
        return Ok(ActionAst::Show(super::parser::unquote(rest.trim())));
    }
    if let Some(rest) = t.strip_prefix("do add wedge ") {
        // do add wedge "text" [width <n>] spinner <ident>
        let r = rest.trim();
        let (text, used) = parse_string_at(r).map_err(|_| AstError::Shape("add wedge missing or invalid quote"))?;
        let mut after = r[used..].trim_start();
        // optional width
        let mut width: usize = 1;
        if let Some(wrest) = after.strip_prefix("width ") {
            let mut j = 0usize;
            while j < wrest.len() && wrest.as_bytes()[j].is_ascii_digit() {
                j += 1;
            }
            if j == 0 {
                return Err(AstError::Shape("add wedge missing width number"));
            }
            width = wrest[..j].parse().map_err(|_| AstError::Shape("invalid wedge width"))?;
            after = wrest[j..].trim_start();
        }
        let spinner = after
            .strip_prefix("spinner ")
            .ok_or(AstError::Shape("add wedge missing 'spinner'"))?
            .trim()
            .to_string();
        return Ok(ActionAst::AddSpinnerWedge { spinner, width, text });
    }
    if let Some(rest) = t.strip_prefix("do add flag ") {
        return Ok(ActionAst::AddFlag(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do add seq flag ") {
        // Syntax: do add seq flag <name> [limit <n>]
        let rest = rest.trim();
        if let Some((name, tail)) = rest.split_once(" limit ") {
            let end: u8 = tail
                .trim()
                .parse()
                .map_err(|_| AstError::Shape("invalid seq flag limit"))?;
            return Ok(ActionAst::AddSeqFlag {
                name: name.trim().to_string(),
                end: Some(end),
            });
        }
        return Ok(ActionAst::AddSeqFlag {
            name: rest.to_string(),
            end: None,
        });
    }
    if let Some(rest) = t.strip_prefix("do remove flag ") {
        return Ok(ActionAst::RemoveFlag(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do reset flag ") {
        return Ok(ActionAst::ResetFlag(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do advance flag ") {
        return Ok(ActionAst::AdvanceFlag(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do replace item ") {
        let rest = rest.trim();
        if let Some((old_sym, new_sym)) = rest.split_once(" with ") {
            return Ok(ActionAst::ReplaceItem {
                old_sym: old_sym.trim().to_string(),
                new_sym: new_sym.trim().to_string(),
            });
        }
        return Err(AstError::Shape("replace item syntax"));
    }
    if let Some(rest) = t.strip_prefix("do replace drop item ") {
        let rest = rest.trim();
        if let Some((old_sym, new_sym)) = rest.split_once(" with ") {
            return Ok(ActionAst::ReplaceDropItem {
                old_sym: old_sym.trim().to_string(),
                new_sym: new_sym.trim().to_string(),
            });
        }
        return Err(AstError::Shape("replace drop item syntax"));
    }
    if let Some(rest) = t.strip_prefix("do spawn npc ") {
        // format: <npc> into room <room>
        let rest = rest.trim();
        if let Some((npc, room)) = rest.split_once(" into room ") {
            return Ok(ActionAst::SpawnNpcIntoRoom {
                npc: npc.trim().to_string(),
                room: room.trim().to_string(),
            });
        }
        return Err(AstError::Shape("spawn npc into room syntax"));
    }
    if let Some(rest) = t.strip_prefix("do despawn npc ") {
        return Ok(ActionAst::DespawnNpc(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do spawn item ") {
        // format: <item> into room <room>
        let rest = rest.trim();
        if let Some((item, tail)) = rest.split_once(" into room ") {
            return Ok(ActionAst::SpawnItemIntoRoom {
                item: item.trim().to_string(),
                room: tail.trim().to_string(),
            });
        }
        if let Some((item, tail)) = rest.split_once(" into container ") {
            return Ok(ActionAst::SpawnItemInContainer {
                item: item.trim().to_string(),
                container: tail.trim().to_string(),
            });
        }
        if let Some((item, tail)) = rest.split_once(" in container ") {
            return Ok(ActionAst::SpawnItemInContainer {
                item: item.trim().to_string(),
                container: tail.trim().to_string(),
            });
        }
        if let Some(item) = rest.strip_suffix(" in inventory") {
            return Ok(ActionAst::SpawnItemInInventory(item.trim().to_string()));
        }
        if let Some(item) = rest.strip_suffix(" in current room") {
            return Ok(ActionAst::SpawnItemCurrentRoom(item.trim().to_string()));
        }
        return Err(AstError::Shape("spawn item syntax"));
    }
    if let Some(rest) = t.strip_prefix("do lock item ") {
        return Ok(ActionAst::LockItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do unlock item ") {
        return Ok(ActionAst::UnlockItemAction(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do set barred message from ") {
        // do set barred message from <exit_from> to <exit_to> "msg"
        let rest = rest.trim();
        if let Some((from, tail)) = rest.split_once(" to ") {
            let tail = tail.trim();
            // tail is: <exit_to> "message..."
            let mut parts = tail.splitn(2, ' ');
            let exit_to = parts
                .next()
                .ok_or(AstError::Shape("barred message missing exit_to"))?
                .to_string();
            let msg_part = parts
                .next()
                .ok_or(AstError::Shape("barred message missing message"))?
                .trim();
            let (msg, _used) =
                parse_string_at(msg_part).map_err(|_| AstError::Shape("barred message invalid quoted text"))?;
            return Ok(ActionAst::SetBarredMessage {
                exit_from: from.trim().to_string(),
                exit_to,
                msg,
            });
        }
        return Err(AstError::Shape("set barred message syntax"));
    }
    if let Some(rest) = t.strip_prefix("do lock exit from ") {
        if let Some((from, tail)) = rest.split_once(" direction ") {
            let tail = tail.trim_start();
            let direction = match parse_string_at(tail) {
                Ok((s, _used)) => s,
                Err(_) => tail.trim().to_string(),
            };
            return Ok(ActionAst::LockExit {
                from_room: from.trim().to_string(),
                direction,
            });
        }
    }
    if let Some(rest) = t.strip_prefix("do unlock exit from ") {
        if let Some((from, tail)) = rest.split_once(" direction ") {
            let tail = tail.trim_start();
            let direction = match parse_string_at(tail) {
                Ok((s, _used)) => s,
                Err(_) => tail.trim().to_string(),
            };
            return Ok(ActionAst::UnlockExit {
                from_room: from.trim().to_string(),
                direction,
            });
        }
    }
    if let Some(rest) = t.strip_prefix("do give item ") {
        // do give item <item> to player from npc <npc>
        let rest = rest.trim();
        if let Some((item, tail)) = rest.split_once(" to player from npc ") {
            return Ok(ActionAst::GiveItemToPlayer {
                item: item.trim().to_string(),
                npc: tail.trim().to_string(),
            });
        }
        return Err(AstError::Shape("give item to player syntax"));
    }
    if let Some(rest) = t.strip_prefix("do reveal exit from ") {
        // format: <from> to <to> direction <dir>
        if let Some((from, tail)) = rest.split_once(" to ") {
            if let Some((to, dir_tail)) = tail.split_once(" direction ") {
                let dir_tail = dir_tail.trim_start();
                let direction = match parse_string_at(dir_tail) {
                    Ok((s, _used)) => s,
                    Err(_) => dir_tail.trim().to_string(),
                };
                return Ok(ActionAst::RevealExit {
                    exit_from: from.trim().to_string(),
                    exit_to: to.trim().to_string(),
                    direction,
                });
            }
        }
        return Err(AstError::Shape("reveal exit syntax"));
    }
    if let Some(rest) = t.strip_prefix("do push player to ") {
        return Ok(ActionAst::PushPlayerTo(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do set item description ") {
        // format: <item> "text"
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let item = &rest[..space];
            let txt = rest[space..].trim();
            let (text, _used) =
                parse_string_at(txt).map_err(|_| AstError::Shape("set item description missing or invalid quote"))?;
            return Ok(ActionAst::SetItemDescription {
                item: item.to_string(),
                text,
            });
        }
        return Err(AstError::Shape("set item description syntax"));
    }
    if let Some(rest) = t.strip_prefix("do npc says ") {
        // format: <npc> "quote"
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let txt = rest[space..].trim();
            let (quote, _used) =
                parse_string_at(txt).map_err(|_| AstError::Shape("npc says missing or invalid quote"))?;
            return Ok(ActionAst::NpcSays {
                npc: npc.to_string(),
                quote,
            });
        }
        return Err(AstError::Shape("npc says syntax"));
    }
    if let Some(rest) = t.strip_prefix("do npc random dialogue ") {
        return Ok(ActionAst::NpcSaysRandom {
            npc: rest.trim().to_string(),
        });
    }
    if let Some(rest) = t.strip_prefix("do npc refuse item ") {
        let rest = rest.trim();
        let mut parts = rest.splitn(2, ' ');
        let npc = parts
            .next()
            .ok_or(AstError::Shape("npc refuse item missing npc"))?
            .to_string();
        let reason_part = parts
            .next()
            .ok_or(AstError::Shape("npc refuse item missing reason"))?
            .trim();
        let (reason, _used) =
            parse_string_at(reason_part).map_err(|_| AstError::Shape("npc refuse missing or invalid quote"))?;
        return Ok(ActionAst::NpcRefuseItem { npc, reason });
    }
    if let Some(rest) = t.strip_prefix("do set npc active ") {
        // format: <npc> <true/false>
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let active_str = rest[space + 1..].trim();
            let active = match active_str {
                "true" => true,
                "false" => false,
                _ => return Err(AstError::Shape("set npc active requires 'true' or 'false'")),
            };
            return Ok(ActionAst::SetNpcActive {
                npc: npc.to_string(),
                active,
            });
        }
        return Err(AstError::Shape("set npc active syntax"));
    }
    if let Some(rest) = t.strip_prefix("do set npc state ") {
        // format: <npc> <state>
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let state = rest[space + 1..].trim();
            return Ok(ActionAst::SetNpcState {
                npc: npc.to_string(),
                state: state.to_string(),
            });
        }
        return Err(AstError::Shape("set npc state syntax"));
    }
    if let Some(rest) = t.strip_prefix("do deny read ") {
        let (msg, _used) =
            parse_string_at(rest.trim()).map_err(|_| AstError::Shape("deny read missing or invalid quote"))?;
        return Ok(ActionAst::DenyRead(msg));
    }
    if let Some(rest) = t.strip_prefix("do restrict item ") {
        return Ok(ActionAst::RestrictItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do set container state ") {
        // do set container state <item> <state|none>
        let mut parts = rest.split_whitespace();
        let item = parts
            .next()
            .ok_or(AstError::Shape("set container state missing item"))?
            .to_string();
        let state_tok = parts
            .next()
            .ok_or(AstError::Shape("set container state missing state"))?;
        let state = match state_tok {
            "none" => None,
            s => Some(s.to_string()),
        };
        return Ok(ActionAst::SetContainerState { item, state });
    }
    if let Some(rest) = t.strip_prefix("do spinner message ") {
        return Ok(ActionAst::SpinnerMessage {
            spinner: rest.trim().to_string(),
        });
    }
    // schedule actions are parsed at the block level in parse_actions_from_body
    if let Some(rest) = t.strip_prefix("do despawn item ") {
        return Ok(ActionAst::DespawnItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do award points ") {
        let n: i64 = rest
            .trim()
            .parse()
            .map_err(|_| AstError::Shape("invalid points number"))?;
        return Ok(ActionAst::AwardPoints(n));
    }
    Err(AstError::Shape("unknown action"))
}

fn parse_actions_from_body(
    body: &str,
    source: &str,
    smap: &SourceMap,
    sets: &HashMap<String, Vec<String>>,
) -> Result<Vec<ActionAst>, AstError> {
    let mut out = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] as char == '#' {
            while i < bytes.len() && (bytes[i] as char) != '\n' {
                i += 1;
            }
            continue;
        }
        if body[i..].starts_with("if ") {
            let if_pos = i;
            let rest = &body[if_pos + 3..];
            let brace_rel = rest.find('{').ok_or(AstError::Shape("missing '{' after if"))?;
            let cond_text = rest[..brace_rel].trim();
            let cond = match parse_condition_text(cond_text, sets) {
                Ok(c) => c,
                Err(AstError::Shape(m)) => {
                    let base = str_offset(source, body);
                    let cond_abs = base + (cond_text.as_ptr() as usize - body.as_ptr() as usize);
                    let (line_no, col) = smap.line_col(cond_abs);
                    let snippet = smap.line_snippet(line_no);
                    return Err(AstError::ShapeAt {
                        msg: m,
                        context: format!(
                            "line {line_no}, col {col}: {snippet}\n{}^",
                            " ".repeat(col.saturating_sub(1))
                        ),
                    });
                },
                Err(e) => return Err(e),
            };
            let block_after = &rest[brace_rel..];
            let inner_body = extract_body(block_after)?;
            let actions = parse_actions_from_body(inner_body, source, smap, sets)?;
            out.push(ActionAst::Conditional {
                condition: Box::new(cond),
                actions,
            });
            let consumed = brace_rel + 1 + inner_body.len() + 1;
            i = if_pos + 3 + consumed;
            continue;
        }
        let remainder = &body[i..];
        let trimmed_remainder = remainder.trim_start();
        let ws_leading = remainder.len() - trimmed_remainder.len();

        match parse_modify_item_action(remainder) {
            Ok((action, used)) => {
                out.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a modify item action")) => {},
            Err(AstError::Shape(m)) => {
                let base = str_offset(source, body);
                let abs = base + i;
                let (line_no, col) = smap.line_col(abs);
                let snippet = smap.line_snippet(line_no);
                return Err(AstError::ShapeAt {
                    msg: m,
                    context: format!(
                        "line {line_no}, col {col}: {snippet}\n{}^",
                        " ".repeat(col.saturating_sub(1))
                    ),
                });
            },
            Err(e) => return Err(e),
        }

        match parse_modify_room_action(remainder) {
            Ok((action, used)) => {
                out.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a modify room action")) => {},
            Err(AstError::Shape(m)) => {
                let base = str_offset(source, body);
                let abs = base + i;
                let (line_no, col) = smap.line_col(abs);
                let snippet = smap.line_snippet(line_no);
                return Err(AstError::ShapeAt {
                    msg: m,
                    context: format!(
                        "line {line_no}, col {col}: {snippet}\n{}^",
                        " ".repeat(col.saturating_sub(1))
                    ),
                });
            },
            Err(e) => return Err(e),
        }

        match parse_modify_npc_action(remainder) {
            Ok((action, used)) => {
                out.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a modify npc action")) => {},
            Err(AstError::Shape(m)) => {
                let base = str_offset(source, body);
                let abs = base + i;
                let (line_no, col) = smap.line_col(abs);
                let snippet = smap.line_snippet(line_no);
                return Err(AstError::ShapeAt {
                    msg: m,
                    context: format!(
                        "line {line_no}, col {col}: {snippet}\n{}^",
                        " ".repeat(col.saturating_sub(1))
                    ),
                });
            },
            Err(e) => return Err(e),
        }

        if trimmed_remainder.starts_with("do schedule in ") || trimmed_remainder.starts_with("do schedule on ") {
            let (action, used) = parse_schedule_action(&body[i + ws_leading..], source, smap, sets)?;
            out.push(action);
            i += ws_leading + used;
            continue;
        }
        if trimmed_remainder.starts_with("do ") {
            let mut j = i;
            while j < bytes.len() && (bytes[j] as char) != '\n' {
                j += 1;
            }
            let line = body[i..j].trim_end();
            match parse_action_from_str(line) {
                Ok(a) => out.push(a),
                Err(AstError::Shape(m)) => {
                    let base = str_offset(source, body);
                    let abs = base + i;
                    let (line_no, col) = smap.line_col(abs);
                    let snippet = smap.line_snippet(line_no);
                    return Err(AstError::ShapeAt {
                        msg: m,
                        context: format!(
                            "line {line_no}, col {col}: {snippet}\n{}^",
                            " ".repeat(col.saturating_sub(1))
                        ),
                    });
                },
                Err(e) => return Err(e),
            }
            i = j;
            continue;
        }
        while i < bytes.len() && (bytes[i] as char) != '\n' {
            i += 1;
        }
    }
    Ok(out)
}

fn parse_modify_item_action(text: &str) -> Result<(ActionAst, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let rest0 = s
        .strip_prefix("do modify item ")
        .ok_or(AstError::Shape("not a modify item action"))?;
    let rest0 = rest0.trim_start();
    let ident_len = rest0.chars().take_while(|&c| is_ident_char(c)).count();
    if ident_len == 0 {
        return Err(AstError::Shape("modify item missing item identifier"));
    }
    let ident_str = &rest0[..ident_len];
    DslParser::parse(Rule::ident, ident_str).map_err(|_| AstError::Shape("modify item has invalid item identifier"))?;
    let item = ident_str.to_string();
    let after_ident = rest0[ident_len..].trim_start();
    if !after_ident.starts_with('{') {
        return Err(AstError::Shape("modify item missing '{' block"));
    }
    let block_slice = after_ident;
    let body = extract_body(block_slice)?;
    // SAFETY: `body` was produced by `extract_body` from `block_slice`, so both pointers
    // lie within the same string slice.
    let start_offset = unsafe { body.as_ptr().offset_from(block_slice.as_ptr()) as usize };
    let block_total_len = start_offset + body.len() + 1;
    let remaining = &block_slice[block_total_len..];
    let consumed = leading_ws + (s.len() - remaining.len());
    let patch_block = &block_slice[..block_total_len];

    let mut pairs = DslParser::parse(Rule::item_patch_block, patch_block).map_err(|e| AstError::Pest(e.to_string()))?;
    let block_pair = pairs
        .next()
        .ok_or(AstError::Shape("modify item block missing statements"))?;
    let mut patch = ItemPatchAst::default();

    for stmt in block_pair.into_inner() {
        match stmt.as_rule() {
            Rule::item_name_patch => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify item name missing value"))?
                    .as_str();
                patch.name = Some(unquote(val));
            },
            Rule::item_desc_patch => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify item description missing value"))?
                    .as_str();
                patch.desc = Some(unquote(val));
            },
            Rule::item_text_patch => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify item text missing value"))?
                    .as_str();
                patch.text = Some(unquote(val));
            },
            Rule::item_portable_patch => {
                let mut inner = stmt.into_inner();
                let tok = inner
                    .next()
                    .ok_or(AstError::Shape("modify item portable missing value"))?
                    .as_str();
                let portable = match tok {
                    "true" => true,
                    "false" => false,
                    _ => return Err(AstError::Shape("portable expects true or false")),
                };
                patch.portable = Some(portable);
            },
            Rule::item_restricted_patch => {
                let mut inner = stmt.into_inner();
                let tok = inner
                    .next()
                    .ok_or(AstError::Shape("modify item restricted missing value"))?
                    .as_str();
                let restricted = match tok {
                    "true" => true,
                    "false" => false,
                    _ => return Err(AstError::Shape("restricted expects true or false")),
                };
                patch.restricted = Some(restricted);
            },
            Rule::item_container_state_patch => {
                let state_word = stmt
                    .as_str()
                    .split_whitespace()
                    .last()
                    .ok_or(AstError::Shape("container state missing value"))?;
                match state_word {
                    "off" => {
                        patch.remove_container_state = true;
                        patch.container_state = None;
                    },
                    "open" => {
                        patch.container_state = Some(ContainerStateAst::Open);
                        patch.remove_container_state = false;
                    },
                    "closed" => {
                        patch.container_state = Some(ContainerStateAst::Closed);
                        patch.remove_container_state = false;
                    },
                    "locked" => {
                        patch.container_state = Some(ContainerStateAst::Locked);
                        patch.remove_container_state = false;
                    },
                    "transparentClosed" => {
                        patch.container_state = Some(ContainerStateAst::TransparentClosed);
                        patch.remove_container_state = false;
                    },
                    "transparentLocked" => {
                        patch.container_state = Some(ContainerStateAst::TransparentLocked);
                        patch.remove_container_state = false;
                    },
                    _ => return Err(AstError::Shape("invalid container state in item patch")),
                }
            },
            Rule::item_add_ability => {
                let mut inner = stmt.into_inner();
                let ability_pair = inner.next().ok_or(AstError::Shape("add ability missing ability id"))?;
                patch.add_abilities.push(parse_patch_ability(ability_pair)?);
            },
            Rule::item_remove_ability => {
                let mut inner = stmt.into_inner();
                let ability_pair = inner
                    .next()
                    .ok_or(AstError::Shape("remove ability missing ability id"))?;
                patch.remove_abilities.push(parse_patch_ability(ability_pair)?);
            },
            _ => return Err(AstError::Shape("unexpected statement in item patch block")),
        }
    }

    Ok((ActionAst::ModifyItem { item, patch }, consumed))
}

fn parse_modify_room_action(text: &str) -> Result<(ActionAst, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let rest0 = s
        .strip_prefix("do modify room ")
        .ok_or(AstError::Shape("not a modify room action"))?;
    let rest0 = rest0.trim_start();
    let ident_len = rest0.chars().take_while(|&c| is_ident_char(c)).count();
    if ident_len == 0 {
        return Err(AstError::Shape("modify room missing room identifier"));
    }
    let ident_str = &rest0[..ident_len];
    DslParser::parse(Rule::ident, ident_str).map_err(|_| AstError::Shape("modify room has invalid room identifier"))?;
    let room = ident_str.to_string();
    let after_ident = rest0[ident_len..].trim_start();
    if !after_ident.starts_with('{') {
        return Err(AstError::Shape("modify room missing '{' block"));
    }
    let block_slice = after_ident;
    let body = extract_body(block_slice)?;
    // SAFETY: `body` was produced by `extract_body` from `block_slice`, so both pointers
    // lie within the same string slice.
    let start_offset = unsafe { body.as_ptr().offset_from(block_slice.as_ptr()) as usize };
    let block_total_len = start_offset + body.len() + 1;
    let remaining = &block_slice[block_total_len..];
    let consumed = leading_ws + (s.len() - remaining.len());
    let patch_block = &block_slice[..block_total_len];

    let mut pairs = DslParser::parse(Rule::room_patch_block, patch_block).map_err(|e| AstError::Pest(e.to_string()))?;
    let block_pair = pairs
        .next()
        .ok_or(AstError::Shape("modify room block missing statements"))?;
    let mut patch = RoomPatchAst::default();

    for stmt in block_pair.into_inner() {
        match stmt.as_rule() {
            Rule::room_patch_name => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify room name missing value"))?
                    .as_str();
                patch.name = Some(unquote(val));
            },
            Rule::room_patch_desc => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify room description missing value"))?
                    .as_str();
                patch.desc = Some(unquote(val));
            },
            Rule::room_patch_remove_exit => {
                let mut inner = stmt.into_inner();
                let dest = inner
                    .next()
                    .ok_or(AstError::Shape("modify room remove exit missing destination"))?
                    .as_str()
                    .to_string();
                patch.remove_exits.push(dest);
            },
            Rule::room_patch_add_exit => {
                let mut inner = stmt.into_inner();
                let dir_pair = inner
                    .next()
                    .ok_or(AstError::Shape("modify room add exit missing direction"))?;
                let direction = if dir_pair.as_rule() == Rule::string {
                    unquote(dir_pair.as_str())
                } else {
                    dir_pair.as_str().to_string()
                };
                let dest_pair = inner
                    .next()
                    .ok_or(AstError::Shape("modify room add exit missing destination"))?;
                let to = dest_pair.as_str().to_string();
                let mut exit = RoomExitPatchAst {
                    direction,
                    to,
                    ..Default::default()
                };
                if let Some(opts) = inner.next() {
                    parse_room_patch_exit_opts(opts, &mut exit)?;
                }
                patch.add_exits.push(exit);
            },
            _ => return Err(AstError::Shape("unexpected statement in room patch block")),
        }
    }

    Ok((ActionAst::ModifyRoom { room, patch }, consumed))
}

fn parse_modify_npc_action(text: &str) -> Result<(ActionAst, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let rest0 = s
        .strip_prefix("do modify npc ")
        .ok_or(AstError::Shape("not a modify npc action"))?;
    let rest0 = rest0.trim_start();
    let ident_len = rest0.chars().take_while(|&c| is_ident_char(c)).count();
    if ident_len == 0 {
        return Err(AstError::Shape("modify npc missing npc identifier"));
    }
    let ident_str = &rest0[..ident_len];
    DslParser::parse(Rule::ident, ident_str).map_err(|_| AstError::Shape("modify npc has invalid npc identifier"))?;
    let npc = ident_str.to_string();
    let after_ident = rest0[ident_len..].trim_start();
    if !after_ident.starts_with('{') {
        return Err(AstError::Shape("modify npc missing '{' block"));
    }
    let block_slice = after_ident;
    let body = extract_body(block_slice)?;
    let start_offset = unsafe { body.as_ptr().offset_from(block_slice.as_ptr()) as usize };
    let block_total_len = start_offset + body.len() + 1;
    let remaining = &block_slice[block_total_len..];
    let consumed = leading_ws + (s.len() - remaining.len());
    let patch_block = &block_slice[..block_total_len];

    let mut pairs = DslParser::parse(Rule::npc_patch_block, patch_block).map_err(|e| AstError::Pest(e.to_string()))?;
    let block_pair = pairs
        .next()
        .ok_or(AstError::Shape("modify npc block missing statements"))?;
    let mut patch = NpcPatchAst::default();
    let mut movement_patch = NpcMovementPatchAst::default();
    let mut movement_touched = false;

    for stmt in block_pair.into_inner() {
        match stmt.as_rule() {
            Rule::npc_patch_name => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify npc name missing value"))?
                    .as_str();
                patch.name = Some(unquote(val));
            },
            Rule::npc_patch_desc => {
                let mut inner = stmt.into_inner();
                let val = inner
                    .next()
                    .ok_or(AstError::Shape("modify npc description missing value"))?
                    .as_str();
                patch.desc = Some(unquote(val));
            },
            Rule::npc_patch_state => {
                let mut inner = stmt.into_inner();
                let state_pair = inner.next().ok_or(AstError::Shape("modify npc state missing value"))?;
                patch.state = Some(parse_npc_state_value(state_pair)?);
            },
            Rule::npc_patch_add_line => {
                let mut inner = stmt.into_inner();
                let line_pair = inner
                    .next()
                    .ok_or(AstError::Shape("modify npc add line missing text"))?;
                let state_pair = inner
                    .next()
                    .ok_or(AstError::Shape("modify npc add line missing state"))?;
                patch.add_lines.push(NpcDialoguePatchAst {
                    line: unquote(line_pair.as_str()),
                    state: parse_npc_state_value(state_pair)?,
                });
            },
            Rule::npc_patch_route => {
                if movement_patch.random_rooms.is_some() {
                    return Err(AstError::Shape("modify npc cannot set both route and random rooms"));
                }
                let rooms = stmt.into_inner().map(|p| p.as_str().to_string()).collect::<Vec<_>>();
                movement_patch.route = Some(rooms);
                movement_patch.random_rooms = None;
                movement_touched = true;
            },
            Rule::npc_patch_random_rooms => {
                if movement_patch.route.is_some() {
                    return Err(AstError::Shape("modify npc cannot set both route and random rooms"));
                }
                let rooms = stmt.into_inner().map(|p| p.as_str().to_string()).collect::<Vec<_>>();
                movement_patch.random_rooms = Some(rooms);
                movement_patch.route = None;
                movement_touched = true;
            },
            Rule::npc_patch_timing_every => {
                let mut inner = stmt.into_inner();
                let num_pair = inner
                    .next()
                    .ok_or(AstError::Shape("modify npc timing every missing turns"))?;
                let turns: i64 = num_pair
                    .as_str()
                    .parse()
                    .map_err(|_| AstError::Shape("modify npc timing every invalid number"))?;
                if turns < 0 {
                    return Err(AstError::Shape("modify npc timing every requires non-negative turns"));
                }
                movement_patch.timing = Some(NpcTimingPatchAst::EveryNTurns(turns as usize));
                movement_touched = true;
            },
            Rule::npc_patch_timing_on => {
                let mut inner = stmt.into_inner();
                let num_pair = inner
                    .next()
                    .ok_or(AstError::Shape("modify npc timing on missing turn"))?;
                let turn: i64 = num_pair
                    .as_str()
                    .parse()
                    .map_err(|_| AstError::Shape("modify npc timing on invalid number"))?;
                if turn < 0 {
                    return Err(AstError::Shape("modify npc timing on requires non-negative turn"));
                }
                movement_patch.timing = Some(NpcTimingPatchAst::OnTurn(turn as usize));
                movement_touched = true;
            },
            Rule::npc_patch_active => {
                let mut inner = stmt.into_inner();
                let bool_pair = inner.next().ok_or(AstError::Shape("modify npc active missing value"))?;
                let val = match bool_pair.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(AstError::Shape("modify npc active expects true or false")),
                };
                movement_patch.active = Some(val);
                movement_touched = true;
            },
            Rule::npc_patch_loop => {
                let mut inner = stmt.into_inner();
                let bool_pair = inner.next().ok_or(AstError::Shape("modify npc loop missing value"))?;
                let val = match bool_pair.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(AstError::Shape("modify npc loop expects true or false")),
                };
                movement_patch.loop_route = Some(val);
                movement_touched = true;
            },
            _ => return Err(AstError::Shape("unexpected statement in npc patch block")),
        }
    }

    if movement_touched {
        patch.movement = Some(movement_patch);
    }

    Ok((ActionAst::ModifyNpc { npc, patch }, consumed))
}

fn parse_room_patch_exit_opts(opts: pest::iterators::Pair<Rule>, exit: &mut RoomExitPatchAst) -> Result<(), AstError> {
    debug_assert_eq!(opts.as_rule(), Rule::exit_opts);
    for opt in opts.into_inner() {
        let opt_text = opt.as_str().trim();
        if opt_text == "hidden" {
            exit.hidden = true;
            continue;
        }
        if opt_text == "locked" {
            exit.locked = true;
            continue;
        }
        let children: Vec<_> = opt.clone().into_inner().collect();
        if let Some(s) = children.iter().find(|p| p.as_rule() == Rule::string) {
            exit.barred_message = Some(unquote(s.as_str()));
            continue;
        }
        if opt_text.starts_with("required_items") && children.iter().all(|p| p.as_rule() == Rule::ident) {
            for item in children {
                exit.required_items.push(item.as_str().to_string());
            }
            continue;
        }
        if opt_text.starts_with("required_flags") {
            for flag_pair in opt.into_inner() {
                match flag_pair.as_rule() {
                    Rule::ident => exit.required_flags.push(flag_pair.as_str().to_string()),
                    Rule::flag_req => {
                        let mut it = flag_pair.into_inner();
                        let ident = it
                            .next()
                            .ok_or(AstError::Shape("flag requirement missing identifier"))?
                            .as_str()
                            .to_string();
                        let base = ident.split('#').next().unwrap_or(&ident).to_string();
                        exit.required_flags.push(base);
                    },
                    _ => {},
                }
            }
            continue;
        }
    }
    Ok(())
}

fn parse_npc_state_value(pair: pest::iterators::Pair<Rule>) -> Result<NpcStateValue, AstError> {
    match pair.as_rule() {
        Rule::npc_state_value => {
            let mut inner = pair.into_inner();
            let next = inner.next().ok_or(AstError::Shape("npc state missing value"))?;
            parse_npc_state_value(next)
        },
        Rule::npc_custom_state => {
            let mut inner = pair.into_inner();
            let ident = inner
                .next()
                .ok_or(AstError::Shape("custom npc state missing identifier"))?;
            Ok(NpcStateValue::Custom(ident.as_str().to_string()))
        },
        Rule::ident => Ok(NpcStateValue::Named(pair.as_str().to_string())),
        _ => Err(AstError::Shape("invalid npc state value")),
    }
}

fn parse_patch_ability(pair: pest::iterators::Pair<Rule>) -> Result<ItemAbilityAst, AstError> {
    debug_assert!(pair.as_rule() == Rule::ability_id);
    let raw = pair.as_str().trim();
    let ability: String = raw.chars().take_while(|c| !c.is_whitespace() && *c != '(').collect();
    if ability.is_empty() {
        return Err(AstError::Shape("ability name missing"));
    }
    let mut inner = pair.into_inner();
    let has_parens = raw.contains('(');
    let target = if ability == "Unlock" && has_parens {
        inner.next().map(|p| p.as_str().to_string())
    } else {
        None
    };
    Ok(ItemAbilityAst { ability, target })
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | ':' | '_' | '#')
}

fn parse_schedule_action(
    text: &str,
    source: &str,
    smap: &SourceMap,
    sets: &HashMap<String, Vec<String>>,
) -> Result<(ActionAst, usize), AstError> {
    let s = text.trim_start();
    let (rest0, is_in) = if let Some(r) = s.strip_prefix("do schedule in ") {
        (r, true)
    } else if let Some(r) = s.strip_prefix("do schedule on ") {
        (r, false)
    } else {
        return Err(AstError::Shape("not a schedule action"));
    };
    // parse number
    let mut idx = 0usize;
    while idx < rest0.len() && rest0.as_bytes()[idx].is_ascii_digit() {
        idx += 1;
    }
    if idx == 0 {
        return Err(AstError::Shape("schedule missing number"));
    }
    let num: usize = rest0[..idx]
        .parse()
        .map_err(|_| AstError::Shape("invalid schedule number"))?;
    let rest1 = &rest0[idx..].trim_start();
    // Find the opening brace of the block and capture header between number and '{'
    let brace_pos = rest1.find('{').ok_or(AstError::Shape("schedule missing '{'"))?;
    let header = rest1[..brace_pos].trim();

    let mut on_false = None;
    let mut note = None;
    let mut cond: Option<ConditionAst> = None;

    if let Some(hdr_after_if) = header.strip_prefix("if ") {
        // Conditional schedule path
        let onfalse_pos = hdr_after_if.find(" onFalse ");
        let note_pos = hdr_after_if.find(" note ");
        let mut cond_end = hdr_after_if.len();
        if let Some(p) = onfalse_pos {
            cond_end = cond_end.min(p);
        }
        if let Some(p) = note_pos {
            cond_end = cond_end.min(p);
        }
        let cond_str = hdr_after_if[..cond_end].trim();
        let extras = hdr_after_if[cond_end..].trim();
        if let Some(idx) = extras.find("onFalse ") {
            let after = &extras[idx + 8..];
            if after.starts_with("cancel") {
                on_false = Some(OnFalseAst::Cancel);
            } else if after.starts_with("retryNextTurn") {
                on_false = Some(OnFalseAst::RetryNextTurn);
            } else if let Some(tail) = after.strip_prefix("retryAfter ") {
                let mut k = 0;
                while k < tail.len() && tail.as_bytes()[k].is_ascii_digit() {
                    k += 1;
                }
                if k == 0 {
                    return Err(AstError::Shape("retryAfter missing turns"));
                }
                let turns: usize = tail[..k]
                    .parse()
                    .map_err(|_| AstError::Shape("invalid retryAfter turns"))?;
                on_false = Some(OnFalseAst::RetryAfter { turns });
            }
        }
        // Note can appear anywhere in header; parse from full header too
        if let Some(n) = extract_note(header) {
            note = Some(n);
        }
        cond = Some(parse_condition_text(cond_str, sets)?);
    } else {
        // Unconditional schedule path; allow optional note in header
        if !header.is_empty() {
            if let Some(n) = extract_note(header) {
                note = Some(n);
            } else {
                // Unknown tokens in header
                // Be forgiving: ignore whitespace-only; otherwise error
                if !header.trim().is_empty() {
                    return Err(AstError::Shape(
                        "unexpected schedule header; expected 'if ...' or 'note \"...\"'",
                    ));
                }
            }
        }
    }

    // Extract block body
    let mut p = brace_pos + 1;
    let bytes2 = rest1.as_bytes();
    let mut depth = 1i32;
    while p < rest1.len() {
        let c = bytes2[p] as char;
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        p += 1;
    }
    if depth != 0 {
        return Err(AstError::Shape("schedule block not closed"));
    }
    let inner_body = &rest1[brace_pos + 1..p];
    let actions = parse_actions_from_body(inner_body, source, smap, sets)?;
    let consumed = s.len() - rest1[p + 1..].len();

    let act = match (is_in, cond) {
        (true, Some(c)) => ActionAst::ScheduleInIf {
            turns_ahead: num,
            condition: Box::new(c),
            on_false,
            actions,
            note,
        },
        (false, Some(c)) => ActionAst::ScheduleOnIf {
            on_turn: num,
            condition: Box::new(c),
            on_false,
            actions,
            note,
        },
        (true, None) => ActionAst::ScheduleIn {
            turns_ahead: num,
            actions,
            note,
        },
        (false, None) => ActionAst::ScheduleOn {
            on_turn: num,
            actions,
            note,
        },
    };
    Ok((act, consumed))
}

#[allow(dead_code)]
fn strip_leading_ws_and_comments(s: &str) -> &str {
    let mut i = 0usize;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] as char == '#' {
            while i < bytes.len() && (bytes[i] as char) != '\n' {
                i += 1;
            }
            continue;
        }
        break;
    }
    &s[i..]
}

// ---------- Error mapping helpers ----------
struct SourceMap {
    line_starts: Vec<usize>,
    src: String,
}
impl SourceMap {
    fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                starts.push(i + 1);
            }
        }
        Self {
            line_starts: starts,
            src: source.to_string(),
        }
    }
    fn line_col(&self, offset: usize) -> (usize, usize) {
        let idx = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = *self.line_starts.get(idx).unwrap_or(&0);
        let line_no = idx + 1;
        let col = offset.saturating_sub(line_start) + 1;
        (line_no, col)
    }
    fn line_snippet(&self, line_no: usize) -> String {
        let start = *self.line_starts.get(line_no - 1).unwrap_or(&0);
        let end = *self.line_starts.get(line_no).unwrap_or(&self.src.len());
        self.src[start..end].trim_end_matches(['\r', '\n']).to_string()
    }
}

fn str_offset(full: &str, slice: &str) -> usize {
    (slice.as_ptr() as usize) - (full.as_ptr() as usize)
}

fn extract_note(header: &str) -> Option<String> {
    if let Some(idx) = header.find("note ") {
        let after = &header[idx + 5..];
        let trimmed = after.trim_start();
        if let Ok((n, _used)) = parse_string_at(trimmed) {
            return Some(n);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braces_in_strings_dont_break_body_scan() {
        let src = r#"
trigger "brace text" when always {
    do show "Shiny {curly} braces"
}
"#;
        parse_trigger(src).expect("should parse");
    }

    #[test]
    fn braces_in_comments_dont_break_body_scan() {
        let src = r#"
trigger "comment braces" when always {
    # { not a block } in comment
    do show "ok"
}
"#;
        parse_trigger(src).expect("should parse");
    }

    #[test]
    fn quoted_strings_support_common_escapes() {
        let src = r#"
trigger "He said:\n\"hi\"" when always {
    do show "Line1\nLine2"
    do npc says gonk "She replied: \"no\""
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert!(ast.name.contains('\n'));
        assert!(ast.name.contains('"'));
        // show contains a newline
        match &ast.actions[0] {
            ActionAst::Show(s) => {
                assert!(s.contains('\n'));
                assert_eq!(s, "Line1\nLine2");
            },
            _ => panic!("expected show"),
        }
        // npc says contains a quote
        match &ast.actions[1] {
            ActionAst::NpcSays { npc, quote } => {
                assert_eq!(npc, "gonk");
                assert!(quote.contains('"'));
            },
            _ => panic!("expected npc says"),
        }
        // TOML may re-escape newlines or include them directly; just ensure both parts appear
        let toml = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("Line1"));
        assert!(toml.contains("Line2"));
    }

    #[test]
    fn schedule_note_supports_escapes() {
        let src = r#"
trigger "note escapes" when always {
  do schedule in 1 note "lineA\nlineB" {
    do show "ok"
  }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        let t = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(t.contains("lineA"));
        assert!(t.contains("lineB"));
    }

    #[test]
    fn modify_item_parses_patch_fields() {
        let src = r#"
trigger "patch locker" when always {
    do modify item locker {
        name "Unlocked locker"
        description "It's open now"
        text "notes"
        portable false
        restricted true
        container state locked
        add ability Unlock ( secret-door )
        add ability Ignite
        remove ability Unlock ( secret-door )
        remove ability Unlock
    }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 1);
        let action = &ast.actions[0];
        match action {
            ActionAst::ModifyItem { item, patch } => {
                assert_eq!(item, "locker");
                assert_eq!(patch.name.as_deref(), Some("Unlocked locker"));
                assert_eq!(patch.desc.as_deref(), Some("It's open now"));
                assert_eq!(patch.text.as_deref(), Some("notes"));
                assert_eq!(patch.portable, Some(false));
                assert_eq!(patch.restricted, Some(true));
                assert_eq!(patch.container_state, Some(ContainerStateAst::Locked));
                assert!(!patch.remove_container_state);
                assert_eq!(patch.add_abilities.len(), 2);
                assert_eq!(patch.add_abilities[0].ability, "Unlock");
                assert_eq!(patch.add_abilities[0].target.as_deref(), Some("secret-door"));
                assert_eq!(patch.add_abilities[1].ability, "Ignite");
                assert!(patch.add_abilities[1].target.is_none());
                assert_eq!(patch.remove_abilities.len(), 2);
                assert_eq!(patch.remove_abilities[0].ability, "Unlock");
                assert_eq!(patch.remove_abilities[0].target.as_deref(), Some("secret-door"));
                assert_eq!(patch.remove_abilities[1].ability, "Unlock");
                assert!(patch.remove_abilities[1].target.is_none());
            },
            other => panic!("expected modify item action, got {other:?}"),
        }
        let toml = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"modifyItem\""));
        assert!(toml.contains("item_sym = \"locker\""));
        assert!(toml.contains("name = \"Unlocked locker\""));
        assert!(toml.contains("desc = \"It's open now\""));
        assert!(toml.contains("portable = false"));
        assert!(toml.contains("restricted = true"));
        assert!(toml.contains("container_state = \"locked\""));
        assert!(toml.contains("add_abilities = ["));
        assert!(toml.contains("remove_abilities = ["));
    }

    #[test]
    fn modify_room_parses_patch_fields() {
        let src = r#"
trigger "patch lab" when always {
    do modify room aperture-lab {
        name "Ruined Lab"
        desc "Charred and broken."
        remove exit portal-room
        add exit "through the vault door" -> stargate-room {
            locked,
            required_items (vault-key),
            required_flags (opened-vault),
            barred "You can't go that way yet."
        }
    }
}
"#;
        let offset = src.find("do modify room").expect("snippet find");
        let snippet = &src[offset..];
        let (helper_action, _used) = super::parse_modify_room_action(snippet).expect("parse helper on snippet");
        assert!(matches!(helper_action, ActionAst::ModifyRoom { .. }));
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 1);
        match &ast.actions[0] {
            ActionAst::ModifyRoom { room, patch } => {
                assert_eq!(room, "aperture-lab");
                assert_eq!(patch.name.as_deref(), Some("Ruined Lab"));
                assert_eq!(patch.desc.as_deref(), Some("Charred and broken."));
                assert_eq!(patch.remove_exits, vec!["portal-room"]);
                assert_eq!(patch.add_exits.len(), 1);
                let exit = &patch.add_exits[0];
                assert_eq!(exit.direction, "through the vault door");
                assert_eq!(exit.to, "stargate-room");
                assert!(exit.locked);
                assert!(!exit.hidden);
                assert_eq!(exit.required_items, vec!["vault-key"]);
                assert_eq!(exit.required_flags, vec!["opened-vault"]);
                assert_eq!(exit.barred_message.as_deref(), Some("You can't go that way yet."));
            },
            other => panic!("expected modify room action, got {other:?}"),
        }
        let toml = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"modifyRoom\""));
        assert!(toml.contains("room_sym = \"aperture-lab\""));
        assert!(toml.contains("remove_exits = [\"portal-room\"]"));
        assert!(toml.contains("add_exits = ["));
        assert!(toml.contains("barred_message = \"You can't go that way yet.\""));
        assert!(toml.contains("required_flags = [{ type = \"simple\", name = \"opened-vault\" }]"));
    }

    #[test]
    fn modify_npc_parses_patch_fields() {
        let src = r#"
trigger "patch emh" when always {
    do modify npc emh {
        name "Emergency Medical Hologram"
        desc "Program updated with bedside manner routines."
        state custom(patched)
        add line "Bedside manner protocols active." to state custom(patched)
        add line "Please state the nature of the medical emergency." to state normal
        route (sickbay, corridor)
        timing every 5 turns
        active false
        loop false
    }
}
"#;
        let offset = src.find("do modify npc").expect("snippet find");
        let snippet = &src[offset..];
        let (helper_action, _used) = super::parse_modify_npc_action(snippet).expect("parse helper on snippet");
        assert!(matches!(helper_action, ActionAst::ModifyNpc { .. }));
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 1);
        match &ast.actions[0] {
            ActionAst::ModifyNpc { npc, patch } => {
                assert_eq!(npc, "emh");
                assert_eq!(patch.name.as_deref(), Some("Emergency Medical Hologram"));
                assert_eq!(
                    patch.desc.as_deref(),
                    Some("Program updated with bedside manner routines.")
                );
                assert!(matches!(patch.state, Some(NpcStateValue::Custom(ref s)) if s == "patched"));
                assert_eq!(patch.add_lines.len(), 2);
                assert!(patch.add_lines.iter().any(
                    |entry| matches!(entry.state, NpcStateValue::Custom(ref s) if s == "patched")
                        && entry.line == "Bedside manner protocols active."
                ));
                assert!(patch.add_lines.iter().any(
                    |entry| matches!(entry.state, NpcStateValue::Named(ref s) if s == "normal")
                        && entry.line == "Please state the nature of the medical emergency."
                ));
                let movement = patch.movement.as_ref().expect("movement patch");
                assert_eq!(movement.route.as_deref().unwrap(), ["sickbay", "corridor"]);
                assert!(movement.random_rooms.is_none());
                assert_eq!(movement.active, Some(false));
                assert_eq!(movement.loop_route, Some(false));
                assert!(matches!(movement.timing, Some(NpcTimingPatchAst::EveryNTurns(5))));
            },
            other => panic!("expected modify npc action, got {other:?}"),
        }
        let toml = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("type = \"modifyNpc\""));
        assert!(toml.contains("npc_sym = \"emh\""));
        assert!(toml.contains("route = [\"sickbay\", \"corridor\"]"));
        assert!(toml.contains("type = \"everyNTurns\""));
        assert!(toml.contains("loop_route = false"));
    }

    #[test]
    fn modify_npc_supports_random_movement() {
        let src = r#"
trigger "patch guard" when always {
    do modify npc guard {
        random rooms (hall, foyer, atrium)
        timing on turn 12
        active true
    }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 1);
        match &ast.actions[0] {
            ActionAst::ModifyNpc { npc, patch } => {
                assert_eq!(npc, "guard");
                let movement = patch.movement.as_ref().expect("movement patch");
                assert!(movement.route.is_none());
                let mut rooms = movement.random_rooms.clone().expect("random rooms");
                rooms.sort();
                let expected = vec!["atrium".to_string(), "foyer".to_string(), "hall".to_string()];
                assert_eq!(rooms, expected);
                assert!(matches!(movement.timing, Some(NpcTimingPatchAst::OnTurn(12))));
                assert_eq!(movement.active, Some(true));
                assert!(movement.loop_route.is_none());
            },
            other => panic!("expected modify npc action, got {other:?}"),
        }
        let toml = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("random_rooms = [\"hall\", \"foyer\", \"atrium\"]"));
        assert!(toml.contains("type = \"onTurn\""));
    }

    #[test]
    fn parse_modify_room_action_helper_handles_basic_block() {
        let snippet = "do modify room lab { name \"Ruined\" }\n";
        let (action, used) = super::parse_modify_room_action(snippet).expect("parse helper");
        assert_eq!(&snippet[..used], "do modify room lab { name \"Ruined\" }");
        match action {
            ActionAst::ModifyRoom { room, patch } => {
                assert_eq!(room, "lab");
                assert_eq!(patch.name.as_deref(), Some("Ruined"));
            },
            other => panic!("expected modify room action, got {other:?}"),
        }
    }

    #[test]
    fn parse_modify_item_action_helper_handles_basic_block() {
        let snippet = "do modify item locker { name \"Ok\" }\n";
        let (action, used) = super::parse_modify_item_action(snippet).expect("parse helper");
        assert_eq!(&snippet[..used], "do modify item locker { name \"Ok\" }");
        match action {
            ActionAst::ModifyItem { item, patch } => {
                assert_eq!(item, "locker");
                assert_eq!(patch.name.as_deref(), Some("Ok"));
            },
            other => panic!("expected modify item action, got {other:?}"),
        }
    }

    #[test]
    fn modify_item_container_state_off_sets_flag() {
        let src = r#"
trigger "patch chest" when always {
    do modify item chest {
        container state off
    }
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        let action = ast.actions.first().expect("expected modify item action");
        match action {
            ActionAst::ModifyItem { item, patch } => {
                assert_eq!(item, "chest");
                assert!(patch.container_state.is_none());
                assert!(patch.remove_container_state);
            },
            other => panic!("expected modify item action, got {other:?}"),
        }
        let toml = crate::compile_trigger_to_toml(&ast).expect("compile ok");
        assert!(toml.contains("remove_container_state = true"));
        assert!(!toml.contains("container_state = \""));
    }

    #[test]
    fn raw_string_with_hash_quotes() {
        let src = "trigger r#\"raw name with \"quotes\"\"# when always {\n  do show r#\"He said \"hi\"\"#\n}\n";
        let asts = super::parse_program(src).expect("parse ok");
        assert!(!asts.is_empty());
        // Ensure value with embedded quotes is preserved (serializer may re-escape)
        let toml = crate::compile_trigger_to_toml(&asts[0]).expect("compile ok");
        assert!(toml.contains("He said"));
        assert!(toml.contains("hi"));
    }

    #[test]
    fn consumable_when_replace_inventory_matches_rule() {
        let mut pairs = DslParser::parse(
            Rule::consumable_when_consumed,
            "when_consumed replace inventory wrapper",
        )
        .expect("parse ok");
        let pair = pairs.next().expect("pair");
        assert_eq!(pair.as_rule(), Rule::consumable_when_consumed);
    }

    #[test]
    fn consumable_block_allows_replace_inventory() {
        let src = "consumable {\n  uses_left 2\n  when_consumed replace inventory wrapper\n}";
        let mut pairs = DslParser::parse(Rule::item_consumable, src).expect("parse ok");
        let pair = pairs.next().expect("pair");
        assert_eq!(pair.as_rule(), Rule::item_consumable);
        let mut inner = pair.into_inner();
        let block = inner.next().expect("block");
        assert_eq!(block.as_rule(), Rule::consumable_block);
        let mut block_inner = block.into_inner();
        let stmt = block_inner.next().expect("stmt");
        assert_eq!(stmt.as_rule(), Rule::consumable_stmt);
        assert_eq!(stmt.into_inner().next().expect("uses").as_rule(), Rule::consumable_uses);
        let stmt = block_inner.next().expect("stmt");
        assert_eq!(stmt.as_rule(), Rule::consumable_stmt);
        assert_eq!(
            stmt.into_inner().next().expect("when").as_rule(),
            Rule::consumable_when_consumed
        );
    }

    #[test]
    fn consumable_block_with_consume_on_and_when_consumed_parses() {
        let src = "consumable {\n  uses_left 1\n  consume_on ability Use\n  when_consumed replace inventory wrapper\n}";
        let mut pairs = DslParser::parse(Rule::item_consumable, src).expect("parse ok");
        let block = pairs.next().expect("pair").into_inner().next().expect("block");
        let mut inner = block.into_inner();
        let mut stmt = inner.next().expect("stmt");
        assert_eq!(stmt.as_rule(), Rule::consumable_stmt);
        assert_eq!(stmt.into_inner().next().expect("uses").as_rule(), Rule::consumable_uses);
        stmt = inner.next().expect("stmt");
        assert_eq!(stmt.as_rule(), Rule::consumable_stmt);
        assert_eq!(
            stmt.into_inner().next().expect("consume_on").as_rule(),
            Rule::consumable_consume_on
        );
        stmt = inner.next().expect("stmt");
        assert_eq!(stmt.as_rule(), Rule::consumable_stmt);
        assert_eq!(
            stmt.into_inner().next().expect("when").as_rule(),
            Rule::consumable_when_consumed
        );
    }

    #[test]
    fn consumable_consume_on_rule_parses() {
        let src = "consume_on ability Use";
        let mut pairs = DslParser::parse(Rule::consumable_consume_on, src).expect("parse ok");
        let pair = pairs.next().expect("pair");
        assert_eq!(pair.as_rule(), Rule::consumable_consume_on);
    }

    #[test]
    fn consumable_consume_on_does_not_consume_when_keyword() {
        let src = "consume_on ability Use when_consumed";
        let mut pairs = DslParser::parse(Rule::consumable_consume_on, src).expect("parse ok");
        let pair = pairs.next().expect("pair");
        // The rule should stop before the trailing keyword to allow the block to parse the next statement.
        assert_eq!(pair.as_str().trim_end(), "consume_on ability Use");
    }

    #[test]
    fn npc_movement_loop_flag_parses_and_compiles() {
        let src = r#"
npc bot {
  name "Maintenance Bot"
  desc "Keeps the corridors tidy."
  location room hub
  state idle
  movement route rooms (hub, hall) timing every_3_turns active true loop false
}
"#;
        let npcs = crate::parse_npcs(src).expect("parse npcs ok");
        assert_eq!(npcs.len(), 1);
        let movement = npcs[0].movement.as_ref().expect("movement present");
        assert_eq!(movement.loop_route, Some(false));

        let toml = crate::compile_npcs_to_toml(&npcs).expect("compile npcs");
        assert!(toml.contains("loop_route = false"));
    }

    #[test]
    fn item_with_consumable_parses() {
        let src = r#"item snack {
  name "Snack"
  desc "Yum"
  portable true
  location inventory player
  consumable {
    uses_left 1
    consume_on ability Use
    when_consumed replace inventory wrapper
  }
}
"#;
        DslParser::parse(Rule::item_def, src).expect("parse ok");
    }

    #[test]
    fn string_literals_preserve_utf8_characters() {
        let s = "\"Pilgrims Welcome – Pancakes\"";
        let parsed = parse_string(s).expect("parse ok");
        assert_eq!(parsed, "Pilgrims Welcome – Pancakes");

        let s2 = "\"It’s fine\"";
        let parsed2 = parse_string(s2).expect("parse ok");
        assert_eq!(parsed2, "It’s fine");
    }

    #[test]
    fn reserved_keywords_are_excluded_from_ident() {
        // Using a keyword as an identifier should fail to parse
        let src = r#"
trigger "bad ident" when enter room trigger {
  do show "won't get here"
}
"#;
        let err = parse_trigger(src).expect_err("expected parse failure");
        match err {
            AstError::Pest(_) | AstError::Shape(_) | AstError::ShapeAt { .. } => {},
        }
    }
}
/// Composite AST collections returned by [`parse_program_full`].
pub type ProgramAstBundle = (
    Vec<TriggerAst>,
    Vec<RoomAst>,
    Vec<ItemAst>,
    Vec<SpinnerAst>,
    Vec<NpcAst>,
    Vec<GoalAst>,
);
