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
//! The engine is designed to be data-driven, loading all game content
//! from TOML configuration files rather than requiring code changes.
//!
//! **Note: While the engine reads TOML and you can write directly in TOML
//! (the first 1/3 of the demo game was written that way), it is vastly easier
//! to use the full capabilities of the engine by creating content using
//! the ['amble_script'] DSL -- and easier still if you use the Zed editor
//! and accompanying Zed Amble extension.

pub const AMBLE_VERSION: &str = env!("CARGO_PKG_VERSION");

// DEV_MODE is enabled or disabled through this const throughout
#[cfg(feature = "dev-mode")]
pub const DEV_MODE: bool = true;

#[cfg(not(feature = "dev-mode"))]
pub const DEV_MODE: bool = false;

// Core modules
pub mod command;
pub mod data_paths;
pub mod dev_command;
pub mod goal;
pub mod health;
pub mod helpers;
pub mod idgen;
pub mod item;
pub mod loader;
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
