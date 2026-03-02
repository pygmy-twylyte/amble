use crate::{ItemId, NpcId, RoomId};
use anyhow::{Context, Result, bail};
use log::{error, info, warn};

use crate::health::{LifeState, LivingEntity};
use crate::item::ItemHolder;
use crate::npc::{NpcState, move_npc};
use crate::spinners::{CoreSpinnerType, SpinnerType};
use crate::view::{View, ViewItem};
use crate::world::{AmbleWorld, Location, WorldObject};

/// Spawn an NPC in a Room. If the NPC is already in the world, it will be moved and a warning logged.
///
/// # Errors
/// Propagates errors when the NPC or destination room cannot be found.
pub fn spawn_npc_in_room(world: &mut AmbleWorld, view: &mut View, npc_id: &NpcId, room_id: &RoomId) -> Result<()> {
    info!(
        "└─ action: spawning NPC '{}' in Room '{room_id}'",
        crate::helpers::symbol_or_unknown(&world.npcs, npc_id),
    );
    if let Some(npc) = world.npcs.get_mut(npc_id)
        && let Some(npc_room) = npc.location.clone().room()
    {
        warn!(
            "spawn called on NPC {} who was already in-game -- MOVING from {npc_room}",
            npc.symbol(),
        );
    }
    move_npc(world, view, npc_id, Location::Room(room_id.clone()))?;
    Ok(())
}

/// Remove an NPC from the world.
///
/// # Errors
/// Returns an error if the NPC cannot be found or movement fails while
/// clearing their location.
pub fn despawn_npc(world: &mut AmbleWorld, view: &mut View, npc_id: &NpcId) -> Result<()> {
    info!("└─ action: despawning NPC '{npc_id}'");
    move_npc(world, view, npc_id, Location::Nowhere)?;
    Ok(())
}

/// Toggle an NPC's active movement flag without relocating them.
///
/// # Errors
/// Returns an error if the NPC does not exist in the world.
pub fn set_npc_active(world: &mut AmbleWorld, npc_id: &NpcId, active: bool) -> Result<()> {
    if let Some(npc) = world.npcs.get_mut(npc_id) {
        if matches!(npc.life_state(), LifeState::Dead) {
            warn!("attempted to change activity of dead NPC {}", npc.symbol());
            return Ok(());
        }
        if let Some(ref mut mvmt) = npc.movement {
            mvmt.active = active;
        }
    } else {
        bail!("error: npc_id {npc_id} not found in world.npcs")
    }
    Ok(())
}

/// Emit feedback when an NPC declines an offered item.
///
/// # Errors
/// Returns an error if either the NPC or their dialogue data cannot be found.
pub fn npc_refuse_item(
    world: &mut AmbleWorld,
    view: &mut View,
    npc_id: &NpcId,
    reason: &str,
    priority: Option<isize>,
) -> Result<()> {
    let npc = world
        .npcs
        .get(npc_id)
        .with_context(|| "failed npc lookup".to_string())?;
    if matches!(npc.life_state(), LifeState::Dead) {
        info!("bypassing refuse_item action for dead NPC '{}'", npc.name());
        return Ok(());
    }
    npc_says(world, view, npc_id, reason, priority)?;

    view.push_with_custom_priority(
        ViewItem::TriggeredEvent(format!("[[npc]]{}[[/npc]] returns it to you.", npc.name())),
        priority,
    );
    info!("└─ action: NpcRefuseItem({}, \"{reason}\"", npc.name());
    Ok(())
}

/// Makes an NPC speak a random line of dialogue based on their current mood.
///
/// # Errors
/// Returns an error if the NPC or required spinner is missing.
pub fn npc_says_random(world: &AmbleWorld, view: &mut View, npc_id: &NpcId, priority: Option<isize>) -> Result<()> {
    let npc = world
        .npcs
        .get(npc_id)
        .with_context(|| format!("action NpcSaysRandom({npc_id}): npc not found"))?;
    if matches!(npc.life_state(), LifeState::Dead) {
        info!("bypassing speech for dead NPC '{}'", npc.symbol());
        return Ok(());
    }
    let ignore_spinner = world
        .spinners
        .get(&SpinnerType::Core(CoreSpinnerType::NpcIgnore))
        .with_context(|| "failed lookup of NpcIgnore spinner".to_string())?;
    let line = npc.random_dialogue(ignore_spinner);
    view.push_with_custom_priority(
        ViewItem::NpcSpeech {
            speaker: npc.name().to_string(),
            quote: line.clone(),
        },
        priority,
    );
    info!("└─ action: NpcSays({}, \"{line}\")", npc.symbol());
    Ok(())
}

/// Makes an NPC speak a specific line of dialogue.
///
/// # Errors
/// Returns an error if the specified NPC doesn't exist in the world.
pub fn npc_says(
    world: &AmbleWorld,
    view: &mut View,
    npc_id: &NpcId,
    quote: &str,
    priority: Option<isize>,
) -> Result<()> {
    let npc = world
        .npcs
        .get(npc_id)
        .with_context(|| format!("action NpcSays({npc_id},_): npc not found"))?;
    if matches!(npc.life_state(), LifeState::Dead) {
        info!("blocked speech for dead NPC '{}'", npc.symbol());
        return Ok(());
    }
    view.push_with_custom_priority(
        ViewItem::NpcSpeech {
            speaker: npc.name.clone(),
            quote: quote.to_string(),
        },
        priority,
    );
    info!("└─ action: NpcSays({}, \"{quote}\")", npc.name());
    Ok(())
}

/// Changes an NPC's behavioral state.
///
/// # Errors
/// Returns an error if the specified NPC doesn't exist in the world.
pub fn set_npc_state(world: &mut AmbleWorld, npc_id: &NpcId, state: &NpcState) -> Result<()> {
    if let Some(npc) = world.npcs.get_mut(npc_id) {
        if npc.state == *state {
            return Ok(());
        }
        npc.state = state.clone();
        info!("└─ action: SetNpcState({}, {state:?})", npc.symbol());
        Ok(())
    } else {
        bail!("SetNpcState({npc_id},_): unknown NPC id");
    }
}

/// Transfers an item from an NPC's inventory to the player's inventory.
///
/// # Errors
/// Returns an error if the NPC or item is missing.
pub fn give_to_player(world: &mut AmbleWorld, npc_id: &NpcId, item_id: &ItemId) -> Result<()> {
    let npc = world
        .npcs
        .get_mut(npc_id)
        .with_context(|| format!("NPC {npc_id} not found"))?;
    if npc.contains_item(item_id.clone()) {
        let item = world
            .items
            .get_mut(item_id)
            .with_context(|| format!("item {item_id} in NPC inventory but missing from world.items"))?;
        item.set_location_inventory();
        npc.remove_item(item_id.clone());
        world.player.add_item(item_id.clone());
        info!("└─ action: GiveItemToPlayer({}, {})", npc.symbol(), item.symbol());
    } else {
        error!("item {item_id} not found in NPC {npc_id} inventory to give to player");
    }
    Ok(())
}
