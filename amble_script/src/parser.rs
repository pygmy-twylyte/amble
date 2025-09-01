use pest::Parser;
use pest_derive::Parser as PestParser;

use crate::{ActionAst, ConditionAst, TriggerAst};

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

    // trigger -> "trigger" ~ quoted ~ "when" ~ "enter" ~ "room" ~ ident ~ block
    let q = it.next().ok_or(AstError::Shape("expected trigger name"))?;
    if q.as_rule() != Rule::quoted {
        return Err(AstError::Shape("expected quoted trigger name"));
    }
    let name = unquote(q.as_str());

    // when condition
    let mut when = it.next().ok_or(AstError::Shape("expected when condition"))?;
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

    // block -> if_block
    let mut block_inner_pairs = block.into_inner();
    let if_block = block_inner_pairs.next().ok_or(AstError::Shape("expected if_block"))?;

    let if_src = if_block.as_str();
    let if_pos = if_src.find("if ").ok_or(AstError::Shape("missing 'if'"))?;
    let brace_pos = if_src.find('{').ok_or(AstError::Shape("missing '{' after if"))?;
    let cond_text = &if_src[if_pos + 3..brace_pos];
    conditions.push(parse_condition_text(cond_text)?);

    let body = {
        let bytes = if_src.as_bytes();
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
        let (s, e) = (start.ok_or(AstError::Shape("missing '{' body start"))?, end.ok_or(AstError::Shape("missing '}' body end"))?);
        &if_src[s..e]
    };
    for line in body.lines() {
        let l = line.trim();
        if l.starts_with("do ") {
            actions.push(parse_action_from_str(l)?);
        }
    }

    Ok(TriggerAst { name, event, conditions, actions })
}

fn unquote(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
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
    if let Some(rest) = t.strip_prefix("do spawn item ") {
        // format: <item> into room <room>
        let rest = rest.trim();
        if let Some((item, tail)) = rest.split_once(" into room ") {
            return Ok(ActionAst::SpawnItemIntoRoom { item: item.trim().to_string(), room: tail.trim().to_string() });
        }
        return Err(AstError::Shape("spawn item syntax"));
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
