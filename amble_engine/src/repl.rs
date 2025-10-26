//! REPL and command handling utilities.
//!
//! The game runs in a read-eval-print loop. This module and its submodules
//! implement the various command handlers that manipulate the [`AmbleWorld`].

pub mod dev;
mod input;
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
use crate::scheduler::OnFalsePolicy;
use crate::spinners::CoreSpinnerType;
use crate::style::GameStyle;
use crate::trigger::{TriggerCondition, dispatch_action};
use crate::world::AmbleWorld;
use crate::{Item, View, ViewItem, WorldObject};

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::hash::BuildHasher;
use uuid::Uuid;
use variantly::Variantly;

use input::{InputEvent, InputManager};

/// Control flow signal used by handlers to exit the REPL.
pub enum ReplControl {
    Continue,
    Quit,
}

/// Run the main read–eval–print loop until the user quits.
///
/// Handles prompting, command parsing, dispatching to the various handler modules,
/// and advancing world time. Returns when a handler signals `Quit`.
///
/// # Errors
/// - Propagates failures from handlers, such as a missing room for the player.
pub fn run_repl(world: &mut AmbleWorld) -> Result<()> {
    #[allow(clippy::enum_glob_use)]
    use Command::*;
    let mut view = View::new();

    let mut input_manager = InputManager::new();
    let mut current_turn = 0;
    world.turn_count = 1;
    loop {
        if world.turn_count > current_turn {
            current_turn += 1;
            info!("================> BEGIN TURN {current_turn} <================");
        }

        // collect status effects for prompt insertion
        let mut status_effects = String::new();
        for status in world.player.status() {
            let s = format!(" [{}]", status.status_style());
            status_effects.push_str(&s);
        }

        let prompt = format!(
            "\n[Turn: {}|Score: {}{}]>> ",
            world.turn_count, world.player.score, status_effects
        )
        .prompt_style()
        .to_string();

        let input_event = if let Ok(event) = input_manager.read_line(&prompt) {
            event
        } else {
            view.push(ViewItem::Error("Failed to read input. Try again.".red().to_string()));
            view.flush();
            continue;
        };

        let mut input = match input_event {
            InputEvent::Line(line) => line,
            InputEvent::Eof => "quit".to_string(),
            InputEvent::Interrupted => {
                view.push(ViewItem::EngineMessage("Command canceled.".to_string()));
                view.flush();
                continue;
            },
        };

        if !input.ends_with('\n') {
            input.push('\n');
        }

        let command = parse_command(&input, &mut view);
        match &command {
            Touch(thing) => touch_handler(world, &mut view, thing)?,
            SetViewMode(mode) => set_viewmode_handler(&mut view, *mode),
            Goals => goals_handler(world, &mut view),
            Help => help_handler(&mut view),
            HelpDev => help_handler_dev(&mut view),
            Quit => {
                if let ReplControl::Quit = quit_handler(world, &mut view)? {
                    view.flush();
                    break;
                }
            },
            Look => look_handler(world, &mut view)?,
            LookAt(thing) => look_at_handler(world, &mut view, thing)?,
            GoBack => go_back_handler(world, &mut view)?,
            MoveTo(direction) => move_to_handler(world, &mut view, direction)?,
            Take(thing) => take_handler(world, &mut view, thing)?,
            TakeFrom { item, container } => take_from_handler(world, &mut view, item, container)?,
            Drop(thing) => drop_handler(world, &mut view, thing)?,
            PutIn { item, container } => put_in_handler(world, &mut view, item, container)?,
            Open(thing) => open_handler(world, &mut view, thing)?,
            Close(thing) => close_handler(world, &mut view, thing)?,
            LockItem(thing) => lock_handler(world, &mut view, thing)?,
            UnlockItem(thing) => unlock_handler(world, &mut view, thing)?,
            Inventory => inv_handler(world, &mut view)?,
            ListSaves => list_saves_handler(&mut view),
            Unknown => {
                view.push(ViewItem::Error(
                    world
                        .spin_core(CoreSpinnerType::UnrecognizedCommand, "Didn't quite catch that?")
                        .italic()
                        .to_string(),
                ));
            },
            TalkTo(npc_name) => talk_to_handler(world, &mut view, npc_name)?,
            GiveToNpc { item, npc } => give_to_npc_handler(world, &mut view, item, npc)?,
            TurnOn(thing) => turn_on_handler(world, &mut view, thing)?,
            TurnOff(thing) => turn_off_handler(world, &mut view, thing)?,
            Read(thing) => read_handler(world, &mut view, thing)?,
            Load(gamefile) => load_handler(world, &mut view, gamefile),
            Save(gamefile) => save_handler(world, &mut view, gamefile)?,
            Theme(theme_name) => theme_handler(&mut view, theme_name)?,
            UseItemOn { verb, tool, target } => {
                use_item_on_handler(world, &mut view, *verb, tool, target)?;
            },
            Ingest { item, mode } => ingest_handler(world, &mut view, item, *mode)?,
            // Commands below only available when crate::DEV_MODE is enabled.
            SpawnItem(item_symbol) => dev_spawn_item_handler(world, &mut view, item_symbol),
            Teleport(room_symbol) => dev_teleport_handler(world, &mut view, room_symbol),
            ListNpcs => dev_list_npcs_handler(world, &mut view),
            ListFlags => dev_list_flags_handler(world, &mut view),
            ListSched => dev_list_sched_handler(world, &mut view),
            SchedCancel(idx) => dev_sched_cancel_handler(world, &mut view, *idx),
            SchedDelay { idx, turns } => dev_sched_delay_handler(world, &mut view, *idx, *turns),
            AdvanceSeq(seq_name) => dev_advance_seq_handler(world, &mut view, seq_name),
            ResetSeq(seq_name) => dev_reset_seq_handler(world, &mut view, seq_name),
            SetFlag(flag_name) => dev_set_flag_handler(world, &mut view, flag_name),
            StartSeq { seq_name, end } => dev_start_seq_handler(world, &mut view, seq_name, end),
        }

        // move NPCs and fire schedule events only if turn was advanced
        if world.turn_count > current_turn {
            check_npc_movement(world, &mut view)?;
            check_scheduled_events(world, &mut view)?;
        }

        // ambients fire regardless of whether a turn was taken
        check_ambient_triggers(world, &mut view)?;
        view.flush();
    }
    Ok(())
}

/// Check the scheduler for any due events and fire them.
///
/// # Errors
/// Returns an error if dispatching a scheduled trigger action fails.
pub fn check_scheduled_events(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let now = world.turn_count;
    while let Some(event) = world.scheduler.pop_due(now) {
        let note_text = event.note.clone().unwrap_or_else(|| "<no note recorded>".to_string());
        let ok = event.condition.as_ref().is_none_or(|c| c.eval(world));

        if ok {
            info!("scheduled event \"{note_text}\" firing --->)");
            for action in event.actions {
                dispatch_action(world, view, &action)?;
            }
        } else {
            match event.on_false {
                OnFalsePolicy::Cancel => {
                    info!("scheduled event \"{note_text}\" canceled (condition false)");
                },
                OnFalsePolicy::RetryAfter(dt) => {
                    let new_turn = now.saturating_add(dt);
                    world.scheduler.schedule_on_if(
                        new_turn,
                        event.condition.clone(),
                        event.on_false.clone(),
                        event.actions.clone(),
                        event.note.clone(),
                    );
                    info!("scheduled event \"{note_text}\" rescheduled for turn {new_turn} (RetryAfter {dt})");
                },
                OnFalsePolicy::RetryNextTurn => {
                    let new_turn = now.saturating_add(1);
                    world.scheduler.schedule_on_if(
                        new_turn,
                        event.condition.clone(),
                        event.on_false.clone(),
                        event.actions.clone(),
                        event.note.clone(),
                    );
                    info!("scheduled event \"{note_text}\" rescheduled for next turn {new_turn}");
                },
            }
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
                // skip if NPC is inactive
                if !movement.active {
                    continue;
                }

                // clear any set pause if expired
                if let Some(resume_turn) = movement.paused_until {
                    if current_turn >= resume_turn {
                        info!("movement pause expiring for npc '{}': resuming activity", npc.symbol);
                        movement.paused_until = None;
                    } else {
                        info!(
                            "npc '{}' is paused for player interaction, skipping movement check",
                            npc.symbol
                        );
                        continue;
                    }
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
fn chance_check(conditions: &[TriggerCondition]) -> bool {
    conditions
        .iter()
        .all(super::trigger::condition::TriggerCondition::chance_value)
}

/// Encapsulates references to different types of `WorldObjects` to allow search across different types.
#[derive(Debug, Variantly, Clone, Copy)]
pub enum WorldEntity<'a> {
    Item(&'a Item),
    Npc(&'a Npc),
}
impl WorldEntity<'_> {
    /// Get the name of the entity
    pub fn name(&self) -> &str {
        match self {
            WorldEntity::Item(item) => item.name(),
            WorldEntity::Npc(npc) => npc.name(),
        }
    }
    /// Get the UUID of the entity
    pub fn id(&self) -> Uuid {
        match self {
            WorldEntity::Item(item) => item.id(),
            WorldEntity::Npc(npc) => npc.id(),
        }
    }
    /// Get the symbol for the entity
    pub fn symbol(&self) -> &str {
        match self {
            WorldEntity::Item(item) => item.symbol(),
            WorldEntity::Npc(npc) => npc.symbol(),
        }
    }
}

/// Searches a list of `WorldEntities`' uuids to find a `WorldObject` with a matching name.
/// Returns Some(&'a `WorldEntity`) or None.
pub fn find_world_object<'a, S: BuildHasher>(
    nearby_ids: impl IntoIterator<Item = &'a Uuid>,
    world_items: &'a HashMap<Uuid, Item>,
    world_npcs: &'a HashMap<Uuid, Npc, S>,
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
