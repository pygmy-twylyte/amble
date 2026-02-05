use crate::{RoomAst, RoomSceneryAst};

use super::helpers::unquote;
use super::{AstError, Rule};

pub(super) fn parse_room_pair(room: pest::iterators::Pair<Rule>, _source: &str) -> Result<RoomAst, AstError> {
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
    let mut scenery: Vec<RoomSceneryAst> = Vec::new();
    let mut scenery_default: Option<String> = None;
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
                if let Some(next) = it.next()
                    && next.as_rule() == Rule::exit_opts
                {
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
                        if children.iter().all(|p| p.as_rule() == Rule::ident) && opt_text.starts_with("required_items")
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
            Rule::room_scenery_default => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing scenery default string"))?;
                scenery_default = Some(unquote(s.as_str()));
            },
            Rule::room_scenery_entry => {
                let mut it = inner_stmt.into_inner();
                let name_pair = it.next().ok_or(AstError::Shape("missing scenery name string"))?;
                let name = unquote(name_pair.as_str());
                let mut desc: Option<String> = None;
                if let Some(desc_pair) = it.next() {
                    let s = desc_pair
                        .into_inner()
                        .next()
                        .ok_or(AstError::Shape("missing scenery desc string"))?;
                    desc = Some(unquote(s.as_str()));
                }
                scenery.push(RoomSceneryAst { name, desc });
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
        scenery,
        scenery_default,
        src_line,
    })
}
