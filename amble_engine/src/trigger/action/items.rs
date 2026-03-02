use crate::{ItemId, RoomId};
use anyhow::{Context, Result, anyhow, bail};
use log::{info, warn};

use crate::helpers::symbol_or_unknown;
use crate::item::{ContainerState, ItemHolder, Movability};
use crate::world::{AmbleWorld, Location, WorldObject};

/// Update an item's container state (or clear it entirely, if it is empty).
///
/// # Errors
/// Returns an error if the item cannot be found in the world.
pub fn set_container_state(world: &mut AmbleWorld, item_id: &ItemId, state: Option<ContainerState>) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        item.container_state = state;
    } else {
        bail!("setting container state for item {item_id}: item not found");
    }
    info!("└─ action: setting container state for item {item_id}: {state:?}");
    Ok(())
}

/// Replace one item with another at the same `Location`.
///
/// # Errors
/// Returns an error if either item cannot be found or if required containers,
/// rooms, or NPCs are missing when transferring ownership.
pub fn replace_item(world: &mut AmbleWorld, old_id: &ItemId, new_id: &ItemId) -> Result<()> {
    let (location, old_sym) = if let Some(old_item) = world.items.get(old_id) {
        (old_item.location.clone(), old_item.symbol.clone())
    } else {
        bail!("replacing item {old_id}: item not found");
    };

    despawn_item(world, old_id)?;

    if let Some(new_item) = world.get_item_mut(new_id) {
        new_item.location = location.clone();
    }

    match &location {
        Location::Item(container_id) => {
            if let Some(container) = world.get_item_mut(container_id) {
                container.add_item(new_id.clone());
            }
        },
        Location::Inventory => world.player.add_item(new_id.clone()),
        Location::Nowhere => warn!("replace_item called on an unspawned item ({old_sym})"),
        Location::Npc(npc_id) => {
            if let Some(npc) = world.npcs.get_mut(npc_id) {
                npc.add_item(new_id.clone());
            }
        },
        Location::Room(room_id) => {
            if let Some(room) = world.rooms.get_mut(room_id) {
                room.add_item(new_id.clone());
            }
        },
    }
    info!(
        "└─ action: ReplaceItem({}, {}) [Location = {location:?}",
        old_sym,
        symbol_or_unknown(&world.items, new_id)
    );
    Ok(())
}

/// Drop an item in the current room and immediately spawn its replacement.
///
/// # Errors
/// Returns an error if either item cannot be found or if despawning/spawning
/// fails due to missing room context.
pub fn replace_drop_item(world: &mut AmbleWorld, old_id: &ItemId, new_id: &ItemId) -> Result<()> {
    despawn_item(world, old_id)?;
    spawn_item_in_current_room(world, new_id)?;
    Ok(())
}

/// Overwrite an item's description text at runtime.
///
/// # Errors
/// Returns an error if the item does not exist.
pub fn set_item_description(world: &mut AmbleWorld, item_id: &ItemId, text: &str) -> Result<()> {
    let item = world
        .get_item_mut(item_id)
        .with_context(|| format!("changing item '{item_id} description"))?;
    item.description = text.to_string();
    info!(
        "└─ action: SetItemDescription({}, \"{}\")",
        symbol_or_unknown(&world.items, item_id.clone()),
        &text[..std::cmp::min(text.len(), 50)]
    );
    Ok(())
}

/// Unlock an item
///
/// # Errors
/// - on invalid item uuid
pub fn unlock_item(world: &mut AmbleWorld, item_id: &ItemId) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        match item.container_state {
            Some(ContainerState::Locked) => {
                item.container_state = Some(ContainerState::Open);
                info!("└─ action: UnlockItem({}) '{}'", item.symbol(), item.name());
            },
            Some(ContainerState::TransparentLocked) => {
                item.container_state = Some(ContainerState::Open);
                info!(
                    "└─ action: UnlockItem({}) '{}' (was transparent locked)",
                    item.symbol(),
                    item.name()
                );
            },
            Some(_) => warn!(
                "action UnlockItem({}): item wasn't locked",
                symbol_or_unknown(&world.items, item_id.clone())
            ),
            None => warn!("action UnlockItem({item_id}): item '{}' isn't a container", item.name()),
        }
        Ok(())
    } else {
        bail!("UnlockItem({item_id}): item id not found")
    }
}

/// Creates an item in a specific room.
///
/// # Errors
/// Returns an error if the specified item or room is missing.
pub fn spawn_item_in_specific_room(world: &mut AmbleWorld, item_id: &ItemId, room_id: &RoomId) -> Result<()> {
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemRoom({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {item_id} missing"))?;
    info!("└─ action: SpawnItemInRoom({}, {room_id})", item.symbol());
    item.set_location_room(room_id.clone());
    world
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| anyhow!("room {room_id} missing"))?
        .add_item(item_id.clone());
    Ok(())
}

/// Creates an item in the player's current room.
///
/// # Errors
/// Returns an error if the item or current room is missing.
pub fn spawn_item_in_current_room(world: &mut AmbleWorld, item_id: &ItemId) -> Result<()> {
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemCurrentRoom({item_id}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    let room_id = world
        .player
        .location
        .room_ref()
        .with_context(|| "SpawnItemCurrentRoom: player not in a room".to_string())?;
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {item_id} missing"))?;

    info!("└─ action: SpawnItemCurrentRoom({})", item.symbol());
    item.set_location_room(room_id.clone());
    world
        .rooms
        .get_mut(room_id)
        .ok_or_else(|| anyhow!("room {room_id} missing"))?
        .add_item(item_id.clone());
    Ok(())
}

/// Creates an item directly in the player's inventory.
///
/// # Errors
/// Returns an error if the item is missing.
pub fn spawn_item_in_inventory(world: &mut AmbleWorld, item_id: &ItemId) -> Result<()> {
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemInInventory({}): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.symbol(),
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {item_id} missing"))?;
    info!("└─ action: SpawnItemInInventory({})", item.symbol());
    item.set_location_inventory();
    world.player.add_item(item_id.clone());
    Ok(())
}

/// Creates an item inside a container item.
///
/// # Errors
/// Returns an error if the item or container is missing.
pub fn spawn_item_in_container(world: &mut AmbleWorld, item_id: &ItemId, container_id: &ItemId) -> Result<()> {
    if let Some(item) = world.items.get(item_id)
        && item.location.is_not_nowhere()
    {
        warn!(
            "SpawnItemInContainer({item_id},_): '{}' already in world -- MOVING item instead (may not be desired!)",
            item.name()
        );
        despawn_item(world, item_id)?;
    }

    let container_sym = symbol_or_unknown(&world.items, container_id.clone());

    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("item {item_id} missing"))?;
    info!("└─ action: SpawnItemInContainer({}, {})", item.symbol(), container_sym);
    item.set_location_item(container_id.clone());
    world
        .items
        .get_mut(container_id)
        .ok_or_else(|| anyhow!("container {container_id} missing"))?
        .add_item(item_id.clone());
    Ok(())
}

/// Locks a container item, preventing access to its contents.
///
/// # Errors
/// Returns an error if the specified item doesn't exist in the world.
pub fn lock_item(world: &mut AmbleWorld, item_id: &ItemId) -> Result<()> {
    if let Some(item) = world.items.get_mut(item_id) {
        if item.container_state.is_some() {
            item.container_state = Some(ContainerState::Locked);
            info!("└─ action: LockItem({})", item.symbol());
        } else {
            warn!(
                "action LockItem({}): '{}' is not a container",
                item.symbol(),
                item.name()
            );
        }
        Ok(())
    } else {
        bail!("item ({item_id}) not found in world.items");
    }
}

/// Completely removes an item from the world.
///
/// # Errors
/// Returns an error if the specified item doesn't exist in the world.
pub fn despawn_item(world: &mut AmbleWorld, item_id: &ItemId) -> Result<()> {
    let item = world
        .items
        .get_mut(item_id)
        .ok_or_else(|| anyhow!("unknown item {item_id}"))?;
    let prev_loc = std::mem::replace(&mut item.location, Location::Nowhere);
    info!(
        "└─ action: DespawnItem({}) - removing from {:?}",
        item.symbol(),
        prev_loc
    );
    match prev_loc {
        Location::Room(id) => {
            if let Some(r) = world.rooms.get_mut(&id) {
                r.remove_item(item_id.clone());
            }
        },
        Location::Item(id) => {
            if let Some(c) = world.items.get_mut(&id) {
                c.remove_item(item_id.clone());
            }
        },
        Location::Npc(id) => {
            if let Some(n) = world.npcs.get_mut(&id) {
                n.remove_item(item_id.clone());
            }
        },
        Location::Inventory => {
            world.player.remove_item(item_id.clone());
        },
        Location::Nowhere => {},
    }
    Ok(())
}

/// Change the movability state of an item.
///
/// # Errors
/// - if the supplied item id is not found in the world item map
pub fn set_item_movability(world: &mut AmbleWorld, item_id: &ItemId, movability: &Movability) -> Result<()> {
    let item = world
        .items
        .get_mut(item_id)
        .with_context(|| format!("item id {item_id} not found"))?;
    item.movability.clone_from(movability);
    Ok(())
}
