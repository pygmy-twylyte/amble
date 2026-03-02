use crate::RoomId;
use anyhow::{Context, Result, anyhow, bail};
use log::info;

use crate::room::Exit;
use crate::world::AmbleWorld;

/// Sets a custom message that will be displayed when a player tries to use a blocked exit.
///
/// # Errors
/// Returns an error if the source room doesn't exist.
pub fn set_barred_message(world: &mut AmbleWorld, exit_from: &RoomId, exit_to: &RoomId, msg: &str) -> Result<()> {
    let room = world
        .rooms
        .get_mut(exit_from)
        .with_context(|| format!("trigger setting barred message: room_id {exit_from} not found"))?;
    let exit = room.exits.iter().find(|exit| exit.1.to == exit_to.clone());
    if let Some(exit) = exit {
        let (direction, mut exit) = (exit.0.clone(), exit.1.clone());
        exit.set_barred_msg(Some(msg.to_string()));
        room.exits.insert(direction, exit);
    }
    info!("└─ action: SetBarredMessage({exit_from} -> {exit_to}, '{msg}')");
    Ok(())
}

/// Locks an exit in a specific direction from a room, preventing player movement.
///
/// # Errors
/// Returns an error if the room or exit is missing.
pub fn lock_exit(world: &mut AmbleWorld, from_room: &RoomId, direction: &String) -> Result<()> {
    if let Some(exit) = world
        .rooms
        .get_mut(from_room)
        .and_then(|rm| rm.exits.get_mut(direction))
    {
        exit.locked = true;
        info!("└─ action: LockExit({direction}, from [{from_room}]");
        Ok(())
    } else {
        bail!("LockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// Unlocks an exit in a specific direction from a room, allowing player movement.
///
/// # Errors
/// Returns an error if the room or exit is missing.
pub fn unlock_exit(world: &mut AmbleWorld, from_room: &RoomId, direction: &String) -> Result<()> {
    if let Some(exit) = world.rooms.get_mut(from_room).and_then(|r| r.exits.get_mut(direction)) {
        exit.locked = false;
        info!("└─ action: UnlockExit({direction}, from [{from_room}])");
        Ok(())
    } else {
        bail!("UnlockExit({from_room}, {direction}): bad room id or exit direction");
    }
}

/// Makes a hidden exit visible and usable, or creates a new exit if none exists.
///
/// # Errors
/// Returns an error if the source room doesn't exist.
pub fn reveal_exit(world: &mut AmbleWorld, direction: &String, exit_from: &RoomId, exit_to: &RoomId) -> Result<()> {
    let exit = world
        .rooms
        .get_mut(exit_from)
        .ok_or_else(|| anyhow!("invalid exit_from room {exit_from}"))?
        .exits
        .entry(direction.clone())
        .or_insert_with(|| Exit::new(exit_to.clone()));
    exit.hidden = false;
    info!("└─ action: RevealExit({direction}, from '{exit_from}', to '{exit_to}')");
    Ok(())
}
