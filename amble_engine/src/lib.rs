#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

//! # Amble game engine.
//!
//! A full-featured text adventure game engine with scripting support.
//!
//! This crate contains the core data structures and logic that power the
//! Amble command-line adventure game. It provides:
//!
//! - **World modeling**: Rooms, items, NPCs with complex interactions
//! - **Command parsing**: Natural language command interpretation
//! - **Trigger system**: Event-driven game logic and scripting
//! - **Save/load**: Complete game state serialization
//! - **Goal tracking**: Quest and objective management
//! - **Rich text output**: Styled terminal output with multiple view modes
//!
//! The engine is designed to be data-driven, loading world content from a
//! compiled `WorldDef` (`world.ron`) rather than requiring code changes.
//!
//! **Note:** The engine still reads some TOML for configuration (themes, scoring,
//! and help commands), but gameplay content should be authored in the
//! [`amble_script`] DSL and compiled to `world.ron`.

pub const AMBLE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Id = String;

// DEV_MODE is enabled or disabled through this const throughout
#[cfg(feature = "dev-mode")]
pub const DEV_MODE: bool = true;

#[cfg(not(feature = "dev-mode"))]
pub const DEV_MODE: bool = false;

// Core modules
pub mod command;
pub mod data_paths;
pub mod dev_command;
pub mod entity_search;
pub mod goal;
pub mod health;
pub mod helpers;
pub mod idgen;
pub mod item;
pub mod loader;
pub mod markup;
pub mod npc;
pub mod player;
pub mod repl;
pub mod room;
pub mod save_files;
pub mod scheduler;
pub mod spinners;
pub mod style;
pub mod theme;
pub mod trigger;
pub mod view;
pub mod world;

// Re-exports for convenience
pub use goal::Goal;
pub use item::{Item, ItemHolder};
pub use loader::load_world;
pub use npc::Npc;
pub use player::Player;
pub use repl::run_repl;
pub use room::Room;
pub use scheduler::Scheduler;
pub use view::{View, ViewItem};
pub use world::{AmbleWorld, Location, WorldObject};
