//! Player -- module for a player in Amble
use crate::{ItemHolder, Location, WorldObject};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// The player-controlled character.
///
/// This struct tracks the player's state, such as inventory, score and flags.
#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub inventory: HashSet<Uuid>,
    pub flags: HashSet<Flag>,
    pub score: usize,
}
impl Player {
    /// Updates one of the existing flags. Emits a warning if the flag isn't found.
    pub fn update_flag<F>(&mut self, name: &str, updater: F)
    where
        F: FnOnce(&mut Flag),
    {
        let target = Flag::Simple { name: name.to_string() };
        if let Some(mut flag) = self.flags.take(&target) {
            updater(&mut flag);
            info!("player flag updated: '{flag}'");
            self.flags.insert(flag);
        } else {
            warn!("update_flag: flag '{name}' not set");
        }
    }

    /// Advances a sequence flag to the next step.
    pub fn advance_flag(&mut self, name: &str) {
        self.update_flag(name, |f| f.advance());
    }
}
impl Default for Player {
    fn default() -> Player {
        Self {
            id: Uuid::new_v4(),
            symbol: "the_candidate".into(),
            name: "The Candidate".into(),
            description: "default".into(),
            location: Location::default(),
            inventory: HashSet::<Uuid>::default(),
            flags: HashSet::<Flag>::default(),
            score: 1,
        }
    }
}
impl WorldObject for Player {
    fn id(&self) -> Uuid {
        self.id
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn location(&self) -> &Location {
        &self.location
    }
}
impl ItemHolder for Player {
    fn add_item(&mut self, item_id: Uuid) {
        self.inventory.insert(item_id);
    }

    fn remove_item(&mut self, item_id: Uuid) {
        self.inventory.remove(&item_id);
    }

    fn contains_item(&self, item_id: Uuid) -> bool {
        self.inventory.contains(&item_id)
    }
}

/// Flags that can be applied to the player
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Flag {
    Simple {
        name: String,
    },
    Sequence {
        name: String,
        #[serde(default)]
        step: u8,
        #[serde(default)]
        end: Option<u8>,
    },
}
impl Flag {
    /// Return string value of the flag.
    pub fn value(&self) -> String {
        match self {
            Self::Simple { name } => name.to_string(),
            Self::Sequence { name, step, .. } => format_sequence_value(name, *step),
        }
    }

    /// Advances to next step of a sequence
    ///
    /// Logs a warning and does nothing if called on a simple flag.
    pub fn advance(&mut self) {
        match self {
            Flag::Simple { name, .. } => {
                warn!("advance() called on non-sequence flag '{name}'");
            },
            Flag::Sequence { name, step, end } => {
                if let Some(final_step) = end {
                    *step = std::cmp::min(*step + 1, *final_step);
                } else {
                    *step += 1;
                }
                info!("sequence '{name}' advanced to step {step}");
            },
        }
    }

    /// Resets to beginning of sequence
    pub fn reset(&mut self) {
        match self {
            Flag::Simple { name } => warn!("reset() called on non-sequence flag '{name}'"),
            Flag::Sequence { name, step, .. } => {
                *step = 0;
                info!("sequence '{name}' reset to step '{step}'")
            },
        }
    }

    /// Create a new simple flag
    pub fn simple(name: &str) -> Flag {
        Flag::Simple { name: name.to_string() }
    }

    /// Create a new sequence flag
    pub fn sequence(name: &str, end: Option<u8>) -> Flag {
        Flag::Sequence {
            name: name.to_string(),
            step: 0u8,
            end,
        }
    }

    /// Get base name of the flag
    pub fn name(&self) -> &str {
        match self {
            Flag::Simple { name } => name,
            Flag::Sequence { name, .. } => name,
        }
    }
}
impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::Simple { name } => write!(f, "{name}"),
            Flag::Sequence { name, step, .. } => write!(f, "{}#{}", name, step),
        }
    }
}
use std::hash::{Hash, Hasher};

impl PartialEq for Flag {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Flag {}

impl Hash for Flag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state)
    }
}

/// Formats a sequence-type flag into a string value
///
/// Format is <name>#<step>, e.g. "hal_reboot#2"
pub fn format_sequence_value(name: &str, step: u8) -> String {
    format!("{name}#{step}")
}
