#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
//! ** Amble **
//! Adventure game / engine project

use amble_engine::{WorldObject, load_world, run_repl};

use anyhow::Result;
use colored::Colorize;

use log::{error, info};

use std::fs;
use std::io::Write;

fn main() -> Result<()> {
    env_logger::init();
    info!("Start: loading Amble world...");
    let mut world = match load_world() {
        Ok(world) => world,
        Err(e) => {
            error!("error loading AmbleWorld: {e}");
            return Err(e);
        }
    };
    info!("AmbleWorld loaded successfully.");
    info!("Starting the game!");
    // clears the screen
    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().unwrap();

    println!(
        "{:^80}",
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
    println!("{}", introduction.italic());

    run_repl(&mut world)
}
