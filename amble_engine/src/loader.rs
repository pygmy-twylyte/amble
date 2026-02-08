//! Loader utilities for building an `AmbleWorld` from serialized data.
//!
//! World content is loaded from the compiled `WorldDef` (RON), while help
//! metadata remains TOML-backed.

pub mod help;
pub mod placement;
pub mod player;
pub mod scoring;
pub mod worlddef;

use crate::loader::placement::{place_items, place_npcs};
use crate::loader::worlddef::{build_world_from_def, load_worlddef};

use crate::data_paths::data_path;
use crate::slug::sanitize_slug;
use crate::trigger::TriggerAction;
use crate::{AmbleWorld, WorldObject};
use amble_data::WorldDef;
use anyhow::{Context, Result, bail};
use log::{info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

static ACTIVE_WORLD_PATH: LazyLock<RwLock<Option<PathBuf>>> = LazyLock::new(|| RwLock::new(None));

#[derive(Debug, Clone)]
pub struct WorldSource {
    pub path: PathBuf,
    pub title: String,
    pub slug: String,
    pub author: String,
    pub version: String,
    pub blurb: String,
}

/// Pin the active world file path so reloads use the same source.
pub fn set_active_world_path(path: PathBuf) {
    if let Ok(mut guard) = ACTIVE_WORLD_PATH.write() {
        *guard = Some(path);
    }
}

/// Return the currently active world file path, if set.
pub fn active_world_path() -> Option<PathBuf> {
    ACTIVE_WORLD_PATH.read().ok().and_then(|guard| guard.clone())
}

/// Discover compiled world files from the data directory.
///
/// Looks for `data/worlds/*.ron` when a `worlds/` directory exists, falling back
/// to `data/world.ron` if none are found.
pub fn discover_world_sources() -> Result<Vec<WorldSource>> {
    let data_root = data_path("");
    let worlds_dir = data_root.join("worlds");
    let mut candidates = Vec::new();

    if worlds_dir.is_dir() {
        for entry in
            fs::read_dir(&worlds_dir).with_context(|| format!("reading world directory {}", worlds_dir.display()))?
        {
            let entry = entry.with_context(|| format!("enumerating {}", worlds_dir.display()))?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("ron") {
                candidates.push(path);
            }
        }
    }

    let fallback = data_root.join("world.ron");
    if fallback.is_file() {
        candidates.push(fallback);
    }

    candidates.sort();
    let mut sources = Vec::new();
    for path in candidates {
        match load_worlddef(&path) {
            Ok(def) => {
                let slug = derive_world_slug(&def, &path);
                let title = derive_world_title(&def, &path);
                sources.push(WorldSource {
                    path,
                    title,
                    slug,
                    author: def.game.author.clone(),
                    version: def.game.version.clone(),
                    blurb: def.game.blurb.clone(),
                });
            },
            Err(err) => {
                warn!("Skipping world file {}: {err}", path.display());
            },
        }
    }

    if sources.is_empty() {
        bail!(
            "no valid world files found in {}",
            if worlds_dir.is_dir() {
                worlds_dir.display().to_string()
            } else {
                data_root.display().to_string()
            }
        );
    }

    Ok(sources)
}
/// Load the `AmbleWorld` from the compiled `WorldDef` file.
///
/// # Errors
/// Errors bubble up from file IO, deserialization, or missing references.
pub fn load_world() -> Result<AmbleWorld> {
    let world_ron_path = active_world_path().unwrap_or_else(|| data_path("world.ron"));
    load_world_from_path(&world_ron_path)
}

/// Load the `AmbleWorld` from a specific compiled `WorldDef` file.
///
/// # Errors
/// Errors bubble up from file IO, deserialization, or missing references.
pub fn load_world_from_path(world_ron_path: &Path) -> Result<AmbleWorld> {
    info!("loading selected world definition from: {}", world_ron_path.display());
    let worlddef = load_worlddef(&world_ron_path).context("while loading worlddef from file")?;

    validate_worlddef(&worlddef)?;
    info!("validation passed, building AmbleWorld for \"{}\"", worlddef.game.title);

    let mut world = build_world_from_def(&worlddef).context("while building world from worlddef")?;
    world.world_slug = derive_world_slug(&worlddef, world_ron_path);
    world.game_title = derive_world_title(&worlddef, world_ron_path);
    info!("{} spinners added to AmbleWorld", world.spinners.len());
    info!("{} rooms added to AmbleWorld", world.rooms.len());
    info!("{} NPCs added to AmbleWorld", world.npcs.len());
    info!("{} items added to AmbleWorld", world.items.len());
    info!("{} triggers added to AmbleWorld", world.triggers.len());
    info!("{} goals added to AmbleWorld", world.goals.len());
    info!("Scoring configuration loaded with {} ranks", world.scoring.ranks.len());

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

    // we gather an estimate of possible maximum points to earn in the world here, but
    // in can be made inaccurate by repeatable awards or mutually exclusive reward paths
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

fn derive_world_slug(def: &WorldDef, path: &Path) -> String {
    let candidate = def.game.slug.trim();
    if !candidate.is_empty() {
        return sanitize_slug(candidate);
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if !stem.is_empty() && stem != "world" {
        return sanitize_slug(stem);
    }
    let title = def.game.title.trim();
    if !title.is_empty() {
        return sanitize_slug(title);
    }
    "world".to_string()
}

fn derive_world_title(def: &WorldDef, path: &Path) -> String {
    let title = def.game.title.trim();
    if !title.is_empty() {
        return title.to_string();
    }
    path.file_stem()
        .and_then(|s| s.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Untitled World")
        .to_string()
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
