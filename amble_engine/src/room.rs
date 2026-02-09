//! Room definitions and spatial utilities.
//!
//! Any location a player or NPC can inhabit is represented as a "Room", whether it's a closet
//! or a canyon.
//!
//! Captures room metadata, exits, and overlays along with helpers used
//! during movement, rendering, and trigger evaluation.

use crate::Id;
use crate::{
    ItemHolder, Location, View, ViewItem, WorldObject,
    health::{LifeState, LivingEntity},
    npc::NpcState,
    player::Flag,
    view::{ExitLine, NpcLine, ViewMode},
    world::{AmbleWorld, item_is_listed, item_is_visible},
};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An exit from one room to another.
///
/// Additional flags and items may be required to traverse it.
pub struct Exit {
    pub to: Id,
    pub hidden: bool,
    pub locked: bool,
    pub required_flags: HashSet<Flag>,
    pub required_items: HashSet<Id>,
    pub barred_message: Option<String>,
}
impl Exit {
    /// Create a basic exit leading to the room with the given id.
    pub fn new(to: Id) -> Self {
        Self {
            to,
            hidden: false,
            locked: false,
            required_flags: HashSet::new(),
            required_items: HashSet::new(),
            barred_message: None,
        }
    }
    /// Change the barred message given to player if they try this exit without meeting requirements.
    pub fn set_barred_msg(&mut self, msg: Option<String>) {
        self.barred_message = msg;
    }
}

/// Conditional text that may be part of a room description, depending on some state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomOverlay {
    pub conditions: Vec<OverlayCondition>,
    pub text: String,
}

/// Room-level scenery entry (look/examine only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomScenery {
    pub name: String,
    pub desc: Option<String>,
}
impl RoomOverlay {
    /// Returns true if an overlay's conditions are all met.
    pub fn applies(&self, room_id: &str, world: &AmbleWorld) -> bool {
        self.conditions.iter().all(|cond| cond.applies(room_id, world))
    }
}

/// Types of conditions that may enable room overlays
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayCondition {
    FlagComplete { flag: String },
    FlagSet { flag: String },
    FlagUnset { flag: String },
    ItemAbsent { item_id: Id },
    ItemInRoom { item_id: Id, room_id: Id },
    ItemPresent { item_id: Id },
    NpcInState { npc_id: Id, mood: NpcState },
    NpcAbsent { npc_id: Id },
    NpcPresent { npc_id: Id },
    PlayerHasItem { item_id: Id },
    PlayerMissingItem { item_id: Id },
}
impl OverlayCondition {
    /// Returns true if this condition currently applies.
    pub fn applies(&self, room_id: &str, world: &AmbleWorld) -> bool {
        let flag_is_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match &self {
            OverlayCondition::FlagComplete { flag } => world
                .player
                .flags
                .get(&Flag::Simple {
                    name: flag.into(),
                    turn_set: 0,
                })
                .is_some_and(Flag::is_complete),
            OverlayCondition::FlagSet { flag } => flag_is_set(flag),
            OverlayCondition::FlagUnset { flag } => !flag_is_set(flag),
            OverlayCondition::ItemPresent { item_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| matches!(&item.location, Location::Room(id) if id.as_str() == room_id)),
            OverlayCondition::ItemAbsent { item_id } => world
                .items
                .get(item_id)
                .is_none_or(|item| !matches!(&item.location, Location::Room(id) if id.as_str() == room_id)),
            OverlayCondition::PlayerHasItem { item_id } => world.player.contains_item(item_id.clone()),
            OverlayCondition::PlayerMissingItem { item_id } => !world.player.contains_item(item_id.clone()),
            OverlayCondition::NpcInState { npc_id, mood } => {
                world.npcs.get(npc_id).is_some_and(|npc| npc.state == *mood)
            },
            OverlayCondition::ItemInRoom { item_id, room_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| matches!(&item.location, Location::Room(id) if id == room_id)),
            OverlayCondition::NpcPresent { npc_id } => {
                world.rooms.get(room_id).is_some_and(|room| room.npcs.contains(npc_id))
            },
            OverlayCondition::NpcAbsent { npc_id } => {
                world.rooms.get(room_id).is_some_and(|room| !room.npcs.contains(npc_id))
            },
        }
    }
}

/// Any visitable location in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Id,
    pub symbol: String,
    pub name: String,
    pub base_description: String,
    pub overlays: Vec<RoomOverlay>,
    #[serde(default)]
    pub scenery: Vec<RoomScenery>,
    #[serde(default)]
    pub scenery_default: Option<String>,
    pub location: Location,
    pub visited: bool,
    pub exits: HashMap<String, Exit>,
    pub contents: HashSet<Id>,
    pub npcs: HashSet<Id>,
}
impl WorldObject for Room {
    fn id(&self) -> Id {
        self.id.clone()
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
    fn add_item(&mut self, item_id: Id) {
        self.contents.insert(item_id);
    }

    fn remove_item(&mut self, item_id: Id) {
        self.contents.remove(&item_id);
    }

    fn contains_item(&self, item_id: Id) -> bool {
        self.contents.contains(&item_id)
    }
}
impl Room {
    /// Displays full description, exit, and NPC information for the `Room`.
    ///
    /// # Errors
    /// Returns an error if any referenced rooms cannot be resolved while showing exits.
    pub fn show(&self, world: &AmbleWorld, view: &mut View, force_mode: Option<ViewMode>) -> Result<()> {
        view.push(ViewItem::RoomDescription {
            name: self.name.clone(),
            description: self.description().to_string(),
            visited: self.visited,
            force_mode,
        });
        self.show_overlays(world, view, force_mode);

        if !self.contents.is_empty() {
            let item_names: Vec<_> = self
                .contents
                .iter()
                .filter(|id| item_is_visible(world, id) && item_is_listed(world, id))
                .filter_map(|id| world.items.get(id).map(|item| item.name().to_string()))
                .collect();
            if !item_names.is_empty() {
                view.push(ViewItem::RoomItems(item_names));
            }
        }
        self.show_exits(world, view)?;
        self.show_npcs(world, view);
        Ok(())
    }

    /// Displays any applicable description overlays.
    pub fn show_overlays(&self, world: &AmbleWorld, view: &mut View, force_mode: Option<ViewMode>) {
        let overlay_text: Vec<String> = self
            .overlays
            .iter()
            .filter(|o| o.applies(self.id.as_str(), world))
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
                    name: match npc.life_state() {
                        // append "(dead)" to NPC's name if they're dead but still present
                        LifeState::Dead => format!("{} (dead)", npc.name()),
                        LifeState::Alive => npc.name.clone(),
                    },
                    description: npc.short_description(),
                })
                .collect();
            view.push(ViewItem::RoomNpcs(npc_lines));
        }
    }

    /// Displays list of available exits from the Room.
    ///
    /// # Errors
    /// Returns an error if a referenced destination room cannot be found.
    pub fn show_exits(&self, world: &AmbleWorld, view: &mut View) -> Result<()> {
        let mut exit_lines = Vec::new();
        for (direction, exit) in &self.exits {
            if exit.hidden {
                continue;
            }
            let target_room = world.rooms.get(&exit.to).ok_or(anyhow!(
                "Room({}) not found ({} exit from Room({})",
                exit.to,
                direction,
                self.id
            ))?;
            exit_lines.push(ExitLine {
                direction: direction.clone(),
                destination: target_room.name().to_string(),
                exit_locked: exit.locked,
                dest_visited: target_room.visited,
            });
        }
        view.push(ViewItem::RoomExits(exit_lines));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Id;
    use crate::{
        health::HealthState,
        item::Item,
        npc::{Npc, NpcState},
        player::Flag,
        view::{View, ViewItem},
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};

    fn create_test_room(id: Id) -> Room {
        Room {
            id,
            symbol: "test_room".into(),
            name: "Test Room".into(),
            base_description: "A test room for testing".into(),
            overlays: vec![],
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        }
    }

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::new_empty();

        let room_id = crate::idgen::new_id();
        let room = create_test_room(room_id.clone());
        world.rooms.insert(room_id.clone(), room);
        world.player.location = Location::Room(room_id.clone());

        let item_id = crate::idgen::new_id();
        let item = Item {
            id: item_id.clone(),
            symbol: "test_item".into(),
            name: "Test Item".into(),
            description: "A test item".into(),
            location: Location::Room(room_id.clone()),
            visibility: crate::item::ItemVisibility::Listed,
            visible_when: None,
            aliases: Vec::new(),
            movability: crate::item::Movability::Free,
            container_state: None,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(item_id.clone(), item);

        let npc_id = crate::idgen::new_id();
        let npc = Npc {
            id: npc_id.clone(),
            symbol: "test_npc".into(),
            name: "Test NPC".into(),
            description: "A test NPC".into(),
            location: Location::Room(room_id.clone()),
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: NpcState::Normal,
            movement: None,
            health: HealthState::new_at_max(10),
        };
        world.npcs.insert(npc_id.clone(), npc);

        world
    }

    #[test]
    fn exit_new_creates_basic_exit() {
        let dest_id = crate::idgen::new_id();
        let exit = Exit::new(dest_id.clone());

        assert_eq!(exit.to, dest_id);
        assert!(!exit.hidden);
        assert!(!exit.locked);
        assert!(exit.required_flags.is_empty());
        assert!(exit.required_items.is_empty());
        assert!(exit.barred_message.is_none());
    }

    #[test]
    fn room_overlay_applies_with_flag_set() {
        let mut world = create_test_world();
        world.player.flags.insert(Flag::simple("test_flag", 0));
        let room_id = world.player.location.room_id().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagSet {
                flag: "test_flag".into(),
            }],
            text: "This overlay should show".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_does_not_apply_without_flag() {
        let world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagSet {
                flag: "nonexistent_flag".into(),
            }],
            text: "This overlay should not show".into(),
        };

        assert!(!overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_flag_unset() {
        let world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagUnset {
                flag: "nonexistent_flag".into(),
            }],
            text: "This overlay should show".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_flag_complete() {
        let mut world = create_test_world();
        let mut seq_flag = Flag::sequence("test_seq", Some(2), 0);
        seq_flag.advance();
        seq_flag.advance(); // Complete
        world.player.flags.insert(seq_flag);
        let room_id = world.player.location.room_id().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagComplete {
                flag: "test_seq".into(),
            }],
            text: "Sequence is complete".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_item_present() {
        let world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();
        let item_id = world.items.keys().next().unwrap().clone();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::ItemPresent { item_id }],
            text: "Item is here".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_item_absent() {
        let world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();
        let nonexistent_item = crate::idgen::new_id();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::ItemAbsent {
                item_id: nonexistent_item,
            }],
            text: "Item is not here".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_player_has_item() {
        let mut world = create_test_world();
        let item_id = world.items.keys().next().unwrap().clone();
        world.items.get_mut(&item_id).unwrap().location = Location::Inventory;
        world.player.inventory.insert(item_id.clone());

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::PlayerHasItem { item_id }],
            text: "You have the item".into(),
        };

        let room_id = world.player.location.room_id().unwrap();
        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_npc_in_mood() {
        let world = create_test_world();
        let npc_id = world.npcs.keys().next().unwrap().clone();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::NpcInState {
                npc_id,
                mood: NpcState::Normal,
            }],
            text: "NPC is in normal mood".into(),
        };

        let room_id = world.player.location.room_id().unwrap();
        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_item_in_room() {
        let world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();
        let item_id = world.items.keys().next().unwrap().clone();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::ItemInRoom {
                item_id,
                room_id: room_id.clone(),
            }],
            text: "Item is in this specific room".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_show_displays_all_sections() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();

        // Add items and NPCs to the room
        let item_id = world.items.keys().next().unwrap().clone();
        let npc_id = world.npcs.keys().next().unwrap().clone();

        world.rooms.get_mut(&room_id).unwrap().contents.insert(item_id);
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        // Add another room for exits
        let other_room_id = crate::idgen::new_id();
        let other_room = create_test_room(other_room_id.clone());
        world.rooms.insert(other_room_id.clone(), other_room);

        // Add exit
        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .exits
            .insert("north".into(), Exit::new(other_room_id));

        let room = world.rooms.get(&room_id).unwrap();
        room.show(&world, &mut view, None).unwrap();

        let items = &view.items;
        assert!(
            items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomDescription { .. }))
        );
        assert!(
            items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomItems(_)))
        );
        assert!(
            items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomExits(_)))
        );
        assert!(
            items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomNpcs(_)))
        );
    }

    #[test]
    fn room_show_overlays_displays_applicable_overlays() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();

        world
            .player
            .flags
            .insert(Flag::simple("show_overlay", world.turn_count));

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagSet {
                flag: "show_overlay".into(),
            }],
            text: "This is an overlay text".into(),
        };

        world.rooms.get_mut(&room_id).unwrap().overlays.push(overlay);

        let room = world.rooms.get(&room_id).unwrap();
        room.show_overlays(&world, &mut view, None);

        if let Some(text) = view.items.iter().find_map(|entry| {
            if let ViewItem::RoomOverlays { text, .. } = &entry.view_item {
                Some(text)
            } else {
                None
            }
        }) {
            assert_eq!(text.len(), 1);
            assert_eq!(text[0], "This is an overlay text");
        } else {
            panic!("Expected RoomOverlays view item");
        }
    }

    #[test]
    fn room_show_npcs_displays_npc_list() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();
        let npc_id = world.npcs.keys().next().unwrap().clone();

        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let room = world.rooms.get(&room_id).unwrap();
        room.show_npcs(&world, &mut view);

        if let Some(npcs) = view.items.iter().find_map(|entry| {
            if let ViewItem::RoomNpcs(npcs) = &entry.view_item {
                Some(npcs)
            } else {
                None
            }
        }) {
            assert_eq!(npcs.len(), 1);
            assert_eq!(npcs[0].name, "Test NPC");
        } else {
            panic!("Expected RoomNpcs view item");
        }
    }

    #[test]
    fn room_show_exits_displays_exit_list() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();

        let dest_room_id = crate::idgen::new_id();
        let dest_room = create_test_room(dest_room_id.clone());
        world.rooms.insert(dest_room_id.clone(), dest_room);

        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .exits
            .insert("north".into(), Exit::new(dest_room_id));

        let room = world.rooms.get(&room_id).unwrap();
        room.show_exits(&world, &mut view).unwrap();

        if let Some(exits) = view.items.iter().find_map(|entry| {
            if let ViewItem::RoomExits(exits) = &entry.view_item {
                Some(exits)
            } else {
                None
            }
        }) {
            assert_eq!(exits.len(), 1);
            assert_eq!(exits[0].direction, "north");
            assert_eq!(exits[0].destination, "Test Room");
            assert!(!exits[0].exit_locked);
            assert!(!exits[0].dest_visited);
        } else {
            panic!("Expected RoomExits view item");
        }
    }

    #[test]
    fn world_object_trait_works() {
        let room = create_test_room(crate::idgen::new_id());
        assert_eq!(room.symbol(), "test_room");
        assert_eq!(room.name(), "Test Room");
        assert_eq!(room.description(), "A test room for testing");
        assert_eq!(room.location(), &Location::Nowhere);
    }

    #[test]
    fn item_holder_add_item_works() {
        let mut room = create_test_room(crate::idgen::new_id());
        let item_id = crate::idgen::new_id();

        room.add_item(item_id.clone());
        assert!(room.contents.contains(&item_id));
    }

    #[test]
    fn item_holder_remove_item_works() {
        let mut room = create_test_room(crate::idgen::new_id());
        let item_id = crate::idgen::new_id();
        room.contents.insert(item_id.clone());

        room.remove_item(item_id.clone());
        assert!(!room.contents.contains(&item_id));
    }

    #[test]
    fn item_holder_contains_item_works() {
        let mut room = create_test_room(crate::idgen::new_id());
        let item_id = crate::idgen::new_id();
        room.contents.insert(item_id.clone());

        assert!(room.contains_item(item_id));
        assert!(!room.contains_item(crate::idgen::new_id()));
    }

    #[test]
    fn room_show_handles_empty_sections() {
        let world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();

        let room = world.rooms.get(&room_id).unwrap();
        room.show(&world, &mut view, None).unwrap();

        // Should handle empty items/npcs/exits gracefully
        let items = &view.items;
        assert!(
            items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomDescription { .. }))
        );
        // Items, NPCs, and exits should not be shown if empty
        assert!(
            !items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomItems(_)))
        );
        assert!(
            !items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomNpcs(_)))
        );
        assert!(
            items
                .iter()
                .any(|entry| matches!(&entry.view_item, ViewItem::RoomExits(_)))
        ); // Empty exits still shown
    }

    #[test]
    fn exit_with_requirements_shows_correct_state() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();

        let dest_room_id = crate::idgen::new_id();
        let dest_room = create_test_room(dest_room_id.clone());
        world.rooms.insert(dest_room_id.clone(), dest_room);

        // Create exit with requirements
        let mut exit = Exit::new(dest_room_id);
        exit.locked = true;
        exit.required_flags.insert(Flag::simple("key_flag", world.turn_count));

        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .exits
            .insert("north".into(), exit);

        let room = world.rooms.get(&room_id).unwrap();
        room.show_exits(&world, &mut view).unwrap();

        if let Some(exits) = view.items.iter().find_map(|entry| {
            if let ViewItem::RoomExits(exits) = &entry.view_item {
                Some(exits)
            } else {
                None
            }
        }) {
            assert_eq!(exits.len(), 1);
            assert!(exits[0].exit_locked);
        }
    }

    #[test]
    fn room_overlay_applies_with_player_missing_item() {
        let world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();
        let missing_item = crate::idgen::new_id();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::PlayerMissingItem { item_id: missing_item }],
            text: "You don't have this item".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_npc_present() {
        let mut world = create_test_world();
        let room_id = world.player.location.room_id().unwrap();
        let npc_id = world.npcs.keys().next().unwrap().clone();
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id.clone());

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::NpcPresent { npc_id }],
            text: "NPC is here".into(),
        };

        assert!(overlay.applies(&room_id, &world));
    }

    #[test]
    fn room_show_exits_errors_if_destination_missing() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.room_id().unwrap();
        let missing_room = crate::idgen::new_id();
        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .exits
            .insert("east".into(), Exit::new(missing_room));

        let room = world.rooms.get(&room_id).unwrap();
        assert!(room.show_exits(&world, &mut view).is_err());
    }
}
