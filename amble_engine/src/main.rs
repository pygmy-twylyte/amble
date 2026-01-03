#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

//! Command-line launcher for the Amble engine.
//!
//! Handles CLI startup, logging configuration, and world loading before
//! entering the interactive REPL.

use amble_engine::save_files::LOG_DIR;
use amble_engine::style::GameStyle;
use amble_engine::theme::init_themes;
use amble_engine::{AMBLE_VERSION, WorldObject, load_world, run_repl};

use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use log::{LevelFilter, info, warn};
use textwrap::{fill, termwidth};

use std::{
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

/// Initialize `env_logger` based on AMBLE_* environment variables.
fn init_logging() -> Result<()> {
    let Ok(raw_level) = env::var("AMBLE_LOG") else {
        return Ok(());
    };

    let trimmed = raw_level.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("off") {
        return Ok(());
    }

    let level = trimmed
        .parse::<LevelFilter>()
        .map_err(|_| anyhow!("invalid AMBLE_LOG value '{trimmed}'. Expected one of error, warn, info, debug, trace"))?;

    let mut builder = env_logger::Builder::new();
    builder.filter_level(level);
    builder.format_timestamp(None);

    let output_choice = env::var("AMBLE_LOG_OUTPUT").unwrap_or_else(|_| "file".to_string());

    match output_choice.to_ascii_lowercase().as_str() {
        "stderr" => {
            builder.target(env_logger::Target::Stderr);
        },
        "stdout" => {
            builder.target(env_logger::Target::Stdout);
        },
        _ => {
            let log_path = env::var_os("AMBLE_LOG_FILE")
                .map(PathBuf::from)
                .map_or_else(|| default_log_path().context("determining default log file path"), Ok)?;

            let mut ready = true;
            if let Some(parent) = log_path.parent() {
                if let Err(error) = fs::create_dir_all(parent) {
                    eprintln!(
                        "AMBLE_LOG: failed to create log directory {} ({error}). Falling back to stderr.",
                        parent.display()
                    );
                    ready = false;
                }
            }

            if ready {
                match OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&log_path)
                {
                    Ok(file) => {
                        builder.target(env_logger::Target::Pipe(Box::new(BufWriter::new(file))));
                        builder.write_style(env_logger::WriteStyle::Never);
                    },
                    Err(error) => {
                        eprintln!(
                            "AMBLE_LOG: failed to open log file {} ({error}). Falling back to stderr.",
                            log_path.display()
                        );
                        builder.target(env_logger::Target::Stderr);
                    },
                }
            } else {
                builder.target(env_logger::Target::Stderr);
            }
        },
    }

    builder
        .try_init()
        .map_err(|err| anyhow!("failed to initialize logger: {err}"))?;

    Ok(())
}

/// Derive a default log file path in the local logs directory.
fn default_log_path() -> Result<PathBuf> {
    Ok(PathBuf::from(LOG_DIR).join(format!("amble-{AMBLE_VERSION}.log")))
}

/// Entry point: loads content, initializes themes, and starts the REPL.
fn main() -> Result<()> {
    init_logging()?;
    info!("Starting Amble engine (version {AMBLE_VERSION})");
    info!("Start: loading game world from files");
    let mut world = load_world().context("while loading AmbleWorld")?;
    info!("AmbleWorld loaded successfully.");

    // Initialize the theme system
    if let Err(e) = init_themes() {
        warn!("Failed to load themes: {e}. Using default theme.");
    }

    // clear the screen
    print!("\x1B[2J\x1B[H");
    std::io::stdout()
        .flush()
        .expect("failed to flush stdout after clearing the screen");
    info!("Starting the game!");

    if !world.game_title.trim().is_empty() {
        println!(
            "{:^width$}",
            world.game_title.trim().bright_yellow().underline(),
            width = termwidth()
        );
    }

    println!(
        "{}",
        fill(
            format!(
                "\nYou are {}: {}\n",
                world.player.name().bold().blue(),
                world.player.description()
            )
            .as_str(),
            termwidth()
        )
    );

    if !world.intro_text.trim().is_empty() {
        println!("{}", fill(&world.intro_text, termwidth()).description_style());
    }

    run_repl(&mut world)
}
