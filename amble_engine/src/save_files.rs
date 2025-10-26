//! Save-game discovery and serialization helpers.
//!
//! Provides file management utilities for listing, loading, and writing
//! player save slots with version awareness.

use crate::{AMBLE_VERSION, AmbleWorld, Location, WorldObject};
use anyhow::{Context, Result};
use log::warn;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const SAVE_DIR: &str = "saved_games";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSlot {
    pub slot: String,
    pub version: String,
    pub path: PathBuf,
    pub file_name: String,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSummary {
    pub player_name: String,
    pub player_location: Option<String>,
    pub turn_count: usize,
    pub score: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFileStatus {
    Ready,
    VersionMismatch { save_version: String, current_version: String },
    Corrupted { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveFileEntry {
    pub slot: String,
    pub version: String,
    pub path: PathBuf,
    pub file_name: String,
    pub modified: Option<SystemTime>,
    pub summary: Option<SaveSummary>,
    pub status: SaveFileStatus,
}

/// Discover save slot files stored in `dir`.
pub fn collect_save_slots(dir: &Path) -> Result<Vec<SaveSlot>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut slots = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry.with_context(|| format!("enumerating {}", dir.display()))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("ron") {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()).map(str::to_string) else {
            continue;
        };
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Some((slot, version)) = stem.rsplit_once("-amble-") else {
            continue;
        };
        if slot.is_empty() {
            continue;
        }
        let modified = entry.metadata().ok().and_then(|meta| meta.modified().ok());
        slots.push(SaveSlot {
            slot: slot.to_string(),
            version: version.to_string(),
            path: path.clone(),
            file_name,
            modified,
        });
    }
    slots.sort_by(|a, b| a.slot.cmp(&b.slot).then(a.version.cmp(&b.version)));
    Ok(slots)
}

/// Build descriptive entries for save files located in `dir`.
pub fn build_save_entries(dir: &Path) -> Result<Vec<SaveFileEntry>> {
    let slots = collect_save_slots(dir)?;
    let mut entries: Vec<_> = slots.into_iter().map(entry_for_slot).collect();
    entries.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.slot.cmp(&b.slot)));
    Ok(entries)
}

/// Format a human-friendly modified time relative to now.
pub fn format_modified(modified: SystemTime) -> String {
    match SystemTime::now().duration_since(modified) {
        Ok(delta) => format_duration(delta),
        Err(_) => "in the future".to_string(),
    }
}

/// Build a full [`SaveFileEntry`] from a discovered save slot.
fn entry_for_slot(slot: SaveSlot) -> SaveFileEntry {
    let mut version = slot.version.clone();
    let (summary, status) = match fs::read_to_string(&slot.path) {
        Ok(raw) => match ron::from_str::<AmbleWorld>(&raw) {
            Ok(world) => {
                version = world.version.clone();
                let status = if world.version == AMBLE_VERSION {
                    SaveFileStatus::Ready
                } else {
                    SaveFileStatus::VersionMismatch {
                        save_version: world.version.clone(),
                        current_version: AMBLE_VERSION.to_string(),
                    }
                };
                let summary = SaveSummary {
                    player_name: world.player.name.clone(),
                    player_location: describe_location(&world),
                    turn_count: world.turn_count,
                    score: world.player.score,
                };
                (Some(summary), status)
            },
            Err(err) => {
                warn!(
                    "failed to parse save '{}' ({}): {}",
                    slot.slot,
                    slot.path.display(),
                    err
                );
                (
                    None,
                    SaveFileStatus::Corrupted {
                        message: format!("parse error: {}", trim_error(err)),
                    },
                )
            },
        },
        Err(err) => {
            warn!("failed to read save '{}' ({}): {}", slot.slot, slot.path.display(), err);
            (
                None,
                SaveFileStatus::Corrupted {
                    message: format!("read error: {}", trim_error(err)),
                },
            )
        },
    };

    SaveFileEntry {
        slot: slot.slot,
        version,
        path: slot.path,
        file_name: slot.file_name,
        modified: slot.modified,
        summary,
        status,
    }
}

/// Render the player's current location into a human-readable label.
fn describe_location(world: &AmbleWorld) -> Option<String> {
    match world.player.location {
        Location::Room(room_id) => world.rooms.get(&room_id).map(|room| room.name.clone()),
        Location::Inventory => Some("Inventory".to_string()),
        Location::Item(item_id) => world.items.get(&item_id).map(|item| format!("Inside {}", item.name())),
        Location::Npc(npc_id) => world.npcs.get(&npc_id).map(|npc| format!("With {}", npc.name())),
        Location::Nowhere => None,
    }
}

/// Convert a duration into a compact "time ago" string.
fn format_duration(duration: Duration) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;

    let secs = duration.as_secs();
    if secs < 30 {
        "just now".to_string()
    } else if secs < MINUTE {
        format!("{secs}s ago")
    } else if secs < HOUR {
        format!("{}m ago", secs / MINUTE)
    } else if secs < DAY {
        format!("{}h ago", secs / HOUR)
    } else if secs < WEEK {
        format!("{}d ago", secs / DAY)
    } else if secs < MONTH {
        format!("{}w ago", secs / WEEK)
    } else if secs < YEAR {
        format!("{}mo ago", secs / MONTH)
    } else {
        format!("{}y ago", secs / YEAR)
    }
}

/// Clamp verbose error messages to a readable length.
fn trim_error(err: impl ToString) -> String {
    let message = err.to_string();
    if message.chars().count() <= 120 {
        return message;
    }
    let mut trimmed = String::new();
    for (idx, ch) in message.chars().enumerate() {
        if idx >= 117 {
            trimmed.push_str("...");
            break;
        }
        trimmed.push(ch);
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Room;
    use anyhow::Result;
    use std::collections::{HashMap, HashSet};
    use tempfile::tempdir;
    use uuid::Uuid;

    #[test]
    fn collect_save_slots_handles_missing_directory() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("missing");
        let slots = collect_save_slots(&path)?;
        assert!(slots.is_empty());
        Ok(())
    }

    #[test]
    fn collect_save_slots_skips_invalid_files() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();
        fs::write(path.join("alpha-amble-0.60.0.ron"), "[]")?;
        fs::write(path.join("notes.txt"), "ignore me")?;
        fs::create_dir_all(path.join("nested"))?;

        let slots = collect_save_slots(path)?;
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot, "alpha");
        assert_eq!(slots[0].version, "0.60.0");
        Ok(())
    }

    #[test]
    fn build_save_entries_reports_status_variants() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();

        let room_id = Uuid::new_v4();
        let room = Room {
            id: room_id,
            symbol: "room_symbol".into(),
            name: "Test Room".into(),
            base_description: "Desc".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };

        let mut world = AmbleWorld::new_empty();
        world.player.name = "Tester".into();
        world.player.score = 42;
        world.turn_count = 7;
        world.player.location = Location::Room(room_id);
        world.rooms.insert(room_id, room);

        let ron = ron::ser::to_string(&world)?;
        fs::write(path.join("alpha-amble-0.60.0.ron"), ron)?;

        let mut old_world = world.clone();
        old_world.version = "0.59.0".into();
        let ron_old = ron::ser::to_string(&old_world)?;
        fs::write(path.join("beta-amble-0.59.0.ron"), ron_old)?;

        fs::write(path.join("gamma-amble-0.60.0.ron"), "this is not valid ron")?;

        let mut entries = build_save_entries(path)?;
        entries.sort_by(|a, b| a.slot.cmp(&b.slot));

        let alpha = entries.iter().find(|entry| entry.slot == "alpha").unwrap();
        assert!(matches!(alpha.status, SaveFileStatus::Ready));
        assert_eq!(alpha.summary.as_ref().unwrap().player_name, "Tester");

        let beta = entries.iter().find(|entry| entry.slot == "beta").unwrap();
        assert!(matches!(beta.status, SaveFileStatus::VersionMismatch { .. }));
        assert_eq!(beta.version, "0.59.0");

        let gamma = entries.iter().find(|entry| entry.slot == "gamma").unwrap();
        assert!(matches!(gamma.status, SaveFileStatus::Corrupted { .. }));
        assert!(gamma.summary.is_none());

        Ok(())
    }
}
