//! Core ID types used across the engine.
//!
//! `RoomId` is already a newtype; `ItemId` and `NpcId` are aliases for now and
//! can be migrated to newtypes later without relocating their definitions again.

use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};

pub type Id = String;

// These aliases are being kept in one place so future newtype migrations are localized.
pub type ItemId = Id;
pub type NpcId = Id;

/// Typed identifier for item-or-npc search results.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityId {
    Item(ItemId),
    Npc(NpcId),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Stable identifier type for a `Room`.
pub struct RoomId(pub(crate) String);
impl RoomId {
    pub fn new(id: &impl ToString) -> Self {
        Self(id.to_string())
    }
}
impl From<&str> for RoomId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}
impl From<String> for RoomId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
impl Deref for RoomId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<str> for RoomId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl PartialEq<String> for RoomId {
    fn eq(&self, other: &String) -> bool {
        *other == self.0
    }
}
impl Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
