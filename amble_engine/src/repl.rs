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
use log::info;
pub use look::*;
pub use movement::*;
pub use npc::*;
pub use system::*;

use crate::command::{Command, parse_command};
use crate::npc::{Npc, calculate_next_location, move_npc, move_scheduled};
use crate::spinners::CoreSpinnerType;
use crate::style::GameStyle;
use crate::trigger::{TriggerCondition, dispatch_action};
use crate::world::AmbleWorld;
use crate::{Item, View, ViewItem, WorldObject};

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
    use Command::*;
    let mut view = View::new();

    loop {
        let mut status_effects = String::new();
        for status in world.player.status() {
            let s = format!(" [{}]", status.status_style());
            status_effects.push_str(&s);
        }
        print!(
            "{}",
            format!(
                "\n[Rel: {}|Score: {}{}]>> ",
                world.turn_count, world.player.score, status_effects
            )
            .prompt_style()
        );
        io::stdout()
            .flush()
            .expect("failed to flush stdout before reading input");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            view.push(ViewItem::Error("Failed to read input. Try again.".red().to_string()));
            view.flush();
            continue;
        }

        match parse_command(&input, &mut view) {
            SetViewMode(mode) => set_viewmode_handler(&mut view, mode),
            Goals => goals_handler(world, &mut view),
            Help => help_handler(&mut view),
            Quit => {
                if let ReplControl::Quit = quit_handler(world, &mut view)? {
                    view.flush();
                    break;
                }
            },
            Look => look_handler(world, &mut view)?,
            LookAt(thing) => look_at_handler(world, &mut view, &thing)?,
            GoBack => go_back_handler(world, &mut view)?,
            MoveTo(direction) => move_to_handler(world, &mut view, &direction)?,
            Take(thing) => take_handler(world, &mut view, &thing)?,
            TakeFrom { item, container } => take_from_handler(world, &mut view, &item, &container)?,
            Drop(thing) => drop_handler(world, &mut view, &thing)?,
            PutIn { item, container } => put_in_handler(world, &mut view, &item, &container)?,
            Open(thing) => open_handler(world, &mut view, &thing)?,
            Close(thing) => close_handler(world, &mut view, &thing)?,
            LockItem(thing) => lock_handler(world, &mut view, &thing)?,
            UnlockItem(thing) => unlock_handler(world, &mut view, &thing)?,
            Inventory => inv_handler(world, &mut view)?,
            Unknown => {
                view.push(ViewItem::Error(
                    world
                        .spin_core(CoreSpinnerType::UnrecognizedCommand, "Didn't quite catch that?")
                        .italic()
                        .to_string(),
                ));
            },
            TalkTo(npc_name) => talk_to_handler(world, &mut view, &npc_name)?,
            Teleport(room_symbol) => dev_teleport_handler(world, &mut view, &room_symbol),
            SpawnItem(item_symbol) => dev_spawn_item_handler(world, &mut view, &item_symbol),
            GiveToNpc { item, npc } => give_to_npc_handler(world, &mut view, &item, &npc)?,
            TurnOn(thing) => turn_on_handler(world, &mut view, &thing)?,
            Read(thing) => read_handler(world, &mut view, &thing)?,
            Load(gamefile) => load_handler(world, &mut view, &gamefile),
            Save(gamefile) => save_handler(world, &mut view, &gamefile)?,
            Theme(theme_name) => theme_handler(&mut view, &theme_name)?,
            UseItemOn { verb, tool, target } => {
                use_item_on_handler(world, &mut view, verb, &tool, &target)?;
            },
            AdvanceSeq(seq_name) => dev_advance_seq_handler(world, &mut view, &seq_name),
            ResetSeq(seq_name) => dev_reset_seq_handler(world, &mut view, &seq_name),
            SetFlag(flag_name) => dev_set_flag_handler(world, &mut view, &flag_name),
            StartSeq { seq_name, end } => dev_start_seq_handler(world, &mut view, &seq_name, &end),
        }
        // We'll update turn count here in a centralized way, but this approach does not
        // take into account commands that return Ok(()) after failing to match a string.
        // Example: "take zpoon" (meant "spoon") -> "What's a zpoon?" response --> Ok(()).
        // That will count as a turn even though it's a no-op. If more granularity is needed,
        // individual command handlers will need to be modified.
        //
        // Only commands / actions that may be part of what's required to solve a puzzle or advance
        // the game count as a turn.
        let turn_taken = matches!(
            parse_command(&input, &mut view),
            Close(_)
                | Drop(_)
                | GiveToNpc { .. }
                | LookAt(_)
                | LockItem(_)
                | MoveTo(_)
                | Open(_)
                | PutIn { .. }
                | Read(_)
                | Take(_)
                | TakeFrom { .. }
                | TalkTo(_)
                | TurnOn(_)
                | UnlockItem(_)
                | UseItemOn { .. }
        );

        if turn_taken {
            world.turn_count += 1;
            info!("----> turn advanced to {} <----", world.turn_count);
            check_npc_movement(world, &mut view)?;
            check_scheduled_events(world, &mut view)?;
        }

        check_ambient_triggers(world, &mut view)?;
        view.flush();
    }
    Ok(())
}

/// Check the scheduler for any due events and fire them.
pub fn check_scheduled_events(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let now = world.turn_count;
    while let Some(event) = world.scheduler.pop_due(now) {
        info!(
            "scheduled event \"{}\" firing --->)",
            event.note.unwrap_or_else(|| "<no note recorded>".to_string())
        );
        for action in event.actions {
            dispatch_action(world, view, &action)?;
        }
    }
    Ok(())
}

/// Check to see if any NPCs are scheduled to move and move them.
/// # Errors
///
pub fn check_npc_movement(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let current_turn = world.turn_count;
    let npc_ids: Vec<Uuid> = world.npcs.keys().copied().collect();

    for npc_id in npc_ids {
        if let Some(npc) = world.npcs.get_mut(&npc_id) {
            if let Some(ref mut movement) = npc.movement {
                if !movement.active {
                    continue;
                }

                if move_scheduled(movement, current_turn) {
                    // just no-ops if next location == current
                    if let Some(new_location) = calculate_next_location(movement)
                        && npc.location != new_location
                    {
                        movement.last_moved_turn = current_turn;
                        move_npc(world, view, npc_id, new_location)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Check and fire any Ambient triggers that apply (runs each time around the REPL loop)
///
/// # Errors
/// - on failed lookup of player's location
pub fn check_ambient_triggers(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let current_room_id = world.player_room_ref()?.id();
    for trigger in &mut world.triggers {
        // if trigger was a one-off and already fired, skip it
        if trigger.fired && trigger.only_once {
            continue;
        }
        // if trigger has a Chance condition and it returns false, skip it
        if !chance_check(&trigger.conditions) {
            continue;
        }
        // push ViewItems for any ambient events that fire at this location
        for cond in &trigger.conditions {
            if let TriggerCondition::Ambient { room_ids, spinner } = cond {
                // empty = applies globally
                if room_ids.is_empty() || room_ids.contains(&current_room_id) {
                    let message = world.spinners.get(spinner).and_then(Spinner::spin).unwrap_or_default();
                    trigger.fired = true;
                    view.push(ViewItem::AmbientEvent(format!("{}", message.ambient_trig_style())));
                }
            }
        }
    }
    Ok(())
}

/// Returns true if there is no `Chance` within `conditions`, or if the `Chance` "roll" returns true.
/// Returns false only if there is a `Chance` condition present *and* it "rolls" false.
fn chance_check(conditions: &Vec<TriggerCondition>) -> bool {
    conditions.iter().all(|cond| cond.chance_value())
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
pub fn entity_not_found(world: &AmbleWorld, view: &mut View, search_text: &str) {
    view.push(ViewItem::Error(format!(
        "\"{}\"? {}",
        search_text.error_style(),
        world.spin_core(CoreSpinnerType::EntityNotFound, "What's that?")
    )));
}
