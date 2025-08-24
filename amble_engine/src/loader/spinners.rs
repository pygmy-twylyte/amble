//! `loader::spinners` module
//!
//! This module implements the loading of spinner data from TOML.
//! It supports both core engine spinners (with built-in defaults) and
//! custom game-specific spinners defined entirely in TOML files.

use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use gametools::{Spinner, Wedge};
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::spinners::{CoreSpinnerType, SpinnerType};

/// Raw spinner data loaded from TOML files
#[derive(Debug, Deserialize, Serialize)]
pub struct RawSpinnerData {
    #[serde(rename = "spinnerType")]
    pub spinner_type_key: String,
    pub values: Vec<String>,
    #[serde(default)]
    pub widths: Vec<usize>,
}

/// Container for multiple spinner definitions in a TOML file
#[derive(Debug, Deserialize, Serialize)]
pub struct SpinnerFile {
    #[serde(rename = "spinners")]
    pub entries: Vec<RawSpinnerData>,
}

impl SpinnerFile {
    /// Convert data loaded from file into the spinner map for the world.
    /// This creates all core spinners with defaults, then applies any overrides from TOML.
    pub fn to_spinner_map(&self) -> HashMap<SpinnerType, Spinner<String>> {
        let mut spinners = HashMap::new();

        // First, create all core spinners with their built-in defaults
        Self::add_core_defaults(&mut spinners);

        // Then apply any overrides/additions from TOML data
        for spin_data in &self.entries {
            let spinner_type = SpinnerType::from_toml_key(&spin_data.spinner_type_key);

            // Validate and process the spinner data
            let wedges = Self::create_wedges_from_data(spin_data);

            if wedges.is_empty() {
                warn!("Spinner '{}' has no valid values, skipping", spin_data.spinner_type_key);
                continue;
            }

            let spinner = Spinner::new(wedges);

            // Log whether this is overriding a core spinner or adding a new one
            if spinner_type.is_core() {
                info!(
                    "Overriding core spinner '{}' with {} values from TOML",
                    spin_data.spinner_type_key,
                    spin_data.values.len()
                );
            } else {
                info!(
                    "Adding custom spinner '{}' with {} values",
                    spin_data.spinner_type_key,
                    spin_data.values.len()
                );
            }

            spinners.insert(spinner_type, spinner);
        }

        info!(
            "Spinner map created with {} total spinners ({} core, {} custom)",
            spinners.len(),
            spinners.keys().filter(|k| k.is_core()).count(),
            spinners.keys().filter(|k| k.is_custom()).count()
        );

        spinners
    }

    /// Add all core spinners with their built-in default values
    fn add_core_defaults(spinners: &mut HashMap<SpinnerType, Spinner<String>>) {
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
            let values = core_type.default_values();
            let widths = core_type.default_widths();

            let wedges: Vec<Wedge<String>> = values
                .iter()
                .zip(widths.iter())
                .map(|(val, &width)| Wedge::new_weighted((*val).to_string(), width))
                .collect();

            let spinner = Spinner::new(wedges);
            spinners.insert(SpinnerType::Core(core_type), spinner);
        }

        info!("Created {} core spinners with default values", core_types.len());
    }

    /// Create spinner wedges from raw TOML data, handling width defaults
    fn create_wedges_from_data(spin_data: &RawSpinnerData) -> Vec<Wedge<String>> {
        if spin_data.values.is_empty() {
            return Vec::new();
        }

        spin_data
            .values
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let width = spin_data.widths.get(i).copied().unwrap_or(1);
                Wedge::new_weighted(val.to_string(), width)
            })
            .collect()
    }
}

/// Load spinners from a TOML file.
/// Returns a map with all core spinners (using defaults) plus any custom spinners defined in the file.
/// Core spinners can be overridden by including them in the TOML file.
pub fn load_spinners(toml_path: &Path) -> Result<HashMap<SpinnerType, Spinner<String>>> {
    match std::fs::read_to_string(toml_path) {
        Ok(file_content) => {
            let spinner_file: SpinnerFile = toml::from_str(&file_content)
                .with_context(|| format!("parsing spinner data from {}", toml_path.display()))?;
            info!("Spinner data loaded from '{}'", toml_path.display());
            Ok(spinner_file.to_spinner_map())
        },
        Err(e) => {
            warn!(
                "Could not read spinner file '{}': {}. Using core defaults only.",
                toml_path.display(),
                e
            );
            // Return just the core defaults if the file can't be read
            let mut spinners = HashMap::new();
            SpinnerFile::add_core_defaults(&mut spinners);
            Ok(spinners)
        },
    }
}

/// Create a spinner map with only the core defaults.
/// Useful for testing or when no TOML file is available.
pub fn create_default_spinners() -> HashMap<SpinnerType, Spinner<String>> {
    let mut spinners = HashMap::new();
    SpinnerFile::add_core_defaults(&mut spinners);
    spinners
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_defaults_creation() {
        let spinners = create_default_spinners();

        // Should have all core spinner types
        assert!(spinners.contains_key(&SpinnerType::Core(CoreSpinnerType::EntityNotFound)));
        assert!(spinners.contains_key(&SpinnerType::Core(CoreSpinnerType::Movement)));
        assert!(spinners.contains_key(&SpinnerType::Core(CoreSpinnerType::TakeVerb)));

        // Should have exactly the number of core types
        assert_eq!(spinners.len(), 10);

        // All should be core spinners
        assert!(spinners.keys().all(|k| k.is_core()));
    }

    #[test]
    fn test_spinner_file_with_custom_spinner() {
        let spinner_file = SpinnerFile {
            entries: vec![RawSpinnerData {
                spinner_type_key: "customTest".to_string(),
                values: vec!["test1".to_string(), "test2".to_string()],
                widths: vec![1, 2],
            }],
        };

        let map = spinner_file.to_spinner_map();

        // Should have all core defaults plus the custom one
        assert!(map.len() > 10);
        assert!(map.contains_key(&SpinnerType::Custom("customTest".to_string())));
    }

    #[test]
    fn test_spinner_file_with_core_override() {
        let spinner_file = SpinnerFile {
            entries: vec![RawSpinnerData {
                spinner_type_key: "movement".to_string(),
                values: vec!["custom move".to_string()],
                widths: vec![1],
            }],
        };

        let map = spinner_file.to_spinner_map();

        // Should still have 10 core spinners, but movement should be overridden
        assert_eq!(map.keys().filter(|k| k.is_core()).count(), 10);

        let movement_spinner = map.get(&SpinnerType::Core(CoreSpinnerType::Movement)).unwrap();
        // This would need more complex testing to verify the content, but we can at least verify it exists
        assert!(movement_spinner.spin().is_some());
    }

    #[test]
    fn test_load_spinners_missing_file() {
        let result = load_spinners(Path::new("/nonexistent/file.toml"));

        // Should succeed with defaults
        assert!(result.is_ok());
        let spinners = result.unwrap();
        assert_eq!(spinners.len(), 10); // Just core defaults
    }

    #[test]
    fn test_spinner_file_parsing() {
        let spinner_file = SpinnerFile {
            entries: vec![
                RawSpinnerData {
                    spinner_type_key: "testCustom".to_string(),
                    values: vec!["value1".to_string(), "value2".to_string()],
                    widths: vec![1, 3],
                },
                RawSpinnerData {
                    spinner_type_key: "movement".to_string(),
                    values: vec!["override move".to_string()],
                    widths: vec![],
                },
            ],
        };

        let spinners = spinner_file.to_spinner_map();
        // Should have core spinners + 1 custom
        assert!(spinners.len() > 10);
        assert!(spinners.contains_key(&SpinnerType::Custom("testCustom".to_string())));
    }

    #[test]
    fn test_empty_values_skipped() {
        let spinner_file = SpinnerFile {
            entries: vec![RawSpinnerData {
                spinner_type_key: "empty".to_string(),
                values: vec![],
                widths: vec![],
            }],
        };

        let map = spinner_file.to_spinner_map();

        // Should not contain the empty spinner
        assert!(!map.contains_key(&SpinnerType::Custom("empty".to_string())));
        // Should still have all core defaults
        assert_eq!(map.keys().filter(|k| k.is_core()).count(), 10);
    }
}
