//! module: goal
//!

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use variantly::Variantly;

use crate::{AmbleWorld, ItemHolder};

/// Groups that goals can be assigned to.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum GoalGroup {
    Required,
    Optional,
    StatusEffect,
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
impl GoalCondition {
    /// Returns true if the condition has been satisfied.
    pub fn satisfied(&self, world: &AmbleWorld) -> bool {
        match self {
            GoalCondition::HasItem { item_id } => world.player.contains_item(*item_id),
            GoalCondition::HasFlag { flag } => world.player.flags.contains(flag),
            GoalCondition::MissingFlag { flag } => !world.player.flags.contains(flag),
            GoalCondition::ReachedRoom { room_id } => {
                if let Some(room) = world.rooms.get(room_id) {
                    room.visited
                } else {
                    false
                }
            }
            GoalCondition::GoalComplete { goal_id } => {
                if let Some(goal) = world.goals.iter().find(|g| g.id == *goal_id) {
                    goal.status(world) == GoalStatus::Complete
                } else {
                    false
                }
            }
        }
    }
}

/// Represents current state of the `Goal`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Variantly)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GoalStatus {
    Inactive,
    Active,
    Complete,
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
    pub failed_when: Option<GoalCondition>,
}
impl Goal {
    /// Determines and returns the current '`GoalStatus`' for this goal.
    pub fn status(&self, world: &AmbleWorld) -> GoalStatus {
        if let Some(fail_condition) = &self.failed_when {
            if fail_condition.satisfied(world) {
                return GoalStatus::Failed;
            }
        }

        if let Some(start_condition) = &self.activate_when {
            if start_condition.satisfied(world) {
                if self.finished_when.satisfied(world) {
                    GoalStatus::Complete
                } else {
                    GoalStatus::Active
                }
            } else {
                GoalStatus::Inactive
            }
        } else if self.finished_when.satisfied(world) {
            GoalStatus::Complete
        } else {
            GoalStatus::Active
        }
    }
}
