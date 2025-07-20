//! Player -- module for a player in Amble
use crate::{ItemHolder, Location, WorldObject};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub inventory: HashSet<Uuid>,
    pub flags: HashSet<String>,
    pub score: usize,
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
            flags: HashSet::<String>::default(),
            score: 0,
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
