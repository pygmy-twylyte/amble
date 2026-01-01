use std::collections::HashMap;

use crate::{ConditionAst, NpcStateValue};

use super::AstError;
use super::helpers::{parse_string_at, split_top_level_commas};

pub(super) fn parse_condition_text(text: &str, sets: &HashMap<String, Vec<String>>) -> Result<ConditionAst, AstError> {
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
            let parsed_state = parse_condition_npc_state(state)?;
            return Ok(ConditionAst::NpcInState {
                npc: npc.to_string(),
                state: parsed_state,
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
        if pct <= 0.0 {
            return Err(AstError::Shape("chance percent must be greater than 0"));
        }
        return Ok(ConditionAst::ChancePercent(pct));
    }
    Err(AstError::Shape("unknown condition"))
}

fn parse_condition_npc_state(token: &str) -> Result<NpcStateValue, AstError> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err(AstError::Shape("npc in state missing state value"));
    }
    if trimmed.len() >= 6 && trimmed[..6].eq_ignore_ascii_case("custom") {
        let mut rest = &trimmed[6..];
        rest = rest.trim_start();
        if rest.starts_with(':') {
            rest = rest[1..].trim_start();
        } else if rest.starts_with('(') {
            rest = rest[1..].trim_start();
            if let Some(idx) = rest.rfind(')') {
                rest = rest[..idx].trim_end();
            }
        }
        rest = rest.trim();
        rest = rest.trim_end_matches(')');
        rest = rest.trim();
        if rest.starts_with('"') {
            let (value, _) =
                parse_string_at(rest).map_err(|_| AstError::Shape("custom npc state invalid quoted string"))?;
            return Ok(NpcStateValue::Custom(value));
        }
        if rest.is_empty() {
            return Err(AstError::Shape("custom npc state missing identifier"));
        }
        return Ok(NpcStateValue::Custom(rest.to_string()));
    }
    if trimmed.starts_with('"') {
        let (value, _) = parse_string_at(trimmed).map_err(|_| AstError::Shape("npc state invalid quoted string"))?;
        return Ok(NpcStateValue::Custom(value));
    }
    Ok(NpcStateValue::Named(trimmed.to_string()))
}
