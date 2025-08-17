//! Amble Content Editor Library
//!
//! A comprehensive GTK4-based content editor for the Amble game engine.
//! This library provides all the functionality for editing game content
//! including rooms, items, NPCs, triggers, and more.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod app;
pub mod data;
pub mod ui;
pub mod utils;
pub mod validation;

// Re-export commonly used types
pub use app::{AmbleEditorApp, EditorState};
pub use data::{EntityReference, ProjectData};
pub use validation::{ValidationError, ValidationResult};

use anyhow::Result;
use log::info;
use std::path::Path;

/// Initialize the editor application
pub fn init() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Initializing Amble Content Editor");

    // Initialize GTK
    gtk4::init()?;
    adw::init()?;

    Ok(())
}

/// Load a project from the specified directory
pub fn load_project(path: &Path) -> Result<ProjectData> {
    info!("Loading project from: {:?}", path);
    ProjectData::load(path)
}

/// Create a new empty project
pub fn new_project() -> ProjectData {
    info!("Creating new empty project");
    ProjectData::new()
}

/// Get the version of the editor
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the application ID for GTK
pub const APP_ID: &str = "com.amble.ContentEditor";

/// Get the application name
pub const APP_NAME: &str = "Amble Content Editor";
