#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
//! ** Amble **
//! Adventure game / engine project

use amble_engine::style::GameStyle;
use amble_engine::{WorldObject, load_world, run_repl};

use anyhow::{Context, Result};
use colored::Colorize;

use log::info;

use std::fs;
use std::io::Write;

fn main() -> Result<()> {
    env_logger::init();
    info!("Start: loading Amble world...");
    let mut world = load_world().context("while loading AmbleWorld")?;
    info!("AmbleWorld loaded successfully.");

    // clear the screen
    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().unwrap();
    info!("Starting the game!");

    println!(
        "{:^84}",
        "AMBLE: AN ADVENTURE IN THE ABSURD"
            .bright_yellow()
            .underline()
    );
    println!(
        "\nYou are {}, {}\n",
        world.player.name().bold().bright_blue(),
        world.player.description()
    );

    let introduction = fs::read_to_string("amble_engine/data/intro.txt")?;
    println!("{}", introduction.description_style());

    run_repl(&mut world)
}
