//! Loader utilities for building an `AmbleWorld` from serialized data.
//!
//! World content is loaded from the compiled `WorldDef` (RON), while scoring
//! and help metadata remain TOML-backed.

pub mod help;
pub mod placement;
pub mod player;
pub mod scoring;
pub mod worlddef;

use crate::loader::placement::{place_items, place_npcs};
use crate::loader::player::{build_player, load_player_def};
use crate::loader::scoring::load_scoring;
use crate::loader::worlddef::{build_world_from_def, load_worlddef};

use crate::data_paths::data_path;
use crate::trigger::TriggerAction;
use crate::{AmbleWorld, WorldObject};
use amble_data::WorldDef;
use anyhow::{Context, Result, bail};
use log::info;
/// Load the `AmbleWorld` from the compiled `WorldDef` file.
///
/// # Errors
/// Errors bubble up from file IO, deserialization, or missing references.
pub fn load_world() -> Result<AmbleWorld> {
    let world_ron_path = data_path("world.ron");
    let player_ron_path = data_path("player.ron");
    let scoring_toml_path = data_path("scoring.toml");

    let worlddef = load_worlddef(&world_ron_path).context("while loading worlddef from file")?;
    validate_worlddef(&worlddef)?;
    let mut world = build_world_from_def(&worlddef).context("while building world from worlddef")?;
    info!("{} spinners added to AmbleWorld", world.spinners.len());
    info!("{} rooms added to AmbleWorld", world.rooms.len());
    info!("{} NPCs added to AmbleWorld", world.npcs.len());
    info!("{} items added to AmbleWorld", world.items.len());
    info!("{} triggers added to AmbleWorld", world.triggers.len());
    info!("{} goals added to AmbleWorld", world.goals.len());

    world.scoring = load_scoring(&scoring_toml_path);
    info!("Scoring configuration loaded with {} ranks", world.scoring.ranks.len());

    let player_def = load_player_def(&player_ron_path).context("while loading player from file")?;
    world.player = build_player(&player_def).context("while building player from definition")?;
    let start_room_id = world
        .player
        .location
        .room_id()
        .context("player start location is not a room")?;
    world.player_path.push(start_room_id.clone());
    info!(
        "player \"{}\" added to AmbleWorld at {}",
        world.player.name(),
        start_room_id
    );

    place_npcs(&mut world)?;
    place_items(&mut world)?;

    for trigger in &world.triggers {
        for action in &trigger.actions {
            if let TriggerAction::AwardPoints { amount, .. } = &action.action
                && *amount > 0
            {
                world.max_score = world.max_score.saturating_add_signed(*amount);
            }
        }
    }

    Ok(world)
}

/// Validate the compiled WorldDef and return a single aggregated error.
fn validate_worlddef(def: &WorldDef) -> Result<()> {
    let errors = amble_data::validate_world(def);
    if errors.is_empty() {
        return Ok(());
    }
    let details = errors
        .into_iter()
        .map(|err| format!("- {err}"))
        .collect::<Vec<_>>()
        .join("\n");
    bail!("worlddef validation failed:\n{details}");
}
