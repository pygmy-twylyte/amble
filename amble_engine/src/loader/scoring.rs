//! Scoring rank definitions and helpers.
//!
//! This module defines the scoring system used when the player quits the game.
//! Ranks are determined by the percentage of maximum score achieved, with
//! each rank having a threshold, name, and description fitting the game's style.

use amble_data::ScoringDef;
use serde::{Deserialize, Serialize};

/// A single scoring rank with its threshold and flavor text.
pub type ScoringRank = amble_data::ScoringRankDef;

/// Complete scoring configuration for the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Sorted list of ranks (highest threshold first)
    pub ranks: Vec<ScoringRank>,
    /// Title displayed above the score summary
    pub report_title: String,
}

impl ScoringConfig {
    /// Returns the appropriate rank for a given completion percentage.
    ///
    /// # Parameters
    /// * `percent` - Percentage of max score achieved (0.0-100.0)
    ///
    /// # Returns
    /// A tuple of (`rank_name`, description) for display to the player.
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

    /// Build a scoring configuration from the compiled WorldDef data.
    pub fn from_def(def: &ScoringDef) -> Self {
        let defaults = ScoringDef::default();
        let mut ranks = if def.ranks.is_empty() {
            defaults.ranks.clone()
        } else {
            def.ranks.clone()
        };
        // Sort ranks by threshold descending (highest first)
        ranks.sort_by(|a, b| {
            b.threshold
                .partial_cmp(&a.threshold)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let report_title = if def.report_title.trim().is_empty() {
            defaults.report_title
        } else {
            def.report_title.clone()
        };

        ScoringConfig { ranks, report_title }
    }
}

impl Default for ScoringConfig {
    fn default() -> Self {
        ScoringConfig::from_def(&ScoringDef::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ranks_are_sorted() {
        let ranks = ScoringConfig::default().ranks;
        for i in 0..ranks.len().saturating_sub(1) {
            assert!(
                ranks[i].threshold >= ranks[i + 1].threshold,
                "Ranks should be sorted descending by threshold"
            );
        }
    }

    #[test]
    fn test_get_rank_exact_match() {
        let config = ScoringConfig::default();
        let (name, _) = config.get_rank(99.0);
        assert_eq!(name, "Quantum Overachiever");
    }

    #[test]
    fn test_get_rank_in_between() {
        let config = ScoringConfig::default();

        let (name, _) = config.get_rank(88.0);
        assert_eq!(name, "Licensed Reality Bender");

        let (name, _) = config.get_rank(50.0);
        assert_eq!(name, "Unpaid Research Assistant");
    }

    #[test]
    fn test_get_rank_edge_cases() {
        let config = ScoringConfig::default();

        let (name, _) = config.get_rank(100.0);
        assert_eq!(name, "Quantum Overachiever");

        let (name, _) = config.get_rank(99.99);
        assert_eq!(name, "Quantum Overachiever");

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
            report_title: "Results".into(),
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
}
