//! System utility command handlers for the Amble game engine.
//!
//! This module provides handlers for meta-game commands that control the game
//! system itself rather than affecting the game world directly. These commands
//! manage game state persistence, user interface settings, help systems, and
//! game termination.
//!
//! # Command Categories
//!
//! ## Game State Management
//! - [`save_handler`] - Serialize and save current game state to disk
//! - [`load_handler`] - Load previously saved game state from disk
//! - [`quit_handler`] - Terminate game session with scoring summary
//!
//! ## User Interface Control
//! - [`set_viewmode_handler`] - Change display verbosity and screen clearing behavior
//!
//! ## Information Systems
//! - [`help_handler`] - Display game help text and command reference
//! - [`goals_handler`] - Show current objectives and completion status
//!
//! # Save System
//!
//! The save system uses RON (Rusty Object Notation) format for human-readable
//! and debuggable save files. Save files include version information to handle
//! compatibility across game updates.
//!
//! ## Save File Format
//! - **Location**: `saved_games/` directory
//! - **Naming**: `{slot_name}-amble-{version}.ron`
//! - **Content**: Complete serialized `AmbleWorld` state
//!
//! ## Version Compatibility
//! - Save files include version metadata
//! - Loading mismatched versions shows warnings but attempts to proceed
//! - Version conflicts are logged for debugging
//!
//! # View Modes
//!
//! The system supports multiple display modes:
//! - **Brief**: Minimal descriptions for visited locations
//! - **Verbose**: Full descriptions always shown
//! - **Clear Verbose**: Full descriptions with screen clearing on movement
//!
//! # Goal Tracking
//!
//! Goals are dynamic objectives that can be:
//! - **Active**: Currently available for completion
//! - **Complete**: Successfully achieved by player actions
//! - **Conditional**: Dependent on game state for availability
//!
//! # Scoring System
//!
//! The quit handler provides comprehensive scoring analysis:
//! - Point-based scoring with maximum possible calculation
//! - Exploration tracking (rooms visited vs. total rooms)
//! - Performance ranking with humorous titles
//! - Detailed game statistics and achievements

use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::goal::GoalStatus;
use crate::helpers::symbol_or_unknown;
use crate::loader::help::{HelpCommand, load_help_data};
use crate::save_files::{self, SAVE_DIR};
use crate::style::GameStyle;
use crate::theme::THEME_MANAGER;

use crate::view::ViewMode;
use crate::{AMBLE_VERSION, Goal, View, ViewItem};
use crate::{AmbleWorld, WorldObject, repl::ReplControl};

use anyhow::{Context, Result};
use log::{info, warn};
use std::path::Path;

/// Changes the display verbosity and screen clearing behavior.
///
/// This handler allows players to customize how room descriptions and other
/// game text are displayed, balancing information density with screen clarity.
/// Different modes suit different play styles and preferences.
///
/// # Parameters
///
/// * `view` - Mutable reference to the player's view for mode changes and feedback
/// * `mode` - The new view mode to activate
///
/// # Available Modes
///
/// - `ClearVerbose`: Clears screen on movement, always shows full descriptions
/// - `Verbose`: Always shows full room descriptions without screen clearing
/// - `Brief`: Shows full descriptions only on first visit and when explicitly looking
///
/// # Behavior
///
/// - Updates the view's display mode immediately
/// - Provides confirmation message explaining the new mode
/// - Mode changes are logged for debugging purposes
/// - Setting persists for the current game session
pub fn set_viewmode_handler(view: &mut View, mode: ViewMode) {
    view.set_mode(mode);
    let msg = match mode {
        ViewMode::ClearVerbose => format!(
            "{} mode set. {}",
            "Clear".highlight(),
            "Screen will be cleared with any movement and full location descriptions will always be shown.".italic()
        ),
        ViewMode::Verbose => format!(
            "{} mode set. {}",
            "Verbose".highlight(),
            "Full location descriptions will always be shown.".italic()
        ),
        ViewMode::Brief => format!(
            "{} mode set. {}",
            "Brief".highlight(),
            "Full location descriptions will only be shown on first visit and with the 'look' command.".italic()
        ),
    };
    view.push(ViewItem::EngineMessage(msg));
    info!("Player changed view mode to {mode:?}");
}

/// Terminates the game session and displays comprehensive scoring summary.
///
/// This handler provides a complete game ending experience, including detailed
/// statistics, performance evaluation, and humorous ranking based on the player's
/// achievements during the session.
///
/// # Parameters
///
/// * `world` - Reference to the game world for final state analysis
/// * `view` - Mutable reference to display the quit summary and statistics
///
/// # Returns
///
/// Returns `ReplControl::Quit` to signal the game loop should terminate.
///
/// # Scoring Analysis
///
/// The function calculates and displays:
/// - **Final score** vs. maximum possible points
/// - **Completion percentage** with performance ranking
/// - **Exploration statistics** (rooms visited vs. available)
/// - **Humorous rank titles** based on achievement level
///
/// # Logging Output
///
/// Comprehensive session data is logged including:
/// - Final score and player statistics
/// - All active flags at game end
/// - Complete inventory listing with item symbols
/// - Session completion metrics
///
/// # Performance Rankings
///
/// Players receive rank titles based on their completion percentage, with
/// rankings defined in `scoring.toml` (or hardcoded defaults if the file
/// cannot be loaded). Each rank has a threshold percentage, a title, and
/// a personalized evaluation message reflecting the player's exploration
/// and puzzle-solving success.
///
/// # Errors
/// This handler never produces an error; it always returns `Ok(ReplControl::Quit)`.
pub fn quit_handler(world: &AmbleWorld, view: &mut View) -> Result<ReplControl> {
    info!("$$ {} quit with a score of {}", world.player.name(), world.player.score);
    let path: Vec<_> = world
        .player_path
        .iter()
        .map(|uuid| symbol_or_unknown(&world.rooms, *uuid))
        .collect();
    let path_str = path.join(" > ");
    info!("$$ PATH ({} moves): {path_str}", path.len().saturating_sub(1));
    info!("$$ FLAGS:");
    world.player.flags.iter().for_each(|i| info!("$$ -> {i}"));
    info!("$$ INVENTORY:");
    world
        .player
        .inventory
        .iter()
        .filter_map(|uuid| world.items.get(uuid))
        .for_each(|i| info!("$$ -> {} ({})", i.name(), i.symbol()));

    #[allow(clippy::cast_precision_loss)]
    let percent = (world.player.score as f32 / world.max_score as f32) * 100.0;

    let (rank, eval) = world.scoring.get_rank(percent);

    let visited = world.rooms.values().filter(|r| r.visited).count();

    view.push(ViewItem::QuitSummary {
        title: world.scoring.report_title.clone(),
        rank: rank.to_string(),
        notes: eval.to_string(),
        score: world.player.score,
        max_score: world.max_score,
        visited,
        max_visited: world.rooms.len(),
    });

    Ok(ReplControl::Quit)
}

/// Displays comprehensive help information including basic instructions and command reference.
///
/// This handler loads and presents help content from external data files, providing
/// players with essential game information and command documentation. The help system
/// is designed to be easily updatable without code changes.
///
/// # Parameters
///
/// * `view` - Mutable reference to display help content to the player
///
/// # Help Content Sources
///
/// - **Basic Text**: `amble_engine/data/help_basic.txt` - General game instructions
/// - **Commands**: `amble_engine/data/help_commands.toml` - Command reference with examples
///
/// # Command Documentation Format
///
/// Commands are documented in TOML format with structure:
/// ```toml
/// [[commands]]
/// command = "drop <object>"
/// description = "Remove an item from inventory and place in current room"
/// ```
///
/// # Error Handling
///
/// If help files cannot be loaded:
/// - Error message displayed to player with styling
/// - Warning logged for debugging
/// - Game continues normally (help failure is non-fatal)
///
/// # Content Organization
///
/// Help is presented in two sections:
/// 1. **Basic Instructions** - Game concepts, objectives, basic interaction
/// 2. **Command Reference** - Detailed list of available commands with syntax
pub fn help_handler(view: &mut View) {
    // Support running from workspace root (engine binary) and from crate dir (unit tests)
    let basic_text_path = if Path::new("amble_engine/data/help_basic.txt").exists() {
        Path::new("amble_engine/data/help_basic.txt")
    } else {
        Path::new("data/help_basic.txt")
    };
    let commands_toml_path = if Path::new("amble_engine/data/help_commands.toml").exists() {
        Path::new("amble_engine/data/help_commands.toml")
    } else {
        Path::new("data/help_commands.toml")
    };

    match load_help_data(basic_text_path, commands_toml_path) {
        Ok(mut help_data) => {
            // Append developer-only commands if DEV_MODE is enabled
            if crate::DEV_MODE {
                let mut dev_cmds: Vec<HelpCommand> = vec![
                    HelpCommand {
                        command: ":npcs".into(),
                        description: "DEV: List all NPCs with location and state.".into(),
                    },
                    HelpCommand {
                        command: ":flags".into(),
                        description: "DEV: List all currently set flags (sequences as name#step).".into(),
                    },
                    HelpCommand {
                        command: ":sched".into(),
                        description: "DEV: List upcoming scheduled events with due turn and notes.".into(),
                    },
                    HelpCommand {
                        command: ":teleport <room_symbol>".into(),
                        description: "DEV: Instantly move to a room (alias :port).".into(),
                    },
                    HelpCommand {
                        command: ":spawn <item_symbol>".into(),
                        description: "DEV: Spawn/move an item into inventory (alias :item).".into(),
                    },
                    HelpCommand {
                        command: ":set-flag <name>".into(),
                        description: "DEV: Create a simple flag on the player.".into(),
                    },
                    HelpCommand {
                        command: ":init-seq <name> <end|none>".into(),
                        description: "DEV: Create a sequence flag with limit or unlimited (none).".into(),
                    },
                    HelpCommand {
                        command: ":adv-seq <name>".into(),
                        description: "DEV: Advance a sequence flag by one step.".into(),
                    },
                    HelpCommand {
                        command: ":reset-seq <name>".into(),
                        description: "DEV: Reset a sequence flag to step 0.".into(),
                    },
                ];
                // Keep the main list readable: append DEV commands at the end
                help_data.commands.append(&mut dev_cmds);
            }
            view.push(ViewItem::Help {
                basic_text: help_data.basic_text,
                commands: help_data.commands,
            });
        },
        Err(e) => {
            view.push(ViewItem::Error(format!(
                "Failed to load help data: {}",
                e.to_string().error_style()
            )));
            warn!("Failed to load help data: {e}");
        },
    }
}

/// Show only developer commands in help (DEV_MODE only).
/// Falls back to a standard disabled message when not in DEV_MODE.
pub fn help_handler_dev(view: &mut View) {
    if !crate::DEV_MODE {
        view.push(ViewItem::Error(
            "Developer commands are disabled in this build."
                .error_style()
                .to_string(),
        ));
        warn!("player attempted to use developer help with DEV_MODE = false");
        return;
    }

    // Load the same help data
    let basic_text_path = if Path::new("amble_engine/data/help_basic.txt").exists() {
        Path::new("amble_engine/data/help_basic.txt")
    } else {
        Path::new("data/help_basic.txt")
    };
    let commands_toml_path = if Path::new("amble_engine/data/help_commands.toml").exists() {
        Path::new("amble_engine/data/help_commands.toml")
    } else {
        Path::new("data/help_commands.toml")
    };

    match load_help_data(basic_text_path, commands_toml_path) {
        Ok(mut help_data) => {
            // Replace commands with only DEV commands
            let mut dev_cmds: Vec<HelpCommand> = vec![
                HelpCommand {
                    command: ":npcs".into(),
                    description: "DEV: List all NPCs with location and state.".into(),
                },
                HelpCommand {
                    command: ":flags".into(),
                    description: "DEV: List all currently set flags (sequences as name#step).".into(),
                },
                HelpCommand {
                    command: ":sched".into(),
                    description: "DEV: List upcoming scheduled events with due turn and notes.".into(),
                },
                HelpCommand {
                    command: ":teleport <room_symbol>".into(),
                    description: "DEV: Instantly move to a room (alias :port).".into(),
                },
                HelpCommand {
                    command: ":spawn <item_symbol>".into(),
                    description: "DEV: Spawn/move an item into inventory (alias :item).".into(),
                },
                HelpCommand {
                    command: ":set-flag <name>".into(),
                    description: "DEV: Create a simple flag on the player.".into(),
                },
                HelpCommand {
                    command: ":init-seq <name> <end|none>".into(),
                    description: "DEV: Create a sequence flag with limit or unlimited (none).".into(),
                },
                HelpCommand {
                    command: ":adv-seq <name>".into(),
                    description: "DEV: Advance a sequence flag by one step.".into(),
                },
                HelpCommand {
                    command: ":reset-seq <name>".into(),
                    description: "DEV: Reset a sequence flag to step 0.".into(),
                },
            ];
            // Only dev commands
            help_data.commands.clear();
            help_data.commands.append(&mut dev_cmds);
            view.push(ViewItem::Help {
                basic_text: help_data.basic_text,
                commands: help_data.commands,
            });
        },
        Err(e) => {
            view.push(ViewItem::Error(format!(
                "Failed to load help data: {}",
                e.to_string().error_style()
            )));
            warn!("Failed to load help data: {e}");
        },
    }
}

/// Displays current game objectives and their completion status.
///
/// This handler presents the player with their current goals, showing both
/// active objectives they can work toward and completed achievements they've
/// already accomplished during their session.
///
/// # Parameters
///
/// * `world` - Reference to the game world for goal evaluation
/// * `view` - Mutable reference to display goal information to the player
///
/// # Goal Categories
///
/// Goals are displayed in two sections:
/// - **Active Goals**: Currently available objectives the player can pursue
/// - **Complete Goals**: Objectives the player has already achieved
///
/// # Goal Status Evaluation
///
/// Goal status is dynamically evaluated based on current world state:
/// - Goals may become active when certain conditions are met
/// - Goals are marked complete when their success conditions are satisfied
/// - Goal availability can change as the player progresses through the game
///
/// # Display Format
///
/// Each goal is presented with:
/// - **Name**: Brief identifier for the objective
/// - **Description**: Detailed explanation of what needs to be accomplished
/// - **Status indication**: Visual distinction between active and completed goals
///
/// # Usage
///
/// Players can use this command to:
/// - Check what objectives are currently available
/// - Review what they've already accomplished
/// - Get reminders about active goals when stuck or planning next actions
pub fn goals_handler(world: &AmbleWorld, view: &mut View) {
    filtered_goals(world, GoalStatus::Active)
        .iter()
        .map(|goal| ViewItem::ActiveGoal {
            name: goal.name.clone(),
            description: goal.description.clone(),
        })
        .for_each(|goal_item| view.push(goal_item));

    filtered_goals(world, GoalStatus::Complete)
        .iter()
        .map(|goal| ViewItem::CompleteGoal {
            name: goal.name.clone(),
            description: goal.description.clone(),
        })
        .for_each(|goal_item| view.push(goal_item));

    info!("{} checked goals status.", world.player.name());
}

/// Lists available save slots and their details.
pub fn list_saves_handler(view: &mut View) {
    match save_files::build_save_entries(Path::new(SAVE_DIR)) {
        Ok(entries) => view.push(ViewItem::SavedGamesList {
            directory: SAVE_DIR.to_string(),
            entries,
        }),
        Err(err) => view.push(ViewItem::Error(format!("Unable to list saved games: {}", err))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::scoring::{ScoringConfig, ScoringRank};

    #[test]
    fn help_includes_dev_commands_when_enabled() {
        if !crate::DEV_MODE {
            // In non-dev builds, this test is not applicable
            return;
        }
        let mut view = View::new();
        help_handler(&mut view);
        // Find Help ViewItem and ensure at least one DEV command is present
        if let Some(ViewItem::Help { commands, .. }) = view.items.iter().find(|i| matches!(i, ViewItem::Help { .. })) {
            let cmds: Vec<&str> = commands.iter().map(|c| c.command.as_str()).collect();
            assert!(cmds.iter().any(|c| *c == ":npcs"));
            assert!(cmds.iter().any(|c| *c == ":flags"));
            assert!(cmds.iter().any(|c| *c == ":sched"));
        } else {
            panic!("Help ViewItem not produced by help_handler");
        }
    }

    #[test]
    fn help_dev_shows_only_dev_commands_when_enabled() {
        if !crate::DEV_MODE {
            return;
        }
        let mut view = View::new();
        help_handler_dev(&mut view);
        if let Some(ViewItem::Help { commands, .. }) = view.items.iter().find(|i| matches!(i, ViewItem::Help { .. })) {
            assert!(!commands.is_empty());
            assert!(commands.iter().all(|c| c.command.starts_with(':')));
        } else {
            panic!(":help dev did not produce a Help ViewItem");
        }
    }

    #[test]
    fn quit_handler_uses_scoring_config() {
        // Disable colored output for consistent test assertions
        colored::control::set_override(false);

        // Create a custom scoring config with simple thresholds
        let custom_scoring = ScoringConfig {
            ranks: vec![
                ScoringRank {
                    threshold: 75.0,
                    name: "Test Master".to_string(),
                    description: "You aced the test.".to_string(),
                },
                ScoringRank {
                    threshold: 50.0,
                    name: "Test Novice".to_string(),
                    description: "You passed.".to_string(),
                },
                ScoringRank {
                    threshold: 0.0,
                    name: "Test Failure".to_string(),
                    description: "Better luck next time.".to_string(),
                },
            ],
            report_title: "Test Scorecard".into(),
        };

        // Create test world with custom scoring
        let mut world = AmbleWorld::new_empty();
        world.scoring = custom_scoring;
        world.player.score = 60;
        world.max_score = 100;

        // Call quit handler
        let mut view = View::new();
        let result = quit_handler(&world, &mut view);

        // Verify it returns Quit control
        assert!(matches!(result, Ok(ReplControl::Quit)));

        // Verify the QuitSummary uses the custom scoring config
        if let Some(ViewItem::QuitSummary {
            rank,
            notes,
            score,
            max_score,
            ..
        }) = view.items.iter().find(|i| matches!(i, ViewItem::QuitSummary { .. }))
        {
            // 60/100 = 60%, should get "Test Novice" rank
            assert_eq!(rank, "Test Novice");
            assert_eq!(notes, "You passed.");
            assert_eq!(*score, 60);
            assert_eq!(*max_score, 100);
        } else {
            panic!("quit_handler did not produce a QuitSummary ViewItem");
        }
    }

    #[test]
    fn quit_handler_uses_default_scoring_when_not_set() {
        colored::control::set_override(false);

        let mut world = AmbleWorld::new_empty();
        world.player.score = 100;
        world.max_score = 100;

        let mut view = View::new();
        let result = quit_handler(&world, &mut view);

        assert!(matches!(result, Ok(ReplControl::Quit)));

        // Verify it uses default ranks (100% = "Quantum Overachiever")
        if let Some(ViewItem::QuitSummary { rank, .. }) =
            view.items.iter().find(|i| matches!(i, ViewItem::QuitSummary { .. }))
        {
            assert_eq!(rank, "Quantum Overachiever");
        } else {
            panic!("quit_handler did not produce a QuitSummary ViewItem");
        }
    }
}

/// Filters the world's goal collection by completion status.
///
/// This utility function extracts goals from the world that match a specific
/// status, enabling the display of active versus completed objectives.
///
/// # Parameters
///
/// * `world` - Reference to the game world containing the goal collection
/// * `status` - The goal status to filter for (Active or Complete)
///
/// # Returns
///
/// Returns a vector of goal references that match the specified status.
///
/// # Behavior
///
/// - Evaluates each goal's status against current world state
/// - Returns only goals matching the requested status
/// - Goal status is computed dynamically, not stored statically
/// - Enables real-time goal status updates as game state changes
pub fn filtered_goals(world: &AmbleWorld, status: GoalStatus) -> Vec<&Goal> {
    world.goals.iter().filter(|goal| goal.status(world) == status).collect()
}

/// Loads a previously saved game state from disk.
///
/// This handler attempts to restore a complete game session from a save file,
/// including all world state, player progress, and game configuration. The
/// system handles version compatibility and provides appropriate feedback
/// for various failure conditions.
///
/// # Parameters
///
/// * `world` - Mutable reference to replace with loaded game state
/// * `view` - Mutable reference to display load results and feedback
/// * `gamefile` - Name of the save slot to load (without path or extension)
///
/// # Save File Location
///
/// Files are loaded from: `saved_games/{gamefile}-amble-{version}.ron`
///
/// # Version Compatibility
///
/// - Save files include version metadata for compatibility checking
/// - Mismatched versions generate warnings but attempt to load anyway
/// - Version conflicts are logged for debugging purposes
/// - Players are informed of version mismatches with clear messaging
///
/// # Error Conditions
///
/// - **File not found**: Save slot doesn't exist or is inaccessible
/// - **Parse error**: Save file is corrupted or from incompatible version
/// - **Version mismatch**: Save file version differs from current game version
///
/// # Success Behavior
///
/// On successful load:
/// - Complete world state is replaced with loaded data
/// - Success message displayed with save file information
/// - Load event is logged with file path details
/// - Game continues from the loaded state
///
/// # Failure Behavior
///
/// On failure:
/// - Appropriate error message displayed to player
/// - Original world state remains unchanged
/// - Error details logged for debugging
/// - Game continues with current state
pub fn load_handler(world: &mut AmbleWorld, view: &mut View, gamefile: &str) {
    let save_dir = Path::new(SAVE_DIR);
    let mut slots = match save_files::collect_save_slots(save_dir) {
        Ok(slots) => slots,
        Err(err) => {
            view.push(ViewItem::Error(format!("Unable to inspect saved games: {err}")));
            return;
        },
    };

    slots.retain(|slot| slot.slot == gamefile);
    slots.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.version.cmp(&b.version)));

    let chosen_slot = slots
        .iter()
        .find(|slot| slot.version == AMBLE_VERSION)
        .cloned()
        .or_else(|| slots.first().cloned());

    let (load_path, loaded_version_hint) = if let Some(slot) = chosen_slot {
        (slot.path.clone(), Some(slot.version))
    } else {
        (save_dir.join(format!("{gamefile}-amble-{AMBLE_VERSION}.ron")), None)
    };

    match fs::read_to_string(load_path.as_path()) {
        Ok(world_ron) => match ron::from_str::<AmbleWorld>(&world_ron) {
            Ok(new_world) => {
                if new_world.version != AMBLE_VERSION {
                    warn!(
                        "player loaded '{gamefile}' (v{}), current version is v{AMBLE_VERSION}",
                        new_world.version
                    );
                    view.push(ViewItem::Error(format!(
                        "{}: '{gamefile}' version is v{} -- does not match current game (v{AMBLE_VERSION}).",
                        "WARNING".bold().yellow(),
                        new_world.version.error_style(),
                    )));
                } else if let Some(original_version) = loaded_version_hint.filter(|version| version != AMBLE_VERSION) {
                    info!(
                        "Loaded '{gamefile}' saved under v{original_version}, metadata indicates current version match."
                    );
                }
                *world = new_world;
                view.push(ViewItem::ActionSuccess(format!(
                    "Saved game {} (v{}) loaded successfully. Sally forth.",
                    gamefile.underline().green(),
                    world.version.highlight()
                )));
                view.push(ViewItem::GameLoaded {
                    save_slot: gamefile.to_string(),
                    save_file: load_path.to_string_lossy().to_string(),
                });
                info!(
                    "Player reloaded AmbleWorld from file '{}' (version {})",
                    load_path.display(),
                    world.version
                );
            },
            Err(err) => {
                view.push(ViewItem::ActionFailure(format!(
                    "Unable to load the {} save file. The Amble engine may have changed since it was created ({}).",
                    gamefile.error_style(),
                    err
                )));
                warn!(
                    "player attempted to load '{gamefile}' from '{}': parse failure ({err})",
                    load_path.display()
                );
            },
        },
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                view.push(ViewItem::Error(format!(
                    "Unable to find {} save file. Load aborted. Type {} to list available saves.",
                    gamefile.error_style(),
                    "`saves`".highlight()
                )));
            } else {
                view.push(ViewItem::ActionFailure(format!(
                    "Unable to load the {} save file ({}).",
                    gamefile.error_style(),
                    err
                )));
            }
            warn!(
                "player attempted to load '{gamefile}' from '{}': {}",
                load_path.display(),
                err
            );
        },
    }
}

/// Saves the current game state to a persistent file on disk.
///
/// This handler serializes the complete game world state using RON format
/// and writes it to a versioned save file. The save system creates organized
/// storage with version compatibility information.
///
/// # Parameters
///
/// * `world` - Reference to the current game world to serialize
/// * `view` - Mutable reference to display save results and feedback
/// * `gamefile` - Name for the save slot (without path or extension)
///
/// # Returns
///
/// Returns `Ok(())` on successful save, or an error if the save operation fails.
///
/// # Save File Organization
///
/// - **Directory**: `saved_games/` (created if it doesn't exist)
/// - **Filename**: `{gamefile}-amble-{version}.ron`
/// - **Format**: RON (Rusty Object Notation) for human readability
///
/// # Serialization Process
///
/// 1. **World serialization** - Convert complete world state to RON format
/// 2. **Directory creation** - Ensure save directory exists
/// 3. **File creation** - Create versioned save file
/// 4. **Data writing** - Write serialized world to file
/// 5. **Confirmation** - Display success message with file details
///
/// # Errors
///
/// Potential failure points:
/// - World serialization errors (corrupted game state)
/// - Directory creation failures (permission issues)
/// - File creation errors (disk space, permissions)
/// - Write operation failures (I/O errors)
///
/// # Success Feedback
///
/// On successful save:
/// - Confirmation message with save slot name
/// - File path information for reference
/// - Encouraging message to continue playing
/// - Logging of save operation for debugging
///
/// # File Format
///
/// RON format provides:
/// - Human-readable save files for debugging
/// - Efficient serialization/deserialization
/// - Version compatibility metadata
/// - Complete world state preservation
pub fn save_handler(world: &AmbleWorld, view: &mut View, gamefile: &str) -> Result<()> {
    // serialize the current AmbleWorld state to RON format
    let world_ron =
        ron::ser::to_string(world).with_context(|| "error converting AmbleWorld to 'ron' format".to_string())?;

    // create save dir if doesn't exist
    fs::create_dir_all(SAVE_DIR).with_context(|| "error creating saved_games folder".to_string())?;

    // create save file
    let save_path = PathBuf::from(SAVE_DIR).join(format!("{gamefile}-amble-{AMBLE_VERSION}.ron"));
    let mut save_file =
        fs::File::create(save_path.as_path()).with_context(|| format!("creating file '{}'", save_path.display()))?;

    // write world to file
    save_file
        .write_all(world_ron.as_bytes())
        .with_context(|| "failed to write AmbleWorld to .ron file".to_string())?;

    // disco!
    view.push(ViewItem::GameSaved {
        save_slot: gamefile.to_string(),
        save_file: save_path.to_string_lossy().to_string(),
    });
    view.push(ViewItem::ActionSuccess(format!(
        "Game saved to slot {} successfully. Amble on...",
        gamefile.underline().green()
    )));
    info!("Player saved game to \"{gamefile}\"");
    Ok(())
}

/// Changes the active UI theme shown in the terminal.
///
/// This handler either applies a requested theme or lists available themes when
/// invoked with an empty argument or the literal `"list"`.
///
/// # Errors
/// Returns an error if the global theme manager cannot be accessed or if applying the
/// requested theme fails.
pub fn theme_handler(view: &mut View, theme_name: &str) -> Result<()> {
    let manager = THEME_MANAGER
        .read()
        .map_err(|_| anyhow::anyhow!("Failed to access theme manager"))?;

    // If no theme name provided or "list" is specified, show available themes
    if theme_name.is_empty() || theme_name == "list" {
        let themes = manager.list_themes();
        let current = manager.current_name();

        view.push(ViewItem::EngineMessage(format!(
            "Available themes: {}",
            themes
                .iter()
                .map(|t| {
                    if t == &current {
                        format!("{t} (current)").status_style().to_string()
                    } else {
                        t.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        )));
        return Ok(());
    }

    // Try to set the requested theme
    match manager.set_theme(theme_name) {
        Ok(()) => {
            view.push(ViewItem::ActionSuccess(format!("Theme changed to '{theme_name}'")));
        },
        Err(_) => {
            view.push(ViewItem::Error(format!(
                "Theme '{theme_name}' not found. Use 'theme list' to see available themes."
            )));
        },
    }

    Ok(())
}
