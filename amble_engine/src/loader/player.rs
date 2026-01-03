//! Player loading helpers.
//!
//! Converts the compiled `WorldDef` player definition into a [`Player`]
//! instance for runtime use.

use std::collections::HashSet;

use anyhow::Result;
use log::info;

use amble_data::PlayerDef;

use crate::Id;
use crate::health::HealthState;
use crate::player::{Flag, Player};
use crate::world::Location;

/// Build `Player` from player definition.
pub fn build_player(def: &PlayerDef) -> Result<Player> {
    let flags: HashSet<Flag> = HashSet::new();

    let player = Player {
        id: "player".to_string(),
        symbol: "player".to_string(),
        name: def.name.clone(),
        description: def.description.clone(),
        location: Location::Room(def.start_room.clone()),
        location_history: Vec::new(),
        inventory: HashSet::<Id>::default(),
        flags,
        score: 0,
        health: HealthState::new_at_max(def.max_hp),
    };
    info!("built player character from definition");
    Ok(player)
}
