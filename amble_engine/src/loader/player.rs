//! Player loading helpers.
//!
//! Converts the compiled `WorldDef` player definition into a [`Player`]
//! instance for runtime use.

use std::collections::HashSet;

use log::info;

use amble_data::PlayerDef;

use crate::health::HealthState;
use crate::player::{Flag, Player};
use crate::world::Location;
use crate::{ItemId, RoomId};

/// Build `Player` from player definition.
pub fn build_player(def: &PlayerDef) -> Player {
    info!("building player character from definition");

    Player {
        id: "player".to_string(),
        symbol: "player".to_string(),
        name: def.name.clone(),
        description: def.description.clone(),
        location: Location::Room(RoomId(def.start_room.clone())),
        location_history: Vec::new(),
        inventory: HashSet::<ItemId>::default(),
        flags: HashSet::<Flag>::default(),
        score: 0,
        health: HealthState::new_at_max(def.max_hp),
    }
}
