use crate::item::ContainerState;
use crate::npc::Npc;
use crate::spinners::{SpinnerType, default_spinners};
use crate::trigger::Trigger;
use crate::{Goal, Item, Player, Room};

use anyhow::{Context, Result, anyhow};
use gametools::Spinner;
use log::info;
use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use variantly::Variantly;

/// Kinds of places where a `WorldObject` may be located.
/// Because Rooms *are* the locations, their location is always `Nowhere`
/// Unspawned/despawned items and NPCs are also located `Nowhere`
#[derive(Copy, Debug, Default, Clone, Serialize, Deserialize, Variantly, PartialEq, Eq)]
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
    #[serde(skip)] // these are hard-coded into spinners.rs
    pub spinners: HashMap<SpinnerType, Spinner<&'static str>>,
    pub npcs: HashMap<Uuid, Npc>,
    pub max_score: usize,
    pub goals: Vec<Goal>,
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
            max_score: 0,
            goals: Vec::new(),
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
    /// # Errors
    /// - if player isn't in a Room or the Room's uuid is not found
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
    /// # Errors
    /// - if player is not in a room or room's UUID is not found
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

/// Constructs a set of all potentially take-able / viewable item (uuids) in a room.
/// Non-portable or restricted items not filtered here -- player discovers
/// that on their own. The scope includes items in room, and items in open containers.
/// Items in closed or locked containers and NPCs are excluded.
///
/// # Errors
/// - if supplied `room_id` is invalid
pub fn nearby_reachable_items(world: &AmbleWorld, room_id: Uuid) -> Result<HashSet<Uuid>> {
    let current_room = world
        .rooms
        .get(&room_id)
        .with_context(|| format!("{room_id} room id not found"))?;
    let room_items = &current_room.contents;
    let mut contained_items = HashSet::new();
    for item_id in room_items {
        if let Some(item) = world.items.get(item_id)
            && item.container_state == Some(ContainerState::Open)
        {
            contained_items.extend(&item.contents);
        }
    }
    Ok(room_items.union(&contained_items).copied().collect())
}
