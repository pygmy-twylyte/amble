use std::collections::HashMap;

use crate::{ActionStmt, ConditionAst, IngestModeAst, TriggerAst};

use super::actions::{
    parse_action_from_str, parse_actions_from_body, parse_modify_item_action, parse_modify_npc_action,
    parse_modify_room_action, parse_schedule_action,
};
use super::conditions::parse_condition_text;
use super::helpers::{SourceMap, extract_body, str_offset, unquote};
use super::{AstError, Rule};

pub(super) fn parse_trigger_pair(
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
        Rule::player_death => ConditionAst::PlayerDeath,
        Rule::npc_death => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("npc death ident"))?.as_str().to_string();
            ConditionAst::NpcDeath(ident)
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
    let mut unconditional_actions: Vec<ActionStmt> = Vec::new();
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
        match parse_schedule_action(remainder, source, smap, sets) {
            Ok((action, used)) => {
                unconditional_actions.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a schedule action")) => {},
            Err(e) => return Err(e),
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
