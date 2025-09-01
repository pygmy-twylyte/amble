use pest::Parser;
use pest_derive::Parser as PestParser;

use crate::{ActionAst, ConditionAst, OnFalseAst, TriggerAst};

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
}

/// Parse a single trigger source string; returns the first trigger found.
pub fn parse_trigger(source: &str) -> Result<TriggerAst, AstError> {
    let mut v = parse_program(source)?;
    v.into_iter().next().ok_or(AstError::Shape("no trigger found"))
}

/// Parse multiple triggers from a full source file.
pub fn parse_program(source: &str) -> Result<Vec<TriggerAst>, AstError> {
    let mut pairs = DslParser::parse(Rule::program, source).map_err(|e| AstError::Pest(e.to_string()))?;
    let pair = pairs.next().ok_or(AstError::Shape("expected program"))?;
    let mut out = Vec::new();
    for trig in pair.into_inner() {
        if trig.as_rule() != Rule::trigger { continue; }
        out.push(parse_trigger_pair(trig)?);
    }
    Ok(out)
}

fn parse_trigger_pair(trig: pest::iterators::Pair<Rule>) -> Result<TriggerAst, AstError> {
    let mut it = trig.into_inner();

    // trigger -> "trigger" ~ quoted ~ [only once]? ~ "when" ~ when_cond ~ block
    let q = it.next().ok_or(AstError::Shape("expected trigger name"))?;
    if q.as_rule() != Rule::quoted {
        return Err(AstError::Shape("expected quoted trigger name"));
    }
    let name = unquote(q.as_str());

    // optional only once
    let mut only_once = false;
    let mut next_pair = it.next().ok_or(AstError::Shape("expected when/only once"))?;
    let mut when = if next_pair.as_rule() == Rule::only_once_kw {
        only_once = true;
        it.next().ok_or(AstError::Shape("expected when condition"))?
    } else {
        next_pair
    };
    if when.as_rule() == Rule::when_cond {
        when = when.into_inner().next().ok_or(AstError::Shape("empty when_cond"))?;
    }
    let event = match when.as_rule() {
        Rule::enter_room => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("enter room ident"))?.as_str().to_string();
            ConditionAst::EnterRoom(ident)
        }
        Rule::take_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("take item ident"))?.as_str().to_string();
            ConditionAst::TakeItem(ident)
        }
        Rule::talk_to_npc => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("talk npc ident"))?.as_str().to_string();
            ConditionAst::TalkToNpc(ident)
        }
        Rule::open_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("open item ident"))?.as_str().to_string();
            ConditionAst::OpenItem(ident)
        }
        Rule::leave_room => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("leave room ident"))?.as_str().to_string();
            ConditionAst::LeaveRoom(ident)
        }
        Rule::look_at_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("look at item ident"))?.as_str().to_string();
            ConditionAst::LookAtItem(ident)
        }
        Rule::use_item => {
            let mut i = when.into_inner();
            let item = i.next().ok_or(AstError::Shape("use item ident"))?.as_str().to_string();
            let ability = i.next().ok_or(AstError::Shape("use item ability"))?.as_str().to_string();
            ConditionAst::UseItem { item, ability }
        }
        Rule::give_to_npc => {
            let mut i = when.into_inner();
            let item = i.next().ok_or(AstError::Shape("give item ident"))?.as_str().to_string();
            let npc = i.next().ok_or(AstError::Shape("give to npc ident"))?.as_str().to_string();
            ConditionAst::GiveToNpc { item, npc }
        }
        Rule::use_item_on_item => {
            let mut i = when.into_inner();
            let tool = i.next().ok_or(AstError::Shape("use tool ident"))?.as_str().to_string();
            let target = i.next().ok_or(AstError::Shape("use target ident"))?.as_str().to_string();
            let interaction = i.next().ok_or(AstError::Shape("use interaction ident"))?.as_str().to_string();
            ConditionAst::UseItemOnItem { tool, target, interaction }
        }
        Rule::act_on_item => {
            let mut i = when.into_inner();
            let action = i.next().ok_or(AstError::Shape("act interaction ident"))?.as_str().to_string();
            let target = i.next().ok_or(AstError::Shape("act target ident"))?.as_str().to_string();
            ConditionAst::ActOnItem { target, action }
        }
        Rule::take_from_npc => {
            let mut i = when.into_inner();
            let item = i.next().ok_or(AstError::Shape("take-from item ident"))?.as_str().to_string();
            let npc = i.next().ok_or(AstError::Shape("take-from npc ident"))?.as_str().to_string();
            ConditionAst::TakeFromNpc { item, npc }
        }
        Rule::insert_item_into => {
            let mut i = when.into_inner();
            let item = i.next().ok_or(AstError::Shape("insert item ident"))?.as_str().to_string();
            let container = i.next().ok_or(AstError::Shape("insert into container ident"))?.as_str().to_string();
            ConditionAst::InsertItemInto { item, container }
        }
        Rule::drop_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("drop item ident"))?.as_str().to_string();
            ConditionAst::DropItem(ident)
        }
        Rule::unlock_item => {
            let mut i = when.into_inner();
            let ident = i.next().ok_or(AstError::Shape("unlock item ident"))?.as_str().to_string();
            ConditionAst::UnlockItem(ident)
        }
        _ => return Err(AstError::Shape("unknown when condition")),
    };

    let block = it.next().ok_or(AstError::Shape("expected block"))?;
    if block.as_rule() != Rule::block {
        return Err(AstError::Shape("expected block"));
    }

    let mut conditions = Vec::new();
    let mut actions = Vec::new();

    // block can be either an if_block or a sequence of do_stmt without conditions
    let block_src = block.as_str();
    let inner = extract_body(block_src)?;
    let leading = strip_leading_ws_and_comments(inner);
    if leading.starts_with("if ") {
        let if_pos = inner.find("if ").ok_or(AstError::Shape("missing 'if'"))?;
        let brace_pos = inner[if_pos..].find('{').ok_or(AstError::Shape("missing '{' after if"))? + if_pos;
        let cond_text = &inner[if_pos + 3..brace_pos];
        conditions.push(parse_condition_text(cond_text.trim())?);
        let if_block_src = &inner[if_pos..];
        let body = extract_body(if_block_src)?;
        actions = parse_actions_from_body(body)?;
    } else {
        actions = parse_actions_from_body(inner)?;
    }

    Ok(TriggerAst { name, event, conditions, actions, only_once })
}

fn unquote(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

fn extract_body(src: &str) -> Result<&str, AstError> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut start = None;
    let mut end = None;
    for (i, &b) in bytes.iter().enumerate() {
        let c = b as char;
        if c == '{' {
            if depth == 0 { start = Some(i + 1); }
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 { end = Some(i); break; }
        }
    }
    let s = start.ok_or(AstError::Shape("missing '{' body start"))?;
    let e = end.ok_or(AstError::Shape("missing '}' body end"))?;
    Ok(&src[s..e])
}

fn parse_condition_text(text: &str) -> Result<ConditionAst, AstError> {
    let t = text.trim();
    if let Some(inner) = t.strip_prefix("all(") {
        let inner = inner.strip_suffix(')').ok_or(AstError::Shape("all() close"))?;
        let parts = split_top_level_commas(inner);
        let mut kids = Vec::new();
        for p in parts { kids.push(parse_condition_text(p)?); }
        return Ok(ConditionAst::All(kids));
    }
    if let Some(inner) = t.strip_prefix("any(") {
        let inner = inner.strip_suffix(')').ok_or(AstError::Shape("any() close"))?;
        let parts = split_top_level_commas(inner);
        let mut kids = Vec::new();
        for p in parts { kids.push(parse_condition_text(p)?); }
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
            let item = rest[space+1..].trim();
            return Ok(ConditionAst::NpcHasItem { npc: npc.to_string(), item: item.to_string() });
        }
        return Err(AstError::Shape("npc has item syntax"));
    }
    if let Some(rest) = t.strip_prefix("npc in state ") {
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let state = rest[space+1..].trim();
            return Ok(ConditionAst::NpcInState { npc: npc.to_string(), state: state.to_string() });
        }
        return Err(AstError::Shape("npc in state syntax"));
    }
    if let Some(rest) = t.strip_prefix("container has item ") {
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let container = &rest[..space];
            let item = rest[space+1..].trim();
            return Ok(ConditionAst::ContainerHasItem { container: container.to_string(), item: item.to_string() });
        }
        return Err(AstError::Shape("container has item syntax"));
    }
    if let Some(rest) = t.strip_prefix("ambient ") {
        let rest = rest.trim();
        if let Some(idx) = rest.find(" in rooms ") {
            let spinner = rest[..idx].trim().to_string();
            let list = rest[idx+10..].trim();
            let rooms: Vec<String> = list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            return Ok(ConditionAst::Ambient { spinner, rooms: Some(rooms) });
        } else {
            return Ok(ConditionAst::Ambient { spinner: rest.to_string(), rooms: None });
        }
    }
    if let Some(rest) = t.strip_prefix("player in room ") {
        return Ok(ConditionAst::PlayerInRoom(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("chance ") {
        let rest = rest.trim();
        let num = rest.strip_suffix('%').ok_or(AstError::Shape("chance percent %"))?;
        let pct: f64 = num.trim().parse().map_err(|_| AstError::Shape("invalid chance percent"))?;
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
                out.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
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
    if let Some(rest) = t.strip_prefix("do add flag ") {
        return Ok(ActionAst::AddFlag(rest.trim().to_string()));
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
    if let Some(rest) = t.strip_prefix("do spawn item ") {
        // format: <item> into room <room>
        let rest = rest.trim();
        if let Some((item, tail)) = rest.split_once(" into room ") {
            return Ok(ActionAst::SpawnItemIntoRoom { item: item.trim().to_string(), room: tail.trim().to_string() });
        }
        if let Some((item, tail)) = rest.split_once(" into container ") {
            return Ok(ActionAst::SpawnItemInContainer { item: item.trim().to_string(), container: tail.trim().to_string() });
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
    if let Some(rest) = t.strip_prefix("do lock exit from ") {
        if let Some((from, tail)) = rest.split_once(" direction ") {
            return Ok(ActionAst::LockExit { from_room: from.trim().to_string(), direction: tail.trim().to_string() });
        }
    }
    if let Some(rest) = t.strip_prefix("do unlock exit from ") {
        if let Some((from, tail)) = rest.split_once(" direction ") {
            return Ok(ActionAst::UnlockExit { from_room: from.trim().to_string(), direction: tail.trim().to_string() });
        }
    }
    if let Some(rest) = t.strip_prefix("do reveal exit from ") {
        // format: <from> to <to> direction <dir>
        if let Some((from, tail)) = rest.split_once(" to ") {
            if let Some((to, dir_tail)) = tail.split_once(" direction ") {
                return Ok(ActionAst::RevealExit { exit_from: from.trim().to_string(), exit_to: to.trim().to_string(), direction: dir_tail.trim().to_string() });
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
            if let Some(r) = txt.strip_prefix('"') {
                if let Some(endq) = r.find('"') {
                    return Ok(ActionAst::SetItemDescription { item: item.to_string(), text: r[..endq].to_string() });
                }
            }
        }
        return Err(AstError::Shape("set item description syntax"));
    }
    if let Some(rest) = t.strip_prefix("do npc says ") {
        // format: <npc> "quote"
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let txt = rest[space..].trim();
            if let Some(r) = txt.strip_prefix('"') {
                if let Some(endq) = r.find('"') { return Ok(ActionAst::NpcSays { npc: npc.to_string(), quote: r[..endq].to_string() }); }
            }
        }
        return Err(AstError::Shape("npc says syntax"));
    }
    if let Some(rest) = t.strip_prefix("do npc says random ") {
        return Ok(ActionAst::NpcSaysRandom { npc: rest.trim().to_string() });
    }
    if let Some(rest) = t.strip_prefix("do set npc state ") {
        // format: <npc> <state>
        let rest = rest.trim();
        if let Some(space) = rest.find(' ') {
            let npc = &rest[..space];
            let state = rest[space+1..].trim();
            return Ok(ActionAst::SetNpcState { npc: npc.to_string(), state: state.to_string() });
        }
        return Err(AstError::Shape("set npc state syntax"));
    }
    if let Some(rest) = t.strip_prefix("do deny read ") {
        if let Some(r) = rest.trim().strip_prefix('"') { if let Some(endq) = r.find('"') { return Ok(ActionAst::DenyRead(r[..endq].to_string())); } }
        return Err(AstError::Shape("deny read syntax"));
    }
    if let Some(rest) = t.strip_prefix("do restrict item ") {
        return Ok(ActionAst::RestrictItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do schedule in ") {
        // maybe without condition; check for 'if'
        let rest = rest.trim();
        let mut idx = 0usize; while idx < rest.len() && rest.as_bytes()[idx].is_ascii_digit() { idx += 1; }
        if idx == 0 { return Err(AstError::Shape("schedule missing number")); }
        let num: usize = rest[..idx].parse().map_err(|_| AstError::Shape("invalid schedule number"))?;
        let rest2 = rest[idx..].trim_start();
        if let Some(r) = rest2.strip_prefix("if ") { /* handled by parse_schedule_action */ }
        else if let Some(brace) = rest2.find('{') {
            let body = &rest2[brace..];
            if let Ok((ActionAst::ScheduleInIf { actions, note, .. }, used)) = parse_schedule_action(&format!("do schedule in {} if chance 100% {}", num, body)) {
                return Ok(ActionAst::ScheduleIn { turns_ahead: num, actions, note });
            }
        }
    }
    if let Some(rest) = t.strip_prefix("do schedule on ") {
        let rest = rest.trim();
        let mut idx = 0usize; while idx < rest.len() && rest.as_bytes()[idx].is_ascii_digit() { idx += 1; }
        if idx == 0 { return Err(AstError::Shape("schedule missing number")); }
        let num: usize = rest[..idx].parse().map_err(|_| AstError::Shape("invalid schedule number"))?;
        let rest2 = rest[idx..].trim_start();
        if let Some(r) = rest2.strip_prefix("if ") { /* handled later */ }
        else if let Some(brace) = rest2.find('{') {
            let body = &rest2[brace..];
            if let Ok((ActionAst::ScheduleOnIf { actions, note, .. }, used)) = parse_schedule_action(&format!("do schedule on {} if chance 100% {}", num, body)) {
                return Ok(ActionAst::ScheduleOn { on_turn: num, actions, note });
            }
        }
    }
    if let Some(rest) = t.strip_prefix("do despawn item ") {
        return Ok(ActionAst::DespawnItem(rest.trim().to_string()));
    }
    if let Some(rest) = t.strip_prefix("do award points ") {
        let n: i64 = rest.trim().parse().map_err(|_| AstError::Shape("invalid points number"))?;
        return Ok(ActionAst::AwardPoints(n));
    }
    Err(AstError::Shape("unknown action"))
}


fn parse_actions_from_body(body: &str) -> Result<Vec<ActionAst>, AstError> {
    let mut out = Vec::new();
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
        if i >= bytes.len() { break; }
        if bytes[i] as char == '#' { while i < bytes.len() && (bytes[i] as char) != '\n' { i += 1; } continue; }
        if body[i..].starts_with("do schedule in ") || body[i..].starts_with("do schedule on ") {
            let (act, used) = parse_schedule_action(&body[i..])?;
            out.push(act);
            i += used;
            continue;
        }
        if body[i..].starts_with("do ") {
            let mut j = i; while j < bytes.len() && (bytes[j] as char) != '\n' { j += 1; }
            let line = body[i..j].trim_end();
            out.push(parse_action_from_str(line)?);
            i = j; continue;
        }
        while i < bytes.len() && (bytes[i] as char) != '\n' { i += 1; }
    }
    Ok(out)
}

fn parse_schedule_action(text: &str) -> Result<(ActionAst, usize), AstError> {
    let s = text.trim_start();
    let (rest, is_in) = if let Some(r) = s.strip_prefix("do schedule in ") { (r, true) } else if let Some(r) = s.strip_prefix("do schedule on ") { (r, false) } else { return Err(AstError::Shape("not a schedule action")); };
    let mut idx = 0usize; while idx < rest.len() && rest.as_bytes()[idx].is_ascii_digit() { idx += 1; }
    if idx == 0 { return Err(AstError::Shape("schedule missing number")); }
    let num: usize = rest[..idx].parse().map_err(|_| AstError::Shape("invalid schedule number"))?;
    let rest = &rest[idx..].trim_start();
    let rest = rest.strip_prefix("if ").ok_or(AstError::Shape("schedule missing 'if'"))?;
    let brace_pos = rest.find('{').ok_or(AstError::Shape("schedule missing '{'"))?;
    let header = &rest[..brace_pos].trim();
    let mut on_false = None; let mut note = None;
    let onfalse_pos = header.find(" onFalse ");
    let note_pos = header.find(" note ");
    let mut cond_end = header.len();
    if let Some(p) = onfalse_pos { cond_end = cond_end.min(p); }
    if let Some(p) = note_pos { cond_end = cond_end.min(p); }
    let cond_str = header[..cond_end].trim();
    let extras = header[cond_end..].trim();
    if let Some(idx) = extras.find("onFalse ") {
        let after = &extras[idx+8..];
        if after.starts_with("cancel") { on_false = Some(OnFalseAst::Cancel); }
        else if after.starts_with("retryNextTurn") { on_false = Some(OnFalseAst::RetryNextTurn); }
        else if let Some(tail) = after.strip_prefix("retryAfter ") {
            let mut k=0; while k < tail.len() && tail.as_bytes()[k].is_ascii_digit() { k+=1; }
            if k==0 { return Err(AstError::Shape("retryAfter missing turns")); }
            let turns: usize = tail[..k].parse().map_err(|_| AstError::Shape("invalid retryAfter turns"))?;
            on_false = Some(OnFalseAst::RetryAfter{ turns });
        }
    }
    if let Some(idx) = extras.find("note ") {
        let after = &extras[idx+5..].trim_start();
        if let Some(r) = after.strip_prefix('"') { if let Some(endq) = r.find('"') { note = Some(r[..endq].to_string()); } }
    }
    let condition = parse_condition_text(cond_str)?;
    let mut p = brace_pos + 1; let bytes2 = rest.as_bytes(); let mut depth = 1i32; while p < rest.len() { let c = bytes2[p] as char; if c == '{' { depth += 1; } else if c == '}' { depth -= 1; if depth == 0 { break; } } p += 1; } if depth != 0 { return Err(AstError::Shape("schedule block not closed")); }
    let inner_body = &rest[brace_pos+1..p];
    let actions = parse_actions_from_body(inner_body)?;
    let consumed = s.len() - rest[p+1..].len();
    Ok(( if is_in { ActionAst::ScheduleInIf { turns_ahead: num, condition: Box::new(condition), on_false, actions, note } } else { ActionAst::ScheduleOnIf { on_turn: num, condition: Box::new(condition), on_false, actions, note } }, consumed))
}


fn strip_leading_ws_and_comments(s: &str) -> &str {
    let mut i = 0usize;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
        if i >= bytes.len() { break; }
        if bytes[i] as char == '#' {
            while i < bytes.len() && (bytes[i] as char) != '\n' { i += 1; }
            continue;
        }
        break;
    }
    &s[i..]
}
