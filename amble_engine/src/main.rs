#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

//! Command-line launcher for the Amble engine.
//!
//! Handles CLI startup, logging configuration, and world loading before
//! entering the interactive REPL.

use amble_engine::save_files::{
    LOG_DIR, SAVE_DIR, SaveFileEntry, build_save_entries_recursive, format_modified, load_save_file,
    save_dir_for_world, set_active_save_dir,
};
use amble_engine::style::GameStyle;
use amble_engine::theme::init_themes;
use amble_engine::{
    AMBLE_VERSION, WorldObject, WorldSource, discover_world_sources, load_world_from_path, run_repl,
    set_active_world_path,
};

use anyhow::{Context, Result, anyhow, bail};
use colored::Colorize;
use log::{LevelFilter, info, warn};
use textwrap::{fill, termwidth};

use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
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

#[derive(Debug, Clone, Copy)]
enum StartupSelection {
    World(usize),
    Save(usize),
}

fn select_startup(worlds: &[WorldSource], saves: &[SaveFileEntry]) -> Result<StartupSelection> {
    if worlds.len() == 1 && saves.is_empty() {
        return Ok(StartupSelection::World(0));
    }

    if worlds.is_empty() && saves.is_empty() {
        bail!("no worlds or saved games found");
    }

    println!("{}", "Choose a world or save:".bright_yellow());
    let mut options = Vec::new();

    if !worlds.is_empty() {
        println!("{}", "Worlds".bold());
        for (idx, world) in worlds.iter().enumerate() {
            let option_idx = options.len() + 1;
            print_world_option(option_idx, world);
            options.push(StartupSelection::World(idx));
        }
        println!();
    }

    if !saves.is_empty() {
        println!("{}", "Saved Games".bold());
        for (idx, entry) in saves.iter().enumerate() {
            let option_idx = options.len() + 1;
            print_save_option(option_idx, entry);
            options.push(StartupSelection::Save(idx));
        }
        println!();
    }

    let chosen = prompt_selection(options.len())?;
    Ok(options[chosen - 1])
}

fn print_world_option(index: usize, world: &WorldSource) {
    let mut details = Vec::new();
    if !world.author.trim().is_empty() {
        details.push(format!("by {}", world.author.npc_style()));
    }
    if !world.version.trim().is_empty() {
        details.push(format!("v{}", world.version.bold()));
    }
    let detail_str = if details.is_empty() {
        String::new()
    } else {
        format!(" ({})", details.join(", "))
    };

    println!("  {index}. {}{detail_str}", world.title.highlight());

    let blurb = world.blurb.trim();
    if !blurb.is_empty() {
        let width = termwidth().saturating_sub(6);
        let wrapped = fill(blurb, width);
        for line in wrapped.lines() {
            println!("      {}", line.description_style());
        }
    }
}

fn print_save_option(index: usize, entry: &SaveFileEntry) {
    let modified = entry.modified.map(format_modified);
    if let Some(summary) = &entry.summary {
        let world_label = if summary.world_title.trim().is_empty() {
            "Unknown world"
        } else {
            summary.world_title.as_str()
        };
        let suffix = if let Some(modified) = modified {
            format!(" — saved {modified}")
        } else {
            String::new()
        };
        println!(
            "  {index}. Load save: {} — {} (turn {}, score {}){suffix}",
            entry.slot.item_style(),
            world_label.highlight(),
            summary.turn_count.to_string().bold(),
            summary.score.to_string().bold()
        );
    } else {
        let suffix = if let Some(modified) = modified {
            format!(" — saved {modified}")
        } else {
            String::new()
        };
        println!("  {index}. Load save: {} (metadata unavailable){suffix}", entry.slot);
    }
}

fn prompt_selection(max: usize) -> Result<usize> {
    let mut line = String::new();
    loop {
        print!("Select a world or save [1-{max}] (Enter=1): ");
        io::stdout().flush()?;
        line.clear();
        io::stdin().read_line(&mut line)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(1);
        }
        if let Ok(choice) = trimmed.parse::<usize>() {
            if (1..=max).contains(&choice) {
                return Ok(choice);
            }
        }
        println!("Invalid selection. Please enter a number between 1 and {max}.");
    }
}

fn match_world_for_save<'a>(entry: &SaveFileEntry, worlds: &'a [WorldSource]) -> Option<&'a WorldSource> {
    let summary = entry.summary.as_ref()?;
    if !summary.world_slug.trim().is_empty() {
        if let Some(world) = worlds.iter().find(|world| world.slug == summary.world_slug) {
            return Some(world);
        }
    }
    if !summary.world_title.trim().is_empty() {
        return worlds.iter().find(|world| world.title == summary.world_title);
    }
    None
}

/// Entry point: loads content, initializes themes, and starts the REPL.
fn main() -> Result<()> {
    init_logging()?;
    info!("Starting Amble engine (version {AMBLE_VERSION})");
    info!("Start: loading game world from files");
    let saves = match build_save_entries_recursive(Path::new(SAVE_DIR)) {
        Ok(entries) => entries,
        Err(err) => {
            warn!("Failed to scan saved games: {err}");
            Vec::new()
        },
    };
    let (worlds, worlds_err) = match discover_world_sources() {
        Ok(entries) => (entries, None),
        Err(err) => {
            warn!("Failed to scan world files: {err}");
            (Vec::new(), Some(err))
        },
    };
    if worlds.is_empty() && saves.is_empty() {
        if let Some(err) = worlds_err {
            return Err(err).context("no worlds or saves available");
        }
        bail!("no worlds or saves available");
    }
    let selection = select_startup(&worlds, &saves).context("while selecting a world")?;
    let mut world = match selection {
        StartupSelection::World(idx) => {
            let world_source = &worlds[idx];
            set_active_world_path(world_source.path.clone());
            load_world_from_path(&world_source.path).context("while loading world")?
        },
        StartupSelection::Save(idx) => {
            let entry = &saves[idx];
            let mut world = load_save_file(&entry.path).context("while loading save file")?;
            if let Some(world_source) = match_world_for_save(entry, &worlds) {
                set_active_world_path(world_source.path.clone());
            }
            if world.game_title.trim().is_empty() {
                world.game_title = "Loaded Save".to_string();
            }
            world
        },
    };
    set_active_save_dir(save_dir_for_world(&world));
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
