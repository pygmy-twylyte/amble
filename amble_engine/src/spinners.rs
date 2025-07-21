//! module: spinners
//!
//! Amble uses the `gametools` crate's `Spinner` module to provide variety
//! in user feedback and intermittent ambient events. The types of spinners
//! used in the game are defined here; the actual spinner data is in the
//! `spinners.toml` file and the spinners are built from that data in the
//! loader::spinners module.
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
    Nauseated,
    NoEffect,
    NpcIgnore,
    QuitMsg,
    TakeVerb,
    UnrecognizedCommand,
}
