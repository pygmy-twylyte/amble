pub mod inventory;
pub mod item;
pub mod look;
pub mod movement;
pub mod npc;
pub mod system;

pub use inventory::*;
pub use item::*;
pub use look::*;
pub use movement::*;
pub use npc::*;
pub use system::*;

use crate::command::{Command, parse_command};
use crate::npc::Npc;
use crate::spinners::SpinnerType;
use crate::style::GameStyle;
use crate::world::AmbleWorld;
use crate::{Item, WorldObject};

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::io::{self, Write};
use uuid::Uuid;
use variantly::Variantly;

/// Enum used to control exit from the repl loop.
pub enum ReplControl {
    Continue,
    Quit,
}

/// Run the main command loop for the game.
/// # Errors
/// - Returns an error if unable to resolve the location (Room) of the player
/// # Panics
/// - Could panic if unable to flush stdout for some reason
pub fn run_repl(world: &mut AmbleWorld) -> Result<()> {
    loop {
        print!("\n[Score: {}/??]> ", world.player.score);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("{}", "Failed to read input. Try again.".red());
            continue;
        }
        println!();

        match parse_command(&input) {
            Command::Help => help_handler()?,
            Command::Quit => {
                if let ReplControl::Quit = quit_handler(world)? {
                    break;
                }
            }
            Command::Look => look_handler(world)?,
            Command::LookAt(thing) => look_at_handler(world, &thing)?,
            Command::MoveTo(direction) => move_to_handler(world, &direction)?,
            Command::Take(thing) => take_handler(world, &thing)?,
            Command::TakeFrom { item, container } => take_from_handler(world, &item, &container)?,
            Command::Drop(thing) => drop_handler(world, &thing)?,
            Command::PutIn { item, container } => put_in_handler(world, &item, &container)?,
            Command::Open(thing) => open_handler(world, &thing)?,
            Command::Close(thing) => close_handler(world, &thing)?,
            Command::LockItem(thing) => lock_handler(world, &thing)?,
            Command::UnlockItem(thing) => unlock_handler(world, &thing)?,
            Command::Inventory => inv_handler(world)?,
            Command::Unknown => {
                println!(
                    "{}",
                    world
                        .spin_spinner(SpinnerType::UnrecognizedCommand, "Didn't quite catch that?")
                        .italic()
                );
            }
            Command::TalkTo(npc_name) => talk_to_handler(world, &npc_name)?,
            Command::Teleport(room_toml_id) => teleport_handler(world, &room_toml_id), // only for development
            Command::GiveToNpc { item, npc } => give_to_npc_handler(world, &item, &npc)?,
            Command::TurnOn(thing) => turn_on_handler(world, &thing)?,
            Command::Read(thing) => read_handler(world, &thing)?,
            Command::Load(gamefile) => load_handler(world, &gamefile)?,
            Command::Save(gamefile) => save_handler(world, &gamefile)?,
            Command::UseItemOn { verb, tool, target } => {
                use_item_on_handler(world, verb, &tool, &target)?;
            }
        }
    }
    Ok(())
}

/// Encapsulates references to different types of `WorldObjects` to allow search across different types.
#[derive(Debug, Variantly, Clone)]
pub enum WorldEntity<'a> {
    Item(&'a Item),
    Npc(&'a Npc),
}

/// Searches a list of `WorldEntities`' uuids to find a `WorldObject` with a matching name.
/// Returns Some(&'a `WorldEntity`) or None.
pub fn find_world_object<'a>(
    nearby_ids: impl IntoIterator<Item = &'a Uuid>,
    world_items: &'a HashMap<Uuid, Item>,
    world_npcs: &'a HashMap<Uuid, Npc>,
    search_term: &str,
) -> Option<WorldEntity<'a>> {
    let lc_term = search_term.to_lowercase();
    for uuid in nearby_ids {
        if let Some(found_item) = world_items.get(uuid) {
            if found_item.name().to_lowercase().contains(&lc_term) {
                return Some(WorldEntity::Item(found_item));
            }
        }
        if let Some(found_npc) = world_npcs.get(uuid) {
            if found_npc.name().to_lowercase().contains(&lc_term) {
                return Some(WorldEntity::Npc(found_npc));
            }
        }
    }
    None
}

/// Feedback to player if an entity search comes up empty
fn entity_not_found(world: &AmbleWorld, search_text: &str) {
    println!(
        "\"{}\"? {}",
        search_text.error_style(),
        world.spin_spinner(SpinnerType::EntityNotFound, "What's that?")
    );
}
