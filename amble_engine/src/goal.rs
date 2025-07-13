//! module: goal
//!

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Groups that goals can be assigned to.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GoalGroup {
    Global,
    Exterior,
    BuildingMain,
    Sublevel1,
}

/// Types of conditions that can activate or complete a goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GoalCondition {
    HasItem { item_id: Uuid },
    HasFlag { flag: String },
    MissingFlag { flag: String },
    ReachedRoom { room_id: Uuid },
    GoalComplete { goal_id: String }, // for activating a goal after another is done
}

/// Represents current state of the `Goal`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GoalStatus {
    Inactive,
    Activated,
    Completed,
    Failed,
}

/// A goal for the player to achieve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group: GoalGroup,
    pub activate_when: Option<GoalCondition>, // None = always active / visible
    pub finished_when: GoalCondition,
}

