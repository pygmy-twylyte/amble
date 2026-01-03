//! Player loading helpers.
//!
//! The player character is described in `player.toml`. This module reads the
//! file and converts the raw representation into a [`Player`] instance.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use crate::Id;
use anyhow::{Context, Result, anyhow};
use log::info;
use serde::Deserialize;

use crate::{
    health::HealthState,
    idgen::{NAMESPACE_CHARACTER, uuid_from_token},
    player::{Flag, Player},
};

use super::{SymbolTable, resolve_location};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
/// Serialized version of Player, used for staged loading from TOML.
pub struct RawPlayer {
    /// The string used to identify the player and generate its UUID.
    pub id: String,
    /// The name of the player character.
    pub name: String,
    /// A brief description of the player character.
    pub description: String,
    /// The player's starting `Location`.
    pub location: HashMap<String, String>,
    /// The player's maximum hit points (health).
    pub max_hp: u32,
    /// Player's inventory PLACEHOLDER. Items should not (cannot) be loaded into inventory here.
    /// To start a player with inventory, create items with a starting `Location::Inventory`.
    #[serde(default)]
    pub inventory: HashMap<String, String>,
    /// Any flags applied to the player at game start. (None by default.)
    #[serde(default)]
    pub flags: HashSet<String>,
    /// Initial score, defaults to zero if not specified.
    #[serde(default)]
    pub score: usize,
}
impl RawPlayer {
    /// Converts tokenized and serialized `RawPlayer` to `Player` object
    /// # Errors
    /// - if unable to resolve the location of the player from the TOML file
    /// - if player UUID not found in symbol table
    pub fn to_player(&self, symbols: &SymbolTable) -> Result<Player> {
        let location = resolve_location(&self.location, symbols)?;
        let id = match symbols.characters.get(&self.id) {
            Some(id) => id.clone(),
            None => {
                return Err(anyhow!("UUID for player ({}) not found in symbol table", self.id));
            },
        };
        let player = Player {
            id,
            symbol: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            location,
            location_history: Vec::new(),
            inventory: HashSet::<Id>::default(),
            flags: HashSet::<Flag>::default(),
            score: self.score,
            health: HealthState::new_at_max(self.max_hp),
        };
        Ok(player)
    }
}

/// Load player data from file
/// # Errors
/// - if unable to read the player.toml file or unable to parse it
pub fn load_player(toml_path: &Path) -> Result<RawPlayer> {
    let player_file =
        fs::read_to_string(toml_path).with_context(|| format!("reading player data from '{}'", toml_path.display()))?;
    let raw_player: RawPlayer =
        toml::from_str(&player_file).with_context(|| format!("parsing player data from '{}'", toml_path.display()))?;
    Ok(raw_player)
}

/// Build `Player` from raw player.
/// # Errors
/// - if symbol lookup fails during conversion of raw player to player instance
pub fn build_player(raw_player: &RawPlayer, symbols: &mut SymbolTable) -> Result<Player> {
    symbols.characters.insert(
        raw_player.id.clone(),
        uuid_from_token(&NAMESPACE_CHARACTER, &raw_player.id),
    );
    info!("added player character to symbol table");
    let player = raw_player
        .to_player(symbols)
        .context("converting raw player to player object")?;
    info!("built player character from raw player");
    Ok(player)
}
