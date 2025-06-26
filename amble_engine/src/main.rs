#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
//! ** Amble **
//! Adventure game / engine project
pub mod command;
pub mod idgen;
pub mod item;
pub mod loader;
pub mod npc;
pub mod player;
pub mod repl;
pub mod room;
pub mod spinners;
pub mod style;
pub mod trigger;
pub mod world;

use crate::item::Item;
use crate::loader::load_world;
use crate::player::Player;
use crate::repl::run_repl;
use crate::room::Room;
use crate::world::AmbleWorld;

use anyhow::Result;
use colored::Colorize;

use log::{error, info};

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use uuid::Uuid;
use variantly::Variantly;

/// Kinds of places where an item may be located.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Variantly, PartialEq, Eq)]
pub enum Location {
    Item(Uuid),
    Inventory,
    #[default]
    Nowhere,
    Npc(Uuid),
    Room(Uuid),
}

/// Methods common to any object in the world.
pub trait WorldObject {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn location(&self) -> &Location;
}

/// Methods common to things that can hold items.
pub trait ItemHolder {
    fn add_item(&mut self, item_id: Uuid);
    fn remove_item(&mut self, item_id: Uuid);
    fn contains_item(&self, item_id: Uuid) -> bool;
}

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

    let introduction = fs::read_to_string("data/intro.txt")?;
    println!("{}", introduction.italic());

    run_repl(&mut world)
}
