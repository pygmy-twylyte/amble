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
use crate::health::{LifeState, LivingEntity};
use crate::loader::load_world;
use crate::npc::{Npc, calculate_next_location, move_npc, move_scheduled};
use crate::scheduler::OnFalsePolicy;
use crate::spinners::CoreSpinnerType;
use crate::style::GameStyle;
use crate::trigger::{TriggerCondition, check_triggers, dispatch_action};
use crate::world::AmbleWorld;
use crate::{Item, View, ViewItem, WorldObject};

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::hash::BuildHasher;
use uuid::Uuid;
use variantly::Variantly;

use input::{InputEvent, InputManager};

const AUTOSAVE_TURNS: usize = 5;

/// Control flow signal used by handlers to exit the REPL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    let mut view = View::new();
    let mut input_manager = InputManager::new();
    let mut current_turn = 0;
    world.turn_count = 1;
    // ---- enter main game loop here ----
    loop {
        current_turn = turn_update(world, current_turn);
        let prompt = build_prompt(world);
        let Ok(input_event) = input_manager.read_line(&prompt) else {
            view.push(ViewItem::Error("Failed to read input. Try again.".red().to_string()));
            view.flush();
            continue;
        };

        let input = match input_event {
            InputEvent::Line(line) => line,
            InputEvent::Eof => "quit".to_string(),
            InputEvent::Interrupted => {
                view.push(ViewItem::EngineMessage("Command canceled.".to_string()));
                view.flush();
                continue;
            },
        };

        // not sure we need this anymore -- let's try commenting it out
        // if !input.ends_with('\n') {
        //     input.push('\n');
        // }

        // parse user input and dispatch an associated `Command`
        let command = parse_command(&input, &mut view);
        let dispatch_result = dispatch_command(&command, world, &mut view)?;
        if dispatch_result.control == ReplControl::Quit {
            view.flush();
            break;
        }
        if let Some(loaded_turn) = dispatch_result.turn {
            current_turn = loaded_turn;
        }

        // fire actions that only take place when a game turn is taken
        if world.turn_count > current_turn {
            // apply all pending health effects and fire any death triggers
            let (player_died, death_events) = run_health_effects(world, &mut view);
            if !death_events.is_empty() {
                check_triggers(world, &mut view, &death_events)?;
            }
            // if player died, flush the view, pause the REPL and ask how to continue
            if player_died {
                view.flush();
                if matches!(
                    handle_player_death(world, &mut view, &mut input_manager, &mut current_turn),
                    ReplControl::Quit
                ) {
                    break;
                }
                continue;
            }
            // move surviving npcs and fire scheduled events
            check_npc_movement(world, &mut view)?;
            check_scheduled_events(world, &mut view)?;
            // autosave if appropriate
            if world.turn_count.is_multiple_of(AUTOSAVE_TURNS)
                && let Err(err) = crate::repl::system::autosave_quiet(world, "autosave")
            {
                view.push(ViewItem::Error(format!("Autosave failed: {err}")));
            }
        }
        // ambient triggers may fire even if turn wasn't advanced
        check_ambient_triggers(world, &mut view)?;
        view.flush();
    }
    Ok(())
}

/// Result returned from the command dispatcher.
#[derive(Debug, Clone, PartialEq)]
struct DispatchResult {
    // Controls whether REPL should continue to run, or quit
    control: ReplControl,
    // Turn number update, if any (needed if game reloaded from file).
    turn: Option<usize>,
}

/// Dispatch a `Command` to its appropriate handler.
///
/// # Errors
/// - propagated from any of the underlying command handlers
fn dispatch_command(command: &Command, world: &mut AmbleWorld, view: &mut View) -> Result<DispatchResult> {
    #[allow(clippy::enum_glob_use)]
    use Command::*;
    let mut dr = DispatchResult {
        control: ReplControl::Continue,
        turn: None,
    };
    match &command {
        Touch(thing) => touch_handler(world, view, thing)?,
        SetViewMode(mode) => set_viewmode_handler(view, *mode),
        Goals => goals_handler(world, view),
        Help => help_handler(view),
        HelpDev => help_handler_dev(view),
        Quit => {
            if let ReplControl::Quit = quit_handler(world, view)? {
                dr.control = ReplControl::Quit;
            }
        },
        Look => look_handler(world, view)?,
        LookAt(thing) => look_at_handler(world, view, thing)?,
        GoBack => go_back_handler(world, view)?,
        MoveTo(direction) => move_to_handler(world, view, direction)?,
        Take(thing) => take_handler(world, view, thing)?,
        TakeFrom { item, container } => take_from_handler(world, view, item, container)?,
        Drop(thing) => drop_handler(world, view, thing)?,
        PutIn { item, container } => put_in_handler(world, view, item, container)?,
        Open(thing) => open_handler(world, view, thing)?,
        Close(thing) => close_handler(world, view, thing)?,
        LockItem(thing) => lock_handler(world, view, thing)?,
        UnlockItem(thing) => unlock_handler(world, view, thing)?,
        Inventory => inv_handler(world, view)?,
        ListSaves => list_saves_handler(view),
        Unknown => {
            view.push(ViewItem::Error(
                world
                    .spin_core(CoreSpinnerType::UnrecognizedCommand, "Didn't quite catch that?")
                    .italic()
                    .to_string(),
            ));
        },
        TalkTo(npc_name) => talk_to_handler(world, view, npc_name)?,
        GiveToNpc { item, npc } => give_to_npc_handler(world, view, item, npc)?,
        TurnOn(thing) => turn_on_handler(world, view, thing)?,
        TurnOff(thing) => turn_off_handler(world, view, thing)?,
        Read(thing) => read_handler(world, view, thing)?,
        Load(gamefile) => {
            if !load_handler(world, view, gamefile) {
                view.push(ViewItem::EngineMessage(
                    format!("- error loading world from '{gamefile}' -")
                        .error_style()
                        .to_string(),
                ));
            }
            // re-sync current turn to loaded game
            dr.turn = Some(world.turn_count.saturating_sub(1));
        },
        Save(gamefile) => save_handler(world, view, gamefile)?,
        Theme(theme_name) => theme_handler(view, theme_name)?,
        UseItemOn { verb, tool, target } => {
            use_item_on_handler(world, view, *verb, tool, target)?;
        },
        Ingest { item, mode } => ingest_handler(world, view, item, *mode)?,
        // Commands below only available when crate::DEV_MODE is enabled.
        SpawnItem(item_symbol) => dev_spawn_item_handler(world, view, item_symbol),
        Teleport(room_symbol) => dev_teleport_handler(world, view, room_symbol),
        ListNpcs => dev_list_npcs_handler(world, view),
        ListFlags => dev_list_flags_handler(world, view),
        ListSched => dev_list_sched_handler(world, view),
        SchedCancel(idx) => dev_sched_cancel_handler(world, view, *idx),
        SchedDelay { idx, turns } => dev_sched_delay_handler(world, view, *idx, *turns),
        AdvanceSeq(seq_name) => dev_advance_seq_handler(world, view, seq_name),
        ResetSeq(seq_name) => dev_reset_seq_handler(world, view, seq_name),
        SetFlag(flag_name) => dev_set_flag_handler(world, view, flag_name),
        StartSeq { seq_name, end } => dev_start_seq_handler(world, view, seq_name, end),
    }
    Ok(dr)
}

/// Updates current turn number and adds turn divider to the log.
fn turn_update(world: &mut AmbleWorld, mut turn: usize) -> usize {
    if world.turn_count > turn {
        turn += 1;
        let loc = world.player_room_ref().expect("player room should exist").name();
        info!(
            "\n====================> BEGIN TURN {turn} <====================\nLocation: '{loc}' | Health {}/{} | Score {}",
            world.player.current_hp(),
            world.player.max_hp(),
            world.player.score
        );
    }
    turn
}

/// Returns the input prompt according to current player/world state.
fn build_prompt(world: &mut AmbleWorld) -> String {
    let mut status_effects = String::new();
    for status in world.player.status() {
        let s = format!(" [{}]", status.status_style());
        status_effects.push_str(&s);
    }

    format!(
        "\n[Turn {} | Health {}/{} | Score: {}{}]>> ",
        world.turn_count,
        world.player.health.current_hp(),
        world.player.health.max_hp(),
        world.player.score,
        status_effects
    )
    .prompt_style()
    .to_string()
}

/// Apply and update health effects for all `LivingEntity` (player and NPCs)
fn run_health_effects(world: &mut AmbleWorld, view: &mut View) -> (bool, Vec<TriggerCondition>) {
    let mut health_view_items = Vec::new();
    let mut death_events = Vec::new();
    let mut player_died = false;

    let player_was_alive = matches!(world.player.life_state(), LifeState::Alive);
    let player_tick = world.player.tick_health_effects();
    health_view_items.extend(player_tick.view_items);
    if player_was_alive && matches!(world.player.life_state(), LifeState::Dead) {
        player_died = true;
        death_events.push(TriggerCondition::PlayerDeath);
        health_view_items.push(ViewItem::CharacterDeath {
            name: world.player.name().to_string(),
            cause: player_tick.death_cause,
            is_player: true,
        });
    }

    let npc_ids: Vec<Uuid> = world.npcs.keys().copied().collect();
    for npc_id in npc_ids {
        let was_alive = world
            .npcs
            .get(&npc_id)
            .is_some_and(|npc| matches!(npc.life_state(), LifeState::Alive));
        if let Some(npc) = world.npcs.get_mut(&npc_id) {
            let tick = npc.tick_health_effects();
            health_view_items.extend(tick.view_items);
            if was_alive && matches!(npc.life_state(), LifeState::Dead) {
                death_events.push(TriggerCondition::NpcDeath(npc_id));
                health_view_items.push(ViewItem::CharacterDeath {
                    name: npc.name().to_string(),
                    cause: tick.death_cause,
                    is_player: false,
                });
                if let Some(movement) = npc.movement.as_mut() {
                    movement.active = false;
                }
            }
        }
    }

    for item in health_view_items {
        view.push(item);
    }

    (player_died, death_events)
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
        if let Some(npc) = world.npcs.get_mut(&npc_id)
            && let Some(ref mut movement) = npc.movement
        {
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

/// Determine how to proceed when the player dies.
///
/// Returns a `ReplControl` variant to indicate whether the REPL should continue to run or exit.
fn handle_player_death(
    world: &mut AmbleWorld,
    view: &mut View,
    input_manager: &mut InputManager,
    current_turn: &mut usize,
) -> ReplControl {
    crate::repl::system::push_quit_summary(world, view);
    view.push(ViewItem::EngineMessage(
        "You have died. Type 'load <slot>', 'restart', or 'quit'.".to_string(),
    ));
    view.flush();

    loop {
        let prompt = "[dead] load <slot> | restart | quit >> ";
        let Ok(input_event) = input_manager.read_line(prompt) else {
            return ReplControl::Quit;
        };
        let mut line = match input_event {
            InputEvent::Line(line) => line,
            InputEvent::Eof | InputEvent::Interrupted => return ReplControl::Quit,
        };
        if !line.ends_with('\n') {
            line.push('\n');
        }
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("quit") {
            return ReplControl::Quit;
        }

        if trimmed.eq_ignore_ascii_case("restart") {
            match load_world() {
                Ok(mut new_world) => {
                    new_world.turn_count = 1;
                    *world = new_world;
                    *current_turn = world.turn_count.saturating_sub(1);
                    view.push(ViewItem::EngineMessage("Restarted from the beginning.".to_string()));
                    return ReplControl::Continue;
                },
                Err(err) => {
                    view.push(ViewItem::Error(format!("Failed to restart: {err}")));
                    view.flush();
                    continue;
                },
            }
        }

        if let Some(rest) = trimmed.strip_prefix("load") {
            let slot = rest.trim();
            let slot = if slot.is_empty() { "autosave" } else { slot };
            if crate::repl::system::load_handler(world, view, slot) {
                *current_turn = world.turn_count.saturating_sub(1);
                return ReplControl::Continue;
            }
            view.push(ViewItem::EngineMessage(format!(
                "Unable to resume from {}. Try another option.",
                slot.error_style()
            )));
        }

        view.push(ViewItem::EngineMessage(
            "Please enter 'load <slot>', 'restart', or 'quit'.".to_string(),
        ));
        view.flush();
    }
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
    world_items: &'a HashMap<Uuid, Item, S>,
    world_npcs: &'a HashMap<Uuid, Npc, S>,
    search_term: &str,
) -> Option<WorldEntity<'a>> {
    let lc_term = search_term.to_lowercase();
    for uuid in nearby_ids {
        if let Some(found_item) = world_items.get(uuid)
            && found_item.name().to_lowercase().contains(&lc_term)
        {
            return Some(WorldEntity::Item(found_item));
        }
        if let Some(found_npc) = world_npcs.get(uuid)
            && found_npc.name().to_lowercase().contains(&lc_term)
        {
            return Some(WorldEntity::Npc(found_npc));
        }
    }
    None
}

/// Feedback to player if an entity search comes up empty
pub fn entity_not_found(world: &AmbleWorld, view: &mut View, search_text: &str) {
    view.push(ViewItem::Error(format!(
        "\"{}\"? {}\n{}",
        search_text.error_style(),
        world.spin_core(CoreSpinnerType::EntityNotFound, "What's that?"),
        "(word not understood in context)".to_string().italic().dimmed()
    )));
}
