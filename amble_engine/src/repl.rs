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
use crate::npc::{calculate_next_location, move_npc, move_scheduled};
use crate::scheduler::{EventCondition, OnFalsePolicy, ScheduledEvent};
use crate::spinners::CoreSpinnerType;
use crate::style::GameStyle;
use crate::trigger::{TriggerCondition, check_triggers, dispatch_action};
use crate::world::AmbleWorld;
use crate::{Location, NpcId, RoomId, View, ViewItem, WorldObject};
use anyhow::{Result, bail};
use colored::Colorize;

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
    world.turn_count = world.turn_count.max(1);
    let mut turn_log_state = TurnLogState::new(world);
    // ---- enter main game loop here ----
    loop {
        turn_log_state.log_if_advanced(world)?;

        let Some(command) = obtain_command(world, &mut view, &mut input_manager)? else {
            continue;
        };

        let dispatch_result = dispatch_command(&command, world, &mut view)?;
        if dispatch_result.control == ReplControl::Quit {
            view.flush();
            break;
        }
        if dispatch_result.world_reloaded {
            turn_log_state.resync(world);
        }

        if dispatch_result.turn_advanced {
            match run_timed_events(world, &mut view, &mut input_manager)? {
                TimedEventsResult::Continue => {},
                TimedEventsResult::Quit => break,
                TimedEventsResult::WorldReloaded => {
                    turn_log_state.resync(world);
                    view.flush();
                    continue;
                },
            }
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

/// Returns the input prompt according to current player/world state.
fn build_prompt(world: &AmbleWorld) -> String {
    let mut status_effects = String::new();
    for status in world.player.status() {
        let s = format!(" [{}]", status.status_style());
        status_effects.push_str(&s);
    }

    format!(
        "\n[Turn {} | Health {}/{} | Score: {}{}]\n→ ",
        world.turn_count,
        world.player.health.current_hp(),
        world.player.health.max_hp(),
        world.player.score,
        status_effects
    )
    .prompt_style()
    .to_string()
}

/// Prompts and returns the raw command input from the user.
fn get_user_input(view: &mut View, input_manager: &mut InputManager, prompt: String) -> Option<String> {
    let Ok(input_event) = input_manager.read_line(&prompt) else {
        view.push(ViewItem::Error("Failed to read input. Try again.".red().to_string()));
        view.flush();
        return None;
    };
    let input = match input_event {
        InputEvent::Line(line) => line,
        InputEvent::Eof => "quit".to_string(),
        InputEvent::Interrupted => {
            view.push(ViewItem::EngineMessage("Command canceled.".to_string()));
            view.flush();
            return None;
        },
    };
    Some(input)
}

/// Checks for an abbreviated (single character) command input and expand it as appropriate
fn expand_abbreviated_input(input: &str) -> &str {
    match input {
        "l" => "look",
        "q" => "quit",
        "u" => "go up",
        "d" => "go down",
        "n" => "go north",
        "e" => "go east",
        "w" => "go west",
        "s" => "go south",
        _ => input,
    }
}

/// Used as a fallback to check if user input is a shortcut for an exit direction, e.g. "rope" -> "down the rope ladder".
/// Returns the appropriate movement command if a single unambiguous exit match is found, otherwise None.
fn check_for_exit_fallback<'a>(input: &str, directions: impl Iterator<Item = &'a String>) -> Option<Command> {
    let lc_input = input.to_lowercase();
    let dir_matches: Vec<_> = directions.filter(|d| d.to_lowercase().contains(&lc_input)).collect();
    if dir_matches.len() != 1 {
        return None;
    }
    Some(Command::MoveTo(dir_matches[0].clone()))
}

/// Parses input text into a Command variant, falling back to a MoveTo command if one of the
/// current room's exits matches the input.
fn parse_with_exit_fallback(world: &AmbleWorld, view: &mut View, input: String) -> Result<Command> {
    let input = expand_abbreviated_input(&input);
    let mut command = parse_command(input, view);
    if matches!(command, Command::Unknown)
        && let Some(move_to_cmd) = check_for_exit_fallback(input, world.player_room_ref()?.exits.keys())
    {
        command = move_to_cmd;
    }
    Ok(command)
}

/// Obtains input from the player and parses it into a Command variant.
///
/// Returns Ok(None) if no user input is obtained / command is cancelled with ctrl-c.
/// Returns Ok(Some(Command)) if a Command variant could be constructed from user input.
///
/// # Errors
/// - if player's room is invalid (propagated from `parse_with_exit_fallback()`)
fn obtain_command(world: &AmbleWorld, view: &mut View, input_mgr: &mut InputManager) -> Result<Option<Command>> {
    let Some(input) = get_user_input(view, input_mgr, build_prompt(world)) else {
        return Ok(None);
    };
    let command = parse_with_exit_fallback(world, view, input.clone())?;
    info!("parsed input \"{input}\" ⇒ Command::{command:?}");
    Ok(Some(command))
}

/// Result returned from the command dispatcher.
#[derive(Debug, Clone, PartialEq)]
struct DispatchResult {
    // Controls whether REPL should continue to run, or quit
    control: ReplControl,
    // True if the world was replaced (e.g. via load command), requiring turn-log resync.
    world_reloaded: bool,
    // True if the turn # was advanced this time around the REPL
    turn_advanced: bool,
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
        world_reloaded: false,
        turn_advanced: false,
    };

    match &command {
        Touch(thing) => dr.turn_advanced = touch_handler(world, view, thing)?,
        SetViewMode(mode) => set_viewmode_handler(view, *mode),
        Goals => goals_handler(world, view),
        Help => help_handler(view),
        HelpDev => help_handler_dev(view),
        Quit => {
            if let ReplControl::Quit = quit_handler(world, view)? {
                dr.control = ReplControl::Quit;
            }
        },
        Look => dr.turn_advanced = look_handler(world, view)?,
        LookAt(thing) => dr.turn_advanced = look_at_handler(world, view, thing)?,
        GoBack => dr.turn_advanced = go_back_handler(world, view)?,
        MoveTo(direction) => dr.turn_advanced = move_to_handler(world, view, direction)?,
        Take(thing) => dr.turn_advanced = take_handler(world, view, thing)?,
        TakeFrom { item, container } => {
            dr.turn_advanced = take_from_handler(world, view, item, container)?;
        },
        Drop(thing) => dr.turn_advanced = drop_handler(world, view, thing)?,
        PutIn { item, container } => {
            dr.turn_advanced = put_in_handler(world, view, item, container)?;
        },
        Open(thing) => dr.turn_advanced = open_handler(world, view, thing)?,
        Close(thing) => dr.turn_advanced = close_handler(world, view, thing)?,
        LockItem(thing) => dr.turn_advanced = lock_handler(world, view, thing)?,
        UnlockItem(thing) => dr.turn_advanced = unlock_handler(world, view, thing)?,
        Inventory => inv_handler(world, view)?,
        ListSaves => list_saves_handler(world, view),
        Unknown => {
            view.push(ViewItem::Error(
                world
                    .spin_core(CoreSpinnerType::UnrecognizedCommand, "Didn't quite catch that?")
                    .italic()
                    .to_string(),
            ));
        },
        TalkTo(npc_name) => {
            dr.turn_advanced = talk_to_handler(world, view, npc_name)?;
        },
        GiveToNpc { item, npc } => {
            dr.turn_advanced = give_to_npc_handler(world, view, item, npc)?;
        },
        TurnOn(thing) => dr.turn_advanced = turn_on_handler(world, view, thing)?,
        TurnOff(thing) => dr.turn_advanced = turn_off_handler(world, view, thing)?,
        Read(thing) => dr.turn_advanced = read_handler(world, view, thing)?,
        Load(gamefile) => {
            if load_handler(world, view, gamefile) {
                dr.world_reloaded = true;
            } else {
                view.push(ViewItem::EngineMessage(
                    format!("- error loading world from '{gamefile}' -")
                        .error_style()
                        .to_string(),
                ));
            }
        },
        Save(gamefile) => save_handler(world, view, gamefile)?,
        Theme(theme_name) => theme_handler(view, theme_name)?,
        UseItemOn { verb, tool, target } => {
            dr.turn_advanced = use_item_on_handler(world, view, *verb, tool, target)?;
        },
        Ingest { item, mode } => {
            dr.turn_advanced = ingest_handler(world, view, item, *mode)?;
        },
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
        DevNote(note) => dev_note_handler(world, view, note),
        StartSeq { seq_name, end } => dev_start_seq_handler(world, view, seq_name, end),
    }
    if dr.turn_advanced {
        world.turn_count = world.turn_count.saturating_add(1);
    }
    Ok(dr)
}

/// Tracks when the REPL needs to emit a turn-divider header to the game log.
struct TurnLogState {
    last_logged_turn: usize,
}

impl TurnLogState {
    fn new(world: &AmbleWorld) -> Self {
        Self {
            last_logged_turn: world.turn_count.saturating_sub(1),
        }
    }

    /// Resync tracker after replacing world state (load/restart).
    fn resync(&mut self, world: &AmbleWorld) {
        self.last_logged_turn = world.turn_count.saturating_sub(1);
    }

    /// Emit turn-divider header once when world turn advances.
    fn log_if_advanced(&mut self, world: &AmbleWorld) -> Result<()> {
        if world.turn_count > self.last_logged_turn {
            self.last_logged_turn = world.turn_count;
            let loc = world.player_room_ref()?.name();
            let turn = world.turn_count;
            info!(
                "\n====================> BEGIN TURN {turn} <====================\nLocation: '{loc}' | Health {}/{} | Score {}",
                world.player.current_hp(),
                world.player.max_hp(),
                world.player.score
            );
        }
        Ok(())
    }
}

/// Overall result of running all timed events for this turn.
enum TimedEventsResult {
    /// Continue looping REPL as usual
    Continue,
    /// Quit game (may be chosen if player is killed by a timed event)
    Quit,
    /// World state has been reloaded from file (after player death)
    WorldReloaded,
}

/// Process any turn-based events that are due this turn.
/// - ticks health effects (damage/heal over time) and handles any deaths
/// - moves NPCs scheduled to do so
/// - fires due events from the scheduler
fn run_timed_events(
    world: &mut AmbleWorld,
    view: &mut View,
    input_mgr: &mut InputManager,
) -> Result<TimedEventsResult> {
    let (player_died, death_events) = run_health_effects(world, view);
    if !death_events.is_empty() {
        check_triggers(world, view, &death_events)?;
    }
    if player_died {
        view.flush();
        return match handle_player_death(world, view, input_mgr) {
            DeathReaction::WorldReloaded => Ok(TimedEventsResult::WorldReloaded),
            DeathReaction::Quit => Ok(TimedEventsResult::Quit),
        };
    }
    // move surviving npcs and fire scheduled events
    tick_npc_movement(world, view)?;
    check_scheduled_events(world, view)?;
    Ok(TimedEventsResult::Continue)
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

    let npc_ids: Vec<NpcId> = world.npcs.keys().cloned().collect();
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

/// User's response to the death prompt.
enum DeathReaction {
    /// Game loaded/restarted and world state was replaced.
    WorldReloaded,
    /// User wants to quit.
    Quit,
}

/// Notifies the player that their character has died and prompts for a decision
/// on how to continue.
///
/// Returns a `DeathReaction` variant to indicate how the REPL should continue.
fn handle_player_death(world: &mut AmbleWorld, view: &mut View, input_manager: &mut InputManager) -> DeathReaction {
    crate::repl::system::push_quit_summary(world, view);
    view.push(ViewItem::EngineMessage(
        "You have died. Type 'load <slot>', 'restart', or 'quit'.".to_string(),
    ));
    view.flush();

    loop {
        let prompt = "[dead] load <slot> | restart | quit >> ";
        let Ok(input_event) = input_manager.read_line(prompt) else {
            return DeathReaction::Quit;
        };
        let mut line = match input_event {
            InputEvent::Line(line) => line,
            InputEvent::Eof | InputEvent::Interrupted => return DeathReaction::Quit,
        };
        if !line.ends_with('\n') {
            line.push('\n');
        }
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("quit") {
            return DeathReaction::Quit;
        }

        if trimmed.eq_ignore_ascii_case("restart") {
            match load_world() {
                Ok(mut new_world) => {
                    new_world.turn_count = 1;
                    crate::save_files::set_active_save_dir(crate::save_files::save_dir_for_world(&new_world));
                    *world = new_world;
                    view.push(ViewItem::EngineMessage("Restarted from the beginning.".to_string()));
                    return DeathReaction::WorldReloaded;
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
                return DeathReaction::WorldReloaded;
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
            apply_on_false_policy(world, now, note_text, event);
        }
    }
    Ok(())
}

/// Applies the policy (retry, cancel, etc) for the given conditional event in the case that its conditions are false.
fn apply_on_false_policy(world: &mut AmbleWorld, now: usize, note_text: String, event: ScheduledEvent) {
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

struct MovementPlan {
    moves: Vec<(NpcId, Location)>,
}

/// Create a `MovementPlan` listing which NPCs need to move, and their destinations.
fn create_movement_plan(world: &mut AmbleWorld) -> MovementPlan {
    let moves = world.npcs.values_mut().fold(Vec::new(), |mut plan, npc| {
        if let Some(ref mut move_opts) = npc.movement
            && move_opts.active
            && move_opts.paused_until.is_none()
            && move_scheduled(move_opts, world.turn_count)
        {
            if let Some(destination) = calculate_next_location(move_opts)
                && npc.location != destination
            {
                plan.push((npc.id.clone(), destination));
            }
        }
        plan
    });
    MovementPlan { moves }
}

/// Check to see if any NPCs are scheduled to move and move them.
/// # Errors
///
pub fn tick_npc_movement(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let planned = create_movement_plan(world);
    planned.moves.iter().try_for_each(|(npc_id, destination)| {
        let Some(movement) = world.npcs.get_mut(npc_id).and_then(|npc| npc.movement.as_mut()) else {
            bail!("movement not found for npc '{}'", npc_id)
        };
        movement.last_moved_turn = world.turn_count;
        move_npc(world, view, npc_id, destination.clone())?;
        Ok(())
    })?;
    Ok(())
}

/// Check and fire any Ambient triggers that apply (runs each time around the REPL loop)
///
/// # Errors
/// - on failed lookup of player's location
pub fn check_ambient_triggers(world: &mut AmbleWorld, view: &mut View) -> Result<()> {
    let current_room_id = world.player_room_id();
    for idx in local_ambient_trigger_idx(world) {
        fire_ambient_spinners(world, view, &current_room_id, idx);
    }
    Ok(())
}

/// Fires an Ambient spinner message connected to the `Trigger` at `idx`
fn fire_ambient_spinners(world: &mut AmbleWorld, view: &mut View, current_room_id: &RoomId, idx: usize) {
    let mut fired = false;
    {
        let trigger = &world.triggers[idx];
        trigger.conditions.for_each_condition(|cond| {
            if let TriggerCondition::Ambient { room_ids, spinner } = cond
                && (room_ids.is_empty() || room_ids.contains(current_room_id))
            {
                let message = world.spinners.get(spinner).and_then(Spinner::spin).unwrap_or_default();
                if !message.is_empty() {
                    view.push(ViewItem::AmbientEvent(format!("{}", message.ambient_trig_style())));
                }
                fired = true;
            }
        });
    }
    if fired && let Some(trigger) = world.triggers.get_mut(idx) {
        trigger.fired = true;
    }
}

/// Returns list of indices to ambient triggers that apply to the current room.
fn local_ambient_trigger_idx(world: &AmbleWorld) -> Vec<usize> {
    world
        .triggers
        .iter()
        .enumerate()
        .filter(|(_, trigger)| is_ambient(&trigger.conditions))
        .filter(|(_, trigger)| trigger.conditions.eval_ambient(world))
        .map(|(idx, _)| idx)
        .collect()
}

/// True if the condition contains an ambient variant.
fn is_ambient(condition: &EventCondition) -> bool {
    condition.any_trigger(|c| matches!(c, TriggerCondition::Ambient { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RoomId;
    use crate::room::{Exit, Room};
    use crate::world::Location;
    use std::collections::{HashMap, HashSet};

    fn build_test_world() -> (AmbleWorld, RoomId, RoomId) {
        let mut world = AmbleWorld::new_empty();
        world.turn_count = 1;

        let room1_id = crate::idgen::new_room_id();
        let room2_id = crate::idgen::new_room_id();

        let room1 = Room {
            id: room1_id.clone(),
            symbol: "r1".into(),
            name: "Room1".into(),
            base_description: "Room1".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        let room2 = Room {
            id: room2_id.clone(),
            symbol: "r2".into(),
            name: "Room2".into(),
            base_description: "Room2".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };

        world.rooms.insert(room1_id.clone(), room1);
        world.rooms.insert(room2_id.clone(), room2);
        world.player.location = Location::Room(room1_id.clone());
        (world, room1_id, room2_id)
    }

    #[test]
    fn turn_log_state_logs_once_and_resyncs_after_reload() {
        let (mut world, _, _) = build_test_world();
        let mut turn_log = TurnLogState::new(&world);
        assert_eq!(turn_log.last_logged_turn, 0);

        turn_log
            .log_if_advanced(&world)
            .expect("initial log_if_advanced failed");
        assert_eq!(turn_log.last_logged_turn, 1);

        turn_log.log_if_advanced(&world).expect("second log_if_advanced failed");
        assert_eq!(turn_log.last_logged_turn, 1);

        world.turn_count = 3;
        turn_log.log_if_advanced(&world).expect("third log_if_advanced failed");
        assert_eq!(turn_log.last_logged_turn, 3);

        world.turn_count = 8;
        turn_log.resync(&world);
        assert_eq!(turn_log.last_logged_turn, 7);
    }

    #[test]
    fn parse_with_exit_fallback_returns_move_to_when_match_is_unique() {
        let (mut world, room1_id, room2_id) = build_test_world();
        let room = world.rooms.get_mut(&room1_id).expect("missing room1");
        room.exits.insert("down the rope ladder".into(), Exit::new(room2_id));

        let mut view = View::new();
        let command = parse_with_exit_fallback(&world, &mut view, "rope".to_string()).expect("parse failed");
        assert_eq!(command, Command::MoveTo("down the rope ladder".to_string()));
    }

    #[test]
    fn parse_with_exit_fallback_leaves_unknown_when_match_is_ambiguous() {
        let (mut world, room1_id, room2_id) = build_test_world();
        let room = world.rooms.get_mut(&room1_id).expect("missing room1");
        room.exits
            .insert("down the rope ladder".into(), Exit::new(room2_id.clone()));
        room.exits.insert("rope bridge".into(), Exit::new(room2_id));

        let mut view = View::new();
        let command = parse_with_exit_fallback(&world, &mut view, "rope".to_string()).expect("parse failed");
        assert_eq!(command, Command::Unknown);
    }

    #[test]
    fn dispatch_command_marks_look_as_turn_advancing() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let turn_before = world.turn_count;
        let result = dispatch_command(&Command::Look, &mut world, &mut view).expect("dispatch failed");
        assert!(result.turn_advanced);
        assert!(!result.world_reloaded);
        assert_eq!(world.turn_count, turn_before + 1);
    }

    #[test]
    fn dispatch_command_marks_failed_move_as_not_turn_advancing() {
        let (mut world, _, _) = build_test_world();
        let mut view = View::new();
        let turn_before = world.turn_count;
        let result =
            dispatch_command(&Command::MoveTo("north".to_string()), &mut world, &mut view).expect("dispatch failed");
        assert!(!result.turn_advanced);
        assert!(!result.world_reloaded);
        assert_eq!(world.turn_count, turn_before);
    }
}
