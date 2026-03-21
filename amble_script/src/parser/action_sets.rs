use std::collections::HashMap;

use crate::{ActionSetSpec, ActionStmt, ConditionAst};

use super::AstError;
use super::actions::parse_actions_from_body;
use super::helpers::SourceMap;

pub(super) fn resolve_action_sets(
    specs: &[ActionSetSpec],
    cond_aliases: &HashMap<String, ConditionAst>,
) -> Result<HashMap<String, Vec<ActionStmt>>, AstError> {
    resolve_action_sets_with_base(specs, cond_aliases, &HashMap::new())
}

pub(super) fn resolve_action_sets_with_base(
    specs: &[ActionSetSpec],
    cond_aliases: &HashMap<String, ConditionAst>,
    base_action_sets: &HashMap<String, Vec<ActionStmt>>,
) -> Result<HashMap<String, Vec<ActionStmt>>, AstError> {
    let mut by_name: HashMap<&str, &ActionSetSpec> = HashMap::new();
    for spec in specs {
        if by_name.insert(spec.name.as_str(), spec).is_some() {
            return Err(AstError::ShapeAt {
                msg: "duplicate action set",
                context: spec.name.clone(),
            });
        }
    }

    let mut resolver = ActionSetResolver {
        specs: by_name,
        cond_aliases,
        base_action_sets,
        resolved: HashMap::new(),
        visiting: Vec::new(),
    };
    let names = resolver.specs.keys().copied().collect::<Vec<_>>();
    for name in names {
        resolver.resolve_action_set(name)?;
    }
    Ok(resolver.resolved)
}

struct ActionSetResolver<'a> {
    specs: HashMap<&'a str, &'a ActionSetSpec>,
    cond_aliases: &'a HashMap<String, ConditionAst>,
    base_action_sets: &'a HashMap<String, Vec<ActionStmt>>,
    resolved: HashMap<String, Vec<ActionStmt>>,
    visiting: Vec<String>,
}

impl ActionSetResolver<'_> {
    fn resolve_action_set(&mut self, name: &str) -> Result<Option<Vec<ActionStmt>>, AstError> {
        if let Some(actions) = self.resolved.get(name) {
            return Ok(Some(actions.clone()));
        }
        let Some(spec) = self.specs.get(name).copied() else {
            return Ok(self.base_action_sets.get(name).cloned());
        };
        if let Some(idx) = self.visiting.iter().position(|current| current == name) {
            let mut cycle = self.visiting[idx..].to_vec();
            cycle.push(name.to_string());
            return Err(AstError::ShapeAt {
                msg: "recursive action set",
                context: cycle.join(" -> "),
            });
        }

        self.visiting.push(name.to_string());
        let parsed = {
            let smap = SourceMap::new(&spec.text);
            let cond_aliases = self.cond_aliases;
            let mut lookup = |candidate: &str| self.resolve_action_set(candidate);
            parse_actions_from_body(&spec.text, &spec.text, &smap, &spec.sets, cond_aliases, &mut lookup)?
        };
        self.visiting.pop();
        self.resolved.insert(name.to_string(), parsed.clone());
        Ok(Some(parsed))
    }
}
