use crate::{GoalAst, GoalCondAst, GoalGroupAst};

use super::helpers::unquote;
use super::{AstError, Rule};

pub(super) fn parse_goal_pair(
    goal: pest::iterators::Pair<Rule>,
    _source: &str,
) -> Result<GoalAst, AstError> {
    let (src_line, _src_col) = goal.as_span().start_pos().line_col();
    let mut it = goal.into_inner();
    let id = it.next().ok_or(AstError::Shape("goal id"))?.as_str().to_string();
    let block = it.next().ok_or(AstError::Shape("goal block"))?;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut group: Option<GoalGroupAst> = None;
    let mut activate_when: Option<GoalCondAst> = None;
    let mut finished_when: Option<GoalCondAst> = None;
    let mut failed_when: Option<GoalCondAst> = None;
    for p in block.into_inner() {
        match p.as_rule() {
            Rule::goal_name => {
                let s = p.into_inner().next().ok_or(AstError::Shape("goal name text"))?.as_str();
                name = Some(unquote(s));
            },
            Rule::goal_desc => {
                let s = p.into_inner().next().ok_or(AstError::Shape("desc text"))?.as_str();
                description = Some(unquote(s));
            },
            Rule::goal_group => {
                let val = p.as_str().split_whitespace().last().unwrap_or("");
                group = Some(match val {
                    "required" => GoalGroupAst::Required,
                    "optional" => GoalGroupAst::Optional,
                    "status-effect" => GoalGroupAst::StatusEffect,
                    _ => GoalGroupAst::Required,
                });
            },
            Rule::goal_start => {
                let cond = p.into_inner().next().ok_or(AstError::Shape("start cond"))?;
                activate_when = Some(parse_goal_cond_pair(cond));
            },
            Rule::goal_done => {
                let cond = p.into_inner().next().ok_or(AstError::Shape("done cond"))?;
                finished_when = Some(parse_goal_cond_pair(cond));
            },
            Rule::goal_fail => {
                let cond = p.into_inner().next().ok_or(AstError::Shape("fail cond"))?;
                failed_when = Some(parse_goal_cond_pair(cond));
            },
            _ => {},
        }
    }
    let name = name.ok_or(AstError::Shape("goal missing name"))?;
    let description = description.ok_or(AstError::Shape("goal missing desc"))?;
    let group = group.ok_or(AstError::Shape("goal missing group"))?;
    let finished_when = finished_when.ok_or(AstError::Shape("goal missing done"))?;
    Ok(GoalAst {
        id,
        name,
        description,
        group,
        activate_when,
        failed_when,
        finished_when,
        src_line,
    })
}

fn parse_goal_cond_pair(p: pest::iterators::Pair<Rule>) -> GoalCondAst {
    let s = p.as_str().trim();
    if let Some(rest) = s.strip_prefix("has flag ") {
        return GoalCondAst::HasFlag(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("missing flag ") {
        return GoalCondAst::MissingFlag(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("has item ") {
        return GoalCondAst::HasItem(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("reached room ") {
        return GoalCondAst::ReachedRoom(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("goal complete ") {
        return GoalCondAst::GoalComplete(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("flag in progress ") {
        return GoalCondAst::FlagInProgress(rest.trim().to_string());
    }
    if let Some(rest) = s.strip_prefix("flag complete ") {
        return GoalCondAst::FlagComplete(rest.trim().to_string());
    }
    GoalCondAst::HasFlag(s.to_string())
}
