use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, bail};
use gametools::{Spinner, Wedge};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::item::{ContainerState, ItemHolder};
use crate::npc::NpcState;
use crate::player::{Flag, Player};
use crate::room::Exit;
use crate::spinners::SpinnerType;
use crate::style::GameStyle;
use crate::world::{AmbleWorld, Location, WorldObject};

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
    NpcRefuseItem { npc_id: Uuid, reason: String },
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

///
/// # Errors
/// - on failed triggered actions due to bad uuids
/// fires the matching trigger action by calling its handler function
pub fn dispatch_action(world: &mut AmbleWorld, action: &TriggerAction) -> Result<()> {
    match action {
        TriggerAction::AddSpinnerWedge { spinner, text, width } => {
            add_spinner_wedge(&mut world.spinners, *spinner, text, *width)?;
        },
        TriggerAction::ResetFlag(flag_name) => reset_flag(&mut world.player, flag_name),
        TriggerAction::AdvanceFlag(flag_name) => advance_flag(&mut world.player, flag_name),
        TriggerAction::SpinnerMessage { spinner } => spinner_message(world, *spinner)?,
        TriggerAction::RestrictItem(item_id) => restrict_item(world, item_id)?,
        TriggerAction::NpcRefuseItem { npc_id, reason } => npc_refuse_item(world, *npc_id, reason)?,
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
            spawn_item_in_container(world, item_id, container_id)?;
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

/// Make NPC refuse a specific item for a specific reason.
/// # Errors
///
pub fn npc_refuse_item(world: &mut AmbleWorld, npc_id: Uuid, reason: &str) -> Result<()> {
    npc_says(world, &npc_id, reason)?;
    let npc_name = world
        .npcs
        .get(&npc_id)
        .with_context(|| "looking up NPC {npc_id} during item refusal")?
        .name();
    println!("{} returns it to you.", npc_name.npc_style());
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
    println!("{}:\n{line}", npc.name().npc_style());
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
    println!("{}:\n{quote}", npc_name.npc_style());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item},
        npc::{Npc, NpcState},
        player::Flag,
        room::Room,
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn build_test_world() -> (AmbleWorld, Uuid, Uuid) {
        let mut world = AmbleWorld::new_empty();
        let room1_id = Uuid::new_v4();
        let room2_id = Uuid::new_v4();

        let room1 = Room {
            id: room1_id,
            symbol: "r1".into(),
            name: "Room1".into(),
            base_description: "Room1".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        let room2 = Room {
            id: room2_id,
            symbol: "r2".into(),
            name: "Room2".into(),
            base_description: "Room2".into(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room1_id, room1);
        world.rooms.insert(room2_id, room2);
        world.player.location = Location::Room(room1_id);
        (world, room1_id, room2_id)
    }

    fn make_item(id: Uuid, location: Location, container_state: Option<ContainerState>) -> Item {
        Item {
            id,
            symbol: "it".into(),
            name: "Item".into(),
            description: "".into(),
            location,
            portable: true,
            container_state,
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
        }
    }

    fn make_npc(id: Uuid, location: Location, state: NpcState) -> Npc {
        Npc {
            id,
            symbol: "n".into(),
            name: "Npc".into(),
            description: "".into(),
            location,
            inventory: HashSet::new(),
            dialogue: HashMap::new(),
            state,
        }
    }

    #[test]
    fn push_player_moves_player_to_room() {
        let (mut world, _start, dest) = build_test_world();
        assert!(push_player(&mut world, &dest).is_ok());
        assert_eq!(world.player.location, Location::Room(dest));
    }

    #[test]
    fn push_player_errors_with_invalid_room() {
        let (mut world, _, _) = build_test_world();
        let bad_room = Uuid::new_v4();
        assert!(push_player(&mut world, &bad_room).is_err());
    }

    #[test]
    fn add_and_remove_flag_updates_player_flags() {
        let (mut world, _, _) = build_test_world();
        let flag = Flag::simple("test");
        add_flag(&mut world, &flag);
        assert!(world.player.flags.contains(&flag));
        remove_flag(&mut world, "test");
        assert!(!world.player.flags.contains(&flag));
    }

    #[test]
    fn reset_and_advance_flag_modifies_sequence() {
        let (mut world, _, _) = build_test_world();
        let flag = Flag::sequence("quest", Some(2));
        world.player.flags.insert(flag);
        advance_flag(&mut world.player, "quest");
        assert!(
            world
                .player
                .flags
                .iter()
                .any(|f| matches!(f, Flag::Sequence { name, step, .. } if name == "quest" && *step == 1))
        );
        reset_flag(&mut world.player, "quest");
        assert!(
            world
                .player
                .flags
                .iter()
                .any(|f| matches!(f, Flag::Sequence { name, step, .. } if name == "quest" && *step == 0))
        );
    }

    #[test]
    fn award_points_modifies_player_score() {
        let (mut world, _, _) = build_test_world();
        award_points(&mut world, 5);
        assert_eq!(world.player.score, 6);
        award_points(&mut world, -3);
        assert_eq!(world.player.score, 3);
    }

    #[test]
    fn restrict_item_sets_restricted_flag() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Room(room_id), None);
        world.items.insert(item_id, item);
        restrict_item(&mut world, &item_id).unwrap();
        assert!(world.items.get(&item_id).unwrap().restricted);
    }

    #[test]
    fn lock_and_unlock_item_changes_state() {
        let (mut world, room_id, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Room(room_id), Some(ContainerState::Open));
        world.items.insert(item_id, item);
        lock_item(&mut world, &item_id).unwrap();
        assert_eq!(
            world.items.get(&item_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
        unlock_item(&mut world, &item_id).unwrap();
        assert_eq!(
            world.items.get(&item_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn lock_and_unlock_exit_changes_state() {
        let (mut world, room1_id, room2_id) = build_test_world();
        world
            .rooms
            .get_mut(&room1_id)
            .unwrap()
            .exits
            .insert("north".into(), Exit::new(room2_id));
        lock_exit(&mut world, &room1_id, &"north".into()).unwrap();
        assert!(world.rooms[&room1_id].exits["north"].locked);
        unlock_exit(&mut world, &room1_id, &"north".into()).unwrap();
        assert!(!world.rooms[&room1_id].exits["north"].locked);
    }

    #[test]
    fn spawn_item_in_specific_room_places_item() {
        let (mut world, _room1, room2) = build_test_world();
        let item_id = Uuid::new_v4();
        let item = make_item(item_id, Location::Nowhere, None);
        world.items.insert(item_id, item);
        spawn_item_in_specific_room(&mut world, &item_id, &room2).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Room(room2));
        assert!(world.rooms[&room2].contents.contains(&item_id));
    }

    #[test]
    fn spawn_item_in_current_room_places_item() {
        let (mut world, room1, _room2) = build_test_world();
        let item_id = Uuid::new_v4();
        world.items.insert(item_id, make_item(item_id, Location::Nowhere, None));
        spawn_item_in_current_room(&mut world, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Room(room1));
        assert!(world.rooms[&room1].contents.contains(&item_id));
    }

    #[test]
    fn spawn_item_in_inventory_adds_to_player() {
        let (mut world, _, _) = build_test_world();
        let item_id = Uuid::new_v4();
        let mut item = make_item(item_id, Location::Nowhere, None);
        item.restricted = true;
        world.items.insert(item_id, item);
        spawn_item_in_inventory(&mut world, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Inventory);
        assert!(world.player.inventory.contains(&item_id));
        assert!(!world.items[&item_id].restricted);
    }

    #[test]
    fn spawn_item_in_container_places_item_inside() {
        let (mut world, room1, _) = build_test_world();
        let container_id = Uuid::new_v4();
        let container = make_item(container_id, Location::Room(room1), Some(ContainerState::Open));
        world.items.insert(container_id, container);
        world.rooms.get_mut(&room1).unwrap().contents.insert(container_id);
        let item_id = Uuid::new_v4();
        world.items.insert(item_id, make_item(item_id, Location::Nowhere, None));
        spawn_item_in_container(&mut world, &item_id, &container_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Item(container_id));
        assert!(world.items[&container_id].contents.contains(&item_id));
    }

    #[test]
    fn despawn_item_removes_item_from_world() {
        let (mut world, room1, _) = build_test_world();
        let item_id = Uuid::new_v4();
        world
            .items
            .insert(item_id, make_item(item_id, Location::Room(room1), None));
        world.rooms.get_mut(&room1).unwrap().contents.insert(item_id);
        despawn_item(&mut world, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Nowhere);
        assert!(!world.rooms[&room1].contents.contains(&item_id));
    }

    #[test]
    fn give_to_player_transfers_item_from_npc() {
        let (mut world, room1, _) = build_test_world();
        let npc_id = Uuid::new_v4();
        let npc = make_npc(npc_id, Location::Room(room1), NpcState::Normal);
        world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id);
        world.npcs.insert(npc_id, npc);
        let item_id = Uuid::new_v4();
        world
            .items
            .insert(item_id, make_item(item_id, Location::Npc(npc_id), None));
        world.npcs.get_mut(&npc_id).unwrap().inventory.insert(item_id);
        give_to_player(&mut world, &npc_id, &item_id).unwrap();
        assert_eq!(world.items[&item_id].location, Location::Inventory);
        assert!(world.player.inventory.contains(&item_id));
        assert!(!world.npcs[&npc_id].inventory.contains(&item_id));
    }
}
