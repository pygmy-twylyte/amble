use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, anyhow};
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    idgen::{NAMESPACE_CHARACTER, uuid_from_token},
    player::Player,
};

use super::{SymbolTable, resolve_location};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
/// Serialized version of Player, used for staged loading from TOML.
pub struct RawPlayer {
    pub id: String,
    pub name: String,
    pub description: String,
    pub location: HashMap<String, String>,
    #[serde(default)]
    pub inventory: HashMap<String, String>,
    #[serde(default)]
    pub flags: HashSet<String>,
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
            Some(id) => *id,
            None => {
                return Err(anyhow!(
                    "UUID for player ({}) not found in symbol table",
                    self.id
                ));
            },
        };
        let player = Player {
            id,
            name: self.name.to_string(),
            description: self.description.to_string(),
            location,
            inventory: HashSet::<Uuid>::default(),
            flags: HashSet::<String>::default(),
            score: self.score,
        };
        Ok(player)
    }
}

/// Load player data from file
/// # Errors
/// - if unable to read the player.toml file or unable to parse it
pub fn load_player(toml_path: &Path) -> Result<RawPlayer> {
    let player_file = fs::read_to_string(toml_path)
        .with_context(|| format!("reading player data from '{}'", toml_path.display()))?;
    let raw_player: RawPlayer = toml::from_str(&player_file)
        .with_context(|| format!("parsing player data from '{}'", toml_path.display()))?;
    Ok(raw_player)
}

/// Build `Player` from raw player.
/// # Errors
/// - if symbol lookup fails during conversion of raw player to player instance
pub fn build_player(raw_player: &RawPlayer, symbols: &mut SymbolTable) -> Result<Player> {
    symbols.characters.insert(
        raw_player.id.to_string(),
        uuid_from_token(&NAMESPACE_CHARACTER, &raw_player.id),
    );
    info!("added player character to symbol table");
    let player = raw_player
        .to_player(symbols)
        .context("converting raw player to player object")?;
    info!("built player character from raw player");
    Ok(player)
}
