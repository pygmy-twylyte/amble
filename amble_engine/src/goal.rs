//! Goal definitions and progress evaluation.
//!
//! Provides the data structures that track player objectives along with
//! helpers for determining whether goal conditions are satisfied.
//!
//! `Goal`s are stateless -- goal completions are not marked or stored -- just
//! three conditions upon creation:
//!
//! - when it becomes active / potentially achievable
//! - when it is considered complete
//! - when it is failed (active but can never be completed)
//!
//! Goals are only evaluated if/when the player issues a "goals" command.
//! When they do, each defined goal's conditions are checked and it is assigned
//! a `GoalStatus`, which is then used to filter/style goals and their descriptions
//! for display.

use crate::Id;
use serde::de::{self, Deserializer, EnumAccess, VariantAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use variantly::Variantly;

use crate::{AmbleWorld, ItemHolder, player::Flag};

/// Groups that goals can be assigned to.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum GoalGroup {
    Required,
    Optional,
    StatusEffect,
}

/// Types of conditions that can activate or complete a goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GoalCondition {
    FlagComplete { flag: String },    // for checking if sequence-type flags are at end
    FlagInProgress { flag: String },  // check if a sequence flag not yet at end
    GoalComplete { goal_id: String }, // for activating a goal after another is done
    HasItem { item_id: Id },
    HasFlag { flag: String },
    MissingFlag { flag: String },
    ReachedRoom { room_id: Id },
}
impl GoalCondition {
    /// Returns true if the condition has been satisfied.
    pub fn satisfied(&self, world: &AmbleWorld) -> bool {
        // Helper closure to check if a flag is set by comparing flag values
        // This works because Flag::value() returns the current state representation
        // with sequence flags in the form <flag>#<step>.
        let flag_is_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);

        match self {
            Self::HasItem { item_id } => world.player.contains_item(item_id.clone()),
            Self::HasFlag { flag } => flag_is_set(flag),
            Self::MissingFlag { flag } => !flag_is_set(flag),
            Self::ReachedRoom { room_id } => {
                if let Some(room) = world.rooms.get(room_id) {
                    room.visited
                } else {
                    false
                }
            },
            Self::GoalComplete { goal_id } => {
                if let Some(goal) = world.goals.iter().find(|g| g.id == *goal_id) {
                    goal.status(world) == GoalStatus::Complete
                } else {
                    false
                }
            },
            Self::FlagInProgress { flag } => world
                .player
                .flags
                .get(&Flag::Simple {
                    name: flag.into(),
                    turn_set: 0,
                })
                .is_some_and(|f| !f.is_complete()),
            Self::FlagComplete { flag } => {
                let target = Flag::simple(flag, world.turn_count);
                world.player.flags.get(&target).is_some_and(Flag::is_complete)
            },
        }
    }
}

/// Represents current state of the `Goal`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Variantly)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GoalStatus {
    Inactive,
    Active,
    Complete,
    Failed,
}

fn deserialize_maybe_value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Maybe<T> {
        Value(T),
        Option(Option<T>),
    }

    match Maybe::deserialize(deserializer)? {
        Maybe::Value(value) => Ok(Some(value)),
        Maybe::Option(opt) => Ok(opt),
    }
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum GoalGroupKind {
    Required,
    Optional,
    StatusEffect,
}

impl GoalGroupKind {
    fn from_str(value: &str) -> Option<Self> {
        let value = value.strip_prefix("r#").unwrap_or(value);
        match value {
            v if v.eq_ignore_ascii_case("required") => Some(Self::Required),
            v if v.eq_ignore_ascii_case("optional") => Some(Self::Optional),
            v if v.eq_ignore_ascii_case("status-effect") => Some(Self::StatusEffect),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for GoalGroupKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = GoalGroupKind;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("goal group identifier")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                GoalGroupKind::from_str(value)
                    .ok_or_else(|| de::Error::unknown_variant(value, &["required", "optional", "status-effect"]))
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(value)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (variant, access) = data.variant::<String>()?;
                access.unit_variant()?;
                self.visit_str(&variant)
            }
        }

        deserializer.deserialize_any(KindVisitor)
    }
}

#[derive(Deserialize)]
struct GoalGroupRepr {
    #[serde(rename = "type")]
    kind: GoalGroupKind,
}

impl<'de> Deserialize<'de> for GoalGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = GoalGroupRepr::deserialize(deserializer)?;
        Ok(match repr.kind {
            GoalGroupKind::Required => GoalGroup::Required,
            GoalGroupKind::Optional => GoalGroup::Optional,
            GoalGroupKind::StatusEffect => GoalGroup::StatusEffect,
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum GoalConditionKind {
    FlagComplete,
    FlagInProgress,
    GoalComplete,
    HasItem,
    HasFlag,
    MissingFlag,
    ReachedRoom,
}

impl GoalConditionKind {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            v if v.eq_ignore_ascii_case("flagComplete") => Some(Self::FlagComplete),
            v if v.eq_ignore_ascii_case("flagInProgress") => Some(Self::FlagInProgress),
            v if v.eq_ignore_ascii_case("goalComplete") => Some(Self::GoalComplete),
            v if v.eq_ignore_ascii_case("hasItem") => Some(Self::HasItem),
            v if v.eq_ignore_ascii_case("hasFlag") => Some(Self::HasFlag),
            v if v.eq_ignore_ascii_case("missingFlag") => Some(Self::MissingFlag),
            v if v.eq_ignore_ascii_case("reachedRoom") => Some(Self::ReachedRoom),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for GoalConditionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = GoalConditionKind;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("goal condition identifier")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                GoalConditionKind::from_str(value).ok_or_else(|| {
                    de::Error::unknown_variant(
                        value,
                        &[
                            "flagComplete",
                            "flagInProgress",
                            "goalComplete",
                            "hasItem",
                            "hasFlag",
                            "missingFlag",
                            "reachedRoom",
                        ],
                    )
                })
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(value)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (variant, access) = data.variant::<String>()?;
                access.unit_variant()?;
                self.visit_str(&variant)
            }
        }

        deserializer.deserialize_any(KindVisitor)
    }
}

#[derive(Deserialize)]
struct GoalConditionRepr {
    #[serde(rename = "type")]
    kind: GoalConditionKind,
    #[serde(default, deserialize_with = "deserialize_maybe_value")]
    flag: Option<String>,
    #[serde(default, deserialize_with = "deserialize_maybe_value")]
    goal_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_maybe_value")]
    item_id: Option<Id>,
    #[serde(default, deserialize_with = "deserialize_maybe_value")]
    room_id: Option<Id>,
}

impl<'de> Deserialize<'de> for GoalCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = GoalConditionRepr::deserialize(deserializer)?;
        Ok(match repr.kind {
            GoalConditionKind::FlagComplete => GoalCondition::FlagComplete {
                flag: repr.flag.ok_or_else(|| de::Error::missing_field("flag"))?,
            },
            GoalConditionKind::FlagInProgress => GoalCondition::FlagInProgress {
                flag: repr.flag.ok_or_else(|| de::Error::missing_field("flag"))?,
            },
            GoalConditionKind::GoalComplete => GoalCondition::GoalComplete {
                goal_id: repr.goal_id.ok_or_else(|| de::Error::missing_field("goal_id"))?,
            },
            GoalConditionKind::HasItem => GoalCondition::HasItem {
                item_id: repr.item_id.ok_or_else(|| de::Error::missing_field("item_id"))?,
            },
            GoalConditionKind::HasFlag => GoalCondition::HasFlag {
                flag: repr.flag.ok_or_else(|| de::Error::missing_field("flag"))?,
            },
            GoalConditionKind::MissingFlag => GoalCondition::MissingFlag {
                flag: repr.flag.ok_or_else(|| de::Error::missing_field("flag"))?,
            },
            GoalConditionKind::ReachedRoom => GoalCondition::ReachedRoom {
                room_id: repr.room_id.ok_or_else(|| de::Error::missing_field("room_id"))?,
            },
        })
    }
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum GoalStatusKind {
    Inactive,
    Active,
    Complete,
    Failed,
}

impl GoalStatusKind {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            v if v.eq_ignore_ascii_case("inactive") => Some(Self::Inactive),
            v if v.eq_ignore_ascii_case("active") => Some(Self::Active),
            v if v.eq_ignore_ascii_case("complete") => Some(Self::Complete),
            v if v.eq_ignore_ascii_case("failed") => Some(Self::Failed),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for GoalStatusKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = GoalStatusKind;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("goal status identifier")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                GoalStatusKind::from_str(value)
                    .ok_or_else(|| de::Error::unknown_variant(value, &["inactive", "active", "complete", "failed"]))
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(value)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (variant, access) = data.variant::<String>()?;
                access.unit_variant()?;
                self.visit_str(&variant)
            }
        }

        deserializer.deserialize_any(KindVisitor)
    }
}

#[derive(Deserialize)]
struct GoalStatusRepr {
    #[serde(rename = "type")]
    kind: GoalStatusKind,
}

impl<'de> Deserialize<'de> for GoalStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = GoalStatusRepr::deserialize(deserializer)?;
        Ok(match repr.kind {
            GoalStatusKind::Inactive => GoalStatus::Inactive,
            GoalStatusKind::Active => GoalStatus::Active,
            GoalStatusKind::Complete => GoalStatus::Complete,
            GoalStatusKind::Failed => GoalStatus::Failed,
        })
    }
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
        if let Some(fail_condition) = &self.failed_when
            && fail_condition.satisfied(world)
        {
            return GoalStatus::Failed;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{Item, Movability},
        player::Flag,
        room::Room,
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::new_empty();

        // Add test room
        let room_id = crate::idgen::new_id();
        let mut room = Room {
            id: room_id.clone(),
            symbol: "test_room".into(),
            name: "Test Room".into(),
            base_description: "A test room".into(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        room.visited = true;
        world.rooms.insert(room_id.clone(), room);

        // Add test item
        let item_id = crate::idgen::new_id();
        let item = Item {
            id: item_id.clone(),
            symbol: "test_item".into(),
            name: "Test Item".into(),
            description: "A test item".into(),
            location: Location::Inventory,
            movability: Movability::Free,
            container_state: None,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(item_id.clone(), item);
        world.player.inventory.insert(item_id.clone());

        // Add test flags
        world.player.flags.insert(Flag::simple("test_flag", world.turn_count));
        world
            .player
            .flags
            .insert(Flag::sequence("test_seq", Some(2), world.turn_count));

        world
    }

    #[test]
    fn goal_condition_flag_complete_works() {
        let mut world = create_test_world();

        // Add completed sequence flag
        let mut seq_flag = Flag::sequence("completed_seq", Some(2), world.turn_count);
        seq_flag.advance(); // step 1
        seq_flag.advance(); // step 2 (complete)
        world.player.flags.insert(seq_flag);

        let condition = GoalCondition::FlagComplete {
            flag: "completed_seq".into(),
        };
        assert!(condition.satisfied(&world));

        let condition = GoalCondition::FlagComplete {
            flag: "nonexistent".into(),
        };
        assert!(!condition.satisfied(&world));
    }

    #[test]
    fn goal_condition_flag_in_progress_works() {
        let mut world = create_test_world();

        // Advance the test sequence flag
        world.player.advance_flag("test_seq");

        let condition = GoalCondition::FlagInProgress {
            flag: "test_seq".into(),
        };
        assert!(condition.satisfied(&world));

        let condition = GoalCondition::FlagInProgress {
            flag: "nonexistent".into(),
        };
        assert!(!condition.satisfied(&world));
    }

    #[test]
    fn goal_condition_goal_complete_works() {
        let world = create_test_world();

        // Create a completed goal
        let completed_goal = Goal {
            id: "completed_goal".into(),
            name: "Completed Goal".into(),
            description: "A completed goal".into(),
            group: GoalGroup::Required,
            activate_when: None,
            finished_when: GoalCondition::HasFlag {
                flag: "test_flag".into(),
            },
            failed_when: None,
        };

        // Create a goal that depends on the completed goal
        let dependent_goal = Goal {
            id: "dependent_goal".into(),
            name: "Dependent Goal".into(),
            description: "A goal that depends on another".into(),
            group: GoalGroup::Optional,
            activate_when: Some(GoalCondition::GoalComplete {
                goal_id: "completed_goal".into(),
            }),
            finished_when: GoalCondition::HasFlag {
                flag: "nonexistent".into(),
            },
            failed_when: None,
        };

        // Test with completed goal in world
        let mut world_with_goals = world.clone();
        world_with_goals.goals.push(completed_goal);
        world_with_goals.goals.push(dependent_goal);

        let condition = GoalCondition::GoalComplete {
            goal_id: "completed_goal".into(),
        };
        assert!(condition.satisfied(&world_with_goals));

        let condition = GoalCondition::GoalComplete {
            goal_id: "nonexistent".into(),
        };
        assert!(!condition.satisfied(&world_with_goals));
    }

    #[test]
    fn goal_condition_has_item_works() {
        let world = create_test_world();
        let item_id = world.player.inventory.iter().next().cloned().unwrap();

        let condition = GoalCondition::HasItem { item_id };
        assert!(condition.satisfied(&world));

        let condition = GoalCondition::HasItem {
            item_id: crate::idgen::new_id(),
        };
        assert!(!condition.satisfied(&world));
    }

    #[test]
    fn goal_condition_has_flag_works() {
        let world = create_test_world();

        let condition = GoalCondition::HasFlag {
            flag: "test_flag".into(),
        };
        assert!(condition.satisfied(&world));

        let condition = GoalCondition::HasFlag {
            flag: "nonexistent".into(),
        };
        assert!(!condition.satisfied(&world));
    }

    #[test]
    fn goal_condition_missing_flag_works() {
        let world = create_test_world();

        let condition = GoalCondition::MissingFlag {
            flag: "nonexistent".into(),
        };
        assert!(condition.satisfied(&world));

        let condition = GoalCondition::MissingFlag {
            flag: "test_flag".into(),
        };
        assert!(!condition.satisfied(&world));
    }

    #[test]
    fn goal_condition_reached_room_works() {
        let world = create_test_world();
        let room_id = world.rooms.keys().next().cloned().unwrap();

        let condition = GoalCondition::ReachedRoom { room_id };
        assert!(condition.satisfied(&world));

        let condition = GoalCondition::ReachedRoom {
            room_id: crate::idgen::new_id(),
        };
        assert!(!condition.satisfied(&world));
    }

    #[test]
    fn goal_status_inactive_when_activation_condition_not_met() {
        let world = create_test_world();

        let goal = Goal {
            id: "test_goal".into(),
            name: "Test Goal".into(),
            description: "A test goal".into(),
            group: GoalGroup::Required,
            activate_when: Some(GoalCondition::HasFlag {
                flag: "nonexistent".into(),
            }),
            finished_when: GoalCondition::HasFlag {
                flag: "test_flag".into(),
            },
            failed_when: None,
        };

        assert_eq!(goal.status(&world), GoalStatus::Inactive);
    }

    #[test]
    fn goal_status_active_when_conditions_met_but_not_finished() {
        let world = create_test_world();

        let goal = Goal {
            id: "test_goal".into(),
            name: "Test Goal".into(),
            description: "A test goal".into(),
            group: GoalGroup::Required,
            activate_when: Some(GoalCondition::HasFlag {
                flag: "test_flag".into(),
            }),
            finished_when: GoalCondition::HasFlag {
                flag: "nonexistent".into(),
            },
            failed_when: None,
        };

        assert_eq!(goal.status(&world), GoalStatus::Active);
    }

    #[test]
    fn goal_status_complete_when_finished_condition_met() {
        let world = create_test_world();

        let goal = Goal {
            id: "test_goal".into(),
            name: "Test Goal".into(),
            description: "A test goal".into(),
            group: GoalGroup::Required,
            activate_when: Some(GoalCondition::HasFlag {
                flag: "test_flag".into(),
            }),
            finished_when: GoalCondition::HasFlag {
                flag: "test_flag".into(),
            },
            failed_when: None,
        };

        assert_eq!(goal.status(&world), GoalStatus::Complete);
    }

    #[test]
    fn goal_status_failed_when_failure_condition_met() {
        let world = create_test_world();

        let goal = Goal {
            id: "test_goal".into(),
            name: "Test Goal".into(),
            description: "A test goal".into(),
            group: GoalGroup::Required,
            activate_when: None,
            finished_when: GoalCondition::HasFlag {
                flag: "nonexistent".into(),
            },
            failed_when: Some(GoalCondition::HasFlag {
                flag: "test_flag".into(),
            }),
        };

        assert_eq!(goal.status(&world), GoalStatus::Failed);
    }

    #[test]
    fn goal_status_active_when_no_activation_condition_and_not_finished() {
        let world = create_test_world();

        let goal = Goal {
            id: "test_goal".into(),
            name: "Test Goal".into(),
            description: "A test goal".into(),
            group: GoalGroup::Required,
            activate_when: None,
            finished_when: GoalCondition::HasFlag {
                flag: "nonexistent".into(),
            },
            failed_when: None,
        };

        assert_eq!(goal.status(&world), GoalStatus::Active);
    }

    #[test]
    fn goal_status_complete_when_no_activation_condition_and_finished() {
        let world = create_test_world();

        let goal = Goal {
            id: "test_goal".into(),
            name: "Test Goal".into(),
            description: "A test goal".into(),
            group: GoalGroup::Required,
            activate_when: None,
            finished_when: GoalCondition::HasFlag {
                flag: "test_flag".into(),
            },
            failed_when: None,
        };

        assert_eq!(goal.status(&world), GoalStatus::Complete);
    }

    #[test]
    fn goal_groups_are_properly_defined() {
        // Test that goal groups serialize/deserialize correctly
        let required = GoalGroup::Required;
        let optional = GoalGroup::Optional;
        let status_effect = GoalGroup::StatusEffect;

        assert_eq!(format!("{required:?}"), "Required");
        assert_eq!(format!("{optional:?}"), "Optional");
        assert_eq!(format!("{status_effect:?}"), "StatusEffect");
    }

    #[test]
    fn goal_status_variants_work() {
        // Test that goal status variants are properly defined
        assert_eq!(GoalStatus::Inactive, GoalStatus::Inactive);
        assert_eq!(GoalStatus::Active, GoalStatus::Active);
        assert_eq!(GoalStatus::Complete, GoalStatus::Complete);
        assert_eq!(GoalStatus::Failed, GoalStatus::Failed);

        assert_ne!(GoalStatus::Inactive, GoalStatus::Active);
        assert_ne!(GoalStatus::Complete, GoalStatus::Failed);
    }
}
