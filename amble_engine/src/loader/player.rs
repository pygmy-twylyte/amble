//! Player loading helpers.
//!
//! The player character is described in `player.ron`. This module reads the
//! file and converts the raw representation into a [`Player`] instance.

use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};

use crate::Id;
use crate::health::HealthState;
use crate::player::{Flag, Player};
use crate::world::Location;

/// Serialized player definition loaded from `player.ron`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub max_hp: u32,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub score: usize,
}

/// Load player data from file.
pub fn load_player_def(path: &Path) -> Result<PlayerDef> {
    let text = fs::read_to_string(path).with_context(|| format!("reading player data from '{}'", path.display()))?;
    ron::from_str(&text).with_context(|| format!("parsing player data from '{}'", path.display()))
}

/// Build `Player` from player definition.
pub fn build_player(def: &PlayerDef) -> Result<Player> {
    let flags: HashSet<Flag> = def.flags.iter().map(|flag| Flag::simple(flag, 0)).collect();

    let player = Player {
        id: def.id.clone(),
        symbol: def.id.clone(),
        name: def.name.clone(),
        description: def.description.clone(),
        location: def.location.clone(),
        location_history: Vec::new(),
        inventory: HashSet::<Id>::default(),
        flags,
        score: def.score,
        health: HealthState::new_at_max(def.max_hp),
    };
    info!("built player character from definition");
    Ok(player)
}
