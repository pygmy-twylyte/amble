use std::collections::HashSet;

use crate::{
    AmbleWorld, ItemHolder, Location, WorldObject,
    item::{ContainerState, ItemAbility, ItemInteractionType},
    npc::NpcMood,
    room::Exit,
    spinners::SpinnerType,
    style::GameStyle,
};
use anyhow::{Context, Result, anyhow, bail};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub name: String,
    pub conditions: Vec<TriggerCondition>,
    pub actions: Vec<TriggerAction>,
    pub only_once: bool,
    pub fired: bool,
}

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
    HasAchievement(String),
    HasVisited(Uuid),
    InRoom(Uuid),
    Insert {
        item: Uuid,
        container: Uuid,
    },
    Leave(Uuid),
    MissingAchievement(String),
    MissingItem(Uuid),
    NpcHasItem {
        npc_id: Uuid,
        item_id: Uuid,
    },
    NpcInMood {
        npc_id: Uuid,
        mood: NpcMood,
    },
    Open(Uuid),
    Take(Uuid),
    TakeFromNpc {
        item_id: Uuid,
        npc_id: Uuid,
    },
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
        match self {
            Self::ContainerHasItem {
                container_id,
                item_id,
            } => {
                if let Some(item) = world.items.get(item_id) {
                    item.location == Location::Item(*container_id)
                } else {
                    false
                }
            }
            Self::HasAchievement(ach) => world.player.achievements.contains(ach),
            Self::MissingAchievement(ach) => !world.player.achievements.contains(ach),
            Self::HasVisited(room_id) => world.rooms.get(room_id).is_some_and(|r| r.visited),
            Self::InRoom(room_id) => *room_id == world.player.location.clone().unwrap_room(),
            Self::NpcHasItem { npc_id, item_id } => world
                .npcs
                .get(npc_id)
                .is_some_and(|npc| npc.contains_item(*item_id)),
            Self::NpcInMood { npc_id, mood } => world
                .npcs
                .get(npc_id)
                .is_some_and(|npc| dbg!(npc.mood) == dbg!(*mood)),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerAction {
    AddAchievement(String),
    AwardPoints(isize),
    DenyRead(String),
    DespawnItem {
        item_id: Uuid,
    },
    GiveItemToPlayer {
        npc_id: Uuid,
        item_id: Uuid,
    },
    LockExit {
        from_room: Uuid,
        direction: String,
    },
    LockItem(Uuid),
    NpcSays {
        npc_id: Uuid,
        quote: String,
    },
    NpcSaysRandom {
        npc_id: Uuid,
    },
    PushPlayerTo(Uuid),
    RestrictItem(Uuid),
    RevealExit {
        exit_from: Uuid,
        exit_to: Uuid,
        direction: String,
    },
    SetNPCMood {
        npc_id: Uuid,
        mood: NpcMood,
    },
    ShowMessage(String),
    SpawnItemCurrentRoom(Uuid),
    SpawnItemInContainer {
        item_id: Uuid,
        container_id: Uuid,
    },
    SpawnItemInInventory(Uuid),
    SpawnItemInRoom {
        item_id: Uuid,
        room_id: Uuid,
    },
    UnlockExit {
        from_room: Uuid,
        direction: String,
    },
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
pub fn check_triggers<'a>(
    world: &'a mut AmbleWorld,
    events: &[TriggerCondition],
) -> Result<Vec<&'a Trigger>> {
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

        // clone needed here to satisfy borrow checker
        let actions = trigger.actions.clone();
        for action in actions {
            dispatch_action(world, &action)?;
        }
    }

    let fired_triggers: Vec<&Trigger> = to_fire.iter().map(|i| &world.triggers[*i]).collect();
    Ok(fired_triggers)
}

/// fires the matching trigger action by calling its handler function
fn dispatch_action(world: &mut AmbleWorld, action: &TriggerAction) -> Result<()> {
    match action {
        TriggerAction::RestrictItem(item_id) => restrict_item(world, item_id)?,
        TriggerAction::NpcSaysRandom { npc_id } => npc_says_random(world, npc_id)?,
        TriggerAction::NpcSays { npc_id, quote } => npc_says(world, npc_id, quote)?,
        TriggerAction::DenyRead(reason) => deny_read(reason),
        TriggerAction::DespawnItem { item_id } => despawn_item(world, item_id)?,
        TriggerAction::GiveItemToPlayer { npc_id, item_id } => {
            give_to_player(world, npc_id, item_id)?;
        }
        TriggerAction::LockItem(item_id) => lock_item(world, item_id)?,
        TriggerAction::PushPlayerTo(room_id) => push_player(world, room_id)?,
        TriggerAction::RevealExit {
            direction,
            exit_from,
            exit_to,
        } => reveal_exit(world, direction, exit_from, exit_to)?,
        TriggerAction::SetNPCMood { npc_id, mood } => set_npc_mood(world, npc_id, *mood)?,
        TriggerAction::ShowMessage(text) => show_message(text),
        TriggerAction::SpawnItemInContainer {
            item_id,
            container_id,
        } => spawn_item_in_container(world, item_id, container_id)?,
        TriggerAction::SpawnItemInInventory(item_id) => spawn_item_in_inventory(world, item_id)?,
        TriggerAction::SpawnItemCurrentRoom(item_id) => spawn_item_in_current_room(world, item_id)?,
        TriggerAction::SpawnItemInRoom { item_id, room_id } => {
            spawn_item_in_specific_room(world, item_id, room_id)?;
        }
        TriggerAction::UnlockItem(item_id) => unlock_item(world, item_id)?,
        TriggerAction::UnlockExit {
            from_room,
            direction,
        } => unlock_exit(world, from_room, direction)?,
        TriggerAction::LockExit {
            from_room,
            direction,
        } => lock_exit(world, from_room, direction)?,
        TriggerAction::AddAchievement(task) => add_achievement(world, task),
        TriggerAction::AwardPoints(amount) => award_points(world, *amount),
    }
    Ok(())
}
/// Changes an item's status to restricted
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

/// award some points to the player (or penalize if amount < 0)
pub fn award_points(world: &mut AmbleWorld, amount: isize) {
    world.player.score = world.player.score.saturating_add_signed(amount);
    info!("└─ action: AwardPoints({amount})");
}

/// grants player with an achievement or task completion
fn add_achievement(world: &mut AmbleWorld, task: &String) {
    world.player.achievements.insert(task.to_string());
    info!("└─ action: AddAchievement(\"{task}\")");
}

/// lock an exit specified by room and direction
fn lock_exit(world: &mut AmbleWorld, from_room: &Uuid, direction: &String) -> Result<()> {
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
fn unlock_exit(world: &mut AmbleWorld, from_room: &Uuid, direction: &String) -> Result<()> {
    if let Some(exit) = world
        .rooms
        .get_mut(from_room)
        .and_then(|r| r.exits.get_mut(direction))
    {
        exit.locked = false;
        info!("└─ action: UnlockExit({direction}, from {from_room})");
        Ok(())
    } else {
        bail!("UnlockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// unlock an item
fn unlock_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        match item.container_state {
            Some(ContainerState::Locked) => {
                item.container_state = Some(ContainerState::Open);
                info!("└─ action: UnlockItem({item_id}) '{}'", item.name());
            }
            Some(_) => warn!("action UnlockItem({item_id}): item wasn't locked"),
            None => warn!(
                "action UnlockItem({item_id}): item '{}' isn't a container",
                item.name()
            ),
        }
        Ok(())
    } else {
        bail!("UnlockItem({item_id}): item id not found")
    }
}

fn spawn_item_in_specific_room(
    world: &mut AmbleWorld,
    item_id: &Uuid,
    room_id: &Uuid,
) -> Result<()> {
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

fn spawn_item_in_current_room(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
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

fn spawn_item_in_inventory(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
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

fn spawn_item_in_container(
    world: &mut AmbleWorld,
    item_id: &Uuid,
    container_id: &Uuid,
) -> Result<()> {
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

fn show_message(text: &String) {
    println!("{} {}", "✦".trig_icon_style(), text.triggered_style());
    info!(
        "└─ action: ShowMessage(\"{}...\")",
        &text[..std::cmp::min(text.len(), 50)]
    );
}

fn set_npc_mood(world: &mut AmbleWorld, npc_id: &Uuid, mood: NpcMood) -> Result<()> {
    if let Some(npc) = world.npcs.get_mut(npc_id) {
        npc.mood = mood;
        info!("└─ action: SetNPCMood({npc_id}, {mood:?})");
        Ok(())
    } else {
        bail!("SetNpcMood({npc_id},_): unknown NPC id");
    }
}

fn reveal_exit(
    world: &mut AmbleWorld,
    direction: &String,
    exit_from: &Uuid,
    exit_to: &Uuid,
) -> Result<()> {
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

fn push_player(world: &mut AmbleWorld, room_id: &Uuid) -> Result<()> {
    if world.rooms.contains_key(room_id) {
        world.player.location = Location::Room(*room_id);
        info!("└─ action: PushPlayerTo({room_id})");
        Ok(())
    } else {
        bail!("tried to push player to unknown room ({room_id})");
    }
}

fn lock_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        if item.container_state.is_some() {
            item.container_state = Some(ContainerState::Locked);
            info!("└─ action: LockItem({item_id})");
        } else {
            warn!(
                "action LockItem({item_id}): '{}' is not a container",
                item.name()
            );
        }
        Ok(())
    } else {
        bail!("item ({item_id}) not found in world.items");
    }
}

fn give_to_player(world: &mut AmbleWorld, npc_id: &Uuid, item_id: &Uuid) -> Result<()> {
    let npc = world
        .npcs
        .get_mut(npc_id)
        .with_context(|| format!("NPC {npc_id} not found"))?;
    if npc.contains_item(*item_id) {
        let item = world.items.get_mut(item_id).with_context(|| {
            format!("item {item_id} in NPC inventory but missing from world.items")
        })?;
        item.set_location_inventory();
        npc.remove_item(*item_id);
        world.player.add_item(*item_id);
        info!("└─ action: GiveItemToPlayer({npc_id}, {item_id})");
        Ok(())
    } else {
        bail!("item {} not found in NPC {} inventory", item_id, npc_id);
    }
}

fn despawn_item(world: &mut AmbleWorld, item_id: &Uuid) -> Result<()> {
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
        }
        Location::Item(id) => {
            if let Some(c) = world.items.get_mut(&id) {
                c.remove_item(*item_id);
            }
        }
        Location::Npc(id) => {
            if let Some(n) = world.npcs.get_mut(&id) {
                n.remove_item(*item_id);
            }
        }
        Location::Inventory => {
            world.player.remove_item(*item_id);
        }
        Location::Nowhere => {}
    }
    info!("└─ action: DespawnItem({item_id})");
    Ok(())
}

/// notify player of the reason a Read(item) command was denied
fn deny_read(reason: &String) {
    println!("You can't read that. {reason}");
    info!("└─ action: DenyRead(\"{reason}\")");
}
