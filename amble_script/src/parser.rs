use pest::Parser;
use pest_derive::Parser as PestParser;

use crate::{ActionAst, ConditionAst, OnFalseAst, RoomAst, TriggerAst};
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
pub fn parse_trigger(source: &str) -> Result<TriggerAst, AstError> {
    let v = parse_program(source)?;
    v.into_iter().next().ok_or(AstError::Shape("no trigger found"))
}

/// Parse multiple triggers from a full source file (triggers only view).
pub fn parse_program(source: &str) -> Result<Vec<TriggerAst>, AstError> {
    let (trigs, _rooms) = parse_program_full(source)?;
    Ok(trigs)
}

/// Parse a full program returning both triggers and rooms.
pub fn parse_program_full(source: &str) -> Result<(Vec<TriggerAst>, Vec<RoomAst>), AstError> {
    let mut pairs = DslParser::parse(Rule::program, source).map_err(|e| AstError::Pest(e.to_string()))?;
    let pair = pairs.next().ok_or(AstError::Shape("expected program"))?;
    let smap = SourceMap::new(source);
    let mut sets: HashMap<String, Vec<String>> = HashMap::new();
    let mut trigger_pairs = Vec::new();
    let mut room_pairs = Vec::new();
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
    Ok((out, rooms))
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
        // Top-level do schedule ... or do ... line
        if inner[i..].starts_with("do schedule in ") || inner[i..].starts_with("do schedule on ") {
            let (actions, used) = parse_schedule_with_if_blocks(&inner[i..], source, smap, sets)?;
            unconditional_actions.extend(actions);
            i += used;
            continue;
        }
        if inner[i..].starts_with("do ") {
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
    let src_line = src_line;
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
                let dir = it.next().ok_or(AstError::Shape("exit direction"))?.as_str().to_string();
                let to = it.next().ok_or(AstError::Shape("exit destination"))?.as_str().to_string();
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
                            if opt_text == "hidden" { hidden = true; continue; }
                            if opt_text == "locked" { locked = true; continue; }

                            // pull children
                            let children: Vec<_> = opt.clone().into_inner().collect();
                            // barred <string>
                            if let Some(s) = children.iter().find(|p| p.as_rule() == Rule::string) {
                                barred_message = Some(unquote(s.as_str()));
                                continue;
                            }
                            // required_items(...): list of idents only
                            if children.iter().all(|p| p.as_rule() == Rule::ident) && opt_text.starts_with("required_items") {
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
                                            let ident = itf.next().ok_or(AstError::Shape("flag ident"))?.as_str().to_string();
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
                exits.push((dir, crate::ExitAst { to, hidden, locked, barred_message, required_flags, required_items }));
            },
            Rule::overlay_stmt => {
                // overlay if <cond_list> { text "..." }
                let mut it = inner_stmt.into_inner();
                // First group: overlay_cond_list
                let conds_pair = it.next().ok_or(AstError::Shape("overlay cond list"))?;
                let mut conds = Vec::new();
                for cp in conds_pair.into_inner() {
                    if cp.as_rule() != Rule::overlay_cond { continue; }
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
                    if let Some(_) = text.strip_prefix("player has item ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::PlayerHasItem(item));
                        continue;
                    }
                    if let Some(_) = text.strip_prefix("player missing item ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::PlayerMissingItem(item));
                        continue;
                    }
                    if let Some(_) = text.strip_prefix("npc present ") {
                        let npc = kids.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::NpcPresent(npc));
                        continue;
                    }
                    if let Some(_) = text.strip_prefix("npc absent ") {
                        let npc = kids.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::NpcAbsent(npc));
                        continue;
                    }
                    if let Some(_) = text.strip_prefix("npc in state ") {
                        let npc = kids.next().ok_or(AstError::Shape("npc id"))?.as_str().to_string();
                        let nxt = kids.next().ok_or(AstError::Shape("state token"))?;
                        let oc = match nxt.as_rule() {
                            Rule::ident => {
                                crate::OverlayCondAst::NpcInState { npc, state: crate::NpcStateValue::Named(nxt.as_str().to_string()) }
                            },
                            Rule::string => {
                                crate::OverlayCondAst::NpcInState { npc, state: crate::NpcStateValue::Custom(unquote(nxt.as_str())) }
                            },
                            _ => {
                                let mut sub = nxt.into_inner();
                                let sval = sub.next().ok_or(AstError::Shape("custom string"))?;
                                crate::OverlayCondAst::NpcInState { npc, state: crate::NpcStateValue::Custom(unquote(sval.as_str())) }
                            },
                        };
                        conds.push(oc);
                        continue;
                    }
                    if let Some(_) = text.strip_prefix("item in room ") {
                        let item = kids.next().ok_or(AstError::Shape("item id"))?.as_str().to_string();
                        let room = kids.next().ok_or(AstError::Shape("room id"))?.as_str().to_string();
                        conds.push(crate::OverlayCondAst::ItemInRoom { item, room });
                        continue;
                    }
                    // Unknown overlay condition; ignore silently per current behavior
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
                overlays.push(crate::OverlayAst { conditions: conds, text: txt });
            },
            _ => {},
        }
    }
    let name = name.ok_or(AstError::Shape("room missing name"))?;
    let desc = desc.ok_or(AstError::Shape("room missing desc"))?;
    Ok(RoomAst { id, name, desc, visited: visited.unwrap_or(false), exits, overlays, src_line })
}

/// Parse only rooms from a source (helper/testing).
pub fn parse_rooms(source: &str) -> Result<Vec<RoomAst>, AstError> {
    let (_, rooms) = parse_program_full(source)?;
    Ok(rooms)
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
            let ch = b[i] as char;
            i += 1;
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
        while i < b.len() {
            let ch = b[i] as char;
            i += 1;
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
    while i < b.len() {
        let ch = b[i] as char;
        i += 1;
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
            } else {
                if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    in_str = false;
                }
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
                if at_line_start { in_comment = true; }
                at_line_start = false;
            },
            '{' => {
                if depth == 0 { start = Some(i + 1); }
                depth += 1;
                at_line_start = false;
            },
            '}' => {
                depth -= 1;
                if depth == 0 { end = Some(i); break; }
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
    if let Some(rest) = t.strip_prefix("do show ") {
        return Ok(ActionAst::Show(super::parser::unquote(rest.trim())));
    }
    if let Some(rest) = t.strip_prefix("do add wedge ") {
        // do add wedge "text" width <n> spinner <ident>
        let r = rest.trim();
        let (text, used) = parse_string_at(r).map_err(|_| AstError::Shape("add wedge missing or invalid quote"))?;
        let after = r[used..].trim_start();
        let after = after
            .strip_prefix("width ")
            .ok_or(AstError::Shape("add wedge missing 'width'"))?;
        let mut j = 0usize;
        while j < after.len() && after.as_bytes()[j].is_ascii_digit() {
            j += 1;
        }
        if j == 0 {
            return Err(AstError::Shape("add wedge missing width number"));
        }
        let width: usize = after[..j].parse().map_err(|_| AstError::Shape("invalid wedge width"))?;
        let after2 = after[j..].trim_start();
        let spinner = after2
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
            return Ok(ActionAst::LockExit {
                from_room: from.trim().to_string(),
                direction: tail.trim().to_string(),
            });
        }
    }
    if let Some(rest) = t.strip_prefix("do unlock exit from ") {
        if let Some((from, tail)) = rest.split_once(" direction ") {
            return Ok(ActionAst::UnlockExit {
                from_room: from.trim().to_string(),
                direction: tail.trim().to_string(),
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
                return Ok(ActionAst::RevealExit {
                    exit_from: from.trim().to_string(),
                    exit_to: to.trim().to_string(),
                    direction: dir_tail.trim().to_string(),
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
        let mut parts = rest.trim().split_whitespace();
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
        if body[i..].starts_with("do schedule in ") || body[i..].starts_with("do schedule on ") {
            let (actions, used) = parse_schedule_with_if_blocks(&body[i..], source, smap, sets)?;
            out.extend(actions);
            i += used;
            continue;
        }
        if body[i..].starts_with("do ") {
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

    if header.starts_with("if ") {
        // Conditional schedule path
        let hdr_after_if = &header[3..];
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

fn parse_schedule_with_if_blocks(
    text: &str,
    source: &str,
    smap: &SourceMap,
    sets: &HashMap<String, Vec<String>>,
) -> Result<(Vec<ActionAst>, usize), AstError> {
    // First, check if this schedule block contains if-blocks by looking at the body
    let s = text.trim_start();
    let (rest0, is_in) = if let Some(r) = s.strip_prefix("do schedule in ") {
        (r, true)
    } else if let Some(r) = s.strip_prefix("do schedule on ") {
        (r, false)
    } else {
        return Err(AstError::Shape("not a schedule action"));
    };

    // Parse number
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

    // If there's already a condition in the header, just delegate to the regular parser
    if header.starts_with("if ") {
        return parse_schedule_action(text, source, smap, sets).map(|(act, used)| (vec![act], used));
    }

    // Parse note from header if present
    let note = if !header.is_empty() { extract_note(header) } else { None };

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
    let consumed = s.len() - rest1[p + 1..].len();

    // Check if the body actually contains if-statements
    if !inner_body.contains("if ") {
        // No if-statements, delegate to regular parser
        return parse_schedule_action(text, source, smap, sets).map(|(act, used)| (vec![act], used));
    }

    // Debug output

    // Parse the body and separate if-blocks from unconditional actions
    let mut result_actions = Vec::new();
    let mut unconditional_lines = Vec::new();

    let lines: Vec<&str> = inner_body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("if ") {
            // Parse if block
            let mut if_condition = line[3..].trim();
            let mut j = i + 1;
            let mut brace_found = false;

            // Look for opening brace (might be on same line or next line)
            if let Some(brace_pos) = if_condition.find('{') {
                if_condition = &if_condition[..brace_pos].trim();
                brace_found = true;
            } else {
                // Look for brace on subsequent lines
                while j < lines.len() && !brace_found {
                    let next_line = lines[j].trim();
                    if next_line == "{" {
                        brace_found = true;
                        j += 1;
                        break;
                    }
                    j += 1;
                }
            }

            if !brace_found {
                return Err(AstError::Shape("if block missing opening brace"));
            }

            // Parse the condition
            let condition = parse_condition_text(if_condition, sets)?;

            // Collect the body of the if block
            let mut if_body_lines = Vec::new();
            let mut brace_count = 1;

            while j < lines.len() && brace_count > 0 {
                let block_line = lines[j];
                for ch in block_line.chars() {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                    }
                }
                if brace_count > 0 {
                    if_body_lines.push(block_line);
                }
                j += 1;
            }

            // Parse actions from the if body
            let if_body_text = if_body_lines.join("\n");
            let if_actions = parse_actions_from_body(&if_body_text, source, smap, sets)?;

            // Create conditional schedule action
            let schedule_action = match is_in {
                true => ActionAst::ScheduleInIf {
                    turns_ahead: num,
                    condition: Box::new(condition),
                    on_false: Some(OnFalseAst::Cancel), // Default policy
                    actions: if_actions,
                    note: note.clone(),
                },
                false => ActionAst::ScheduleOnIf {
                    on_turn: num,
                    condition: Box::new(condition),
                    on_false: Some(OnFalseAst::Cancel), // Default policy
                    actions: if_actions,
                    note: note.clone(),
                },
            };
            result_actions.push(schedule_action);
            i = j;
            continue;
        }

        // Regular do line
        if line.starts_with("do ") && !line.starts_with("do schedule") {
            unconditional_lines.push(line);
        }

        i += 1;
    }

    // If we have unconditional actions, create an unconditional schedule action
    if !unconditional_lines.is_empty() {
        let mut unconditional_actions = Vec::new();
        for line in unconditional_lines {
            let action = parse_action_from_str(line)?;
            unconditional_actions.push(action);
        }

        let schedule_action = match is_in {
            true => ActionAst::ScheduleIn {
                turns_ahead: num,
                actions: unconditional_actions,
                note: note.clone(),
            },
            false => ActionAst::ScheduleOn {
                on_turn: num,
                actions: unconditional_actions,
                note: note.clone(),
            },
        };
        result_actions.push(schedule_action);
    }

    Ok((result_actions, consumed))
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
