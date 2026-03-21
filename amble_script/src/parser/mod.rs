//! Parser and AST builders for the Amble DSL.
//!
//! Wraps the Pest-generated grammar with helpers that construct the
//! compiler's abstract syntax tree for triggers, rooms, items, and more.

use pest::Parser;
use pest_derive::Parser as PestParser;

use std::collections::HashMap;

use crate::{ActionSetSpec, ConditionAliasSpec, GoalAst, ItemAst, NpcAst, RoomAst, SpinnerAst, TriggerAst};

mod action_sets;
mod actions;
mod conditions;
mod game;
mod goal;
mod helpers;
mod item;
mod npc;
mod room;
mod spinner;
mod trigger;

#[cfg(test)]
use actions::{parse_modify_item_action, parse_modify_npc_action, parse_modify_room_action};
#[cfg(test)]
use helpers::parse_string;

use conditions::{
    resolve_condition_aliases as resolve_condition_aliases_impl,
    resolve_condition_aliases_with_base as resolve_condition_aliases_with_base_impl,
};
use game::parse_game_pair;
use goal::parse_goal_pair;
use helpers::SourceMap;
use item::parse_item_pair;
use npc::parse_npc_pair;
use room::parse_room_pair;
use spinner::parse_spinner_pair;
use trigger::parse_trigger_pair;

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
    let (_, triggers, ..) = parse_program_full(source)?;
    Ok(triggers)
}

/// Parse a full program returning triggers, rooms, items, and spinners.
///
/// # Errors
/// Returns an error when parsing fails or when the grammar encounters an
/// unexpected shape.
pub fn parse_program_full(source: &str) -> Result<ProgramAstBundle, AstError> {
    parse_program_full_with_context(source, &HashMap::new(), &HashMap::new())
}

/// Parse a full program using a caller-provided map of resolved condition aliases.
///
/// This is primarily useful for multi-file callers that want aliases defined in
/// one source file to be available in another.
///
/// # Errors
/// Returns an error when parsing fails or when the grammar encounters an
/// unexpected shape.
pub fn parse_program_full_with_aliases(
    source: &str,
    aliases: &HashMap<String, crate::ConditionAst>,
) -> Result<ProgramAstBundle, AstError> {
    parse_program_full_with_context(source, aliases, &HashMap::new())
}

/// Parse a full program using caller-provided maps of resolved condition aliases
/// and action sets.
///
/// This is primarily useful for multi-file callers that want shared definitions
/// from other sources to be available in the current file.
///
/// # Errors
/// Returns an error when parsing fails or when the grammar encounters an
/// unexpected shape.
pub fn parse_program_full_with_context(
    source: &str,
    aliases: &HashMap<String, crate::ConditionAst>,
    action_sets: &HashMap<String, Vec<crate::ActionStmt>>,
) -> Result<ProgramAstBundle, AstError> {
    let mut pairs = DslParser::parse(Rule::program, source).map_err(|e| AstError::Pest(e.to_string()))?;
    let pair = pairs.next().ok_or(AstError::Shape("expected program"))?;
    let smap = SourceMap::new(source);
    let (sets, local_alias_specs, local_action_set_specs) = collect_sets_and_specs(&pair)?;
    let local_aliases = resolve_condition_aliases_with_base_impl(&local_alias_specs, aliases)?;
    let mut merged_aliases = aliases.clone();
    merged_aliases.extend(local_aliases);
    let local_action_sets =
        action_sets::resolve_action_sets_with_base(&local_action_set_specs, &merged_aliases, action_sets)?;
    let mut merged_action_sets = action_sets.clone();
    merged_action_sets.extend(local_action_sets);
    let mut game_pair = None;
    let mut trigger_pairs = Vec::new();
    let mut room_pairs = Vec::new();
    let mut item_pairs = Vec::new();
    let mut spinner_pairs = Vec::new();
    let mut npc_pairs = Vec::new();
    let mut goal_pairs = Vec::new();
    for item in pair.clone().into_inner() {
        match item.as_rule() {
            Rule::set_decl | Rule::cond_decl | Rule::action_set_decl => {},
            Rule::game_def => {
                if game_pair.is_some() {
                    return Err(AstError::Shape("multiple game blocks"));
                }
                game_pair = Some(item);
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
    let mut triggers = Vec::new();
    for trig in trigger_pairs {
        let mut ts = parse_trigger_pair(trig, source, &smap, &sets, &merged_aliases, &merged_action_sets)?;
        triggers.append(&mut ts);
    }
    let mut rooms = Vec::new();
    for rp in room_pairs {
        let r = parse_room_pair(rp, source)?;
        rooms.push(r);
    }
    let mut items = Vec::new();
    for ip in item_pairs {
        let it = parse_item_pair(ip, source, &sets, &merged_aliases)?;
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
    let game = if let Some(gp) = game_pair {
        Some(parse_game_pair(gp, source)?)
    } else {
        None
    };
    Ok((game, triggers, rooms, items, spinners, npcs, goals))
}

/// Collect top-level condition alias definitions from a source file.
///
/// The returned specs preserve the room-set environment from the defining
/// source so aliases can later be resolved globally across files.
///
/// # Errors
/// Returns an error if the source cannot be parsed as a program.
pub fn collect_condition_alias_specs(source: &str) -> Result<Vec<ConditionAliasSpec>, AstError> {
    let mut pairs = DslParser::parse(Rule::program, source).map_err(|e| AstError::Pest(e.to_string()))?;
    let pair = pairs.next().ok_or(AstError::Shape("expected program"))?;
    let (_, specs, _) = collect_sets_and_specs(&pair)?;
    Ok(specs)
}

/// Collect top-level action set definitions from a source file.
///
/// The returned specs preserve the room-set environment from the defining
/// source so conditions inside action sets can be resolved globally across
/// files.
///
/// # Errors
/// Returns an error if the source cannot be parsed as a program.
pub fn collect_action_set_specs(source: &str) -> Result<Vec<ActionSetSpec>, AstError> {
    let mut pairs = DslParser::parse(Rule::program, source).map_err(|e| AstError::Pest(e.to_string()))?;
    let pair = pairs.next().ok_or(AstError::Shape("expected program"))?;
    let (_, _, specs) = collect_sets_and_specs(&pair)?;
    Ok(specs)
}

/// Resolve collected condition alias specs into reusable condition ASTs.
///
/// # Errors
/// Returns an error when aliases are duplicated, recursive, or reference an
/// invalid condition.
pub fn resolve_condition_aliases(
    specs: &[ConditionAliasSpec],
) -> Result<HashMap<String, crate::ConditionAst>, AstError> {
    resolve_condition_aliases_impl(specs)
}

pub fn resolve_action_sets(
    specs: &[ActionSetSpec],
    cond_aliases: &HashMap<String, crate::ConditionAst>,
) -> Result<HashMap<String, Vec<crate::ActionStmt>>, AstError> {
    action_sets::resolve_action_sets(specs, cond_aliases)
}

fn collect_sets_and_specs(
    pair: &pest::iterators::Pair<'_, Rule>,
) -> Result<
    (
        HashMap<String, Vec<String>>,
        Vec<ConditionAliasSpec>,
        Vec<ActionSetSpec>,
    ),
    AstError,
> {
    let mut sets: HashMap<String, Vec<String>> = HashMap::new();
    for item in pair.clone().into_inner() {
        if item.as_rule() != Rule::set_decl {
            continue;
        }
        let mut it = item.into_inner();
        let name = it.next().ok_or(AstError::Shape("set name"))?.as_str().to_string();
        let list_pair = it.next().ok_or(AstError::Shape("set list"))?;
        let mut vals = Vec::new();
        for p in list_pair.into_inner() {
            if p.as_rule() == Rule::ident {
                vals.push(p.as_str().to_string());
            }
        }
        sets.insert(name, vals);
    }

    let mut cond_specs = Vec::new();
    for item in pair.clone().into_inner() {
        if item.as_rule() != Rule::cond_decl {
            continue;
        }
        let mut it = item.into_inner();
        let name = it
            .next()
            .ok_or(AstError::Shape("condition alias name"))?
            .as_str()
            .to_string();
        let cond = it
            .next()
            .ok_or(AstError::Shape("condition alias condition"))?
            .as_str()
            .trim()
            .to_string();
        cond_specs.push(ConditionAliasSpec {
            name,
            text: cond,
            sets: sets.clone(),
        });
    }

    let mut action_set_specs = Vec::new();
    for item in pair.clone().into_inner() {
        if item.as_rule() != Rule::action_set_decl {
            continue;
        }
        let mut it = item.into_inner();
        let name = it
            .next()
            .ok_or(AstError::Shape("action set name"))?
            .as_str()
            .to_string();
        let block = it.next().ok_or(AstError::Shape("action set block"))?;
        let body = helpers::extract_body(block.as_str())?.to_string();
        action_set_specs.push(ActionSetSpec {
            name,
            text: body,
            sets: sets.clone(),
        });
    }

    Ok((sets, cond_specs, action_set_specs))
}

/// Parse only rooms from a source (helper/testing).
/// Parse only room definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into rooms.
pub fn parse_rooms(source: &str) -> Result<Vec<RoomAst>, AstError> {
    let (_, _, rooms, _, _, _, _) = parse_program_full(source)?;
    Ok(rooms)
}

/// Parse only items from a source (helper/testing).
/// Parse only item definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into items.
pub fn parse_items(source: &str) -> Result<Vec<ItemAst>, AstError> {
    let (_, _, _, items, _, _, _) = parse_program_full(source)?;
    Ok(items)
}

/// Parse only spinners from a source (helper/testing).
/// Parse only spinner definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into spinners.
pub fn parse_spinners(source: &str) -> Result<Vec<SpinnerAst>, AstError> {
    let (_, _, _, _, spinners, _, _) = parse_program_full(source)?;
    Ok(spinners)
}

/// Parse only npcs from a source (helper/testing).
/// Parse only NPC definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into NPCs.
pub fn parse_npcs(source: &str) -> Result<Vec<NpcAst>, AstError> {
    let (_, _, _, _, _, npcs, _) = parse_program_full(source)?;
    Ok(npcs)
}

/// Parse only goal definitions from the given source.
///
/// # Errors
/// Returns an error if the source cannot be parsed into goals.
pub fn parse_goals(source: &str) -> Result<Vec<GoalAst>, AstError> {
    let (_, _, _, _, _, _, goals) = parse_program_full(source)?;
    Ok(goals)
}

/// Composite AST collections returned by [`parse_program_full`].
pub type ProgramAstBundle = (
    Option<crate::GameAst>,
    Vec<TriggerAst>,
    Vec<RoomAst>,
    Vec<ItemAst>,
    Vec<SpinnerAst>,
    Vec<NpcAst>,
    Vec<GoalAst>,
);
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActionAst, ActionStmt, ConditionAst, ContainerStateAst, MovabilityAst, NpcStateValue, NpcTimingPatchAst,
    };

    #[test]
    fn game_block_parses() {
        let src = r#"
game {
  title "Demo"
  slug "demo"
  author "Jane Doe"
  version "0.1.0"
  blurb "Short and sweet."
  intro "Intro"
  player {
    name "The Candidate"
    desc "A test player."
    max_hp 20
    start room foyer
  }
  scoring {
    rank 0.0 "Rookie" "Keep going."
  }
}
"#;
        let (game, ..) = parse_program_full(src).expect("valid game block");
        let game = game.expect("game block parsed");
        assert_eq!(game.title, "Demo");
        assert_eq!(game.slug.as_deref(), Some("demo"));
        assert_eq!(game.author.as_deref(), Some("Jane Doe"));
        assert_eq!(game.version.as_deref(), Some("0.1.0"));
        assert_eq!(game.blurb.as_deref(), Some("Short and sweet."));
        assert_eq!(game.player.start_room, "foyer");
        let scoring = game.scoring.expect("scoring parsed");
        assert_eq!(scoring.ranks.len(), 1);
        assert_eq!(scoring.ranks[0].threshold, 0.0);
    }

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
        match &ast.actions[0].action {
            ActionAst::Show(s) => {
                assert!(s.contains('\n'));
                assert_eq!(s, "Line1\nLine2");
            },
            _ => panic!("expected show"),
        }
        // npc says contains a quote
        match &ast.actions[1].action {
            ActionAst::NpcSays { npc, quote } => {
                assert_eq!(npc, "gonk");
                assert!(quote.contains('"'));
            },
            _ => panic!("expected npc says"),
        }
    }

    #[test]
    fn compact_take_event_parses() {
        let src = r#"
trigger "take towel" when take towel {
    do show "ok"
}
"#;
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.event, ConditionAst::TakeItem("towel".into()));
    }

    #[test]
    fn compact_take_from_events_parse() {
        let src = r#"
trigger "take from npc" when take dull_longsword from npc black_knight {
    do show "npc"
}

trigger "take from item" when take keycard from item locker {
    do show "item"
}
"#;
        let (_, triggers, ..) = parse_program_full(src).expect("parse ok");
        assert_eq!(
            triggers[0].event,
            ConditionAst::TakeFromNpc {
                item: "dull_longsword".into(),
                npc: "black_knight".into(),
            }
        );
        assert_eq!(
            triggers[1].event,
            ConditionAst::TakeFromItem {
                loot: "keycard".into(),
                container: "locker".into(),
            }
        );
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
        match &ast.actions[0].action {
            ActionAst::ScheduleIn { note, .. } => {
                let note = note.as_deref().expect("note should be present");
                assert!(note.contains("lineA"));
                assert!(note.contains("lineB"));
            },
            other => panic!("expected schedule action, got {other:?}"),
        }
    }

    #[test]
    fn schedule_if_chance_requires_positive_percent() {
        let src = r#"
trigger "bad chance" when always {
  do schedule in 1 if chance 0% {
    do show "nope"
  }
}
"#;
        let err = parse_trigger(src).expect_err("expected parse failure");
        match err {
            AstError::Shape(msg) => assert_eq!(msg, "chance percent must be greater than 0"),
            other => panic!("expected shape error, got {other:?}"),
        }
    }

    #[test]
    fn schedule_actions_reject_zero_turn_health_effects() {
        let src = r#"
trigger "zero turns" when always {
  do schedule in 1 {
    do damage player 1 for 0 turns cause "noop"
  }
}
"#;
        let err = parse_trigger(src).expect_err("expected parse failure");
        match err {
            AstError::ShapeAt { msg, context } => {
                assert_eq!(msg, "health action turns must be a positive number");
                assert!(context.contains("for 0 turns"), "{context}");
            },
            other => panic!("expected shape-at error, got {other:?}"),
        }
    }

    #[test]
    fn modify_item_parses_patch_fields() {
        let src = r#"
trigger "patch locker" when always {
    do modify item locker {
        name "Unlocked locker"
        description "It's open now"
        text "notes"
        movability restricted "It's not yours to take."
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
        let action = &ast.actions[0].action;
        match action {
            ActionAst::ModifyItem { item, patch } => {
                assert_eq!(item, "locker");
                assert_eq!(patch.name.as_deref(), Some("Unlocked locker"));
                assert_eq!(patch.desc.as_deref(), Some("It's open now"));
                assert_eq!(patch.text.as_deref(), Some("notes"));
                assert_eq!(
                    patch.movability,
                    Some(MovabilityAst::Restricted {
                        reason: "It's not yours to take.".into()
                    })
                );
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
        assert!(matches!(&helper_action.action, ActionAst::ModifyRoom { .. }));
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 1);
        match &ast.actions[0].action {
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
        assert!(matches!(&helper_action.action, ActionAst::ModifyNpc { .. }));
        let ast = parse_trigger(src).expect("parse ok");
        assert_eq!(ast.actions.len(), 1);
        match &ast.actions[0].action {
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
        match &ast.actions[0].action {
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
    }

    #[test]
    fn parse_modify_room_action_helper_handles_basic_block() {
        let snippet = "do modify room lab { name \"Ruined\" }\n";
        let (action, used) = super::parse_modify_room_action(snippet).expect("parse helper");
        assert_eq!(&snippet[..used], "do modify room lab { name \"Ruined\" }");
        match &action.action {
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
        let (action, used) = super::parse_modify_item_action(
            snippet,
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
        )
        .expect("parse helper");
        assert_eq!(&snippet[..used], "do modify item locker { name \"Ok\" }");
        match &action.action {
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
        match &action.action {
            ActionAst::ModifyItem { item, patch } => {
                assert_eq!(item, "chest");
                assert!(patch.container_state.is_none());
                assert!(patch.remove_container_state);
            },
            other => panic!("expected modify item action, got {other:?}"),
        }
    }

    #[test]
    fn raw_string_with_hash_quotes() {
        let src = "trigger r#\"raw name with \"quotes\"\"# when always {\n  do show r#\"He said \"hi\"\"#\n}\n";
        let asts = super::parse_program(src).expect("parse ok");
        assert!(!asts.is_empty());
        match &asts[0].actions[0].action {
            ActionAst::Show(msg) => {
                assert!(msg.contains("He said"));
                assert!(msg.contains("hi"));
            },
            other => panic!("expected show action, got {other:?}"),
        }
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
    fn npc_movement_loop_flag_parses() {
        let src = r#"
npc bot {
  name "Maintenance Bot"
  desc "Keeps the corridors tidy."
  location room hub
  max_hp 5
  state idle
  movement route rooms (hub, hall) timing every_3_turns active true loop false
}
"#;
        let npcs = crate::parse_npcs(src).expect("parse npcs ok");
        assert_eq!(npcs.len(), 1);
        let movement = npcs[0].movement.as_ref().expect("movement present");
        assert_eq!(movement.loop_route, Some(false));
    }

    #[test]
    fn item_with_consumable_parses() {
        let src = r#"item snack {
  name "Snack"
  desc "Yum"
  movability free
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
    fn condition_aliases_expand_in_same_file_conditions() {
        let src = r#"
let cond radio_ready = all(has item hint_radio, has flag hint-radio-on)
let cond hint_needed = all(radio_ready, missing flag puzzle-solved)

item receiver {
  name "Receiver"
  desc "Listens."
  movability free
  location nowhere "spawn later"
  visible when radio_ready
}

trigger "Radio Hint" when always {
  if hint_needed {
    do show "Need a hint."
  }
}
"#;
        let (_game, triggers, _rooms, items, _spinners, _npcs, _goals) =
            parse_program_full(src).expect("alias parse succeeds");

        assert_eq!(triggers.len(), 1);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].visible_when,
            Some(ConditionAst::All(vec![
                ConditionAst::HasItem("hint_radio".into()),
                ConditionAst::HasFlag("hint-radio-on".into()),
            ]))
        );
        assert_eq!(
            triggers[0].conditions,
            vec![ConditionAst::All(vec![
                ConditionAst::All(vec![
                    ConditionAst::HasItem("hint_radio".into()),
                    ConditionAst::HasFlag("hint-radio-on".into()),
                ]),
                ConditionAst::MissingFlag("puzzle-solved".into()),
            ])]
        );
    }

    #[test]
    fn condition_aliases_resolve_across_files() {
        let defs = r#"
let cond radio_ready = all(has item hint_radio, has flag hint-radio-on)
"#;
        let usage = r#"
trigger "Radio Hint" when always {
  if radio_ready {
    do show "Ready."
  }
}
"#;
        let mut specs = super::collect_condition_alias_specs(defs).expect("collect alias defs");
        specs.extend(super::collect_condition_alias_specs(usage).expect("usage has no local aliases"));
        let aliases = super::resolve_condition_aliases(&specs).expect("resolve aliases");
        let (_game, triggers, ..) =
            super::parse_program_full_with_aliases(usage, &aliases).expect("cross-file alias parse succeeds");

        assert_eq!(
            triggers[0].conditions,
            vec![ConditionAst::All(vec![
                ConditionAst::HasItem("hint_radio".into()),
                ConditionAst::HasFlag("hint-radio-on".into()),
            ])]
        );
    }

    #[test]
    fn parse_program_full_with_aliases_merges_local_aliases() {
        let defs = r#"
let cond radio_ready = all(has item hint_radio, has flag hint-radio-on)
"#;
        let usage = r#"
let cond hint_needed = all(radio_ready, missing flag puzzle-solved)

trigger "Radio Hint" when always {
  if hint_needed {
    do show "Need a hint."
  }
}
"#;
        let aliases =
            super::resolve_condition_aliases(&super::collect_condition_alias_specs(defs).expect("collect defs"))
                .expect("resolve shared aliases");
        let (_game, triggers, ..) =
            super::parse_program_full_with_aliases(usage, &aliases).expect("local aliases should merge");

        assert_eq!(
            triggers[0].conditions,
            vec![ConditionAst::All(vec![
                ConditionAst::All(vec![
                    ConditionAst::HasItem("hint_radio".into()),
                    ConditionAst::HasFlag("hint-radio-on".into()),
                ]),
                ConditionAst::MissingFlag("puzzle-solved".into()),
            ])]
        );
    }

    #[test]
    fn recursive_condition_aliases_are_rejected() {
        let src = r#"
let cond a = b
let cond b = a
"#;
        let specs = super::collect_condition_alias_specs(src).expect("collect aliases");
        let err = super::resolve_condition_aliases(&specs).expect_err("recursive aliases should fail");
        assert!(format!("{err}").contains("recursive condition alias"));
    }

    #[test]
    fn condition_aliases_can_be_adjacent_in_any_group() {
        let src = r#"
let cond radio_ready = all(has item hint_radio, has flag hint-radio-on)
let cond hint_not_needed = any(has flag read-plaque-2, has item firewood)
let cond hint_gate = any(radio_ready, hint_not_needed)
"#;
        let specs = super::collect_condition_alias_specs(src).expect("collect aliases");
        let aliases = super::resolve_condition_aliases(&specs).expect("resolve aliases");
        assert!(matches!(aliases.get("hint_gate"), Some(ConditionAst::Any(kids)) if kids.len() == 2));
    }

    #[test]
    fn action_sets_expand_in_same_file_triggers() {
        let src = r#"
let actions common_steps = {
  do add flag radio-ready
  do show "Ready."
}

trigger "Radio Hint" when always {
  run common_steps
}
"#;
        let (_game, triggers, ..) = parse_program_full(src).expect("action set parse succeeds");

        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].actions.len(), 2);
        assert_eq!(
            triggers[0].actions[0],
            ActionStmt::new(ActionAst::AddFlag("radio-ready".into()))
        );
        assert_eq!(
            triggers[0].actions[1],
            ActionStmt::new(ActionAst::Show("Ready.".into()))
        );
    }

    #[test]
    fn action_sets_resolve_across_files() {
        let defs = r#"
let actions common_steps = {
  do add flag radio-ready
}
"#;
        let usage = r#"
trigger "Radio Hint" when always {
  run common_steps
}
"#;
        let aliases = HashMap::new();
        let action_sets = super::resolve_action_sets(
            &super::collect_action_set_specs(defs).expect("collect action sets"),
            &aliases,
        )
        .expect("resolve action sets");
        let (_game, triggers, ..) = super::parse_program_full_with_context(usage, &aliases, &action_sets)
            .expect("cross-file action set parse succeeds");

        assert_eq!(
            triggers[0].actions,
            vec![ActionStmt::new(ActionAst::AddFlag("radio-ready".into()))]
        );
    }

    #[test]
    fn action_sets_can_use_condition_aliases_and_nested_runs() {
        let src = r#"
let cond radio_ready = has flag radio-on
let actions inner_steps = {
  do add flag nested-ready
}
let actions outer_steps = {
  if radio_ready {
    run inner_steps
  }
}

trigger "Radio Hint" when always {
  run outer_steps
}
"#;
        let (_game, triggers, ..) = parse_program_full(src).expect("nested action sets parse succeeds");

        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].actions.len(), 1);
        match &triggers[0].actions[0].action {
            ActionAst::Conditional { condition, actions } => {
                assert_eq!(**condition, ConditionAst::HasFlag("radio-on".into()));
                assert_eq!(
                    actions,
                    &vec![ActionStmt::new(ActionAst::AddFlag("nested-ready".into()))]
                );
            },
            other => panic!("expected conditional action, got {other:?}"),
        }
    }

    #[test]
    fn parse_program_full_with_context_merges_local_action_sets() {
        let defs = r#"
let actions base_steps = {
  do add flag radio-ready
}
"#;
        let usage = r#"
let actions hint_steps = {
  run base_steps
  do show "Need a hint."
}

trigger "Radio Hint" when always {
  run hint_steps
}
"#;
        let aliases = HashMap::new();
        let action_sets = super::resolve_action_sets(
            &super::collect_action_set_specs(defs).expect("collect action sets"),
            &aliases,
        )
        .expect("resolve action sets");
        let (_game, triggers, ..) = super::parse_program_full_with_context(usage, &aliases, &action_sets)
            .expect("local action sets should merge");

        assert_eq!(
            triggers[0].actions,
            vec![
                ActionStmt::new(ActionAst::AddFlag("radio-ready".into())),
                ActionStmt::new(ActionAst::Show("Need a hint.".into())),
            ]
        );
    }

    #[test]
    fn recursive_action_sets_are_rejected() {
        let src = r#"
let actions a = {
  run b
}
let actions b = {
  run a
}
"#;
        let aliases = HashMap::new();
        let specs = super::collect_action_set_specs(src).expect("collect action sets");
        let err = super::resolve_action_sets(&specs, &aliases).expect_err("recursive action sets should fail");
        assert!(format!("{err}").contains("recursive action set"));
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
