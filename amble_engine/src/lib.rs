#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

pub const AMBLE_VERSION: &str = env!("CARGO_PKG_VERSION");

// Core modules
pub mod command;
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
