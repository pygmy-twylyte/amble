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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
impl RoomOverlay {
    /// Returns true if an overlay's conditions are all met.
    pub fn applies(&self, room_id: Uuid, world: &AmbleWorld) -> bool {
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
    ItemAbsent { item_id: Uuid },
    ItemInRoom { item_id: Uuid, room_id: Uuid },
    ItemPresent { item_id: Uuid },
    NpcInState { npc_id: Uuid, mood: NpcState },
    NpcPresent { npc_id: Uuid },
    PlayerHasItem { item_id: Uuid },
    PlayerMissingItem { item_id: Uuid },
}
impl OverlayCondition {
    /// Returns true if this condition currently applies.
    pub fn applies(&self, room_id: Uuid, world: &AmbleWorld) -> bool {
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
                .is_some_and(|item| matches!(item.location, Location::Room(id) if id == room_id)),
            OverlayCondition::ItemAbsent { item_id } => world
                .items
                .get(item_id)
                .is_none_or(|item| !matches!(item.location, Location::Room(id) if id == room_id)),
            OverlayCondition::PlayerHasItem { item_id } => world.player.contains_item(*item_id),
            OverlayCondition::PlayerMissingItem { item_id } => !world.player.contains_item(*item_id),
            OverlayCondition::NpcInState { npc_id, mood } => {
                world.npcs.get(npc_id).is_some_and(|npc| npc.state == *mood)
            },
            OverlayCondition::ItemInRoom { item_id, room_id } => world
                .items
                .get(item_id)
                .is_some_and(|item| matches!(item.location, Location::Room(id) if id == *room_id)),
            OverlayCondition::NpcPresent { npc_id } => {
                world.rooms.get(&room_id).is_some_and(|room| room.npcs.contains(npc_id))
            },
        }
    }
}

/// Any visitable location in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::Item,
        npc::{Npc, NpcState},
        player::Flag,
        view::{View, ViewItem},
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn create_test_room(id: Uuid) -> Room {
        Room {
            id,
            symbol: "test_room".into(),
            name: "Test Room".into(),
            base_description: "A test room for testing".into(),
            overlays: vec![],
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        }
    }

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::new_empty();

        let room_id = Uuid::new_v4();
        let room = create_test_room(room_id);
        world.rooms.insert(room_id, room);
        world.player.location = Location::Room(room_id);

        let item_id = Uuid::new_v4();
        let item = Item {
            id: item_id,
            symbol: "test_item".into(),
            name: "Test Item".into(),
            description: "A test item".into(),
            location: Location::Room(room_id),
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.items.insert(item_id, item);

        let npc_id = Uuid::new_v4();
        let npc = Npc {
            id: npc_id,
            symbol: "test_npc".into(),
            name: "Test NPC".into(),
            description: "A test NPC".into(),
            location: Location::Room(room_id),
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: NpcState::Normal,
            movement: None,
        };
        world.npcs.insert(npc_id, npc);

        world
    }

    #[test]
    fn exit_new_creates_basic_exit() {
        let dest_id = Uuid::new_v4();
        let exit = Exit::new(dest_id);

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

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagSet {
                flag: "test_flag".into(),
            }],
            text: "This overlay should show".into(),
        };

        assert!(overlay.applies(world.player.location.unwrap_room(), &world));
    }

    #[test]
    fn room_overlay_does_not_apply_without_flag() {
        let world = create_test_world();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagSet {
                flag: "nonexistent_flag".into(),
            }],
            text: "This overlay should not show".into(),
        };

        assert!(!overlay.applies(world.player.location.unwrap_room(), &world));
    }

    #[test]
    fn room_overlay_applies_with_flag_unset() {
        let world = create_test_world();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagUnset {
                flag: "nonexistent_flag".into(),
            }],
            text: "This overlay should show".into(),
        };

        assert!(overlay.applies(world.player.location.unwrap_room(), &world));
    }

    #[test]
    fn room_overlay_applies_with_flag_complete() {
        let mut world = create_test_world();
        let mut seq_flag = Flag::sequence("test_seq", Some(2), 0);
        seq_flag.advance();
        seq_flag.advance(); // Complete
        world.player.flags.insert(seq_flag);

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::FlagComplete {
                flag: "test_seq".into(),
            }],
            text: "Sequence is complete".into(),
        };

        assert!(overlay.applies(world.player.location.unwrap_room(), &world));
    }

    #[test]
    fn room_overlay_applies_with_item_present() {
        let world = create_test_world();
        let room_id = world.player.location.unwrap_room();
        let item_id = *world.items.keys().next().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::ItemPresent { item_id }],
            text: "Item is here".into(),
        };

        assert!(overlay.applies(room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_item_absent() {
        let world = create_test_world();
        let room_id = world.player.location.unwrap_room();
        let nonexistent_item = Uuid::new_v4();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::ItemAbsent {
                item_id: nonexistent_item,
            }],
            text: "Item is not here".into(),
        };

        assert!(overlay.applies(room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_player_has_item() {
        let mut world = create_test_world();
        let item_id = *world.items.keys().next().unwrap();
        world.items.get_mut(&item_id).unwrap().location = Location::Inventory;
        world.player.inventory.insert(item_id);

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::PlayerHasItem { item_id }],
            text: "You have the item".into(),
        };

        assert!(overlay.applies(world.player.location.unwrap_room(), &world));
    }

    #[test]
    fn room_overlay_applies_with_npc_in_mood() {
        let world = create_test_world();
        let npc_id = *world.npcs.keys().next().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::NpcInState {
                npc_id,
                mood: NpcState::Normal,
            }],
            text: "NPC is in normal mood".into(),
        };

        assert!(overlay.applies(world.player.location.unwrap_room(), &world));
    }

    #[test]
    fn room_overlay_applies_with_item_in_room() {
        let world = create_test_world();
        let room_id = world.player.location.unwrap_room();
        let item_id = *world.items.keys().next().unwrap();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::ItemInRoom { item_id, room_id }],
            text: "Item is in this specific room".into(),
        };

        assert!(overlay.applies(room_id, &world));
    }

    #[test]
    fn room_show_displays_all_sections() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.unwrap_room();

        // Add items and NPCs to the room
        let item_id = *world.items.keys().next().unwrap();
        let npc_id = *world.npcs.keys().next().unwrap();

        world.rooms.get_mut(&room_id).unwrap().contents.insert(item_id);
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        // Add another room for exits
        let other_room_id = Uuid::new_v4();
        let other_room = create_test_room(other_room_id);
        world.rooms.insert(other_room_id, other_room);

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
                .any(|item| matches!(item, ViewItem::RoomDescription { .. }))
        );
        assert!(items.iter().any(|item| matches!(item, ViewItem::RoomItems(_))));
        assert!(items.iter().any(|item| matches!(item, ViewItem::RoomExits(_))));
        assert!(items.iter().any(|item| matches!(item, ViewItem::RoomNpcs(_))));
    }

    #[test]
    fn room_show_overlays_displays_applicable_overlays() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.unwrap_room();

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

        if let Some(ViewItem::RoomOverlays { text, .. }) = view
            .items
            .iter()
            .find(|item| matches!(item, ViewItem::RoomOverlays { .. }))
        {
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
        let room_id = world.player.location.unwrap_room();
        let npc_id = *world.npcs.keys().next().unwrap();

        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let room = world.rooms.get(&room_id).unwrap();
        room.show_npcs(&world, &mut view);

        if let Some(ViewItem::RoomNpcs(npcs)) = view.items.iter().find(|item| matches!(item, ViewItem::RoomNpcs(_))) {
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
        let room_id = world.player.location.unwrap_room();

        let dest_room_id = Uuid::new_v4();
        let dest_room = create_test_room(dest_room_id);
        world.rooms.insert(dest_room_id, dest_room);

        world
            .rooms
            .get_mut(&room_id)
            .unwrap()
            .exits
            .insert("north".into(), Exit::new(dest_room_id));

        let room = world.rooms.get(&room_id).unwrap();
        room.show_exits(&world, &mut view).unwrap();

        if let Some(ViewItem::RoomExits(exits)) = view.items.iter().find(|item| matches!(item, ViewItem::RoomExits(_)))
        {
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
        let room = create_test_room(Uuid::new_v4());
        assert_eq!(room.symbol(), "test_room");
        assert_eq!(room.name(), "Test Room");
        assert_eq!(room.description(), "A test room for testing");
        assert_eq!(room.location(), &Location::Nowhere);
    }

    #[test]
    fn item_holder_add_item_works() {
        let mut room = create_test_room(Uuid::new_v4());
        let item_id = Uuid::new_v4();

        room.add_item(item_id);
        assert!(room.contents.contains(&item_id));
    }

    #[test]
    fn item_holder_remove_item_works() {
        let mut room = create_test_room(Uuid::new_v4());
        let item_id = Uuid::new_v4();
        room.contents.insert(item_id);

        room.remove_item(item_id);
        assert!(!room.contents.contains(&item_id));
    }

    #[test]
    fn item_holder_contains_item_works() {
        let mut room = create_test_room(Uuid::new_v4());
        let item_id = Uuid::new_v4();
        room.contents.insert(item_id);

        assert!(room.contains_item(item_id));
        assert!(!room.contains_item(Uuid::new_v4()));
    }

    #[test]
    fn room_show_handles_empty_sections() {
        let world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.unwrap_room();

        let room = world.rooms.get(&room_id).unwrap();
        room.show(&world, &mut view, None).unwrap();

        // Should handle empty items/npcs/exits gracefully
        let items = &view.items;
        assert!(
            items
                .iter()
                .any(|item| matches!(item, ViewItem::RoomDescription { .. }))
        );
        // Items, NPCs, and exits should not be shown if empty
        assert!(!items.iter().any(|item| matches!(item, ViewItem::RoomItems(_))));
        assert!(!items.iter().any(|item| matches!(item, ViewItem::RoomNpcs(_))));
        assert!(items.iter().any(|item| matches!(item, ViewItem::RoomExits(_)))); // Empty exits still shown
    }

    #[test]
    fn exit_with_requirements_shows_correct_state() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.unwrap_room();

        let dest_room_id = Uuid::new_v4();
        let dest_room = create_test_room(dest_room_id);
        world.rooms.insert(dest_room_id, dest_room);

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

        if let Some(ViewItem::RoomExits(exits)) = view.items.iter().find(|item| matches!(item, ViewItem::RoomExits(_)))
        {
            assert_eq!(exits.len(), 1);
            assert!(exits[0].exit_locked);
        }
    }

    #[test]
    fn room_overlay_applies_with_player_missing_item() {
        let world = create_test_world();
        let room_id = world.player.location.unwrap_room();
        let missing_item = Uuid::new_v4();

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::PlayerMissingItem { item_id: missing_item }],
            text: "You don't have this item".into(),
        };

        assert!(overlay.applies(room_id, &world));
    }

    #[test]
    fn room_overlay_applies_with_npc_present() {
        let mut world = create_test_world();
        let room_id = world.player.location.unwrap_room();
        let npc_id = *world.npcs.keys().next().unwrap();
        world.rooms.get_mut(&room_id).unwrap().npcs.insert(npc_id);

        let overlay = RoomOverlay {
            conditions: vec![OverlayCondition::NpcPresent { npc_id }],
            text: "NPC is here".into(),
        };

        assert!(overlay.applies(room_id, &world));
    }

    #[test]
    fn room_show_exits_errors_if_destination_missing() {
        let mut world = create_test_world();
        let mut view = View::new();
        let room_id = world.player.location.unwrap_room();
        let missing_room = Uuid::new_v4();
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
