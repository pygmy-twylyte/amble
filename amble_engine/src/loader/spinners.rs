//! loader::spinners module
//!
//! This module implements the loading of spinner data from TOML.
use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use gametools::{Spinner, Wedge};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::spinners::SpinnerType;

/// The structure used for first stage loading of spinner data from file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSpinnerData {
    pub spinner_type: SpinnerType,
    pub values: Vec<String>,
    #[serde(default)]
    pub widths: Vec<usize>,
}

/// Wrapper for vector of spinner data from TOML file
#[derive(Debug, Serialize, Deserialize)]
pub struct SpinnerFile {
    #[serde(rename = "spinners")]
    pub entries: Vec<RawSpinnerData>,
}
impl SpinnerFile {
    /// Convert data loaded from file into the spinner map for the world.
    pub fn to_spinner_map(&self) -> HashMap<SpinnerType, Spinner<String>> {
        let mut spinners = HashMap::new();
        for spin_data in &self.entries {
            let wedges: Vec<Wedge<String>> = spin_data
                .values
                .iter()
                .enumerate()
                .map(|(i, val)| {
                    Wedge::new_weighted(val.to_string(), *spin_data.widths.get(i).unwrap_or(&1))
                })
                .collect();
            if spinners
                .insert(spin_data.spinner_type, Spinner::new(wedges))
                .is_some()
            {
                warn!(
                    "duplicate entry for spinner type {:?}",
                    spin_data.spinner_type
                );
            };
        }
        spinners
    }
}

pub fn load_spinners(toml_path: &Path) -> Result<HashMap<SpinnerType, Spinner<String>>> {
    let file = std::fs::read_to_string(toml_path)
        .with_context(|| format!("reading spinner data from {}", toml_path.display()))?;
    let spinner_file: SpinnerFile = toml::from_str(&file)
        .with_context(|| format!("parsing spinner data from {}", toml_path.display()))?;
    Ok(spinner_file.to_spinner_map())
}
