use crate::{NpcAst, NpcMovementAst, NpcMovementTypeAst, NpcStateValue};

use super::helpers::{parse_string_at, unquote};
use super::{AstError, Rule};

pub(super) fn parse_npc_pair(
    npc: pest::iterators::Pair<Rule>,
    _source: &str,
) -> Result<NpcAst, AstError> {
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
    let mut max_hp: Option<u32> = None;
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
            Rule::npc_max_hp => {
                let tok = stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing max_hp number"))?;
                let hp: i64 = tok
                    .as_str()
                    .parse()
                    .map_err(|_| AstError::Shape("npc max_hp must be a number"))?;
                if hp <= 0 {
                    return Err(AstError::Shape("npc max_hp must be positive"));
                }
                max_hp = Some(hp as u32);
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
                if let Some(open) = s.find('(')
                    && let Some(close_rel) = s[open + 1..].find(')')
                {
                    let inner = &s[open + 1..open + 1 + close_rel];
                    for tok in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                        rooms.push(tok.to_string());
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
                if let Some(rest) = txt.strip_prefix("max_hp ") {
                    let hp: i64 = rest
                        .trim()
                        .parse()
                        .map_err(|_| AstError::Shape("npc max_hp must be a number"))?;
                    if hp <= 0 {
                        return Err(AstError::Shape("npc max_hp must be positive"));
                    }
                    max_hp = Some(hp as u32);
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
                    if let Some(open) = txt.find('(')
                        && let Some(close_rel) = txt[open + 1..].find(')')
                    {
                        let inner = &txt[open + 1..open + 1 + close_rel];
                        for tok in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                            rooms.push(tok.to_string());
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
                    if let Some(open_idx) = after_key.find('{')
                        && let Some(close_rel) = after_key[open_idx + 1..].rfind('}')
                    {
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
            },
        }
    }
    let name = name.ok_or(AstError::Shape("npc missing name"))?;
    let desc = desc.ok_or(AstError::Shape("npc missing desc"))?;
    let location = location.ok_or(AstError::Shape("npc missing location"))?;
    let max_hp = max_hp.ok_or(AstError::Shape("npc missing max_hp"))?;
    let state = state.unwrap_or(NpcStateValue::Named("normal".to_string()));
    Ok(NpcAst {
        id,
        name,
        desc,
        max_hp,
        location,
        state,
        movement,
        dialogue,
        src_line,
    })
}
