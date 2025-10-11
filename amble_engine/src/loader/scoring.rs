//! Scoring rank definitions and loader.
//!
//! This module defines the scoring system used when the player quits the game.
//! Ranks are determined by the percentage of maximum score achieved, with
//! each rank having a threshold, name, and description fitting the game's style.

use anyhow::{Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A single scoring rank with its threshold and flavor text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringRank {
    /// Minimum percentage (0.0-100.0) required to achieve this rank
    pub threshold: f32,
    /// Display name of the rank
    pub name: String,
    /// Humorous one-sentence evaluation of the player's performance
    pub description: String,
}

/// Wrapper for the TOML file containing scoring ranks.
#[derive(Debug, Deserialize)]
struct ScoringFile {
    ranks: Vec<ScoringRank>,
}

/// Complete scoring configuration for the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Sorted list of ranks (highest threshold first)
    pub ranks: Vec<ScoringRank>,
}

impl ScoringConfig {
    /// Returns the appropriate rank for a given completion percentage.
    ///
    /// # Parameters
    /// * `percent` - Percentage of max score achieved (0.0-100.0)
    ///
    /// # Returns
    /// A tuple of (rank_name, description) for display to the player.
    pub fn get_rank(&self, percent: f32) -> (&str, &str) {
        for rank in &self.ranks {
            if percent >= rank.threshold {
                return (&rank.name, &rank.description);
            }
        }

        // Fallback to the last rank if no match (should never happen if 0.0 threshold exists)
        if let Some(last_rank) = self.ranks.last() {
            (&last_rank.name, &last_rank.description)
        } else {
            ("Unknown Rank", "No scoring data available.")
        }
    }
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            ranks: default_scoring_ranks(),
        }
    }
}

/// Returns hardcoded default scoring ranks.
///
/// These defaults are used if `scoring.toml` cannot be loaded or parsed.
/// Ranks are sorted from highest to lowest threshold.
fn default_scoring_ranks() -> Vec<ScoringRank> {
    vec![
        ScoringRank {
            threshold: 99.0,
            name: "Completionist".to_string(),
            description: "Wow! You solved all of the puzzles, visted all areas, and did everything there was to do!"
                .to_string(),
        },
        ScoringRank {
            threshold: 90.0,
            name: "Master Explorer".to_string(),
            description: "A nearly flawless run. You found and solved just about everything.".to_string(),
        },
        ScoringRank {
            threshold: 80.0,
            name: "Seasoned Explorer".to_string(),
            description: "You covered an impressive amount of ground.".to_string(),
        },
        ScoringRank {
            threshold: 70.0,
            name: "Explorer".to_string(),
            description: "You tackled a strong majority of the game content.".to_string(),
        },
        ScoringRank {
            threshold: 60.0,
            name: "Chief Assistant Explorer".to_string(),
            description: "Solid effort. Some areas or opportunities went unnoticed.".to_string(),
        },
        ScoringRank {
            threshold: 50.0,
            name: "Master Assistant Explorer".to_string(),
            description: "Good instincts, but spotty coverage. You completed a bare majority of the game.".to_string(),
        },
        ScoringRank {
            threshold: 40.0,
            name: "Assistant Explorer".to_string(),
            description: "You seemed to lose steam just before things really started getting interesting.".to_string(),
        },
        ScoringRank {
            threshold: 25.0,
            name: "Scout".to_string(),
            description: "You opened a box, tripped on a rug, and called it a day.".to_string(),
        },
        ScoringRank {
            threshold: 10.0,
            name: "Scribe".to_string(),
            description: "You jotted some notes, but missed some major portions of the game while doing it."
                .to_string(),
        },
        ScoringRank {
            threshold: 0.0,
            name: "Casual Observer".to_string(),
            description: "Did you… play? Were you even awake?".to_string(),
        },
    ]
}

/// Loads scoring configuration from a TOML file, falling back to defaults on error.
///
/// # Parameters
/// * `toml_path` - Path to the `scoring.toml` file
///
/// # Returns
/// A `ScoringConfig` with either loaded or default ranks. This function never
/// fails - it returns defaults if the file cannot be loaded or parsed.
///
/// # Logging
/// - `info!` on successful load
/// - `warn!` if file cannot be read or parsed (with fallback to defaults)
pub fn load_scoring(toml_path: &Path) -> ScoringConfig {
    match try_load_scoring(toml_path) {
        Ok(config) => {
            info!(
                "{} scoring ranks loaded from '{}'",
                config.ranks.len(),
                toml_path.display()
            );
            config
        },
        Err(e) => {
            warn!(
                "Could not load scoring data from '{}': {}. Using hardcoded defaults.",
                toml_path.display(),
                e
            );
            ScoringConfig {
                ranks: default_scoring_ranks(),
            }
        },
    }
}

/// Attempts to load scoring configuration from a TOML file.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
fn try_load_scoring(toml_path: &Path) -> Result<ScoringConfig> {
    let scoring_file = fs::read_to_string(toml_path)
        .with_context(|| format!("reading scoring data from '{}'", toml_path.display()))?;

    let wrapper: ScoringFile = toml::from_str(&scoring_file)
        .with_context(|| format!("parsing scoring data from '{}'", toml_path.display()))?;

    let mut ranks = wrapper.ranks;

    // Sort ranks by threshold descending (highest first)
    ranks.sort_by(|a, b| {
        b.threshold
            .partial_cmp(&a.threshold)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(ScoringConfig { ranks })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ranks_are_sorted() {
        let ranks = default_scoring_ranks();
        for i in 0..ranks.len() - 1 {
            assert!(
                ranks[i].threshold >= ranks[i + 1].threshold,
                "Ranks should be sorted descending by threshold"
            );
        }
    }

    #[test]
    fn test_get_rank_exact_match() {
        let config = ScoringConfig {
            ranks: default_scoring_ranks(),
        };

        let (name, _) = config.get_rank(100.0);
        assert_eq!(name, "Quantum Overachiever");

        let (name, _) = config.get_rank(90.0);
        assert_eq!(name, "Senior Field Operative");

        let (name, _) = config.get_rank(0.0);
        assert_eq!(name, "Amnesiac Test Subject");
    }

    #[test]
    fn test_get_rank_in_between() {
        let config = ScoringConfig {
            ranks: default_scoring_ranks(),
        };

        let (name, _) = config.get_rank(92.5);
        assert_eq!(name, "Senior Field Operative");

        let (name, _) = config.get_rank(76.3);
        assert_eq!(name, "Licensed Reality Bender");

        let (name, _) = config.get_rank(50.0);
        assert_eq!(name, "Unpaid Research Assistant");
    }

    #[test]
    fn test_get_rank_edge_cases() {
        let config = ScoringConfig {
            ranks: default_scoring_ranks(),
        };

        let (name, _) = config.get_rank(100.0);
        assert_eq!(name, "Quantum Overachiever");

        let (name, _) = config.get_rank(99.99);
        assert_eq!(name, "Senior Field Operative");

        let (name, _) = config.get_rank(0.01);
        assert_eq!(name, "Amnesiac Test Subject");
    }

    #[test]
    fn test_custom_scoring_config() {
        let config = ScoringConfig {
            ranks: vec![
                ScoringRank {
                    threshold: 80.0,
                    name: "Expert".to_string(),
                    description: "You mastered the challenge.".to_string(),
                },
                ScoringRank {
                    threshold: 50.0,
                    name: "Competent".to_string(),
                    description: "You did reasonably well.".to_string(),
                },
                ScoringRank {
                    threshold: 0.0,
                    name: "Novice".to_string(),
                    description: "You tried.".to_string(),
                },
            ],
        };

        let (name, desc) = config.get_rank(95.0);
        assert_eq!(name, "Expert");
        assert_eq!(desc, "You mastered the challenge.");

        let (name, desc) = config.get_rank(65.0);
        assert_eq!(name, "Competent");
        assert_eq!(desc, "You did reasonably well.");

        let (name, desc) = config.get_rank(25.0);
        assert_eq!(name, "Novice");
        assert_eq!(desc, "You tried.");
    }

    #[test]
    fn test_default_trait_provides_working_config() {
        let config = ScoringConfig::default();
        assert!(!config.ranks.is_empty());

        let (name, _) = config.get_rank(100.0);
        assert_eq!(name, "Quantum Overachiever");
    }
}
