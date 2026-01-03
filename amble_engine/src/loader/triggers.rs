//! Helpers for loading [`Trigger`] definitions from TOML.
//!
//! Triggers combine game conditions with actions. This module converts the raw
//! text representation into fully linked structures that the engine can
//! evaluate at runtime.

pub mod raw_action;
pub mod raw_condition;

use raw_action::RawActionStmt;
use raw_condition::RawTriggerCondition;

use std::{fs, path::Path};

use anyhow::{Context, Result};
use log::info;
use serde::Deserialize;

use crate::scheduler::EventCondition;
use crate::trigger::Trigger;

use super::SymbolTable;

#[derive(Deserialize)]
struct RawTriggerFile {
    triggers: Vec<RawTrigger>,
}

#[derive(Debug, Deserialize)]
pub struct RawTrigger {
    pub name: String,
    pub conditions: Vec<RawTriggerCondition>,
    pub actions: Vec<RawActionStmt>,
    #[serde(default)]
    pub only_once: bool,
}
impl RawTrigger {
    /// Convert a `RawTrigger` loaded from TOML to a `Trigger`
    /// # Errors
    /// - on failed symbol lookups / failure to convert any component of the trigger
    pub fn to_trigger(&self, symbols: &SymbolTable) -> Result<Trigger> {
        let mut conditions = Vec::new();
        let mut actions = Vec::new();
        for cond in &self.conditions {
            conditions.push(EventCondition::Trigger(cond.to_condition(symbols)?));
        }
        for act in &self.actions {
            actions.push(act.to_action(symbols)?);
        }
        Ok(Trigger {
            name: self.name.clone(),
            conditions: EventCondition::All(conditions),
            actions,
            only_once: self.only_once,
            fired: false,
        })
    }
}

/// Load `RawTrigger` representations from TOML
/// # Errors
/// - on failed file access or TOML parsing
pub fn load_raw_triggers(toml_path: &Path) -> Result<Vec<RawTrigger>> {
    let trigger_file =
        fs::read_to_string(toml_path).with_context(|| format!("reading triggers from \"{}\"", toml_path.display()))?;
    let wrapper: RawTriggerFile = toml::from_str(&trigger_file)?;
    info!(
        "{} raw triggers loaded from '{}'",
        wrapper.triggers.len(),
        toml_path.display(),
    );
    Ok(wrapper.triggers)
}
/// Build `Triggers` from `RawTriggers` loaded from TOML.
/// # Errors
/// - on failed conversion of any raw to real trigger
pub fn build_triggers(raw_triggers: &[RawTrigger], symbols: &SymbolTable) -> Result<Vec<Trigger>> {
    let triggers: Vec<Trigger> = raw_triggers
        .iter()
        .map(|rt| rt.to_trigger(symbols))
        .collect::<Result<_, _>>()?;
    info!("{} triggers built from raw triggers", triggers.len());
    Ok(triggers)
}
