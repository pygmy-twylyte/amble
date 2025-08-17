#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

//! Amble game engine library.
//!
//! A comprehensive text adventure game engine with scripting support.
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
//! The engine is designed to be data-driven, loading most game content
//! from TOML configuration files rather than requiring code changes.

pub const AMBLE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEV_MODE: bool = true;

// Core modules
pub mod command;
pub mod dev_command;
pub mod goal;
pub mod idgen;
pub mod item;
pub mod loader;
pub mod npc;
pub mod player;
pub mod repl;
pub mod room;
pub mod spinners;
pub mod style;
pub mod trigger;
pub mod view;
pub mod world;

// Re-exports for convenience
pub use goal::Goal;
pub use item::{Item, ItemHolder};
pub use loader::load_world;
pub use player::Player;
pub use repl::run_repl;
pub use room::Room;
pub use view::{View, ViewItem};
pub use world::{AmbleWorld, Location, WorldObject};
