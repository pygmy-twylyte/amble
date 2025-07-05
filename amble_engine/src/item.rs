use crate::{Location, WorldObject, style::GameStyle, world::AmbleWorld};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use uuid::Uuid;
use variantly::Variantly;

/// Methods common to things that can hold items.
pub trait ItemHolder {
    fn add_item(&mut self, item_id: Uuid);
    fn remove_item(&mut self, item_id: Uuid);
    fn contains_item(&self, item_id: Uuid) -> bool;
}

/// Things an item can do.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ItemAbility {
    CutWood,
    Ignite,
    Insulate,
    Pluck,
    Pry,
    Read,
    Sharpen,
    Smash,
    TurnOn,
    TurnOff,
    Unlock(Option<Uuid>),
    Use,
}
impl Display for ItemAbility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemAbility::CutWood => write!(f, "cut wood"),
            ItemAbility::Ignite => write!(f, "ignite"),
            ItemAbility::Insulate => write!(f, "insulate"),
            ItemAbility::Read => write!(f, "read"),
            ItemAbility::Sharpen => write!(f, "sharpen"),
            ItemAbility::TurnOn => write!(f, "turn on"),
            ItemAbility::TurnOff => write!(f, "turn off"),
            ItemAbility::Unlock(_) => write!(f, "unlock"),
            ItemAbility::Use => write!(f, "use"),
            ItemAbility::Pluck => write!(f, "pluck"),
            ItemAbility::Pry => write!(f, "pry"),
            ItemAbility::Smash => write!(f, "smash"),
        }
    }
}

/// Things you can do to an item, but only with certain other items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ItemInteractionType {
    Break,
    Burn,
    Cover,
    Cut,
    Handle,
    Move,
    Open,
    Sharpen,
    Turn,
    Unlock,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Variantly)]
#[serde(rename_all = "camelCase")]
pub enum ContainerState {
    Open,
    Closed,
    Locked,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub portable: bool,
    pub container_state: Option<ContainerState>,
    pub restricted: bool,
    pub contents: HashSet<Uuid>,
    pub abilities: HashSet<ItemAbility>,
    pub interaction_requires: HashMap<ItemInteractionType, ItemAbility>,
    pub text: Option<String>,
}
impl WorldObject for Item {
    fn id(&self) -> Uuid {
        self.id
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
impl ItemHolder for Item {
    fn add_item(&mut self, item_id: Uuid) {
        if self.container_state.is_some() && self.id.ne(&item_id) {
            self.contents.insert(item_id);
        }
    }
    fn remove_item(&mut self, item_id: Uuid) {
        if self.container_state.is_some() {
            self.contents.remove(&item_id);
        }
    }
    fn contains_item(&self, item_id: Uuid) -> bool {
        self.contents.contains(&item_id)
    }
}
impl Item {
    /// Returns true if item's contents can be accessed.
    pub fn is_accessible(&self) -> bool {
        self.container_state.is_some_and(|cs| cs.is_open())
    }
    /// Set location to a `Room` by UUID
    pub fn set_location_room(&mut self, room_id: Uuid) {
        self.location = Location::Room(room_id);
    }
    /// Set location to another `Item` by UUID
    pub fn set_location_item(&mut self, container_id: Uuid) {
        self.location = Location::Item(container_id);
    }
    /// Set location to player inventory by UUID
    pub fn set_location_inventory(&mut self) {
        // once a restricted item has been obtained, must no longer be so
        // if given back to an NPC it can be optionally re-restricted using a trigger action
        self.restricted = false;
        self.location = Location::Inventory;
    }
    /// Set location to NPC inventory by UUID
    pub fn set_location_npc(&mut self, npc_id: Uuid) {
        self.location = Location::Npc(npc_id);
    }
    /// Show item description (and any contents if a container and open).
    pub fn show(&self, world: &AmbleWorld) {
        println!("{}", self.name().item_style().underline());
        println!("{}", self.description().description_style());
        if self.container_state.is_some() {
            println!("{}", "Contents:".bold());
            if self.is_accessible() {
                if self.contents.is_empty() {
                    println!("{}", "\tEmpty".italic().dimmed());
                } else {
                    self.contents
                        .iter()
                        .filter_map(|item_id| world.items.get(item_id))
                        .for_each(|i| println!("\t{}", i.name().item_style()));
                }
            } else {
                let action = if self.container_state.is_some_and(|cs| cs.is_locked()) {
                    "unlock".bold().red()
                } else {
                    "open".bold().green()
                };
                println!("You'll have to {action} it first.");
            }
        }
    }

    /// Checks if an item requires a something special for a particular interaction
    pub fn requires_capability_for(&self, inter: ItemInteractionType) -> Option<ItemAbility> {
        self.interaction_requires.get(&inter).copied()
    }

    /// Returns the reason the item can't be accessed (as a container), if any
    pub fn access_denied_reason(&self) -> Option<String> {
        match self.container_state {
            Some(ContainerState::Open) => None,
            Some(ContainerState::Closed) => {
                let reason = format!("The {} is {}.", self.name().item_style(), "closed".bold());
                Some(reason)
            }
            Some(ContainerState::Locked) => {
                let reason = format!("The {} is {}.", self.name().item_style(), "locked".bold());
                Some(reason)
            }
            None => {
                let reason = format!("The {} isn't a container.", self.name().item_style());
                Some(reason)
            }
        }
    }
}
