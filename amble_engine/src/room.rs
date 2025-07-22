use crate::{ItemHolder, Location, WorldObject, style::GameStyle, world::AmbleWorld};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Exit {
    pub to: Uuid,
    pub hidden: bool,
    pub locked: bool,
    pub required_flags: HashSet<String>,
    pub required_items: HashSet<Uuid>,
    pub barred_message: Option<String>,
}
impl Exit {
    pub fn new(to: Uuid) -> Self {
        Self {
            to,
            hidden: false,
            locked: false,
            required_flags: HashSet::new(),
            required_items: HashSet::new(),
            barred_message: None,
        }
    }
}

/// Conditional text that may be part of a room description, depending on some state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomOverlay {
    pub condition: OverlayCondition,
    pub text: String,
}
impl RoomOverlay {
    /// Returns true if an overlay's condition is met.
    pub fn applies(&self, room_id: Uuid, world: &AmbleWorld) -> bool {
        match &self.condition {
            OverlayCondition::FlagSet { flag } => world.player.flags.contains(flag),
            OverlayCondition::FlagUnset { flag } => !world.player.flags.contains(flag),
            OverlayCondition::ItemPresent { item_id } => world.items.get(item_id).map_or(
                false,
                |item| matches!(item.location, Location::Room(id) if id == room_id),
            ),
            OverlayCondition::ItemAbsent { item_id } => world.items.get(item_id).map_or(
                true,
                |item| !matches!(item.location, Location::Room(id) if id == room_id),
            ),
        }
    }
}

/// Types of conditions that may enable room overlays
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayCondition {
    FlagSet { flag: String },
    FlagUnset { flag: String },
    ItemPresent { item_id: Uuid },
    ItemAbsent { item_id: Uuid },
}

/// Any visitable location in the game world.
#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub base_description: String,
    pub overlays: Vec<RoomOverlay>,
    pub location: Location,
    pub visited: bool,
    pub exits: HashMap<String, Exit>,
    pub contents: HashSet<Uuid>,
    pub npcs: HashSet<Uuid>,
}
impl WorldObject for Room {
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
        &self.base_description
    }

    fn location(&self) -> &Location {
        &self.location
    }
}
impl ItemHolder for Room {
    fn add_item(&mut self, item_id: Uuid) {
        self.contents.insert(item_id);
    }

    fn remove_item(&mut self, item_id: Uuid) {
        self.contents.remove(&item_id);
    }

    fn contains_item(&self, item_id: Uuid) -> bool {
        self.contents.contains(&item_id)
    }
}
impl Room {
    /// Displays full description, exit, and NPC information for the `Room`.
    pub fn show(&self, world: &AmbleWorld) -> Result<()> {
        let banner = self.name.room_titlebar_style();
        println!("{banner:^80}");
        println!("{}", self.base_description.description_style());
        self.show_overlays(world);

        if !self.contents.is_empty() {
            println!("{}", "You see:".subheading_style());
            self.contents
                .iter()
                .filter_map(|item_id| world.items.get(item_id))
                .enumerate()
                .for_each(|(n, item)| println!("    {}) {}", n + 1, item.name.item_style()));
        }
        self.show_exits(world)?;
        self.show_npcs(world);
        println!();
        Ok(())
    }

    /// Displays any applicable description overlays.
    pub fn show_overlays(&self, world: &AmbleWorld) {
        for overlay in &self.overlays {
            if overlay.applies(self.id(), world) {
                println!("{}", overlay.text.description_style());
                println!();
            }
        }
    }

    /// Displays list of NPCs present in the `Room`
    pub fn show_npcs(&self, world: &AmbleWorld) {
        if !self.npcs.is_empty() {
            println!("{}", "Others here:".subheading_style());
            self.npcs
                .iter()
                .filter_map(|npc_id| world.npcs.get(npc_id))
                .for_each(|npc| {
                    println!("\t{} - {}", npc.name.npc_style(), npc.description.description_style());
                });
            println!();
        }
    }

    /// Displays list of available exits from the Room.
    pub fn show_exits(&self, world: &AmbleWorld) -> Result<()> {
        println!("\n{}", "Exits:".subheading_style());
        for (direction, exit) in &self.exits {
            let target_room = world.rooms.get(&exit.to).ok_or(anyhow!(
                "Room({}) not found ({} exit from Room({})",
                exit.to,
                direction,
                self.id
            ))?;
            match target_room {
                room if room.visited && exit.locked => {
                    println!(
                        "\t🡺 {} ({})",
                        direction.exit_locked_style(),
                        target_room.name().room_style()
                    );
                },
                room if room.visited => {
                    println!(
                        "\t🡺 {} ({})",
                        direction.exit_visited_style(),
                        target_room.name().room_style()
                    );
                },
                _ if exit.locked => println!("\t🡺 {}", direction.exit_locked_style()),
                _ => println!("\t🡺 {}", direction.exit_unvisited_style()),
            }
        }
        Ok(())
    }
}
