//! NPC Module

use anyhow::{Context, Result, bail};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use colored::Colorize;
use gametools::Spinner;
use rand::{prelude::IndexedRandom, seq::IteratorRandom};

use uuid::Uuid;

use crate::{
    ItemHolder, Location, View, ViewItem, WorldObject, helpers::room_symbol_from_id, style::GameStyle,
    view::ContentLine, world::AmbleWorld,
};

/// A non-playable character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Npc {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub inventory: HashSet<Uuid>,
    pub dialogue: HashMap<NpcState, Vec<String>>,
    pub state: NpcState,
    pub movement: Option<NpcMovement>,
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

/// Paramaters that define when and where mobile NPCs should move.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcMovement {
    pub movement_type: MovementType,
    pub timing: MovementTiming,
    pub active: bool,
    pub last_moved_turn: usize,
}

/// Type and route of NPC movement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MovementType {
    Route {
        rooms: Vec<Uuid>,
        current_idx: usize,
        loop_route: bool,
    },
    RandomSet {
        rooms: HashSet<Uuid>,
    },
}

/// Defines schedule for NPC movements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MovementTiming {
    EveryNTurns { turns: usize },
    OnTurn { turn: usize },
}

/// Returns true if movement should occur according to the '`NpcMovement`' parameters given.
pub fn move_scheduled(movement: &NpcMovement, current_turn: usize) -> bool {
    match &movement.timing {
        MovementTiming::EveryNTurns { turns } => current_turn % turns == 0,
        MovementTiming::OnTurn { turn } => current_turn == *turn,
    }
}

/// Returns `Location` NPC is set to move to next, if any.
pub fn calculate_next_location(movement: &mut NpcMovement) -> Option<Location> {
    use crate::npc::MovementType::{RandomSet, Route};
    match &mut movement.movement_type {
        Route {
            rooms,
            current_idx,
            loop_route,
        } => {
            let next_idx = if *loop_route {
                (*current_idx + 1) % rooms.len()
            } else {
                *current_idx + 1
            };
            if let Some(room_id) = rooms.get(next_idx) {
                *current_idx = next_idx;
                Some(Location::Room(*room_id))
            } else {
                None
            }
        },
        RandomSet { rooms } => rooms
            .iter()
            .choose(&mut rand::rng())
            .map(|room_id| Location::Room(*room_id)),
    }
}

/// Moves an NPC to a new `Location`.
/// # Errors
/// - if '`move_to`' is a location other than a 'Room' or 'Nowhere'
pub fn move_npc(world: &mut AmbleWorld, view: &mut View, npc_id: Uuid, move_to: Location) -> Result<()> {
    // update location in NPC instance
    let npc = world
        .npcs
        .get(&npc_id)
        .with_context(|| format!("looking up npc_id {npc_id} for move"))?;

    // only valid locations to move to are "nowhere" (a despawn) or a room (spawn/move)
    if move_to.is_not_room() && move_to.is_not_nowhere() {
        bail!("tried to move NPC to invalid location {move_to:?}")
    }
    let current_room_sym = match npc.location {
        Location::Room(uuid) => world
            .rooms
            .get(&uuid)
            .map(|room| room.symbol.clone())
            .expect("npc Room should be valid"),
        _ => "<nowhere>".to_string(),
    };
    let dest_room_sym = match move_to {
        Location::Room(uuid) => world
            .rooms
            .get(&uuid)
            .map(|room| room.symbol.clone())
            .expect("move_to Room should be valid"),
        _ => "<nowhere>".to_string(),
    };

    info!(
        "moving NPC '{}' from [{}] to [{}]",
        npc.symbol, current_room_sym, dest_room_sym
    );

    // TODO: check player location here -- push a ViewItem to View if the NPC is either
    // entering or leaving the player's room.

    // get source and destination ids, or None where not a room
    let from_room_id = match &npc.location {
        Location::Room(uuid) => Some(*uuid),
        _ => None,
    };
    let to_room_id = match &move_to {
        Location::Room(uuid) => Some(*uuid),
        _ => None,
    };

    // needed for message to player if NPC entering / leaving their current location
    let player_room_id = world.player_room_ref()?.id();

    // update npc list in from/to rooms as appropriate
    if let Some(uuid) = from_room_id {
        if uuid == player_room_id {
            view.push(ViewItem::TriggeredEvent(format!("{} left.", npc.name().npc_style())));
            info!("{} ({}) left the Candidate's location.", npc.name(), npc.symbol());
        }
        world.rooms.get_mut(&uuid).map(|room| room.npcs.remove(&npc_id));
    }
    if let Some(uuid) = to_room_id {
        if uuid == player_room_id {
            view.push(ViewItem::TriggeredEvent(format!("{} entered.", npc.name().npc_style())));
            info!("{} ({}) arrived at the Candidate's location.", npc.name(), npc.symbol());
        }
        world.rooms.get_mut(&uuid).map(|room| room.npcs.insert(npc_id));
    }

    // finally update NPC instance's location field
    world.npcs.get_mut(&npc_id).map(|npc| npc.location = move_to);

    Ok(())
}

/// Represents the demeanor of an 'Npc', which may affect default dialogue and behavior.
///
/// NPC states affect which dialogue lines are given in response to a `TalkTo` command. They
/// can also be used as trigger conditions, and state can be changed by triggers. Room
/// overlays can also change according to NPC presence / state. Custom states allow for
/// other "moods" and can be used to pin selections of dialogue to particular game states.
/// Ex: player does something -> puzzle advanced to puzzle#2 -> trigger sets custom NPC state
/// "player_at_puzzle_step_2" which has specific dialogue.
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::Item,
        view::{View, ViewItem},
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn create_test_npc() -> Npc {
        let mut dialogue = HashMap::new();
        dialogue.insert(NpcState::Normal, vec!["Hello there!".into(), "Nice weather!".into()]);
        dialogue.insert(NpcState::Happy, vec!["What a wonderful day!".into()]);
        dialogue.insert(NpcState::Mad, vec!["Go away!".into(), "I'm not talking to you!".into()]);

        Npc {
            id: Uuid::new_v4(),
            symbol: "test_npc".into(),
            name: "Test NPC".into(),
            description: "A test NPC".into(),
            location: Location::Nowhere,
            inventory: HashSet::new(),
            dialogue,
            state: NpcState::Normal,
            movement: None,
        }
    }

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::new_empty();

        let item_id = Uuid::new_v4();
        let item = Item {
            id: item_id,
            symbol: "test_item".into(),
            name: "Test Item".into(),
            description: "A test item".into(),
            location: Location::Nowhere,
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

        world
    }

    #[test]
    fn npc_state_from_key_parses_standard_states() {
        assert_eq!(NpcState::from_key("sad"), NpcState::Sad);
        assert_eq!(NpcState::from_key("bored"), NpcState::Bored);
        assert_eq!(NpcState::from_key("normal"), NpcState::Normal);
        assert_eq!(NpcState::from_key("happy"), NpcState::Happy);
        assert_eq!(NpcState::from_key("mad"), NpcState::Mad);
        assert_eq!(NpcState::from_key("tired"), NpcState::Tired);
    }

    #[test]
    fn npc_state_from_key_parses_custom_states() {
        let custom = NpcState::from_key("custom:excited");
        assert_eq!(custom, NpcState::Custom("excited".into()));

        let custom2 = NpcState::from_key("custom:some_state");
        assert_eq!(custom2, NpcState::Custom("some_state".into()));
    }

    #[test]
    fn npc_state_from_key_defaults_to_normal_for_unknown() {
        assert_eq!(NpcState::from_key("unknown_state"), NpcState::Normal);
        assert_eq!(NpcState::from_key("invalid"), NpcState::Normal);
        assert_eq!(NpcState::from_key(""), NpcState::Normal);
    }

    #[test]
    fn npc_state_as_key_converts_correctly() {
        assert_eq!(NpcState::Sad.as_key(), "sad");
        assert_eq!(NpcState::Bored.as_key(), "bored");
        assert_eq!(NpcState::Normal.as_key(), "normal");
        assert_eq!(NpcState::Happy.as_key(), "happy");
        assert_eq!(NpcState::Mad.as_key(), "mad");
        assert_eq!(NpcState::Tired.as_key(), "tired");
        assert_eq!(NpcState::Custom("excited".into()).as_key(), "custom:excited");
    }

    #[test]
    fn npc_state_display_works() {
        assert_eq!(format!("{}", NpcState::Happy), "Happy");
        assert_eq!(format!("{}", NpcState::Bored), "Bored");
        assert_eq!(format!("{}", NpcState::Mad), "Mad");
        assert_eq!(format!("{}", NpcState::Normal), "Normal");
        assert_eq!(format!("{}", NpcState::Sad), "Sad");
        assert_eq!(format!("{}", NpcState::Tired), "Tired");
        assert_eq!(format!("{}", NpcState::Custom("test".into())), "Custom");
    }

    #[test]
    fn npc_random_dialogue_returns_appropriate_line() {
        use gametools::{Spinner, Wedge};

        let npc = create_test_npc();
        let ignore_spinner = Spinner::new(vec![Wedge::new("Ignores you.".into())]);

        // Test normal state dialogue
        let dialogue = npc.random_dialogue(&ignore_spinner);
        let normal_lines = &npc.dialogue[&NpcState::Normal];
        assert!(normal_lines.contains(&dialogue) || dialogue == "Ignores you.");
    }

    #[test]
    fn npc_random_dialogue_returns_fallback_for_missing_state() {
        use gametools::{Spinner, Wedge};

        let mut npc = create_test_npc();
        npc.state = NpcState::Tired; // State not in dialogue map
        let ignore_spinner = Spinner::new(vec![Wedge::new("Ignores you.".into())]);

        let dialogue = npc.random_dialogue(&ignore_spinner);
        assert_eq!(dialogue, "Ignores you.");
    }

    #[test]
    fn npc_show_displays_description_and_inventory() {
        let world = create_test_world();
        let mut view = View::new();
        let item_id = *world.items.keys().next().unwrap();

        let mut npc = create_test_npc();
        npc.inventory.insert(item_id);

        npc.show(&world, &mut view);

        // Check that the view contains the expected items
        let items = &view.items;
        assert!(items.iter().any(|item| matches!(item, ViewItem::NpcDescription { .. })));
        assert!(items.iter().any(|item| matches!(item, ViewItem::NpcInventory(_))));

        if let Some(ViewItem::NpcDescription { name, description }) = items
            .iter()
            .find(|item| matches!(item, ViewItem::NpcDescription { .. }))
        {
            assert_eq!(name, "Test NPC");
            assert_eq!(description, "A test NPC");
        }

        if let Some(ViewItem::NpcInventory(inventory)) =
            items.iter().find(|item| matches!(item, ViewItem::NpcInventory(_)))
        {
            assert_eq!(inventory.len(), 1);
            assert_eq!(inventory[0].item_name, "Test Item");
        }
    }

    #[test]
    fn npc_show_handles_empty_inventory() {
        let world = create_test_world();
        let mut view = View::new();
        let npc = create_test_npc();

        npc.show(&world, &mut view);

        if let Some(ViewItem::NpcInventory(inventory)) =
            view.items.iter().find(|item| matches!(item, ViewItem::NpcInventory(_)))
        {
            assert!(inventory.is_empty());
        }
    }

    #[test]
    fn world_object_trait_works() {
        let npc = create_test_npc();
        assert_eq!(npc.symbol(), "test_npc");
        assert_eq!(npc.name(), "Test NPC");
        assert_eq!(npc.description(), "A test NPC");
        assert_eq!(npc.location(), &Location::Nowhere);
    }

    #[test]
    fn item_holder_add_item_works() {
        let mut npc = create_test_npc();
        let item_id = Uuid::new_v4();

        npc.add_item(item_id);
        assert!(npc.inventory.contains(&item_id));
    }

    #[test]
    fn item_holder_remove_item_works() {
        let mut npc = create_test_npc();
        let item_id = Uuid::new_v4();
        npc.inventory.insert(item_id);

        npc.remove_item(item_id);
        assert!(!npc.inventory.contains(&item_id));
    }

    #[test]
    fn item_holder_contains_item_works() {
        let mut npc = create_test_npc();
        let item_id = Uuid::new_v4();
        npc.inventory.insert(item_id);

        assert!(npc.contains_item(item_id));
        assert!(!npc.contains_item(Uuid::new_v4()));
    }

    #[test]
    fn npc_state_equality_and_hash_work() {
        let state1 = NpcState::Happy;
        let state2 = NpcState::Happy;
        let state3 = NpcState::Mad;

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);

        let mut state_map = HashMap::new();
        state_map.insert(state1, "happy dialogue");
        assert_eq!(state_map.get(&state2), Some(&"happy dialogue"));
    }

    #[test]
    fn npc_state_custom_equality_works() {
        let custom1 = NpcState::Custom("excited".into());
        let custom2 = NpcState::Custom("excited".into());
        let custom3 = NpcState::Custom("angry".into());

        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
        assert_ne!(custom1, NpcState::Happy);
    }
}
