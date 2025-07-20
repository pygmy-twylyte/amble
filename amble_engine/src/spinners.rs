use gametools::spinners::{Spinner, Wedge};
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

pub trait SpinnerExt {
    fn from_strs(words: &[&'static str]) -> Self;
}
impl SpinnerExt for Spinner<&'static str> {
    fn from_strs(words: &[&'static str]) -> Self {
        let wedges = words.iter().map(|w| Wedge::new(*w)).collect::<Vec<_>>();
        Spinner::new(wedges)
    }
}
