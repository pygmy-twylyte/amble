use crate::{
    ItemHolder, Location, View, ViewItem, WorldObject,
    npc::NpcState,
    player::Flag,
    view::{ExitLine, NpcLine, ViewMode},
    world::AmbleWorld,
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
/// An exit from one room to another.
///
/// Additional flags and items may be required to traverse it.
pub struct Exit {
    pub to: Uuid,
    pub hidden: bool,
    pub locked: bool,
    pub required_flags: HashSet<Flag>,
    pub required_items: HashSet<Uuid>,
    pub barred_message: Option<String>,
}
impl Exit {
    /// Create a basic exit leading to the room with the given UUID.
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
        let flag_is_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match &self.condition {
            OverlayCondition::FlagComplete { flag } => world
                .player
                .flags
                .get(&Flag::Simple { name: flag.into() })
                .is_some_and(Flag::is_complete),
            OverlayCondition::FlagSet { flag } => flag_is_set(flag),
            OverlayCondition::FlagUnset { flag } => !flag_is_set(flag),
            OverlayCondition::ItemPresent { item_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| matches!(item.location, Location::Room(id) if id == room_id)),
            OverlayCondition::ItemAbsent { item_id } => world
                .items
                .get(item_id)
                .is_none_or(|item| !matches!(item.location, Location::Room(id) if id == room_id)),
            OverlayCondition::PlayerHasItem { item_id } => world.player.contains_item(*item_id),
            OverlayCondition::PlayerMissingItem { item_id } => !world.player.contains_item(*item_id),
            OverlayCondition::NpcInMood { npc_id, mood } => {
                world.npcs.get(npc_id).is_some_and(|npc| npc.state == *mood)
            },
            OverlayCondition::ItemInRoom { item_id, room_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| matches!(item.location, Location::Room(id) if id == *room_id)),
        }
    }
}

/// Types of conditions that may enable room overlays
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayCondition {
    FlagComplete { flag: String },
    FlagSet { flag: String },
    FlagUnset { flag: String },
    ItemPresent { item_id: Uuid },
    ItemAbsent { item_id: Uuid },
    PlayerHasItem { item_id: Uuid },
    PlayerMissingItem { item_id: Uuid },
    NpcInMood { npc_id: Uuid, mood: NpcState },
    ItemInRoom { item_id: Uuid, room_id: Uuid },
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
    pub fn show(&self, world: &AmbleWorld, view: &mut View, force_mode: Option<ViewMode>) -> Result<()> {
        view.push(ViewItem::RoomDescription {
            name: self.name.to_string(),
            description: self.description().to_string(),
            visited: self.visited,
            force_mode,
        });
        self.show_overlays(world, view, force_mode);

        if !self.contents.is_empty() {
            let item_names: Vec<_> = self
                .contents
                .iter()
                .filter_map(|id| world.items.get(id).map(|item| item.name().to_string()))
                .collect();
            view.push(ViewItem::RoomItems(item_names));
        }
        self.show_exits(world, view)?;
        self.show_npcs(world, view);
        println!();
        Ok(())
    }

    /// Displays any applicable description overlays.
    pub fn show_overlays(&self, world: &AmbleWorld, view: &mut View, force_mode: Option<ViewMode>) {
        let overlay_text: Vec<String> = self
            .overlays
            .iter()
            .filter(|o| o.applies(self.id(), world))
            .map(|o| o.text.clone())
            .collect();
        view.push(ViewItem::RoomOverlays {
            text: overlay_text,
            force_mode,
        });
    }

    /// Displays list of NPCs present in the `Room`
    pub fn show_npcs(&self, world: &AmbleWorld, view: &mut View) {
        if !self.npcs.is_empty() {
            let npc_lines: Vec<NpcLine> = self
                .npcs
                .iter()
                .filter_map(|npc_id| world.npcs.get(npc_id))
                .map(|npc| NpcLine {
                    name: npc.name.clone(),
                    description: npc.description.clone(),
                })
                .collect();
            view.push(ViewItem::RoomNpcs(npc_lines));
        }
    }

    /// Displays list of available exits from the Room.
    pub fn show_exits(&self, world: &AmbleWorld, view: &mut View) -> Result<()> {
        let mut exit_lines = Vec::new();
        for (direction, exit) in &self.exits {
            let target_room = world.rooms.get(&exit.to).ok_or(anyhow!(
                "Room({}) not found ({} exit from Room({})",
                exit.to,
                direction,
                self.id
            ))?;
            exit_lines.push(ExitLine {
                direction: direction.to_string(),
                destination: target_room.name().to_string(),
                exit_locked: exit.locked,
                dest_visited: target_room.visited,
            });
        }
        view.push(ViewItem::RoomExits(exit_lines));
        Ok(())
    }
}
