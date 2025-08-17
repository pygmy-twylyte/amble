//! module: spinners
//!
//! Random text generation system for varied game responses.
//!
//! Amble uses the `gametools` crate's `Spinner` module to provide variety
//! in user feedback and intermittent ambient events. Spinners are weighted
//! random text generators that help avoid repetitive messages.
//!
//! The types of spinners used in the game are defined here as the [`SpinnerType`] enum.
//! The actual spinner data (text and weights) is loaded from `spinners.toml`
//! and the spinners are built from that data in the `loader::spinners` module.
//!

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpinnerType {
    AmbientAA3B,
    AmbientInterior,
    AmbientWoodland,
    DestinationUnknown,
    EntityNotFound,
    Movement,
    Muzak,
    Nauseated,
    NoEffect,
    NpcIgnore,
    QuitMsg,
    TakeVerb,
    UnrecognizedCommand,
}
