use std::collections::HashMap;

use crate::{
    ConditionAst, ConsumableAst, ConsumableWhenAst, ContainerStateAst, ItemAbilityAst, ItemAst, ItemLocationAst,
    ItemVisibilityAst, MovabilityAst,
};

use super::conditions::parse_condition_text;
use super::helpers::{parse_movability_opt, unquote};
use super::{AstError, Rule};

pub(super) fn parse_item_pair(
    item: pest::iterators::Pair<Rule>,
    _source: &str,
    sets: &HashMap<String, Vec<String>>,
) -> Result<ItemAst, AstError> {
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
    let mut movability: Option<MovabilityAst> = None;
    let mut location: Option<ItemLocationAst> = None;
    let mut visibility: Option<ItemVisibilityAst> = None;
    let mut visible_when: Option<ConditionAst> = None;
    let mut aliases: Vec<String> = Vec::new();
    let mut container_state: Option<ContainerStateAst> = None;
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
            Rule::item_movability => {
                let raw = stmt
                    .as_str()
                    .split_once(' ')
                    .map(|(_, rest)| rest)
                    .ok_or(AstError::Shape("movability missing value"))?;
                movability = Some(parse_movability_opt(raw)?);
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
            Rule::item_visibility => {
                let val = stmt
                    .as_str()
                    .split_whitespace()
                    .last()
                    .ok_or(AstError::Shape("missing visibility value"))?;
                visibility = Some(match val {
                    "listed" => ItemVisibilityAst::Listed,
                    "scenery" => ItemVisibilityAst::Scenery,
                    "hidden" => ItemVisibilityAst::Hidden,
                    _ => return Err(AstError::Shape("unknown visibility value")),
                });
            },
            Rule::item_visible_when => {
                let cond_pair = stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing visibility condition"))?;
                let cond_text = cond_pair.as_str().trim();
                visible_when = Some(parse_condition_text(cond_text, sets)?);
            },
            Rule::item_aliases => {
                let aliases_list = stmt
                    .into_inner()
                    .filter(|p| p.as_rule() == Rule::string)
                    .map(|p| unquote(p.as_str()))
                    .collect::<Vec<_>>();
                aliases.extend(aliases_list);
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
                // Store as (interaction, ability) to match engine mapping
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
                            if val <= 0 {
                                return Err(AstError::Shape("consumable uses must be > 0"));
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
    let movability = movability.unwrap_or(MovabilityAst::Free);
    let location = location.ok_or(AstError::Shape("item missing location"))?;
    let visibility = visibility.unwrap_or(ItemVisibilityAst::Listed);
    Ok(ItemAst {
        id,
        name,
        desc,
        movability,
        location,
        visibility,
        visible_when,
        aliases,
        container_state,
        abilities,
        text,
        interaction_requires: requires,
        consumable,
        src_line,
    })
}
