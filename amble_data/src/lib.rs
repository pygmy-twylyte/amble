//! Shared data model for Amble content.

pub mod defs;
pub mod validate;

pub use defs::*;
pub use validate::{ValidationError, validate_world};
