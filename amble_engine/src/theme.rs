//! Theme system for customizable color schemes in the game.
//!
//! This module provides a theming system that allows players to switch between
//! different color schemes. Themes can be loaded from a TOML file and applied
//! dynamically during gameplay.

use anyhow::{Context, Result};
use colored::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// RGB color representation for theme configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThemeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ThemeColor {
    /// Create a new ThemeColor from RGB values
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert to colored::Color for use with the colored crate
    pub fn to_color(&self) -> Color {
        Color::TrueColor {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

/// A complete color theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ThemeColors,
}

/// All color settings for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Prompt and status
    pub prompt: ThemeColor,
    pub prompt_bg: ThemeColor,
    pub status: ThemeColor,

    // General styles
    pub highlight: ThemeColor,
    pub transition: ThemeColor,
    pub subheading: ThemeColor,

    // Items
    pub item: ThemeColor,
    pub item_text: ThemeColor,

    // NPCs
    pub npc: ThemeColor,
    pub npc_quote: ThemeColor,
    pub npc_movement: ThemeColor,

    // Rooms
    pub room: ThemeColor,
    pub room_titlebar: ThemeColor,
    pub description: ThemeColor,
    pub overlay: ThemeColor,

    // Triggers and events
    pub triggered: ThemeColor,
    pub trig_icon: ThemeColor,
    pub ambient_icon: ThemeColor,
    pub ambient_trig: ThemeColor,

    // Exits
    pub exit_visited: ThemeColor,
    pub exit_locked: ThemeColor,
    pub exit_unvisited: ThemeColor,

    // Feedback
    pub error: ThemeColor,
    pub error_icon: ThemeColor,
    pub denied: ThemeColor,

    // Goals
    pub goal_active: ThemeColor,
    pub goal_complete: ThemeColor,

    // UI sections
    pub section: ThemeColor,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "The default color scheme".to_string(),
            colors: ThemeColors::default(),
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            // Prompt and status
            prompt: ThemeColor::new(250, 200, 100),
            prompt_bg: ThemeColor::new(50, 51, 50),
            status: ThemeColor::new(20, 220, 100),

            // General styles
            highlight: ThemeColor::new(255, 255, 0), // yellow
            transition: ThemeColor::new(40, 210, 160),
            subheading: ThemeColor::new(255, 255, 255), // white for bold

            // Items
            item: ThemeColor::new(220, 180, 40),
            item_text: ThemeColor::new(40, 180, 40),

            // NPCs
            npc: ThemeColor::new(50, 200, 50),
            npc_quote: ThemeColor::new(100, 250, 250),
            npc_movement: ThemeColor::new(40, 180, 220),

            // Rooms
            room: ThemeColor::new(223, 77, 10),
            room_titlebar: ThemeColor::new(223, 77, 10),
            description: ThemeColor::new(102, 208, 250),
            overlay: ThemeColor::new(75, 180, 255),

            // Triggers and events
            triggered: ThemeColor::new(230, 230, 30),
            trig_icon: ThemeColor::new(230, 80, 80),
            ambient_icon: ThemeColor::new(80, 80, 230),
            ambient_trig: ThemeColor::new(150, 230, 30),

            // Exits
            exit_visited: ThemeColor::new(110, 220, 110),
            exit_locked: ThemeColor::new(200, 50, 50),
            exit_unvisited: ThemeColor::new(220, 180, 40),

            // Feedback
            error: ThemeColor::new(230, 30, 30),
            error_icon: ThemeColor::new(255, 0, 0), // bright red
            denied: ThemeColor::new(230, 30, 30),

            // Goals
            goal_active: ThemeColor::new(220, 40, 220),
            goal_complete: ThemeColor::new(110, 20, 110),

            // UI sections
            section: ThemeColor::new(75, 80, 75),
        }
    }
}

/// Container for theme data loaded from TOML
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeData {
    pub themes: Vec<Theme>,
}

/// Manages the catalog of available themes and the active selection.
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current_theme: Arc<RwLock<Theme>>,
}

impl ThemeManager {
    /// Create a new theme manager seeded with the built-in default theme.
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        let default_theme = Theme::default();
        themes.insert("default".to_string(), default_theme.clone());

        Self {
            themes,
            current_theme: Arc::new(RwLock::new(default_theme)),
        }
    }

    /// Load themes from a TOML file
    pub fn load_themes_from_file(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            // If the themes file doesn't exist, that's okay - we'll just use defaults
            return Ok(());
        }

        let contents = fs::read_to_string(path).with_context(|| format!("Failed to read themes file at {:?}", path))?;

        let theme_data: ThemeData = toml::from_str(&contents).with_context(|| "Failed to parse themes TOML")?;

        for theme in theme_data.themes {
            self.themes.insert(theme.name.clone(), theme);
        }

        Ok(())
    }

    /// Return an alphabetized list of installed theme names.
    pub fn list_themes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Switch to a different theme by name.
    pub fn set_theme(&self, name: &str) -> Result<()> {
        let theme = self
            .themes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Theme '{}' not found", name))?;

        let mut current = self
            .current_theme
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire theme lock"))?;
        *current = theme.clone();

        Ok(())
    }

    /// Get the currently active theme.
    pub fn current(&self) -> Arc<RwLock<Theme>> {
        Arc::clone(&self.current_theme)
    }

    /// Return the name of the active theme, falling back to `default`.
    pub fn current_name(&self) -> String {
        self.current_theme
            .read()
            .map(|t| t.name.clone())
            .unwrap_or_else(|_| "default".to_string())
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global theme manager shared across modules.
lazy_static::lazy_static! {
    pub static ref THEME_MANAGER: RwLock<ThemeManager> = RwLock::new(ThemeManager::new());
}

/// Initialize the theme system by loading themes from the data directory
pub fn init_themes() -> Result<()> {
    let themes_path = Path::new("amble_engine/data/themes.toml");

    let mut manager = THEME_MANAGER
        .write()
        .map_err(|_| anyhow::anyhow!("Failed to acquire theme manager lock"))?;

    manager.load_themes_from_file(themes_path)?;

    Ok(())
}

/// Snapshot the color palette for the active theme, defaulting if unavailable.
pub fn current_theme_colors() -> ThemeColors {
    THEME_MANAGER
        .read()
        .ok()
        .and_then(|m| {
            let current = m.current();
            current.read().ok().map(|t| t.colors.clone())
        })
        .unwrap_or_default()
}
