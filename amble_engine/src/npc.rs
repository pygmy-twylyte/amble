//! NPC Module

use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use colored::Colorize;
use gametools::Spinner;
use rand::prelude::IndexedRandom;

use uuid::Uuid;

use crate::{ItemHolder, Location, View, ViewItem, WorldObject, view::ContentLine, world::AmbleWorld};

/// Represents the demeanor of an 'Npc', which may affect default dialogue and behavior
#[derive(Clone, Debug, variantly::Variantly, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NpcState {
    Bored,
    Happy,
    Mad,
    Normal,
    Sad,
    Tired,
    Custom(String),
}
impl Display for NpcState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Happy => write!(f, "Happy"),
            Self::Bored => write!(f, "Bored"),
            Self::Mad => write!(f, "Mad"),
            Self::Normal => write!(f, "Normal"),
            Self::Sad => write!(f, "Sad"),
            Self::Tired => write!(f, "Tired"),
            Self::Custom(_) => write!(f, "Custom"),
        }
    }
}
impl NpcState {
    pub fn from_key(key: &str) -> Self {
        match key {
            "sad" => NpcState::Sad,
            "bored" => NpcState::Bored,
            "normal" => NpcState::Normal,
            "happy" => NpcState::Happy,
            "mad" => NpcState::Mad,
            "tired" => NpcState::Tired,
            other if other.starts_with("custom:") => NpcState::Custom(other.trim_start_matches("custom:").to_string()),
            _ => {
                warn!("Unknown NpcState key in dialogue map: {key}");
                NpcState::Normal
            },
        }
    }

    pub fn as_key(&self) -> String {
        match self {
            NpcState::Sad => "sad".into(),
            NpcState::Bored => "bored".into(),
            NpcState::Normal => "normal".into(),
            NpcState::Happy => "happy".into(),
            NpcState::Mad => "mad".into(),
            NpcState::Tired => "tired".into(),
            NpcState::Custom(s) => format!("custom:{s}"),
        }
    }
}

/// A non-playable character.
#[derive(Debug, Serialize, Deserialize)]
pub struct Npc {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub inventory: HashSet<Uuid>,
    pub dialogue: HashMap<NpcState, Vec<String>>,
    pub state: NpcState,
}
impl Npc {
    /// Returns a random line of dialogue from within the NPCs current Mood.
    pub fn random_dialogue(&self, ignore_spinner: &Spinner<String>) -> String {
        if let Some(lines) = self.dialogue.get(&self.state) {
            let mut rng = rand::rng();
            lines
                .choose(&mut rng)
                .unwrap_or(&"Stands mute.".italic().dimmed().to_string())
                .to_string()
        } else {
            warn!(
                "Npc {}({}): failed dialogue lookup for mood: {:?}",
                self.name(),
                self.id(),
                self.state
            );
            ignore_spinner.spin().unwrap_or("Ignores you.".to_string())
        }
    }
    /// Display NPC info to the player
    pub fn show(&self, world: &AmbleWorld, view: &mut View) {
        view.push(ViewItem::NpcDescription {
            name: self.name.clone(),
            description: self.description.clone(),
        });
        view.push(ViewItem::NpcInventory(
            self.inventory
                .iter()
                .filter_map(|id| world.items.get(id))
                .map(|item| ContentLine {
                    item_name: item.name.clone(),
                    restricted: item.restricted,
                })
                .collect(),
        ));
    }
}
impl WorldObject for Npc {
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
impl ItemHolder for Npc {
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
