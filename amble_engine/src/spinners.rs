//! module: spinners
//!
//! Random text generation system for varied game responses.
//!
//! Amble uses the `gametools` crate's `Spinner` module to provide variety
//! in user feedback and intermittent ambient events. Spinners are weighted
//! random text generators that help avoid repetitive messages.
//!
//! The engine now supports two types of spinners:
//! - **Core spinners** (`CoreSpinnerType`) are essential for engine operation
//! - **Custom spinners** are defined per-game in TOML data files
//!
//! Core spinners have built-in defaults but can be overridden in TOML files.
//! Custom spinners are completely defined by game data.
//!

use serde::{Deserialize, Serialize};

/// Core spinner types that are essential for the engine to function.
/// These have built-in defaults but can be overridden in TOML files.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoreSpinnerType {
    /// Used when entity lookups fail
    EntityNotFound,
    /// Used when movement direction is invalid
    DestinationUnknown,
    /// Used for travel messages between rooms
    Movement,
    /// Used when actions have no effect
    NoEffect,
    /// Used when NPCs don't respond to talk attempts
    NpcIgnore,
    /// Used for variety in "take" command responses
    TakeVerb,
    /// Used for unrecognized player commands
    UnrecognizedCommand,
    /// Used for quit/exit messages
    QuitMsg,
    /// Used for "NPC entered" messages
    NpcEntered,
    /// Used for "NPC left" messages
    NpcLeft,
}

/// Represents either a core spinner type or a custom game-specific spinner.
/// Custom spinners are identified by string keys and defined entirely in TOML.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpinnerType {
    /// Core engine spinner with built-in defaults
    Core(CoreSpinnerType),
    /// Custom game-specific spinner identified by string key
    Custom(String),
}

impl SpinnerType {
    /// Create a custom spinner type from a string key
    pub fn custom(key: impl Into<String>) -> Self {
        SpinnerType::Custom(key.into())
    }

    /// Returns true if this is a core spinner type
    pub fn is_core(&self) -> bool {
        matches!(self, SpinnerType::Core(_))
    }

    /// Returns true if this is a custom spinner type
    pub fn is_custom(&self) -> bool {
        matches!(self, SpinnerType::Custom(_))
    }

    /// Get the string representation used in TOML files
    pub fn as_toml_key(&self) -> String {
        match self {
            SpinnerType::Core(core) => core.as_toml_key(),
            SpinnerType::Custom(key) => key.clone(),
        }
    }

    /// Parse a spinner type from a TOML key string
    pub fn from_toml_key(key: &str) -> Self {
        // Try to parse as core spinner first
        if let Some(core) = CoreSpinnerType::from_toml_key(key) {
            SpinnerType::Core(core)
        } else {
            // Otherwise treat as custom spinner
            SpinnerType::Custom(key.to_string())
        }
    }
}

impl CoreSpinnerType {
    /// Get the string representation used in TOML files
    pub fn as_toml_key(&self) -> String {
        match self {
            CoreSpinnerType::EntityNotFound => "entityNotFound".to_string(),
            CoreSpinnerType::DestinationUnknown => "destinationUnknown".to_string(),
            CoreSpinnerType::Movement => "movement".to_string(),
            CoreSpinnerType::NoEffect => "noEffect".to_string(),
            CoreSpinnerType::NpcIgnore => "npcIgnore".to_string(),
            CoreSpinnerType::TakeVerb => "takeVerb".to_string(),
            CoreSpinnerType::UnrecognizedCommand => "unrecognizedCommand".to_string(),
            CoreSpinnerType::QuitMsg => "quitMsg".to_string(),
            CoreSpinnerType::NpcLeft => "npcLeft".to_string(),
            CoreSpinnerType::NpcEntered => "npcEntered".to_string(),
        }
    }

    /// Parse a core spinner type from a TOML key string
    pub fn from_toml_key(key: &str) -> Option<Self> {
        match key {
            "entityNotFound" => Some(CoreSpinnerType::EntityNotFound),
            "destinationUnknown" => Some(CoreSpinnerType::DestinationUnknown),
            "movement" => Some(CoreSpinnerType::Movement),
            "noEffect" => Some(CoreSpinnerType::NoEffect),
            "npcIgnore" => Some(CoreSpinnerType::NpcIgnore),
            "takeVerb" => Some(CoreSpinnerType::TakeVerb),
            "unrecognizedCommand" => Some(CoreSpinnerType::UnrecognizedCommand),
            "quitMsg" => Some(CoreSpinnerType::QuitMsg),
            "npcEntered" => Some(CoreSpinnerType::NpcEntered),
            "npcLeft" => Some(CoreSpinnerType::NpcLeft),
            _ => None,
        }
    }

    /// Get the built-in default values for this core spinner type
    pub fn default_values(&self) -> Vec<&'static str> {
        match self {
            CoreSpinnerType::EntityNotFound => vec![
                "What's that?",
                "You made that up.",
                "Never heard of it.",
                "You don't see that here.",
                "I don't recognize that.",
            ],
            CoreSpinnerType::DestinationUnknown => vec![
                "Can't get there from here.",
                "Which way is that?",
                "Your feet refuse to obey such nonsense.",
                "That direction folds in on itself and vanishes.",
            ],
            CoreSpinnerType::Movement => vec![
                "You move that direction...",
                "You head that way...",
                "You go on...",
                "Heading that direction...",
            ],
            CoreSpinnerType::NoEffect => vec![
                "You try it. It doesn't seem to help.",
                "Nothing happens.",
                "That doesn't seem to work.",
                "No effect.",
            ],
            CoreSpinnerType::NpcIgnore => vec![
                "Has nothing to say.",
                "Ignores you.",
                "Isn't in the mood to talk.",
                "Stands mute.",
            ],
            CoreSpinnerType::TakeVerb => vec!["take", "grab", "get", "pick up"],
            CoreSpinnerType::UnrecognizedCommand => vec![
                "I don't understand that.",
                "Unrecognized command.",
                "Try 'help' for available commands.",
                "That doesn't make sense.",
            ],
            CoreSpinnerType::QuitMsg => vec!["Goodbye!", "Thanks for playing!", "See you later!"],
            CoreSpinnerType::NpcEntered => vec![
                "enters.",
                "ambles in.",
                "arrives.",
                "shows up.",
                "turns up.",
                "drops in.",
            ],
            CoreSpinnerType::NpcLeft => vec!["leaves.", "departs.", "exits.", "goes away.", "takes off."],
        }
    }

    /// Get the default widths for this core spinner type
    pub fn default_widths(&self) -> Vec<usize> {
        let count = self.default_values().len();
        vec![1; count] // Equal weight for all default values
    }
}

// Convenience From implementations for easier migration
impl From<CoreSpinnerType> for SpinnerType {
    fn from(core: CoreSpinnerType) -> Self {
        SpinnerType::Core(core)
    }
}

impl From<String> for SpinnerType {
    fn from(key: String) -> Self {
        SpinnerType::Custom(key)
    }
}

impl From<&str> for SpinnerType {
    fn from(key: &str) -> Self {
        SpinnerType::Custom(key.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_spinner_toml_key_roundtrip() {
        let core_types = [
            CoreSpinnerType::EntityNotFound,
            CoreSpinnerType::DestinationUnknown,
            CoreSpinnerType::Movement,
            CoreSpinnerType::NoEffect,
            CoreSpinnerType::NpcIgnore,
            CoreSpinnerType::TakeVerb,
            CoreSpinnerType::UnrecognizedCommand,
            CoreSpinnerType::QuitMsg,
            CoreSpinnerType::NpcEntered,
            CoreSpinnerType::NpcLeft,
        ];

        for core_type in core_types {
            let key = core_type.as_toml_key();
            let parsed = CoreSpinnerType::from_toml_key(&key);
            assert_eq!(Some(core_type), parsed, "Failed roundtrip for {:?}", core_type);
        }
    }

    #[test]
    fn spinner_type_from_toml_key() {
        // Test core spinner parsing
        let core_spinner = SpinnerType::from_toml_key("entityNotFound");
        assert_eq!(core_spinner, SpinnerType::Core(CoreSpinnerType::EntityNotFound));

        // Test custom spinner parsing
        let custom_spinner = SpinnerType::from_toml_key("ambientForest");
        assert_eq!(custom_spinner, SpinnerType::Custom("ambientForest".to_string()));
    }

    #[test]
    fn spinner_type_convenience_methods() {
        let core = SpinnerType::Core(CoreSpinnerType::Movement);
        let custom = SpinnerType::Custom("test".to_string());

        assert!(core.is_core());
        assert!(!core.is_custom());
        assert!(!custom.is_core());
        assert!(custom.is_custom());
    }

    #[test]
    fn core_spinner_defaults() {
        for core_type in [
            CoreSpinnerType::EntityNotFound,
            CoreSpinnerType::DestinationUnknown,
            CoreSpinnerType::Movement,
            CoreSpinnerType::NoEffect,
            CoreSpinnerType::NpcIgnore,
            CoreSpinnerType::TakeVerb,
            CoreSpinnerType::UnrecognizedCommand,
            CoreSpinnerType::QuitMsg,
            CoreSpinnerType::NpcEntered,
            CoreSpinnerType::NpcLeft,
        ] {
            let values = core_type.default_values();
            let widths = core_type.default_widths();

            assert!(!values.is_empty(), "{:?} should have default values", core_type);
            assert_eq!(
                values.len(),
                widths.len(),
                "{:?} values and widths should match",
                core_type
            );
        }
    }
}
