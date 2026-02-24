//! Core ID types used across the engine.
//!
//! `RoomId`, `ItemId`, and `NpcId` are stable newtypes centralized here.

use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, fmt::Display, ops::Deref};

pub type Id = String;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ItemId(pub(crate) String);
impl ItemId {
    pub fn new(id: &impl ToString) -> Self {
        Self(id.to_string())
    }
}
impl From<&str> for ItemId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}
impl From<String> for ItemId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
impl Deref for ItemId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<str> for ItemId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Borrow<str> for ItemId {
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl Borrow<String> for ItemId {
    fn borrow(&self) -> &String {
        &self.0
    }
}
impl PartialEq<String> for ItemId {
    fn eq(&self, other: &String) -> bool {
        *other == self.0
    }
}
impl Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NpcId(pub(crate) String);
impl NpcId {
    pub fn new(id: &impl ToString) -> Self {
        Self(id.to_string())
    }
}
impl From<&str> for NpcId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}
impl From<String> for NpcId {
    fn from(id: String) -> Self {
        Self(id)
    }
}
impl Deref for NpcId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<str> for NpcId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Borrow<str> for NpcId {
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl Borrow<String> for NpcId {
    fn borrow(&self) -> &String {
        &self.0
    }
}
impl PartialEq<String> for NpcId {
    fn eq(&self, other: &String) -> bool {
        *other == self.0
    }
}
impl Display for NpcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
impl Borrow<str> for RoomId {
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl Borrow<String> for RoomId {
    fn borrow(&self) -> &String {
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
