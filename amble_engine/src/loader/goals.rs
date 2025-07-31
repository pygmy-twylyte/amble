//! module `loader::goals`

use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    Goal,
    goal::{GoalCondition, GoalGroup},
    loader::SymbolTable,
};

/// The raw version of a `Goal` loaded from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawGoal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group: GoalGroup,
    pub activate_when: Option<RawGoalCondition>, // None = always active / visible
    pub finished_when: RawGoalCondition,
    pub failed_when: Option<RawGoalCondition>,
}
impl RawGoal {
    /// Converts a `RawGoal` from TOML to a `Goal`
    /// # Errors
    /// - on failed symbol lookup
    pub fn to_goal(&self, symbols: &SymbolTable) -> Result<Goal> {
        let act_when = self
            .activate_when
            .as_ref()
            .map(|raw| raw.to_goal_condition(symbols))
            .transpose()?;
        let fail_when = self
            .failed_when
            .as_ref()
            .map(|raw| raw.to_goal_condition(symbols))
            .transpose()?;
        let done_when = self.finished_when.to_goal_condition(symbols)?;

        Ok(Goal {
            id: self.id.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            group: self.group,
            activate_when: act_when,
            finished_when: done_when,
            failed_when: fail_when,
        })
    }
}

/// The raw version of a `GoalCondition` from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RawGoalCondition {
    HasItem { item_sym: String },
    HasFlag { flag: String },
    MissingFlag { flag: String },
    ReachedRoom { room_sym: String },
    GoalComplete { goal_id: String }, // for activating a goal after another is done
    FlagComplete { flag: String },    // for determining whether a sequence is at end
}

impl RawGoalCondition {
    /// Converts a `RawGoalCondition` from TOML to a `GoalCondition`
    /// # Errors
    /// - on failed symbol lookup
    pub fn to_goal_condition(&self, symbols: &SymbolTable) -> Result<GoalCondition> {
        match self {
            Self::FlagComplete { flag } => Ok(GoalCondition::FlagComplete { flag: flag.to_string() }),
            Self::GoalComplete { goal_id } => Ok(GoalCondition::GoalComplete {
                goal_id: goal_id.to_string(),
            }),
            Self::HasFlag { flag } => Ok(GoalCondition::HasFlag { flag: flag.to_string() }),
            Self::MissingFlag { flag } => Ok(GoalCondition::MissingFlag { flag: flag.to_string() }),
            Self::HasItem { item_sym } => {
                if let Some(uuid) = symbols.items.get(item_sym) {
                    Ok(GoalCondition::HasItem { item_id: *uuid })
                } else {
                    bail!("converting RawGoalCondition::HasItem({item_sym}): symbol not found");
                }
            },
            Self::ReachedRoom { room_sym } => {
                if let Some(uuid) = symbols.rooms.get(room_sym) {
                    Ok(GoalCondition::ReachedRoom { room_id: *uuid })
                } else {
                    bail!("converting RawGoalCondition::ReachedRoom({room_sym}): symbol not found");
                }
            },
        }
    }
}

/// Needed for loading a "bare" goal vector from TOML
#[derive(Deserialize)]
pub struct RawGoalFile {
    pub goals: Vec<RawGoal>,
}

/// Load `RawGoals` vector from TOML file.
/// # Errors
/// - on failed read of file indicated in `toml_path`
/// - on error parsing the file
pub fn load_raw_goals(toml_path: &Path) -> Result<Vec<RawGoal>> {
    let goal_file =
        fs::read_to_string(toml_path).with_context(|| format!("reading goal data from '{}'", toml_path.display()))?;
    let wrapper: RawGoalFile = toml::from_str(&goal_file)?;
    info!(
        "{} raw goals loaded from '{}'",
        wrapper.goals.len(),
        toml_path.display()
    );
    Ok(wrapper.goals)
}

/// Build `Goals` from `RawGoals`
/// # Errors
/// - on failed uuid lookups in the symbol table
pub fn build_goals(raw_goals: &[RawGoal], symbols: &SymbolTable) -> Result<Vec<Goal>> {
    let goals: Vec<Goal> = raw_goals
        .iter()
        .map(|rg| rg.to_goal(symbols))
        .collect::<Result<_, _>>()?;
    info!("{} goals built from raw goals", goals.len());
    Ok(goals)
}
