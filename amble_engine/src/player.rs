//! Player -- module for a player in Amble
use crate::{ItemHolder, Location, WorldObject, world::AmbleWorld};

use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub inventory: HashSet<Uuid>,
    pub achievements: HashSet<String>,
    pub score: usize,
}
impl Default for Player {
    fn default() -> Player {
        Self {
            id: Uuid::new_v4(),
            name: "default".into(),
            description: "default".into(),
            location: Location::default(),
            inventory: HashSet::<Uuid>::default(),
            achievements: HashSet::<String>::default(),
            score: 0,
        }
    }
}
impl WorldObject for Player {
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
impl Player {
    /// Return a string containing the name of the player's current location
    pub fn location_name(&self, world: &AmbleWorld) -> String {
        match world.player_room_ref() {
            Ok(room) => room.name.to_string(),
            Err(e) => {
                error!("when looking up player location: {e}");
                "<unknown>".to_string()
            }
        }
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
