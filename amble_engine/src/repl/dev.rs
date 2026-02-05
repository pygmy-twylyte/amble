//! Development mode command handlers for the Amble game engine.
//!
//! This module provides handlers for special development commands that are only
//! available when the game is running in `DEV_MODE`. These commands allow developers
//! and content creators to manipulate game state for testing, debugging, and
//! content creation purposes.
//!
//! # Security Note
//!
//! All functions in this module bypass normal game restrictions and can modify
//! world state in ways that would not be possible through regular gameplay.
//! They should only be available in development builds or when explicitly
//! enabled by developers.
//!
//! # Available Commands
//!
//! ## Item Manipulation
//! - [`dev_spawn_item_handler`] - Instantly add any item to player inventory
//!
//! ## Player Movement
//! - [`dev_teleport_handler`] - Instantly transport player to any room
//!
//! ## Playtest Notes
//! - [`dev_note_handler`] - Append a playtest note with location metadata
//!
//! ## Flag Management
//! - [`dev_start_seq_handler`] - Create new sequence flags with custom limits
//! - [`dev_set_flag_handler`] - Add simple boolean flags to player
//! - [`dev_advance_seq_handler`] - Advance sequence flags by one step
//! - [`dev_reset_seq_handler`] - Reset sequence flags back to step 0
//!
//! # Logging
//!
//! All dev commands use `warn`-level logging to create an audit trail of
//! development actions, making it easy to track what changes were made
//! during testing sessions.

use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use log::{info, warn};
use time::{OffsetDateTime, format_description};

use crate::helpers::symbol_or_unknown;
use crate::save_files::LOG_DIR;
use crate::scheduler::{EventCondition, OnFalsePolicy, ScheduledEvent};
use crate::slug::sanitize_slug;
use crate::trigger::TriggerCondition;
use crate::{
    AmbleWorld, Location, View, ViewItem, WorldObject,
    idgen::{NAMESPACE_ITEM, NAMESPACE_ROOM, uuid_from_token},
    player::Flag,
    style::GameStyle,
    trigger::{self, spawn_item_in_inventory},
};

/// Spawns an item directly into the player's inventory (`DEV_MODE` only).
///
/// This development command instantly adds any item to the player's inventory
/// by its symbol name. If the item already exists elsewhere in the world, it
/// will be moved rather than duplicated to maintain world consistency.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `symbol` - The item's symbol identifier from the game data files
///
/// # Behavior
///
/// - Converts the symbol to the item's id using the item namespace
/// - If the item exists, moves it to inventory (removing from current location)
/// - Displays success message with item name
/// - If item doesn't exist, displays error message
/// - All actions are logged at `warn` level for audit trail
///
/// # Security
///
/// This function bypasses all normal game restrictions including:
/// - Item portability checks
/// - Container access restrictions
/// - Inventory space limitations
/// - Story progression requirements
///
/// # Panics
/// None -- `item_id` is already known to be valid and exist in symbol table before `expect()` is called
pub fn dev_spawn_item_handler(world: &mut AmbleWorld, view: &mut View, symbol: &str) {
    let item_id = uuid_from_token(&NAMESPACE_ITEM, symbol);
    if world.items.contains_key(&item_id) {
        spawn_item_in_inventory(world, &item_id).expect("should not err; item_id already known to be valid");
        info!("player used DEV_MODE SpawnItem({symbol})");
        view.push(ViewItem::ActionSuccess(format!("Item '{symbol}' moved to inventory.")));
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "No item matching '{}' found in AmbleWorld data.",
            symbol.error_style()
        )));
    }
}

/// Instantly teleports the player to any room by its room id (`DEV_MODE` only).
///
/// This development command bypasses all normal movement restrictions and
/// immediately transports the player to the specified room. This is useful
/// for testing different areas, debugging room connections, or quickly
/// navigating during content development.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing rooms and player
/// * `view` - Mutable reference to the player's view for feedback and room display
/// * `room_id` - The room's string identifier from world data
///
/// # Behavior
///
/// - Converts the room id using the room namespace
/// - If the room exists, immediately updates player location
/// - Displays the new room with full description (as if entering for first time)
/// - If room doesn't exist, displays error message
/// - All teleportation is logged at `warn` level for audit trail
///
/// # Security
///
/// This function bypasses all normal movement restrictions including:
/// - Locked exits and doors
/// - Required items or keys
/// - Story progression flags
/// - Movement-blocking conditions
pub fn dev_teleport_handler(world: &mut AmbleWorld, view: &mut View, room_id: &str) {
    let room_uuid = uuid_from_token(&NAMESPACE_ROOM, room_id);
    if let Some(room) = world.rooms.get(&room_uuid) {
        world.player.location = Location::Room(room_uuid);
        warn!(
            "DEV only command used: Teleported player to {} ({})",
            room.name(),
            room.id()
        );
        view.push(ViewItem::ActionSuccess("You teleported...".to_string()));
        let _ = room.show(world, view, None);
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "Teleport failed: Lookup of '{room_id}' failed."
        )));
    }
}

/// Creates a new sequence flag with optional step limit (`DEV_MODE` only).
///
/// This development command creates a sequence flag that can track multi-step
/// progress through game events. Sequence flags start at step 0 and can be
/// advanced, reset, or checked by trigger conditions.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `seq_name` - Name identifier for the new sequence flag
/// * `end` - Maximum number of steps ("none" for unlimited, or numeric string)
///
/// # Behavior
///
/// - Parses the end parameter: "none" creates unlimited sequence, numbers set limit
/// - Creates new sequence flag with specified name and limit
/// - Adds flag to player's flag collection (replacing any existing flag with same name)
/// - Displays success message with flag details
/// - All flag creation is logged at `warn` level for audit trail
///
/// # Examples
///
/// - `start_seq puzzle_steps 5` - Creates sequence limited to 5 steps
/// - `start_seq story_progress none` - Creates unlimited sequence
pub fn dev_start_seq_handler(world: &mut AmbleWorld, view: &mut View, seq_name: &str, end: &str) {
    let limit = if end.to_lowercase() == "none" {
        None
    } else {
        end.parse::<u8>().ok()
    };
    let seq = Flag::sequence(seq_name, limit, world.turn_count);
    view.push(ViewItem::ActionSuccess(format!(
        "Sequence flag '{}' started with step limit {limit:?}.",
        seq.value()
    )));
    warn!("DEV_MODE command StartSeq used: '{}' set, limit {limit:?}", seq.value());
    trigger::add_flag(world, view, &seq);
}

/// Sets a simple boolean flag on the player (`DEV_MODE` only).
///
/// This development command creates or sets a simple flag that represents
/// a boolean condition or completed state. Simple flags are either present
/// (true) or absent (false) and are commonly used for unlock conditions,
/// story progression markers, or achievement tracking.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `flag_name` - Name identifier for the simple flag to set
///
/// # Behavior
///
/// - Creates a simple (non-sequence) flag with the specified name
/// - Adds flag to player's flag collection (replacing any existing flag with same name)
/// - Displays success message confirming flag was set
/// - All flag setting is logged at `warn` level for audit trail
///
/// # Use Cases
///
/// - Unlocking areas or content that requires specific flags
/// - Testing trigger conditions that depend on flag states
/// - Simulating completed story events or achievements
pub fn dev_set_flag_handler(world: &mut AmbleWorld, view: &mut View, flag_name: &str) {
    let flag = Flag::simple(flag_name, world.turn_count);
    view.push(ViewItem::ActionSuccess(format!("Simple flag '{}' set.", flag.value())));
    warn!("DEV_MODE command SetFlag used: '{}' set.", flag.value());
    trigger::add_flag(world, view, &flag);
}

/// Record a development note to the daily log file (`DEV_MODE` only).
pub fn dev_note_handler(world: &AmbleWorld, view: &mut View, note: &str) {
    let log_dir = world_log_dir(world);
    if let Err(err) = fs::create_dir_all(&log_dir) {
        view.push(ViewItem::ActionFailure(format!(
            "Unable to create logs directory: {err}"
        )));
        warn!("DEV_MODE note failed creating log dir '{}': {err}", log_dir.display());
        return;
    }

    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let log_path = note_log_path(now, world);
    let timestamp = format_note_timestamp(now);
    let note_text = note.trim();
    let note_text = if note_text.is_empty() { "<empty>" } else { note_text };

    let mut line = String::new();
    if let Ok(room) = world.player_room_ref() {
        let _ = write!(
            &mut line,
            "- [ ] {timestamp} turn={} room={} \"{}\" | {note_text}",
            world.turn_count,
            room.symbol(),
            room.name()
        );
    } else {
        let location = describe_note_location(world);
        let _ = write!(
            &mut line,
            "- [ ] {timestamp} turn={} {location} | {note_text}",
            world.turn_count
        );
    }

    let is_new_file = !log_path.exists();
    match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(mut file) => {
            if is_new_file {
                let heading = format_note_heading(now);
                if let Err(err) = writeln!(file, "# Notes for {heading}\n") {
                    view.push(ViewItem::ActionFailure(format!(
                        "Failed to write note header to {}: {err}",
                        log_path.display()
                    )));
                    warn!("DEV_MODE note failed writing header to {}: {err}", log_path.display());
                    return;
                }
            }
            if let Err(err) = writeln!(file, "{line}") {
                view.push(ViewItem::ActionFailure(format!(
                    "Failed to write note to {}: {err}",
                    log_path.display()
                )));
                warn!("DEV_MODE note failed writing to {}: {err}", log_path.display());
                return;
            }
        },
        Err(err) => {
            view.push(ViewItem::ActionFailure(format!(
                "Failed to open note log {}: {err}",
                log_path.display()
            )));
            warn!("DEV_MODE note failed opening {}: {err}", log_path.display());
            return;
        },
    }

    view.push(ViewItem::ActionSuccess("Note recorded.".to_string()));
    warn!("DEV_MODE command used: :note");
}

fn world_log_dir(world: &AmbleWorld) -> PathBuf {
    let slug = if !world.world_slug.trim().is_empty() {
        sanitize_slug(&world.world_slug)
    } else if !world.game_title.trim().is_empty() {
        sanitize_slug(&world.game_title)
    } else {
        "world".to_string()
    };
    PathBuf::from(LOG_DIR).join(slug)
}

fn note_log_path(now: OffsetDateTime, world: &AmbleWorld) -> PathBuf {
    let fmt = format_description::parse("[year]-[month]-[day]").expect("valid note date format");
    let date = now.format(&fmt).unwrap_or_else(|_| "unknown-date".to_string());
    world_log_dir(world).join(format!("notes-{date}.md"))
}

fn format_note_timestamp(now: OffsetDateTime) -> String {
    let fmt = format_description::parse("[hour]:[minute]").expect("valid note timestamp format");
    now.format(&fmt).unwrap_or_else(|_| "00:00".to_string())
}

fn format_note_heading(now: OffsetDateTime) -> String {
    let fmt = format_description::parse("[weekday repr:long], [month repr:long] [day padding:none], [year]")
        .expect("valid note heading format");
    now.format(&fmt).unwrap_or_else(|_| "Unknown Date".to_string())
}

fn describe_note_location(world: &AmbleWorld) -> String {
    match &world.player.location {
        Location::Room(room_id) => world
            .rooms
            .get(room_id)
            .map(|room| format!("room={} \"{}\"", room.symbol(), room.name()))
            .unwrap_or_else(|| format!("room_id={room_id}")),
        _ => String::from("loc=<unknown>"),
    }
}

/// Advances a sequence flag to its next step (`DEV_MODE` only).
///
/// This development command increments a sequence flag by one step, simulating
/// progress through a multi-step process. This is useful for testing triggers
/// that depend on specific sequence flag values or for advancing past points
/// in development.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `seq_name` - Name of the sequence flag to advance
///
/// # Behavior
///
/// - Advances the named sequence flag by one step (if it exists)
/// - Respects the flag's step limit (won't advance beyond maximum)
/// - Displays current step value after advancement
/// - If flag doesn't exist, displays error message with helpful suggestion
/// - All advancement is logged at `warn` level for audit trail
///
/// # Use Cases
///
/// - Testing multi-step puzzle logic
/// - Simulating story progression
/// - Debugging sequence-dependent triggers
/// - Fast-forwarding through lengthy sequences during testing
pub fn dev_advance_seq_handler(world: &mut AmbleWorld, view: &mut View, seq_name: &str) {
    let target = Flag::simple(seq_name, world.turn_count);

    // Check if the flag exists before trying to advance it
    if world.player.flags.contains(&target) {
        world.player.advance_flag(seq_name);
        if let Some(flag) = world.player.flags.get(&target) {
            view.push(ViewItem::ActionSuccess(format!(
                "Sequence '{}' advanced to [{}].",
                flag.name(),
                flag.value()
            )));
            warn!(
                "DEV_MODE AdvanceSeq used: '{}' advanced to [{}].",
                flag.name(),
                flag.value()
            );
        }
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "No sequence flag '{}' found. Use :init-seq to create it first.",
            seq_name.error_style()
        )));
        warn!("DEV_MODE AdvanceSeq failed: sequence flag '{seq_name}' not found");
    }
}

/// Resets a sequence flag back to step 0 (`DEV_MODE` only).
///
/// This development command resets a sequence flag to its initial state,
/// effectively undoing any progress made on that sequence. This is useful
/// for testing the beginning of multi-step processes or resetting puzzle
/// states during development.
///
/// # Parameters
///
/// * `world` - Mutable reference to the game world containing the player
/// * `view` - Mutable reference to the player's view for feedback messages
/// * `seq_name` - Name of the sequence flag to reset
///
/// # Behavior
///
/// - Resets the named sequence flag to step 0 (if it exists)
/// - Preserves the flag's step limit setting
/// - Displays current step value after reset (should be 0)
/// - If flag doesn't exist, displays error message with helpful suggestion
/// - All resets are logged at `warn` level for audit trail
///
/// # Use Cases
///
/// - Testing puzzle logic from the beginning
/// - Resetting story sequences for repeated testing
/// - Debugging sequence initialization
/// - Clearing progress to test different approaches
pub fn dev_reset_seq_handler(world: &mut AmbleWorld, view: &mut View, seq_name: &str) {
    let target = Flag::simple(seq_name, world.turn_count);

    // Check if the flag exists before trying to reset it
    if world.player.flags.contains(&target) {
        world.player.reset_flag(seq_name);
        if let Some(flag) = world.player.flags.get(&target) {
            view.push(ViewItem::ActionSuccess(format!(
                "Sequence '{}' reset to [{}].",
                flag.name(),
                flag.value()
            )));
            warn!("DEV_MODE ResetSeq used: '{}' reset to [{}].", flag.name(), flag.value());
        }
    } else {
        view.push(ViewItem::ActionFailure(format!(
            "No sequence flag '{}' found. Use :init-seq to create it first.",
            seq_name.error_style()
        )));
        warn!("DEV_MODE ResetSeq failed: sequence flag '{seq_name}' not found");
    }
}

/// Lists all NPCs with their current location and state (`DEV_MODE` only).
///
/// Displays a developer-focused summary of all NPCs in the world, including
/// their symbol, name, location, and current state.
pub fn dev_list_npcs_handler(world: &mut AmbleWorld, view: &mut View) {
    let mut npcs: Vec<_> = world
        .npcs
        .values()
        .map(|npc| {
            let loc = match npc.location() {
                Location::Room(uuid) => symbol_or_unknown(&world.rooms, uuid),
                Location::Nowhere => "<nowhere>".to_string(),
                other => format!("<{other:?}>"),
            };
            (
                npc.name().to_string(),
                npc.symbol().to_string(),
                loc,
                npc.state.as_key(),
            )
        })
        .collect();
    npcs.sort_by(|a, b| a.0.cmp(&b.0));

    if npcs.is_empty() {
        view.push(ViewItem::EngineMessage("No NPCs found in world.".to_string()));
    } else {
        let mut msg = String::new();
        let _ = writeln!(msg, "NPCs [{}]:", npcs.len());
        for (name, symbol, loc, state) in npcs {
            let _ = writeln!(msg, " - {name} ({symbol}) @ {loc} [{state}]");
        }
        view.push(ViewItem::EngineMessage(msg));
    }
    warn!("DEV_MODE command used: :npcs (list NPCs)");
}

/// Lists all player flags currently set (`DEV_MODE` only).
/// Simple flags are displayed as their name, sequence flags as name#step.
pub fn dev_list_flags_handler(world: &mut AmbleWorld, view: &mut View) {
    let mut flags: Vec<String> = world
        .player
        .flags
        .iter()
        .map(super::super::player::Flag::value)
        .collect();
    flags.sort();
    if flags.is_empty() {
        view.push(ViewItem::EngineMessage("No flags are currently set.".to_string()));
    } else {
        let mut msg = String::new();
        let _ = writeln!(msg, "Flags [{}]:", flags.len());
        for f in flags {
            let _ = writeln!(msg, " - {f}");
        }
        view.push(ViewItem::EngineMessage(msg));
    }
    warn!("DEV_MODE command used: :flags (list flags)");
}

/// Lists upcoming scheduled events from the world's scheduler (`DEV_MODE` only).
pub fn dev_list_sched_handler(world: &mut AmbleWorld, view: &mut View) {
    let now = world.turn_count;
    let mut upcoming: Vec<(usize, usize)> = world.scheduler.heap.iter().map(|rev| rev.0).collect();
    upcoming.sort_unstable();

    if upcoming.is_empty() {
        view.push(ViewItem::EngineMessage("No events currently scheduled.".to_string()));
        warn!("DEV_MODE command used: :sched (no events)");
        return;
    }

    let mut msg = String::new();
    let _ = writeln!(msg, "Scheduled events [{}], current turn = {now}:", upcoming.len());
    for (turn_due, idx) in upcoming {
        let (note, actions_len, cond_opt, policy) = if let Some(ev) = world.scheduler.events.get(idx) {
            (
                ev.note.clone().unwrap_or_else(|| "<no note>".to_string()),
                ev.actions.len(),
                ev.condition.clone(),
                ev.on_false.clone(),
            )
        } else {
            ("<no note>".to_string(), 0, None, OnFalsePolicy::Cancel)
        };

        // Header line stays compact for grep-ability
        let _ = writeln!(msg, " - turn {turn_due:>4} | idx {idx:>3} | actions: {actions_len}");

        // Metadata lines are separated for readability
        let _ = writeln!(msg, "   on_false: {}", summarize_on_false(&policy));
        if let Some(cond) = &cond_opt {
            let _ = writeln!(msg, "   cond: {}", summarize_event_condition(world, cond));
        } else {
            msg.push_str("   cond: <none>\n");
        }
        let _ = writeln!(msg, "   note: {note}");
        msg.push('\n');
    }
    view.push(ViewItem::EngineMessage(msg));
    warn!("DEV_MODE command used: :sched (list schedule)");
}

/// Summarize an `OnFalsePolicy` to a compact, human-friendly string.
///
/// This keeps scheduler listings tidy while still conveying retry behavior.
fn summarize_on_false(p: &OnFalsePolicy) -> String {
    match p {
        OnFalsePolicy::Cancel => "cancel".to_string(),
        OnFalsePolicy::RetryAfter(n) => format!("retry+{n}"),
        OnFalsePolicy::RetryNextTurn => "retry-next".to_string(),
    }
}

/// Render an `EventCondition` using world symbols for concise logging.
fn summarize_event_condition(world: &AmbleWorld, ec: &EventCondition) -> String {
    match ec {
        EventCondition::Trigger(tc) => summarize_trigger_condition(world, tc),
        EventCondition::All(list) => {
            let parts: Vec<String> = list.iter().map(|c| summarize_event_condition(world, c)).collect();
            format!("all({})", parts.join(", "))
        },
        EventCondition::Any(list) => {
            let parts: Vec<String> = list.iter().map(|c| summarize_event_condition(world, c)).collect();
            format!("any({})", parts.join(", "))
        },
    }
}

/// Render a `TriggerCondition` using world symbols for concise logging.
fn summarize_trigger_condition(world: &AmbleWorld, tc: &TriggerCondition) -> String {
    match tc {
        TriggerCondition::Touch(item) => {
            format!("touch:{}", symbol_or_unknown(&world.items, item))
        },
        TriggerCondition::Ingest { item_id, mode } => {
            format!("ingest:{}:{mode}", symbol_or_unknown(&world.items, item_id))
        },
        TriggerCondition::HasFlag(f) => format!("hasFlag:{f}"),
        TriggerCondition::MissingFlag(f) => format!("missingFlag:{f}"),
        TriggerCondition::FlagInProgress(f) => format!("flagInProgress:{f}"),
        TriggerCondition::FlagComplete(f) => format!("flagComplete:{f}"),
        TriggerCondition::HasItem(item) => format!("hasItem:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::MissingItem(item) => format!("missingItem:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::InRoom(room) => format!("inRoom:{}", symbol_or_unknown(&world.rooms, room)),
        TriggerCondition::Enter(room) => format!("enter:{}", symbol_or_unknown(&world.rooms, room)),
        TriggerCondition::Leave(room) => format!("leave:{}", symbol_or_unknown(&world.rooms, room)),
        TriggerCondition::HasVisited(room) => format!("visited:{}", symbol_or_unknown(&world.rooms, room)),
        TriggerCondition::WithNpc(npc) => format!("withNpc:{}", symbol_or_unknown(&world.npcs, npc)),
        TriggerCondition::NpcInState { npc_id, mood } => format!(
            "npcState:{}:{}",
            symbol_or_unknown(&world.npcs, npc_id),
            format!("{mood:?}").to_lowercase()
        ),
        TriggerCondition::NpcHasItem { npc_id, item_id } => format!(
            "npcHasItem:{}:{}",
            symbol_or_unknown(&world.npcs, npc_id),
            symbol_or_unknown(&world.items, item_id)
        ),
        TriggerCondition::GiveToNpc { item_id, npc_id } => format!(
            "giveToNpc:{}->{}",
            symbol_or_unknown(&world.items, item_id),
            symbol_or_unknown(&world.npcs, npc_id)
        ),
        TriggerCondition::LookAt(item) => format!("lookAt:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::Open(item) => format!("open:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::Unlock(item) => format!("unlock:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::Drop(item) => format!("drop:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::Take(item) => format!("take:{}", symbol_or_unknown(&world.items, item)),
        TriggerCondition::TakeFromNpc { item_id, npc_id } => format!(
            "takeFromNpc:{}<-{}",
            symbol_or_unknown(&world.items, item_id),
            symbol_or_unknown(&world.npcs, npc_id)
        ),
        TriggerCondition::Insert { item, container } => format!(
            "putIn:{}->{}",
            symbol_or_unknown(&world.items, item),
            symbol_or_unknown(&world.items, container)
        ),
        TriggerCondition::ContainerHasItem { container_id, item_id } => format!(
            "containerHas:{}:{}",
            symbol_or_unknown(&world.items, container_id),
            symbol_or_unknown(&world.items, item_id)
        ),
        TriggerCondition::UseItem { item_id, ability } => format!(
            "useItem:{}:{}",
            symbol_or_unknown(&world.items, item_id),
            format!("{ability:?}").to_lowercase()
        ),
        TriggerCondition::UseItemOnItem {
            interaction,
            target_id,
            tool_id,
        } => format!(
            "useItemOn:{}->{}:{}",
            symbol_or_unknown(&world.items, tool_id),
            symbol_or_unknown(&world.items, target_id),
            format!("{interaction:?}").to_lowercase()
        ),
        TriggerCondition::PlayerDeath => "playerDeath".to_string(),
        TriggerCondition::NpcDeath(npc_id) => format!("npcDeath:{}", symbol_or_unknown(&world.npcs, npc_id)),
        TriggerCondition::TalkToNpc(npc) => format!("talkToNpc:{}", symbol_or_unknown(&world.npcs, npc)),
        TriggerCondition::ActOnItem { target_id, action } => format!(
            "actOnItem:{}:{}",
            symbol_or_unknown(&world.items, target_id),
            format!("{action:?}").to_lowercase()
        ),
        TriggerCondition::Ambient { spinner, .. } => format!("ambient:{}", spinner.as_toml_key()),
        TriggerCondition::Chance { one_in } => format!("chance:1-in-{one_in:.0}"),
    }
}

/// Cancel a scheduled event by its internal index (`DEV_MODE` only).
pub fn dev_sched_cancel_handler(world: &mut AmbleWorld, view: &mut View, idx: usize) {
    if idx >= world.scheduler.events.len() {
        view.push(ViewItem::ActionFailure(format!(
            "No scheduled event found at index {idx}."
        )));
        return;
    }
    let ev = &mut world.scheduler.events[idx];
    let was_placeholder = ev.on_turn == 0 && ev.actions.is_empty() && ev.note.is_none() && ev.condition.is_none();
    if was_placeholder {
        let msg = format!("Event at index {idx} is already cleared/canceled.");
        view.push(ViewItem::ActionFailure(msg));
        return;
    }
    let note = ev.note.clone().unwrap_or_else(|| "<no note>".to_string());
    *ev = ScheduledEvent::default();
    warn!("DEV_MODE: canceled scheduled event idx {idx} (note: {note})");
    view.push(ViewItem::ActionSuccess(format!("Scheduled event {idx} canceled.")));
}

/// Delay a scheduled event by N turns (`DEV_MODE` only).
pub fn dev_sched_delay_handler(world: &mut AmbleWorld, view: &mut View, idx: usize, turns: usize) {
    if idx >= world.scheduler.events.len() {
        let msg = format!("No scheduled event found at index {idx}.");
        view.push(ViewItem::ActionFailure(msg));
        return;
    }
    let ev = world.scheduler.events.get(idx).cloned().unwrap_or_default();
    if ev.on_turn == 0 && ev.actions.is_empty() && ev.note.is_none() && ev.condition.is_none() {
        let msg = format!("Event at index {idx} is empty/cleared.");
        view.push(ViewItem::ActionFailure(msg));
        return;
    }
    let new_turn = ev.on_turn.saturating_add(turns);
    let note = ev.note.clone();
    world
        .scheduler
        .schedule_on_if(new_turn, ev.condition, ev.on_false, ev.actions, note.clone());
    // clear the original (leave its heap entries as harmless placeholders when due)
    if let Some(slot) = world.scheduler.events.get_mut(idx) {
        *slot = ScheduledEvent::default();
    }
    warn!("DEV_MODE: delayed scheduled event idx {idx} by {turns} turns (new turn {new_turn})");
    view.push(ViewItem::ActionSuccess(format!(
        "Scheduled event {idx} delayed by {turns} turn(s) (now on {new_turn})."
    )));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AmbleWorld, View, ViewItem,
        health::HealthState,
        player::{Flag, Player},
    };

    fn create_test_world() -> AmbleWorld {
        let mut world = AmbleWorld::new_empty();
        world.player = Player::default();
        // Disable colored output for consistent test assertions
        colored::control::set_override(false);
        world
    }

    #[test]
    fn dev_advance_seq_handler_shows_error_for_nonexistent_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        dev_advance_seq_handler(&mut world, &mut view, "nonexistent_flag");

        // Should have an error message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionFailure(msg) = &view.items[0].view_item {
            // Strip ANSI color codes for comparison
            let clean_msg = msg
                .replace("\u{1b}[38;2;230;30;30m", "")
                .replace("\u{1b}[31m", "")
                .replace("\u{1b}[0m", "");
            assert!(clean_msg.contains("No sequence flag 'nonexistent_flag' found"));
            assert!(clean_msg.contains("Use :init-seq to create it first"));
        } else {
            panic!("Expected ActionFailure, got {:?}", view.items[0]);
        }
    }

    #[test]
    fn dev_advance_seq_handler_works_for_existing_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        // First create a sequence flag
        let flag = Flag::sequence("test_seq", Some(3), world.turn_count);
        world.player.flags.insert(flag);

        dev_advance_seq_handler(&mut world, &mut view, "test_seq");

        // Should have a success message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionSuccess(msg) = &view.items[0].view_item {
            assert!(msg.contains("Sequence 'test_seq' advanced to [test_seq#1]"));
        } else {
            panic!("Expected ActionSuccess, got {:?}", view.items[0]);
        }
    }

    #[test]
    fn dev_reset_seq_handler_shows_error_for_nonexistent_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        dev_reset_seq_handler(&mut world, &mut view, "nonexistent_flag");

        // Should have an error message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionFailure(msg) = &view.items[0].view_item {
            // Strip ANSI color codes for comparison
            let clean_msg = msg
                .replace("\u{1b}[38;2;230;30;30m", "")
                .replace("\u{1b}[31m", "")
                .replace("\u{1b}[0m", "");
            assert!(clean_msg.contains("No sequence flag 'nonexistent_flag' found"));
            assert!(clean_msg.contains("Use :init-seq to create it first"));
        } else {
            panic!("Expected ActionFailure, got {:?}", view.items[0].view_item);
        }
    }

    #[test]
    fn dev_reset_seq_handler_works_for_existing_flag() {
        let mut world = create_test_world();
        let mut view = View::new();

        // First create and advance a sequence flag
        let mut flag = Flag::sequence("test_seq", Some(3), world.turn_count);
        if let Flag::Sequence { step, .. } = &mut flag {
            *step = 2; // Set to step 2
        }
        world.player.flags.insert(flag);

        dev_reset_seq_handler(&mut world, &mut view, "test_seq");

        // Should have a success message
        assert_eq!(view.items.len(), 1);
        if let ViewItem::ActionSuccess(msg) = &view.items[0].view_item {
            assert!(msg.contains("Sequence 'test_seq' reset to [test_seq#0]"));
        } else {
            panic!("Expected ActionSuccess, got {:?}", view.items[0]);
        }
    }

    #[test]
    fn dev_list_flags_handler_empty_and_nonempty() {
        let mut world = create_test_world();
        let mut view = View::new();
        // empty
        dev_list_flags_handler(&mut world, &mut view);
        assert!(
            view.items
                .iter()
                .any(|i| matches!(&i.view_item, ViewItem::EngineMessage(msg) if msg.contains("No flags")))
        );

        // add flags and check listing
        view.items.clear();
        world.player.flags.insert(Flag::simple("alpha", world.turn_count));
        let mut seq = Flag::sequence("beta", Some(3), world.turn_count);
        if let Flag::Sequence { step, .. } = &mut seq {
            *step = 2;
        }
        world.player.flags.insert(seq);
        dev_list_flags_handler(&mut world, &mut view);
        let combined = view
            .items
            .iter()
            .filter_map(|i| match &i.view_item {
                ViewItem::EngineMessage(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(combined.contains("Flags [2]"));
        assert!(combined.contains("alpha"));
        assert!(combined.contains("beta#2"));
    }

    #[test]
    fn dev_list_npcs_handler_outputs_expected_format() {
        use crate::npc::Npc;
        use crate::room::Room;
        use std::collections::{HashMap, HashSet};

        let mut world = AmbleWorld::new_empty();
        let mut view = View::new();
        // room
        let room_id = crate::idgen::new_id();
        world.rooms.insert(
            room_id.clone(),
            Room {
                id: room_id.clone(),
                symbol: "test_room".into(),
                name: "Test Room".into(),
                base_description: String::new(),
                overlays: vec![],
                scenery: Vec::new(),
                scenery_default: None,
                location: Location::Nowhere,
                visited: false,
                exits: HashMap::new(),
                contents: HashSet::new(),
                npcs: HashSet::new(),
            },
        );
        // npc
        let npc_id = crate::idgen::new_id();
        let npc = Npc {
            id: npc_id.clone(),
            symbol: "npc_sym".into(),
            name: "Zed".into(),
            description: String::new(),
            location: Location::Room(room_id.clone()),
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state: crate::npc::NpcState::Normal,
            movement: None,
            health: HealthState::new_at_max(10),
        };
        world.npcs.insert(npc_id, npc);

        dev_list_npcs_handler(&mut world, &mut view);
        let combined = view
            .items
            .iter()
            .filter_map(|i| match &i.view_item {
                ViewItem::EngineMessage(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(combined.contains("NPCs [1]"));
        assert!(combined.contains("Zed (npc_sym) @ test_room [normal]"));
    }

    #[test]
    fn dev_list_sched_handler_shows_entries() {
        use crate::trigger::{ScriptedAction, TriggerAction};
        let mut world = create_test_world();
        let mut view = View::new();
        // empty
        dev_list_sched_handler(&mut world, &mut view);
        assert!(
            view.items
                .iter()
                .any(|i| matches!(&i.view_item, ViewItem::EngineMessage(msg) if msg.contains("No events")))
        );

        // add an event
        view.items.clear();
        world.scheduler.schedule_on(
            5,
            vec![ScriptedAction::new(TriggerAction::ShowMessage("m".into()))],
            Some("test".into()),
        );
        dev_list_sched_handler(&mut world, &mut view);
        let combined = view
            .items
            .iter()
            .filter_map(|i| match &i.view_item {
                ViewItem::EngineMessage(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(combined.contains("Scheduled events [1]"));
        assert!(combined.contains("turn    5"));
        assert!(combined.contains("test"));
    }
}
