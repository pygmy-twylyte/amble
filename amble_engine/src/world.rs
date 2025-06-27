use crate::npc::Npc;
use crate::spinners::{SpinnerType, default_spinners};
use crate::trigger::Trigger;
use crate::{Item, Player, Room};

use anyhow::{Result, anyhow};
use gametools::Spinner;
use log::info;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use uuid::Uuid;
use variantly::Variantly;

/// Kinds of places where a WorldObject may be located.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Variantly, PartialEq, Eq)]
pub enum Location {
    Item(Uuid),
    Inventory,
    #[default]
    Nowhere,
    Npc(Uuid),
    Room(Uuid),
}

/// Methods common to any object in the world.
pub trait WorldObject {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn location(&self) -> &Location;
}

/// The Amble world.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AmbleWorld {
    pub rooms: HashMap<Uuid, Room>,
    pub items: HashMap<Uuid, Item>,
    pub triggers: Vec<Trigger>,
    pub player: Player,
    #[serde(skip)] // these are hard-coded into source
    pub spinners: HashMap<SpinnerType, Spinner<&'static str>>,
    pub npcs: HashMap<Uuid, Npc>,
}
impl AmbleWorld {
    /// Create a new empty world with a default player.
    pub fn new_empty() -> AmbleWorld {
        let world = Self {
            rooms: HashMap::new(),
            npcs: HashMap::new(),
            items: HashMap::new(),
            triggers: Vec::new(),
            player: Player::default(),
            spinners: default_spinners(),
        };
        info!("new, empty 'AmbleWorld' created");
        info!("{} spinners added to 'AmbleWorld'", world.spinners.len());
        world
    }

    /// Returns a random string (&'static str) from the selected spinner type, or a supplied default.
    pub fn spin_spinner(&self, spin_type: SpinnerType, default: &'static str) -> &'static str {
        self.spinners
            .get(&spin_type)
            .and_then(gametools::Spinner::spin)
            .unwrap_or(default)
    }

    /// Obtain a reference to the room the player occupies.
    pub fn player_room_ref(&self) -> Result<&Room> {
        match self.player.location {
            Location::Room(uuid) => self
                .rooms
                .get(&uuid)
                .ok_or_else(|| anyhow!("player's room UUID ({}) not found in world", uuid)),
            _ => Err(anyhow!(
                "player not in a room - located at {:?}",
                self.player.location
            )),
        }
    }

    /// Obtain a mutable reference to the room the player occupies.
    pub fn player_room_mut(&mut self) -> Result<&mut Room> {
        match self.player.location {
            Location::Room(uuid) => self
                .rooms
                .get_mut(&uuid)
                .ok_or_else(|| anyhow!("player's room UUID ({}) not found in world", uuid)),
            _ => Err(anyhow!(
                "player not in a room - located at {:?}",
                self.player.location
            )),
        }
    }

    /// Get mutable reference to a world item.
    pub fn get_item_mut(&mut self, item_id: Uuid) -> Option<&mut Item> {
        self.items.get_mut(&item_id)
    }
}
