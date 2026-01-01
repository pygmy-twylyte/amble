use std::borrow::Cow;
use std::collections::HashMap;

use pest::Parser;

use crate::{
    ActionAst, ActionStmt, ConditionAst, ContainerStateAst, ItemAbilityAst, ItemPatchAst, NpcDialoguePatchAst,
    NpcMovementPatchAst, NpcPatchAst, NpcStateValue, NpcTimingPatchAst, OnFalseAst, RoomExitPatchAst, RoomPatchAst,
};

use super::conditions::parse_condition_text;
use super::helpers::{
    SourceMap, extract_body, extract_note, is_ident_char, parse_movability_opt, parse_string_at, str_offset, unquote,
};
use super::{AstError, DslParser, Rule};

fn parse_action_core(text: &str) -> Result<ActionAst, AstError> {
    let t = text.trim();
    if let Some(rest) = t.strip_prefix("do show ") {
        return Ok(ActionAst::Show(unquote(rest.trim())));
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
    if let Some(rest) = t.strip_prefix("do damage player ") {
        return parse_player_health_action(rest, true);
    }
    if let Some(rest) = t.strip_prefix("do heal player ") {
        return parse_player_health_action(rest, false);
    }
    if let Some(rest) = t.strip_prefix("do remove player effect ") {
        let r = rest.trim();
        let (cause, _) =
            parse_string_at(r).map_err(|_| AstError::Shape("remove player effect missing or invalid cause"))?;
        return Ok(ActionAst::RemovePlayerEffect { cause });
    }
    if let Some(rest) = t.strip_prefix("do damage npc ") {
        return parse_npc_health_action(rest, true);
    }
    if let Some(rest) = t.strip_prefix("do heal npc ") {
        return parse_npc_health_action(rest, false);
    }
    if let Some(rest) = t.strip_prefix("do remove npc ") {
        let rest = rest.trim();
        if let Some((npc, tail)) = rest.split_once(" effect ") {
            let r = tail.trim();
            let (cause, _) =
                parse_string_at(r).map_err(|_| AstError::Shape("remove npc effect missing or invalid cause"))?;
            return Ok(ActionAst::RemoveNpcEffect {
                npc: npc.trim().to_string(),
                cause,
            });
        }
        return Err(AstError::Shape("remove npc effect syntax"));
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
    if let Some(rest) = t.strip_prefix("do lock exit from ")
        && let Some((from, tail)) = rest.split_once(" direction ")
    {
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
    if let Some(rest) = t.strip_prefix("do unlock exit from ")
        && let Some((from, tail)) = rest.split_once(" direction ")
    {
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
        if let Some((from, tail)) = rest.split_once(" to ")
            && let Some((to, dir_tail)) = tail.split_once(" direction ")
        {
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
    if let Some(rest) = t.strip_prefix("do set item movability ") {
        let rest = rest.trim();
        let (item, mov_text) = rest
            .split_once(' ')
            .ok_or(AstError::Shape("set item movability missing item or value"))?;
        let movability = parse_movability_opt(mov_text)?;
        return Ok(ActionAst::SetItemMovability {
            item: item.to_string(),
            movability,
        });
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
        let rest = rest.trim();
        let mut parts = rest.splitn(2, ' ');
        let amount_part = parts.next().ok_or(AstError::Shape("award points missing amount"))?;
        let amount: i64 = amount_part
            .parse()
            .map_err(|_| AstError::Shape("invalid points number"))?;
        let remainder = parts
            .next()
            .ok_or(AstError::Shape("award points missing reason"))?
            .trim();
        let reason_text = remainder
            .strip_prefix("reason")
            .ok_or(AstError::Shape("award points missing reason keyword"))?
            .trim();
        let (reason, _used) =
            parse_string_at(reason_text).map_err(|_| AstError::Shape("award points missing or invalid reason"))?;
        return Ok(ActionAst::AwardPoints { amount, reason });
    }
    Err(AstError::Shape("unknown action"))
}

fn parse_player_health_action(rest: &str, is_damage: bool) -> Result<ActionAst, AstError> {
    let (amount, turns, cause) =
        parse_amount_turns_and_cause(rest, if is_damage { "damage player" } else { "heal player" })?;
    Ok(if is_damage {
        ActionAst::DamagePlayer { amount, turns, cause }
    } else {
        ActionAst::HealPlayer { amount, turns, cause }
    })
}

fn parse_npc_health_action(rest: &str, is_damage: bool) -> Result<ActionAst, AstError> {
    let rest = rest.trim();
    let Some((npc, tail)) = rest.split_once(' ') else {
        return Err(AstError::Shape("npc health action missing npc id"));
    };
    let (amount, turns, cause) = parse_amount_turns_and_cause(tail, if is_damage { "damage npc" } else { "heal npc" })?;
    Ok(if is_damage {
        ActionAst::DamageNpc {
            npc: npc.trim().to_string(),
            amount,
            turns,
            cause,
        }
    } else {
        ActionAst::HealNpc {
            npc: npc.trim().to_string(),
            amount,
            turns,
            cause,
        }
    })
}

fn parse_amount_turns_and_cause(rest: &str, label: &str) -> Result<(u32, Option<usize>, String), AstError> {
    let rest = rest.trim();
    let Some((amount_tok, mut tail)) = rest.split_once(' ') else {
        return Err(AstError::Shape("health action missing amount"));
    };
    let amount: u32 = amount_tok
        .parse()
        .map_err(|_| AstError::Shape("health action amount must be a positive number"))?;
    let mut turns: Option<usize> = None;
    tail = tail.trim_start();
    if let Some(after_for) = tail.strip_prefix("for ") {
        let after_for = after_for.trim_start();
        let mut len = 0usize;
        while len < after_for.len() && after_for.as_bytes()[len].is_ascii_digit() {
            len += 1;
        }
        if len == 0 {
            return Err(AstError::Shape("health action missing turns count after 'for'"));
        }
        let tval: usize = after_for[..len]
            .parse()
            .map_err(|_| AstError::Shape("health action turns must be a positive number"))?;
        if tval == 0 {
            return Err(AstError::Shape("health action turns must be a positive number"));
        }
        turns = Some(tval);
        tail = after_for[len..].trim_start();
        tail = tail
            .strip_prefix("turns")
            .ok_or(AstError::Shape("health action missing 'turns' keyword"))?
            .trim_start();
    }
    let cause_tail = tail
        .strip_prefix("cause")
        .ok_or_else(|| {
            AstError::Shape(match label {
                l if l.contains("damage player") => "damage player missing 'cause'",
                l if l.contains("heal player") => "heal player missing 'cause'",
                l if l.contains("damage npc") => "damage npc missing 'cause'",
                l if l.contains("heal npc") => "heal npc missing 'cause'",
                _ => "health action missing 'cause'",
            })
        })?
        .trim_start();
    let (cause, _used) =
        parse_string_at(cause_tail).map_err(|_| AstError::Shape("health action cause missing or invalid quote"))?;
    Ok((amount, turns, cause))
}

pub(super) fn parse_actions_from_body(
    body: &str,
    source: &str,
    smap: &SourceMap,
    sets: &HashMap<String, Vec<String>>,
) -> Result<Vec<ActionStmt>, AstError> {
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
            out.push(ActionStmt::new(ActionAst::Conditional {
                condition: Box::new(cond),
                actions,
            }));
            let consumed = brace_rel + 1 + inner_body.len() + 1;
            i = if_pos + 3 + consumed;
            continue;
        }
        let remainder = &body[i..];
        let trimmed_remainder = remainder.trim_start();

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

        match parse_schedule_action(remainder, source, smap, sets) {
            Ok((action, used)) => {
                out.push(action);
                i += used;
                continue;
            },
            Err(AstError::Shape("not a schedule action")) => {},
            Err(e) => return Err(e),
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
pub(super) fn parse_modify_item_action(text: &str) -> Result<(ActionStmt, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let (priority, rest_after_do) = match strip_priority_clause(s) {
        Ok(v) => v,
        Err(AstError::Shape("not a do action")) => return Err(AstError::Shape("not a modify item action")),
        Err(e) => return Err(e),
    };
    let rest0 = rest_after_do
        .strip_prefix("modify item ")
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
            Rule::item_movability_patch => {
                let raw = stmt
                    .as_str()
                    .split_once(' ')
                    .map(|(_, rest)| rest)
                    .ok_or(AstError::Shape("modify item movability missing value"))?;
                patch.movability = Some(parse_movability_opt(raw)?);
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

    Ok((
        ActionStmt {
            priority,
            action: ActionAst::ModifyItem { item, patch },
        },
        consumed,
    ))
}
pub(super) fn parse_modify_room_action(text: &str) -> Result<(ActionStmt, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let (priority, rest_after_do) = match strip_priority_clause(s) {
        Ok(v) => v,
        Err(AstError::Shape("not a do action")) => return Err(AstError::Shape("not a modify room action")),
        Err(e) => return Err(e),
    };
    let rest0 = rest_after_do
        .strip_prefix("modify room ")
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

    Ok((
        ActionStmt {
            priority,
            action: ActionAst::ModifyRoom { room, patch },
        },
        consumed,
    ))
}
pub(super) fn parse_modify_npc_action(text: &str) -> Result<(ActionStmt, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let (priority, rest_after_do) = match strip_priority_clause(s) {
        Ok(v) => v,
        Err(AstError::Shape("not a do action")) => return Err(AstError::Shape("not a modify npc action")),
        Err(e) => return Err(e),
    };
    let rest0 = rest_after_do
        .strip_prefix("modify npc ")
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
                if turns <= 0 {
                    return Err(AstError::Shape("modify npc timing every requires positive turns"));
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
                if turn <= 0 {
                    return Err(AstError::Shape("modify npc timing on requires a positive turn"));
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

    Ok((
        ActionStmt {
            priority,
            action: ActionAst::ModifyNpc { npc, patch },
        },
        consumed,
    ))
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
pub(super) fn parse_schedule_action(
    text: &str,
    source: &str,
    smap: &SourceMap,
    sets: &HashMap<String, Vec<String>>,
) -> Result<(ActionStmt, usize), AstError> {
    let s = text.trim_start();
    let leading_ws = text.len() - s.len();
    let (priority, rest_after_do) = match strip_priority_clause(s) {
        Ok(v) => v,
        Err(AstError::Shape("not a do action")) => return Err(AstError::Shape("not a schedule action")),
        Err(e) => return Err(e),
    };
    let (rest0, is_in) = if let Some(r) = rest_after_do.strip_prefix("schedule in ") {
        (r, true)
    } else if let Some(r) = rest_after_do.strip_prefix("schedule on ") {
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
    let consumed = leading_ws + (s.len() - rest1[p + 1..].len());

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
    Ok((ActionStmt { priority, action: act }, consumed))
}
fn strip_priority_clause(text: &str) -> Result<(Option<isize>, &str), AstError> {
    let rest = text.strip_prefix("do").ok_or(AstError::Shape("not a do action"))?;
    let mut after = rest.trim_start();
    if let Some(rem) = after.strip_prefix("priority") {
        after = rem.trim_start();
        let mut idx = 0usize;
        if after.starts_with('-') {
            idx += 1;
        }
        while idx < after.len() && after.as_bytes()[idx].is_ascii_digit() {
            idx += 1;
        }
        if idx == 0 || (idx == 1 && after.starts_with('-')) {
            return Err(AstError::Shape("priority missing number"));
        }
        let num: isize = after[..idx]
            .parse()
            .map_err(|_| AstError::Shape("invalid priority number"))?;
        let tail = after[idx..].trim_start();
        return Ok((Some(num), tail));
    }
    Ok((None, after))
}
pub(super) fn parse_action_from_str(text: &str) -> Result<ActionStmt, AstError> {
    let trimmed = text.trim();
    let (priority, rest) = strip_priority_clause(trimmed)?;
    let source = if priority.is_some() {
        Cow::Owned(format!("do {rest}"))
    } else {
        Cow::Borrowed(trimmed)
    };
    let action = parse_action_core(&source)?;
    Ok(ActionStmt { priority, action })
}
