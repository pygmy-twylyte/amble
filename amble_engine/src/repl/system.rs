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
use crate::loader::help::load_help_data;
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
/// - **ClearVerbose**: Clears screen on movement, always shows full descriptions
/// - **Verbose**: Always shows full room descriptions without screen clearing
/// - **Brief**: Shows full descriptions only on first visit and when explicitly looking
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
/// Players receive titles ranging from "Quantum Overachiever" (100%) to
/// "Amnesiac Test Subject" (0%), each with personalized evaluation messages
/// that reflect their exploration and puzzle-solving success.
pub fn quit_handler(world: &AmbleWorld, view: &mut View) -> Result<ReplControl> {
    info!("{} quit with a score of {}", world.player.name(), world.player.score);
    info!("ending flags:");
    world.player.flags.iter().for_each(|i| info!("flag> {i}"));
    info!("ending inventory:");
    world
        .player
        .inventory
        .iter()
        .filter_map(|uuid| world.items.get(uuid))
        .for_each(|i| info!("- {} ({})", i.name(), i.symbol()));

    let percent = (world.player.score as f32 / world.max_score as f32) * 100.0;

    let (rank, eval) = match percent {
        100.0 => (
            "Quantum Overachiever",
            "You saw the multiverse, understood it, then filed a bug report.",
        ),
        p if p >= 90.0 => (
            "Senior Field Operative",
            "A nearly flawless run. Someone give this candidate a promotion.",
        ),
        p if p >= 75.0 => (
            "Licensed Reality Bender",
            "Impressive grasp of nonlinear environments and cake-based paradoxes.",
        ),
        p if p >= 60.0 => (
            "Rogue Intern, Level II",
            "You got the job done, and only melted one small pocket universe.",
        ),
        p if p >= 45.0 => (
            "Unpaid Research Assistant",
            "Solid effort. Some concepts may have slipped through dimensional cracks.",
        ),
        p if p >= 30.0 => (
            "Junior Sandwich Technician",
            "Good instincts, questionable execution. Especially with condiments.",
        ),
        p if p >= 15.0 => (
            "Volunteer Tour Guide",
            "You wandered. You looked at stuff. It was something.",
        ),
        p if p >= 5.0 => (
            "Mailbox Stuffing Trainee",
            "You opened a box, tripped on a rug, and called it a day.",
        ),
        p if p >= 1.0 => (
            "Accidental Hire",
            "We're not sure how you got in. Please return your lanyard.",
        ),
        _ => ("Amnesiac Test Subject", "Did you… play? Were you even awake?"),
    };

    let visited = world.rooms.values().filter(|r| r.visited).count();

    view.push(ViewItem::QuitSummary {
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
    let basic_text_path = Path::new("amble_engine/data/help_basic.txt");
    let commands_toml_path = Path::new("amble_engine/data/help_commands.toml");

    match load_help_data(basic_text_path, commands_toml_path) {
        Ok(help_data) => {
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
    let load_path = PathBuf::from("saved_games").join(format!("{gamefile}-amble-{AMBLE_VERSION}.ron"));
    if let Ok(world_ron) = fs::read_to_string(load_path.as_path()) {
        if let Ok(new_world) = ron::from_str::<AmbleWorld>(&world_ron) {
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
            }
            *world = new_world;
            view.push(ViewItem::ActionSuccess(format!(
                "Saved game {} loaded successfully. Sally forth.",
                gamefile.underline().green()
            )));
            view.push(ViewItem::GameLoaded {
                save_slot: gamefile.to_string(),
                save_file: load_path.to_string_lossy().to_string(),
            });
            info!("Player reloaded AmbleWorld from file '{}'", load_path.display());
        } else {
            view.push(ViewItem::ActionFailure(format!(
                "Unable to load the {} save file. The Amble engine may have changed since it was created.",
                gamefile.error_style()
            )));
            warn!("player attempted to load '{gamefile}': failed to parse, likely version conflict");
        }
    } else {
        view.push(ViewItem::Error(format!(
            "Unable to find {} save file. Load aborted.",
            gamefile.error_style()
        )));
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
/// # Error Handling
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
    fs::create_dir_all("saved_games").with_context(|| "error creating saved_games folder".to_string())?;

    // create save file
    let save_path = PathBuf::from("saved_games").join(format!("{gamefile}-amble-{AMBLE_VERSION}.ron"));
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

/// Handler for the theme command - changes the active color scheme
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
                        format!("{} (current)", t).status_style().to_string()
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
        Ok(_) => {
            view.push(ViewItem::ActionSuccess(format!("Theme changed to '{}'", theme_name)));
        },
        Err(_) => {
            view.push(ViewItem::Error(format!(
                "Theme '{}' not found. Use 'theme list' to see available themes.",
                theme_name
            )));
        },
    }

    Ok(())
}
