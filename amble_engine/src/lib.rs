#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

//! Amble game engine library.
//!
//! This crate contains the core data structures and logic that power the
//! command line adventure game. It exposes a small API used by the binary
//! in `main.rs` and by tooling.

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
pub mod world;

// Re-exports for convenience
pub use goal::Goal;
pub use item::{Item, ItemHolder};
pub use loader::load_world;
pub use player::Player;
pub use repl::run_repl;
pub use room::Room;
pub use world::{AmbleWorld, Location, WorldObject};
