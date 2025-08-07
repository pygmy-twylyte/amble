//! REPL and command handling utilities.
//!
//! The game runs in a read-eval-print loop. This module and its submodules
//! implement the various command handlers that manipulate the [`AmbleWorld`].

pub mod dev;
pub mod inventory;
pub mod item;
pub mod look;
pub mod movement;
pub mod npc;
pub mod system;

pub use dev::*;
use gametools::Spinner;
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
use crate::trigger::TriggerCondition;
use crate::world::AmbleWorld;
use crate::{Item, View, WorldObject};

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
/// - Returns an error if unable to flush stdout
///
/// # Panics
/// This function does not expect to panic.
pub fn run_repl(world: &mut AmbleWorld) -> Result<()> {
    let mut view = View::new();

    loop {
        view.reset();
        print!("\n[Score: {}/{}]> ", world.player.score, world.max_score);
        io::stdout()
            .flush()
            .expect("failed to flush stdout before reading input");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("{}", "Failed to read input. Try again.".red());
            continue;
        }
        println!();

        match parse_command(&input) {
            Command::Goals => goals_handler(world),
            Command::Help => help_handler(),
            Command::Quit => {
                if let ReplControl::Quit = quit_handler(world)? {
                    break;
                }
            },
            Command::Look => look_handler(world, &mut view)?,
            Command::LookAt(thing) => look_at_handler(world, &thing)?,
            Command::MoveTo(direction) => move_to_handler(world, &mut view, &direction)?,
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
            },
            Command::TalkTo(npc_name) => talk_to_handler(world, &npc_name)?,
            Command::Teleport(room_symbol) => dev_teleport_handler(world, &mut view, &room_symbol),
            Command::SpawnItem(item_symbol) => dev_spawn_item_handler(world, &item_symbol),
            Command::GiveToNpc { item, npc } => give_to_npc_handler(world, &item, &npc)?,
            Command::TurnOn(thing) => turn_on_handler(world, &thing)?,
            Command::Read(thing) => read_handler(world, &thing)?,
            Command::Load(gamefile) => load_handler(world, &gamefile),
            Command::Save(gamefile) => save_handler(world, &gamefile)?,
            Command::UseItemOn { verb, tool, target } => {
                use_item_on_handler(world, verb, &tool, &target)?;
            },
            Command::AdvanceSeq(seq_name) => dev_advance_seq_handler(world, &seq_name),
            Command::ResetSeq(seq_name) => dev_reset_seq_handler(world, &seq_name),
            Command::SetFlag(flag_name) => dev_set_flag_handler(world, &flag_name),
            Command::StartSeq { seq_name, end } => dev_start_seq_handler(world, &seq_name, &end),
        }
        check_ambient_triggers(world)?;
        view.flush();
    }
    Ok(())
}

/// Check and fire any Ambient triggers that apply (run each time around the REPL loop)
///
/// # Errors
/// - on failed lookup of player's location
pub fn check_ambient_triggers(world: &mut AmbleWorld) -> Result<()> {
    let current_room_id = world.player_room_ref()?.id();
    for trigger in &mut world.triggers {
        if trigger.fired && trigger.only_once {
            continue;
        }

        for cond in &trigger.conditions {
            if let TriggerCondition::Ambient { room_ids, spinner } = cond {
                // empty = applies globally
                if room_ids.is_empty() || room_ids.contains(&current_room_id) {
                    let message = world.spinners.get(spinner).and_then(Spinner::spin).unwrap_or_default();
                    if !message.is_empty() {
                        trigger.fired = true;
                        println!("\n{} {}", "❉".ambient_icon_style(), message.ambient_trig_style());
                    }
                }
            }
        }
    }
    Ok(())
}

/// Encapsulates references to different types of `WorldObjects` to allow search across different types.
#[derive(Debug, Variantly, Clone, Copy)]
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
