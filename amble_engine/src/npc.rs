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

use crate::{ItemHolder, Location, WorldObject, style::GameStyle, world::AmbleWorld};

/// Represents the demeanor of an 'Npc', which may affect default dialogue and behavior
#[derive(Copy, Clone, Debug, variantly::Variantly, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum NpcMood {
    Bored,
    Happy,
    Mad,
    Normal,
    Sad,
    Tired,
}
impl Display for NpcMood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NpcMood::Bored => write!(f, "Bored"),
            NpcMood::Happy => write!(f, "Happy"),
            NpcMood::Mad => write!(f, "Mad"),
            NpcMood::Normal => write!(f, "Normal"),
            NpcMood::Sad => write!(f, "Sad"),
            NpcMood::Tired => write!(f, "Tired"),
        }
    }
}

/// A non-playable character.
#[derive(Debug, Serialize, Deserialize)]
pub struct Npc {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub inventory: HashSet<Uuid>,
    pub dialogue: HashMap<NpcMood, Vec<String>>,
    pub mood: NpcMood,
}
impl Npc {
    /// Returns a random line of dialogue from within the NPCs current Mood.
    pub fn random_dialogue(&self, ignore_spinner: &Spinner<&'static str>) -> String {
        if let Some(lines) = self.dialogue.get(&self.mood) {
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
                self.mood
            );
            ignore_spinner.spin().unwrap_or("Ignores you.").to_string()
        }
    }
    /// Display NPC info to the player
    pub fn show(&self, world: &AmbleWorld) {
        println!(
            "{} {}",
            self.name().npc_style().bold(),
            format!("({})", self.mood).italic().dimmed()
        );
        println!("{}\n", self.description().description_style());
        println!("{}", "Inventory".item_style().underline().bold());
        if self.inventory.is_empty() {
            println!("{}", "No items available.".italic().dimmed());
        } else {
            self.inventory
                .iter()
                .filter_map(|id| world.items.get(id))
                .for_each(|item| {
                    if item.restricted {
                        println!("\t{}🔒", item.name().item_style());
                    } else {
                        println!("\t{}", item.name().item_style());
                    }
                });
        }
    }
}
impl WorldObject for Npc {
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
