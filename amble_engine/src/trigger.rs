use std::collections::{HashMap, HashSet};

use crate::{
    AmbleWorld, ItemHolder, Location, Player, WorldObject,
    item::{ContainerState, ItemAbility, ItemInteractionType},
    npc::NpcState,
    player::Flag,
    room::Exit,
    spinners::SpinnerType,
    style::GameStyle,
};
use anyhow::{Context, Result, anyhow, bail};

use gametools::{Spinner, Wedge};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A specified response to a particular set of game conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub conditions: Vec<TriggerCondition>,
    pub actions: Vec<TriggerAction>,
    pub only_once: bool,
    pub fired: bool,
}

/// Game states and player actions that can be detected by a `Trigger`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TriggerCondition {
    Ambient {
        room_ids: HashSet<Uuid>, // empty = applies everywhere
        spinner: SpinnerType,
    },
    ContainerHasItem {
        container_id: Uuid,
        item_id: Uuid,
    },
    Drop(Uuid),
    Enter(Uuid),
    GiveToNpc {
        item_id: Uuid,
        npc_id: Uuid,
    },
    HasItem(Uuid),
    HasFlag(String),
    HasVisited(Uuid),
    InRoom(Uuid),
    Insert {
        item: Uuid,
        container: Uuid,
    },
    Leave(Uuid),
    MissingFlag(String),
    MissingItem(Uuid),
    NpcHasItem {
        npc_id: Uuid,
        item_id: Uuid,
    },
    NpcInState {
        npc_id: Uuid,
        mood: NpcState,
    },
    Open(Uuid),
    Take(Uuid),
    TakeFromNpc {
        item_id: Uuid,
        npc_id: Uuid,
    },
    TalkToNpc(Uuid),
    UseItem {
        item_id: Uuid,
        ability: ItemAbility,
    },
    UseItemOnItem {
        interaction: ItemInteractionType,
        target_id: Uuid,
        tool_id: Uuid,
    },
    Unlock(Uuid),
    WithNpc(Uuid),
}

impl TriggerCondition {
    fn matches_event_in(&self, events: &[TriggerCondition]) -> bool {
        events.contains(self)
    }

    fn is_ongoing(&self, world: &AmbleWorld) -> bool {
        let player_flag_set = |flag_str: &str| world.player.flags.iter().any(|f| f.value() == *flag_str);
        match self {
            Self::ContainerHasItem { container_id, item_id } => {
                if let Some(item) = world.items.get(item_id) {
                    item.location == Location::Item(*container_id)
                } else {
                    false
                }
            },
            Self::HasFlag(flag) => player_flag_set(flag),
            Self::MissingFlag(flag) => !player_flag_set(flag),
            Self::HasVisited(room_id) => world.rooms.get(room_id).is_some_and(|r| r.visited),
            Self::InRoom(room_id) => *room_id == world.player.location.unwrap_room(),
            Self::NpcHasItem { npc_id, item_id } => {
                world.npcs.get(npc_id).is_some_and(|npc| npc.contains_item(*item_id))
            },
            Self::NpcInState { npc_id, mood } => world.npcs.get(npc_id).is_some_and(|npc| npc.state == *mood),
            Self::HasItem(item_id) => world.player.contains_item(*item_id),
            Self::MissingItem(item_id) => !world.player.contains_item(*item_id),
            Self::WithNpc(npc_id) => world
                .npcs
                .get(npc_id)
                .is_some_and(|npc| npc.location == world.player.location),
            _ => false,
        }
    }
}
/// Types of actions that can be fired by a `Trigger` based on a set of `TriggerConditions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerAction {
    AddFlag(Flag),
    AddSpinnerWedge { spinner: SpinnerType, text: String, width: usize },
    AdvanceFlag(String),
    RemoveFlag(String),
    AwardPoints(isize),
    DenyRead(String),
    DespawnItem { item_id: Uuid },
    GiveItemToPlayer { npc_id: Uuid, item_id: Uuid },
    LockExit { from_room: Uuid, direction: String },
    LockItem(Uuid),
    NpcSays { npc_id: Uuid, quote: String },
    NpcSaysRandom { npc_id: Uuid },
    PushPlayerTo(Uuid),
    ResetFlag(String),
    RestrictItem(Uuid),
    RevealExit { exit_from: Uuid, exit_to: Uuid, direction: String },
    SetNPCState { npc_id: Uuid, state: NpcState },
    ShowMessage(String),
    SpawnItemCurrentRoom(Uuid),
    SpawnItemInContainer { item_id: Uuid, container_id: Uuid },
    SpawnItemInInventory(Uuid),
    SpawnItemInRoom { item_id: Uuid, room_id: Uuid },
    SpinnerMessage { spinner: SpinnerType },
    UnlockExit { from_room: Uuid, direction: String },
    UnlockItem(Uuid),
}

/// Determines if a matching trigger condition exists in a list of triggers.
/// Useful to see if a `TriggerCondition` just sent to `check_triggers` did anything.
pub fn triggers_contain_condition<F>(list: &[&Trigger], matcher: F) -> bool
where
    F: Fn(&TriggerCondition) -> bool,
{
    list.iter().any(|t| t.conditions.iter().any(&matcher))
}

/// Determine which triggers meet conditions to fire now, fire them, and return a list of fired triggers.
///
/// # Errors
/// - on any failed uuid lookup during trigger dispatch
pub fn check_triggers<'a>(world: &'a mut AmbleWorld, events: &[TriggerCondition]) -> Result<Vec<&'a Trigger>> {
    // collect map of indices to triggers that should fire now
    let to_fire: Vec<_> = world
        .triggers
        .iter()
        .enumerate()
        .filter(|(_, t)| !t.only_once || !t.fired)
        .filter(|(_, t)| {
            t.conditions
                .iter()
                .all(|c| c.matches_event_in(events) || c.is_ongoing(world))
        })
        .map(|(i, _)| i)
        .collect();

    // mark each trigger as fired if a one-off and log it
    for i in &to_fire {
        let trigger = &mut world.triggers[*i];
        info!("Trigger fired: {}", trigger.name);
        if trigger.only_once {
            trigger.fired = true;
        }

        let actions = trigger.actions.clone();
        for action in actions {
            dispatch_action(world, &action)?;
        }
    }

    let fired_triggers: Vec<&Trigger> = to_fire.iter().map(|i| &world.triggers[*i]).collect();
    Ok(fired_triggers)
}

/// fires the matching trigger action by calling its handler function
///
/// # Errors
/// - on failed triggered actions due to bad uuids
fn dispatch_action(world: &mut AmbleWorld, action: &TriggerAction) -> Result<()> {
    match action {
        TriggerAction::AddSpinnerWedge { spinner, text, width } => {
            add_spinner_wedge(&mut world.spinners, *spinner, text, *width)?
        },
        TriggerAction::ResetFlag(flag_name) => reset_flag(&mut world.player, flag_name),
        TriggerAction::AdvanceFlag(flag_name) => advance_flag(&mut world.player, flag_name),
        TriggerAction::SpinnerMessage { spinner } => spinner_message(world, *spinner)?,
        TriggerAction::RestrictItem(item_id) => restrict_item(world, item_id)?,
        TriggerAction::NpcSaysRandom { npc_id } => npc_says_random(world, npc_id)?,
        TriggerAction::NpcSays { npc_id, quote } => npc_says(world, npc_id, quote)?,
        TriggerAction::DenyRead(reason) => deny_read(reason),
        TriggerAction::DespawnItem { item_id } => despawn_item(world, item_id)?,
        TriggerAction::GiveItemToPlayer { npc_id, item_id } => {
            give_to_player(world, npc_id, item_id)?;
        },
        TriggerAction::LockItem(item_id) => lock_item(world, item_id)?,
        TriggerAction::PushPlayerTo(room_id) => push_player(world, room_id)?,
        TriggerAction::RevealExit {
            direction,
            exit_from,
            exit_to,
        } => reveal_exit(world, direction, exit_from, exit_to)?,
        TriggerAction::SetNPCState { npc_id, state } => set_npc_state(world, npc_id, state)?,
        TriggerAction::ShowMessage(text) => show_message(text),
        TriggerAction::SpawnItemInContainer { item_id, container_id } => {
            spawn_item_in_container(world, item_id, container_id)?
        },
        TriggerAction::SpawnItemInInventory(item_id) => spawn_item_in_inventory(world, item_id)?,
        TriggerAction::SpawnItemCurrentRoom(item_id) => spawn_item_in_current_room(world, item_id)?,
        TriggerAction::SpawnItemInRoom { item_id, room_id } => {
            spawn_item_in_specific_room(world, item_id, room_id)?;
        },
        TriggerAction::UnlockItem(item_id) => unlock_item(world, item_id)?,
        TriggerAction::UnlockExit { from_room, direction } => unlock_exit(world, from_room, direction)?,
        TriggerAction::LockExit { from_room, direction } => lock_exit(world, from_room, direction)?,
        TriggerAction::AddFlag(flag) => add_flag(world, flag),
        TriggerAction::RemoveFlag(flag) => remove_flag(world, flag),
        TriggerAction::AwardPoints(amount) => award_points(world, *amount),
    }
    Ok(())
}

/// Add a wedge to one of the spinners.
///
/// # Errors
/// - if spinner type is not found
pub fn add_spinner_wedge(
    spinners: &mut HashMap<SpinnerType, Spinner<String>>,
    spin_type: SpinnerType,
    text: &str,
    width: usize,
) -> Result<()> {
    let wedge = Wedge::new_weighted(text.to_string(), width);
    let spinref = spinners
        .get_mut(&spin_type)
        .with_context(|| format!("add_spinner_wedge(_, {spin_type:?}, _, _): spinner not found"))?;
    *spinref = spinref.add_wedge(wedge);
    Ok(())
}

/// Reset a sequence flag to the first step (0).
pub fn reset_flag(player: &mut Player, flag_name: &str) {
    info!("└─ action: ResetFlag(\"{flag_name}\")");
    player.reset_flag(flag_name);
}

/// Advance a sequence flag to the next step.
pub fn advance_flag(player: &mut Player, flag_name: &str) {
    info!("└─ action: AdvanceFlag(\"{flag_name}\")");
    player.advance_flag(flag_name);
}

/// Displays a triggered, randomized message (or sometimes none) from one of the world spinners.
///
/// # Errors
/// - if requested spinner type isn't found
pub fn spinner_message(world: &mut AmbleWorld, spinner_type: SpinnerType) -> Result<()> {
    if let Some(spinner) = world.spinners.get(&spinner_type) {
        let msg = spinner.spin().unwrap_or_default();
        if !msg.is_empty() {
            println!("\n{} {}", "❉".ambient_icon_style(), msg.ambient_trig_style());
        }
        info!("└─ action: SpinnerMessage(\"{msg}\")");
        Ok(())
    } else {
        bail!("action SpinnerMessage({spinner_type:?}): no spinner found for type");
    }
}

/// Remove a flag that's been applied to the player.
pub fn remove_flag(world: &mut AmbleWorld, flag: &str) {
    let target = Flag::simple(flag);
    if world.player.flags.remove(&target) {
        info!("└─ action: RemoveFlag(\"{flag}\")");
    } else {
        warn!("└─ action: RemoveFlag(\"{flag}\") - flag was not set");
    }
}
/// Changes an item's status to restricted
///
/// # Errors
/// - on failed item lookup
pub fn restrict_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        item.restricted = true;
        info!("└─ action: RestrictItem({item_id}) \"{}\"", item.name());
        Ok(())
    } else {
        bail!("action RestrictItem({item_id}): item not found");
    }
}
/// Trigger random dialogue (based on mood) from NPC
///
/// # Errors
/// - on failed NPC or `NpcIgnore` spinner lookups
pub fn npc_says_random(world: &AmbleWorld, npc_id: &Uuid) -> Result<()> {
    let npc = world
        .npcs
        .get(npc_id)
        .with_context(|| format!("action NpcSaysRandom({npc_id}): npc not found"))?;
    let ignore_spinner = world
        .spinners
        .get(&SpinnerType::NpcIgnore)
        .with_context(|| "failed lookup of NpcIgnore spinner".to_string())?;
    let line = npc.random_dialogue(ignore_spinner);
    println!("{}: {line}", npc.name().npc_style());
    info!("└─ action: NpcSays({}, \"{line}\")", npc.name());
    Ok(())
}

/// Trigger specific dialogue from an NPC
/// # Errors
/// - on failed NPC uuid lookup
pub fn npc_says(world: &AmbleWorld, npc_id: &Uuid, quote: &str) -> Result<()> {
    let npc_name = world
        .npcs
        .get(npc_id)
        .with_context(|| format!("action NpcSays({npc_id},_): npc not found"))?
        .name();
    println!("{}: {quote}", npc_name.npc_style());
    info!("└─ action: NpcSays({npc_name}, \"{quote}\")");
    Ok(())
}

/// Award some points to the player (or penalize if amount < 0)
pub fn award_points(world: &mut AmbleWorld, amount: isize) {
    world.player.score = world.player.score.saturating_add_signed(amount);
    info!("└─ action: AwardPoints({amount})");
}

/// Adds a status flag to the player
pub fn add_flag(world: &mut AmbleWorld, flag: &Flag) {
    world.player.flags.insert(flag.clone());
    info!("└─ action: AddFlag(\"{flag}\")");
}

/// lock an exit specified by room and direction
/// # Errors
/// - on invalid room or exit direction
pub fn lock_exit(world: &mut AmbleWorld, from_room: &Uuid, direction: &String) -> Result<()> {
    if let Some(exit) = world
        .rooms
        .get_mut(from_room)
        .and_then(|rm| rm.exits.get_mut(direction))
    {
        exit.locked = true;
        info!("└─ action: LockExit({direction}, from {from_room})");
        Ok(())
    } else {
        bail!("LockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// unlock an exit specified by room and direction
/// # Errors
/// - on invalid room or exit direction
pub fn unlock_exit(world: &mut AmbleWorld, from_room: &Uuid, direction: &String) -> Result<()> {
    if let Some(exit) = world.rooms.get_mut(from_room).and_then(|r| r.exits.get_mut(direction)) {
        exit.locked = false;
        info!("└─ action: UnlockExit({direction}, from {from_room})");
        Ok(())
    } else {
        bail!("UnlockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// Unlock an item
/// # Errors
/// - on invalid item uuid
pub fn unlock_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        match item.container_state {
            Some(ContainerState::Locked) => {
                item.container_state = Some(ContainerState::Open);
                info!("└─ action: UnlockItem({item_id}) '{}'", item.name());
            },
            Some(_) => warn!("action UnlockItem({item_id}): item wasn't locked"),
            None => warn!("action UnlockItem({item_id}): item '{}' isn't a container", item.name()),
        }
        Ok(())
    } else {
        bail!("UnlockItem({item_id}): item id not found")
    }
}

/// Spawn an `Item` in a specific `Room`
/// # Errors
/// - on failed item or room lookup
pub fn spawn_item_in_specific_room(world: &mut AmbleWorld, item_id: &Uuid, room_id: &Uuid) -> Result<()> {
    // warn and remove item from world if it's already somewhere to avoid dups
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemRoom({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    // spawn in specified room as intended
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;
    info!("└─ action: SpawnItemInRoom({item_id}, {room_id})");
    item.set_location_room(*room_id);
    world
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| anyhow!("room {} missing", room_id))?
        .add_item(*item_id);
    Ok(())
}

/// Spawn an `Item` in the `Room` the player currently occupies
/// # Errors
/// - on failed item or room lookup
pub fn spawn_item_in_current_room(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    // warn and remove item from world if it's already somewhere to avoid dups
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemCurrentRoom({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    // then spawn at current location as intended
    let room_id = world
        .player
        .location
        .room_ref()
        .with_context(|| "SpawnItemCurrentRoom: player not in a room".to_string())?;
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;

    info!("└─ action: SpawnItemCurrentRoom({item_id})");
    item.set_location_room(*room_id);
    world
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| anyhow!("room {} missing", room_id))?
        .add_item(*item_id);
    Ok(())
}

/// Spawn an `Item` in player's inventory
/// # Errors
/// - on failed item lookup
pub fn spawn_item_in_inventory(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    // warn and remove item from world if it's already somewhere to avoid dups
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemInInventory({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }
    // add item to player inventory as intended
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;
    info!("└─ action: SpawnItemInInventory({item_id})");
    item.set_location_inventory();
    world.player.add_item(*item_id);
    Ok(())
}

/// Spawn an `Item` within a container `Item`
/// # Errors
/// - on failed item or container lookup
pub fn spawn_item_in_container(world: &mut AmbleWorld, item_id: &Uuid, container_id: &Uuid) -> Result<()> {
    // if item is already in-world, warn and remove it to avoid duplications / inconsistent state
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemInContainer({item_id},_): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    // then spawn again in the desired location
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {} missing", item_id))?;
    info!("└─ action: SpawnItemInContainer({item_id}, {container_id})");
    item.set_location_item(*container_id);
    world
        .items
        .get_mut(container_id)
        .ok_or_else(|| anyhow!("container {} missing", container_id))?
        .add_item(*item_id);
    Ok(())
}

/// Show a message to the player.
pub fn show_message(text: &String) {
    println!("{} {}", "✦".trig_icon_style(), text.triggered_style());
    info!(
        "└─ action: ShowMessage(\"{}...\")",
        &text[..std::cmp::min(text.len(), 50)]
    );
}

/// Set the state of a specified `Npc`
/// # Errors
/// - on failed npc lookup
pub fn set_npc_state(world: &mut AmbleWorld, npc_id: &Uuid, state: &NpcState) -> Result<()> {
    if let Some(npc) = world.npcs.get_mut(npc_id) {
        npc.state = state.clone();
        info!("└─ action: SetNpcState({npc_id}, {state:?})");
        Ok(())
    } else {
        bail!("SetNpcState({npc_id},_): unknown NPC id");
    }
}

/// Reveal or create a new exit from a `Room`
/// # Errors
/// - on invalid `exit_from` room uuid
pub fn reveal_exit(world: &mut AmbleWorld, direction: &String, exit_from: &Uuid, exit_to: &Uuid) -> Result<()> {
    let exit = world
        .rooms
        .get_mut(exit_from)
        .ok_or_else(|| anyhow!("invalid exit_from room {}", exit_from))?
        .exits
        .entry(direction.clone())
        .or_insert_with(|| Exit::new(*exit_to));
    exit.hidden = false;
    info!("└─ action: RevealExit({direction}, from {exit_from}, to {exit_to})");
    Ok(())
}

/// Move player to another `Room`
/// # Errors
/// - on failed room lookup
pub fn push_player(world: &mut AmbleWorld, room_id: &Uuid) -> Result<()> {
    if world.rooms.contains_key(room_id) {
        world.player.location = Location::Room(*room_id);
        info!("└─ action: PushPlayerTo({room_id})");
        Ok(())
    } else {
        bail!("tried to push player to unknown room ({room_id})");
    }
}

/// Lock an `Item`
/// # Errors
/// - if attempt to lock an item that isn't a container
/// - if specified container is not found
pub fn lock_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        if item.container_state.is_some() {
            item.container_state = Some(ContainerState::Locked);
            info!("└─ action: LockItem({item_id})");
        } else {
            warn!("action LockItem({item_id}): '{}' is not a container", item.name());
        }
        Ok(())
    } else {
        bail!("item ({item_id}) not found in world.items");
    }
}

/// Move an `Item` from an `Npc` to the player's inventory.
/// # Errors
/// - on failed item or npc lookup
pub fn give_to_player(world: &mut AmbleWorld, npc_id: &Uuid, item_id: &Uuid) -> Result<()> {
    let npc = world
        .npcs
        .get_mut(npc_id)
        .with_context(|| format!("NPC {npc_id} not found"))?;
    if npc.contains_item(*item_id) {
        let item = world
            .items
            .get_mut(item_id)
            .with_context(|| format!("item {item_id} in NPC inventory but missing from world.items"))?;
        item.set_location_inventory();
        npc.remove_item(*item_id);
        world.player.add_item(*item_id);
        info!("└─ action: GiveItemToPlayer({npc_id}, {item_id})");
        Ok(())
    } else {
        bail!("item {} not found in NPC {} inventory", item_id, npc_id);
    }
}

/// Remove an `Item` from the world.
/// Sets item location to "Nowhere" and removes it from wherever it was
/// # Errors
/// - on failed item lookup
pub fn despawn_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("unknown item {}", item_id))?;
    let prev_loc = std::mem::replace(&mut item.location, Location::Nowhere);
    match prev_loc {
        Location::Room(id) => {
            if let Some(r) = world.rooms.get_mut(&id) {
                r.remove_item(*item_id);
            }
        },
        Location::Item(id) => {
            if let Some(c) = world.items.get_mut(&id) {
                c.remove_item(*item_id);
            }
        },
        Location::Npc(id) => {
            if let Some(n) = world.npcs.get_mut(&id) {
                n.remove_item(*item_id);
            }
        },
        Location::Inventory => {
            world.player.remove_item(*item_id);
        },
        Location::Nowhere => {},
    }
    info!("└─ action: DespawnItem({item_id})");
    Ok(())
}

/// Notify player of the reason a Read(item) command was denied
pub fn deny_read(reason: &String) {
    println!("{}", reason.denied_style());
    info!("└─ action: DenyRead(\"{reason}\")");
}
