use crate::{Item, ItemId, NpcId, RoomId};
use anyhow::{Context, Result, bail};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::helpers::symbol_or_unknown;
use crate::item::{ContainerState, ItemAbility, ItemVisibility, Movability};
use crate::npc::{MovementTiming, MovementType, Npc, NpcMovement, NpcState};
use crate::player::Flag;
use crate::room::Exit;
use crate::scheduler::EventCondition;
use crate::world::{AmbleWorld, WorldObject};

/// Patch that can be applied to modify multiple properties of an `Item` at once
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ItemPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub text: Option<String>,
    pub movability: Option<Movability>,
    pub container_state: Option<ContainerState>,
    #[serde(default)]
    pub remove_container_state: bool,
    pub visibility: Option<ItemVisibility>,
    pub visible_when: Option<EventCondition>,
    pub aliases: Option<Vec<String>>,
    #[serde(default)]
    pub add_abilities: Vec<ItemAbility>,
    #[serde(default)]
    pub remove_abilities: Vec<ItemAbility>,
}

/// Patch that can be applied to modify multiple properties of a `Room` at once.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RoomPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub remove_exits: Vec<RoomId>,
    #[serde(default)]
    pub add_exits: Vec<RoomExitPatch>,
}

/// Exit data used when adding an exit via `RoomPatch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RoomExitPatch {
    pub direction: String,
    pub to: RoomId,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub required_flags: HashSet<Flag>,
    #[serde(default)]
    pub required_items: HashSet<ItemId>,
    #[serde(default)]
    pub barred_message: Option<String>,
}

/// Represents a line of dialogue to be appended to a specific NPC state when applying an `NpcPatch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NpcDialoguePatch {
    pub state: NpcState,
    pub line: String,
}

/// Movement updates that may accompany an `NpcPatch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NpcMovementPatch {
    #[serde(default)]
    pub route: Option<Vec<RoomId>>,
    #[serde(default)]
    pub random_rooms: Option<HashSet<RoomId>>,
    pub timing: Option<MovementTiming>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

impl NpcMovementPatch {
    /// Return `true` if any movement-related field should be updated.
    pub fn has_updates(&self) -> bool {
        self.route.is_some()
            || self.random_rooms.is_some()
            || self.timing.is_some()
            || self.active.is_some()
            || self.loop_route.is_some()
    }

    fn wants_new_instance(&self) -> bool {
        self.route.is_some() || self.random_rooms.is_some()
    }
}

/// Patch that can be applied to modify multiple properties of an `Npc` at once.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NpcPatch {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub state: Option<NpcState>,
    #[serde(default)]
    pub add_lines: Vec<NpcDialoguePatch>,
    pub movement: Option<NpcMovementPatch>,
}

/// Modifies multiple properties of an `Item` at once by applying an `ItemPatch`.
///
/// # Errors
/// Returns an error if the target item cannot be found or if referenced
/// container/item identifiers inside the patch are missing.
pub fn modify_item(world: &mut AmbleWorld, item_id: ItemId, patch: &ItemPatch) -> Result<()> {
    info!(
        "└─ action: modifying item {} using patch: {:?}",
        symbol_or_unknown(&world.items, &item_id),
        patch
    );
    let patched = apply_item_patch(world, &item_id, patch)?;
    world.items.insert(item_id, patched);
    Ok(())
}

/// Modifies multiple properties of a `Room` at once by applying a `RoomPatch`.
///
/// # Errors
/// Returns an error if the target room cannot be found or if referenced exits
/// or items in the patch cannot be resolved.
pub fn modify_room(world: &mut AmbleWorld, room_id: &RoomId, patch: &RoomPatch) -> Result<()> {
    info!("└─ action: modifying room {room_id} using patch: {patch:?}");
    apply_room_patch(world, room_id, patch)?;
    Ok(())
}

/// Modifies multiple properties of an `Npc` at once by applying an `NpcPatch`.
///
/// # Errors
/// Returns an error if the NPC cannot be found or if movement patches contain
/// inconsistent data (such as empty routes or unknown rooms).
pub fn modify_npc(world: &mut AmbleWorld, npc_id: &NpcId, patch: &NpcPatch) -> Result<()> {
    info!(
        "└─ action: modifying npc {} using patch: {:?}",
        symbol_or_unknown(&world.npcs, npc_id),
        patch
    );
    apply_npc_patch(world, npc_id, patch)?;
    Ok(())
}

/// Applies a `RoomPatch` to the targeted `Room`, mutating it in place.
fn apply_room_patch(world: &mut AmbleWorld, room_id: &RoomId, patch: &RoomPatch) -> Result<()> {
    let mut removal_plan: Vec<(String, RoomId)> = Vec::new();
    {
        let room_ref = world
            .rooms
            .get(room_id)
            .with_context(|| format!("patching a room: RoomId ({room_id}) not found in world room map"))?;
        for target_room_id in &patch.remove_exits {
            if let Some(direction) = room_ref
                .exits
                .iter()
                .find_map(|(dir, exit)| (exit.to.as_str() == target_room_id.as_str()).then_some(dir.clone()))
            {
                removal_plan.push((direction, target_room_id.clone()));
            } else {
                warn!(
                    "modifyRoom patch attempted to remove exit to '{target_room_id}' but room '{room_id}' has no such exit"
                );
            }
        }
    }

    let room = world.rooms.get_mut(room_id).expect("room existence validated above");

    if let Some(ref new_name) = patch.name {
        room.name.clone_from(new_name);
    }

    if let Some(ref new_desc) = patch.desc {
        room.base_description.clone_from(new_desc);
    }

    for (direction, _) in &removal_plan {
        room.exits.remove(direction);
    }

    for addition in &patch.add_exits {
        let mut exit = Exit::new(addition.to.clone());
        exit.hidden = addition.hidden;
        exit.locked = addition.locked;
        exit.barred_message.clone_from(&addition.barred_message);
        exit.required_flags.clone_from(&addition.required_flags);
        exit.required_items.clone_from(&addition.required_items);
        room.exits.insert(addition.direction.clone(), exit);
    }

    Ok(())
}

/// Applies an `NpcPatch` to the targeted `Npc`, mutating it in place.
fn apply_npc_patch(world: &mut AmbleWorld, npc_id: &NpcId, patch: &NpcPatch) -> Result<()> {
    let npc = world
        .npcs
        .get_mut(npc_id)
        .with_context(|| format!("patching an npc: NpcId ({npc_id}) not found in world npc map"))?;

    if let Some(ref new_name) = patch.name {
        npc.name.clone_from(new_name);
    }
    if let Some(ref new_desc) = patch.desc {
        npc.description.clone_from(new_desc);
    }
    if let Some(ref new_state) = patch.state {
        npc.state = new_state.clone();
    }

    if !patch.add_lines.is_empty() {
        for addition in &patch.add_lines {
            npc.dialogue
                .entry(addition.state.clone())
                .or_default()
                .push(addition.line.clone());
        }
    }

    if let Some(movement_patch) = &patch.movement
        && movement_patch.has_updates()
    {
        let current_turn = world.turn_count;
        apply_npc_movement_patch(current_turn, npc, movement_patch)?;
    }

    Ok(())
}

fn apply_npc_movement_patch(current_turn: usize, npc: &mut Npc, patch: &NpcMovementPatch) -> Result<()> {
    if patch.route.is_some() && patch.random_rooms.is_some() {
        bail!(
            "modifyNpc patch for '{}' cannot set both a route and random movement set",
            npc.symbol()
        );
    }

    if let Some(route) = &patch.route
        && route.is_empty()
    {
        bail!(
            "modifyNpc patch for '{}' requires at least one room in a movement route",
            npc.symbol()
        );
    }
    if let Some(random) = &patch.random_rooms
        && random.is_empty()
    {
        bail!(
            "modifyNpc patch for '{}' requires at least one room in a random movement set",
            npc.symbol()
        );
    }

    let npc_symbol = npc.symbol().to_string();

    if npc.movement.is_none() && patch.wants_new_instance() {
        npc.movement = Some(NpcMovement {
            movement_type: if let Some(route) = &patch.route {
                MovementType::Route {
                    rooms: route.clone(),
                    current_idx: 0,
                    loop_route: patch.loop_route.unwrap_or(true),
                }
            } else if let Some(random) = &patch.random_rooms {
                MovementType::RandomSet { rooms: random.clone() }
            } else {
                warn!("modifyNpc patch for '{npc_symbol}' requested new movement without route or random rooms");
                return Ok(());
            },
            timing: patch.timing.clone().unwrap_or(MovementTiming::EveryNTurns { turns: 1 }),
            active: patch.active.unwrap_or(true),
            last_moved_turn: current_turn,
            paused_until: None,
        });
    }

    if let Some(movement) = npc.movement.as_mut() {
        if let Some(route) = &patch.route {
            let loop_setting = patch.loop_route.unwrap_or(match &movement.movement_type {
                MovementType::Route { loop_route, .. } => *loop_route,
                MovementType::RandomSet { .. } => true,
            });
            movement.movement_type = MovementType::Route {
                rooms: route.clone(),
                current_idx: 0,
                loop_route: loop_setting,
            };
        } else if let Some(random) = &patch.random_rooms {
            movement.movement_type = MovementType::RandomSet { rooms: random.clone() };
        }

        if let Some(loop_setting) = patch.loop_route {
            if let MovementType::Route { loop_route, .. } = &mut movement.movement_type {
                *loop_route = loop_setting;
            } else {
                warn!("modifyNpc patch attempted to set loop on a non-route movement for '{npc_symbol}'");
            }
        }

        if let Some(timing) = &patch.timing {
            movement.timing = timing.clone();
            movement.last_moved_turn = current_turn;
        }

        if let Some(active) = patch.active {
            movement.active = active;
        }
    } else {
        warn!("modifyNpc patch for '{npc_symbol}' requested movement updates but NPC has no movement configured");
    }

    Ok(())
}

/// Clones an `Item`, modifies contents, and returns the updated `Item`
fn apply_item_patch(world: &mut AmbleWorld, item_id: &ItemId, patch: &ItemPatch) -> Result<Item> {
    let mut patched = if let Some(old_item) = world.items.get(item_id) {
        old_item.clone()
    } else {
        bail!("patching an item: ItemId ({item_id}) not found in world item map");
    };

    if let Some(ref new_name) = patch.name {
        patched.name.clone_from(new_name);
    }

    if let Some(ref new_desc) = patch.desc {
        patched.description.clone_from(new_desc);
    }

    if patch.text.is_some() {
        patched.text.clone_from(&patch.text);
    }

    if let Some(ref new_mov) = patch.movability {
        patched.movability.clone_from(new_mov);
    }

    if let Some(new_visibility) = patch.visibility {
        patched.visibility = new_visibility;
    }

    if patch.visible_when.is_some() {
        patched.visible_when.clone_from(&patch.visible_when);
    }

    if let Some(new_aliases) = &patch.aliases {
        patched.aliases.clone_from(new_aliases);
    }

    if patch.remove_container_state {
        if patch.container_state.is_some() {
            error!("modifyItem patch cannot set and remove container state simultaneously");
        } else if !patched.contents.is_empty() {
            error!(
                "modifyItem cannot remove container state from '{}' while it still holds items",
                patched.symbol
            );
        } else {
            patched.container_state = None;
        }
    } else if let Some(new_state) = patch.container_state {
        patched.container_state = Some(new_state);
    }

    for removal in &patch.remove_abilities {
        patched.abilities.remove(removal);
    }
    for addition in &patch.add_abilities {
        patched.abilities.insert(addition.clone());
    }

    Ok(patched)
}
