use crate::{GameAst, PlayerAst, ScoringAst, ScoringRankAst};

use super::helpers::unquote;
use super::{AstError, Rule};

pub(super) fn parse_game_pair(game: pest::iterators::Pair<Rule>, _source: &str) -> Result<GameAst, AstError> {
    let mut it = game.into_inner();
    let block = it.next().ok_or(AstError::Shape("expected game block"))?;
    if block.as_rule() != Rule::game_block {
        return Err(AstError::Shape("expected game block"));
    }

    let mut title: Option<String> = None;
    let mut intro: Option<String> = None;
    let mut player: Option<PlayerAst> = None;
    let mut scoring: Option<ScoringAst> = None;

    for stmt in block.into_inner() {
        let inner_stmt = {
            let mut it = stmt.clone().into_inner();
            if let Some(p) = it.next() { p } else { stmt.clone() }
        };
        match inner_stmt.as_rule() {
            Rule::game_title => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing game title string"))?;
                title = Some(unquote(s.as_str()));
            },
            Rule::game_intro => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing game intro string"))?;
                intro = Some(unquote(s.as_str()));
            },
            Rule::game_player => {
                let block = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing player block"))?;
                player = Some(parse_player_block(block)?);
            },
            Rule::game_scoring => {
                let block = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing scoring block"))?;
                scoring = Some(parse_scoring_block(block)?);
            },
            _ => {},
        }
    }

    Ok(GameAst {
        title: title.ok_or(AstError::Shape("missing game title"))?,
        intro: intro.ok_or(AstError::Shape("missing game intro"))?,
        player: player.ok_or(AstError::Shape("missing player block"))?,
        scoring,
    })
}

fn parse_player_block(block: pest::iterators::Pair<Rule>) -> Result<PlayerAst, AstError> {
    if block.as_rule() != Rule::player_block {
        return Err(AstError::Shape("expected player block"));
    }

    let mut name: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut max_hp: Option<u32> = None;
    let mut start_room: Option<String> = None;

    for stmt in block.into_inner() {
        let inner_stmt = {
            let mut it = stmt.clone().into_inner();
            if let Some(p) = it.next() { p } else { stmt.clone() }
        };
        match inner_stmt.as_rule() {
            Rule::player_name => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing player name string"))?;
                name = Some(unquote(s.as_str()));
            },
            Rule::player_desc => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing player desc string"))?;
                desc = Some(unquote(s.as_str()));
            },
            Rule::player_max_hp => {
                let tok = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing player max_hp"))?;
                let value = tok
                    .as_str()
                    .parse::<u32>()
                    .map_err(|_| AstError::Shape("player max_hp must be a positive integer"))?;
                max_hp = Some(value);
            },
            Rule::player_start => {
                let mut it = inner_stmt.into_inner();
                let room = it
                    .next()
                    .ok_or(AstError::Shape("missing player start room id"))?
                    .as_str()
                    .to_string();
                start_room = Some(room);
            },
            _ => {},
        }
    }

    Ok(PlayerAst {
        name: name.ok_or(AstError::Shape("missing player name"))?,
        description: desc.ok_or(AstError::Shape("missing player description"))?,
        max_hp: max_hp.ok_or(AstError::Shape("missing player max_hp"))?,
        start_room: start_room.ok_or(AstError::Shape("missing player start room"))?,
    })
}

fn parse_scoring_block(block: pest::iterators::Pair<Rule>) -> Result<ScoringAst, AstError> {
    if block.as_rule() != Rule::scoring_block {
        return Err(AstError::Shape("expected scoring block"));
    }

    let mut report_title: Option<String> = None;
    let mut ranks: Vec<ScoringRankAst> = Vec::new();

    for stmt in block.into_inner() {
        let inner_stmt = {
            let mut it = stmt.clone().into_inner();
            if let Some(p) = it.next() { p } else { stmt.clone() }
        };
        match inner_stmt.as_rule() {
            Rule::scoring_title => {
                let s = inner_stmt
                    .into_inner()
                    .next()
                    .ok_or(AstError::Shape("missing scoring report_title string"))?;
                report_title = Some(unquote(s.as_str()));
            },
            Rule::scoring_rank => {
                let mut it = inner_stmt.into_inner();
                let threshold_tok = it.next().ok_or(AstError::Shape("missing scoring rank threshold"))?;
                let threshold = threshold_tok
                    .as_str()
                    .parse::<f32>()
                    .map_err(|_| AstError::Shape("invalid scoring rank threshold"))?;
                let name = it.next().ok_or(AstError::Shape("missing scoring rank name"))?;
                let description = it.next().ok_or(AstError::Shape("missing scoring rank description"))?;
                ranks.push(ScoringRankAst {
                    threshold,
                    name: unquote(name.as_str()),
                    description: unquote(description.as_str()),
                });
            },
            _ => {},
        }
    }

    if ranks.is_empty() {
        return Err(AstError::Shape("scoring block requires at least one rank"));
    }

    Ok(ScoringAst { report_title, ranks })
}
