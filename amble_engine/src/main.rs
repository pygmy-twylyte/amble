#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

//! ** Amble **
//! Adventure game / engine project

use amble_engine::style::GameStyle;
use amble_engine::{AMBLE_VERSION, WorldObject, load_world, run_repl};

use anyhow::{Context, Result};
use colored::Colorize;
use textwrap::{fill, termwidth};

use log::info;

use std::fs;
use std::io::Write;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Amble (version {AMBLE_VERSION})");
    info!("Start: loading 'AmbleWorld' from files");
    let mut world = load_world().context("while loading AmbleWorld")?;
    info!("AmbleWorld loaded successfully.");

    // clear the screen
    print!("\x1B[2J\x1B[H");
    std::io::stdout()
        .flush()
        .expect("failed to flush stdout after clearing the screen");
    info!("Starting the game!");

    println!(
        "{:^width$}",
        "AMBLE: AN ADVENTURE".bright_yellow().underline(),
        width = termwidth()
    );
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

    let introduction = include_str!("../data/intro.txt");
    println!("{}", fill(introduction, termwidth()).description_style());

    run_repl(&mut world)
}
