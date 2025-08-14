//! `repl::item` module
//!
//! Contains repl loop handlers for commands that affect item state

use std::collections::HashSet;

use crate::{
    AmbleWorld, View, ViewItem, WorldObject,
    item::{ContainerState, ItemAbility, ItemInteractionType, consume},
    loader::items::interaction_requirement_met,
    repl::{entity_not_found, find_world_object},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::{Context, Result};
use colored::Colorize;
use log::{info, warn};
use uuid::Uuid;

/// Use one item on another item in a specific way
pub fn use_item_on_handler(
    world: &mut AmbleWorld,
    view: &mut View,
    interaction: ItemInteractionType,
    tool: &str,
    target: &str,
) -> Result<()> {
    // make sure we can find valid matches for tool and target items and notify player if not
    let items_nearby = &world.player_room_ref()?.contents;
    let target_scope: HashSet<_> = items_nearby.union(&world.player.inventory).collect();
    let maybe_target =
        find_world_object(target_scope, &world.items, &world.npcs, target).and_then(super::WorldEntity::item);
    let maybe_tool =
        find_world_object(&world.player.inventory, &world.items, &world.npcs, tool).and_then(super::WorldEntity::item);
    if maybe_target.is_none() {
        view.push(ViewItem::ActionFailure(format!(
            "You don't see any {} nearby.",
            target.error_style()
        )));
        return Ok(());
    }
    if maybe_tool.is_none() {
        view.push(ViewItem::ActionFailure(format!(
            "You don't have any {} in inventory.",
            tool.error_style()
        )));
        return Ok(());
    }
    // unwrap OK here because we just checked for None above
    let target = maybe_target.unwrap();
    let tool = maybe_tool.unwrap();
    let target_name = target.name().to_string();
    let target_id = target.id();
    let tool_name = tool.name().to_string();
    let tool_id = tool.id();
    let tool_is_consumable = tool.consumable.is_some();

    // Can you even do this to the target? ("burn water with lighter" -> you can't burn water!)
    // This may have unwanted consequences when items don't have a specifice reaction defined but it's still
    // a reasonable command to try -- e.g. "clean flamethrower with towel" -- reasonable to do but won't have a defined requirement for cleaning
    // message would be "You can't clean the flamethrower!"... problematic.
    // if !target.interaction_requires.contains_key(&interaction) {
    //     view.push(ViewItem::ActionFailure(format!(
    //         "You can't {interaction} the {target_name}!"
    //     )));
    // }

    // check if these items can interact in this way
    if !interaction_requirement_met(interaction, target, tool) {
        view.push(ViewItem::ActionFailure(format!(
            "You can't do that with a {}!",
            tool.name().item_style(),
        )));
        info!(
            "Player tried to {:?} {} ({}) with {} ({})",
            interaction,
            target.name(),
            target.id(),
            tool.name(),
            tool.id()
        );
        return Ok(());
    }
    // do the interaction as appropriate
    let sent_interaction = interaction;
    let sent_target_id = target.id();

    // This is needed for the UseItem TriggerCondition. ItemAbility::Use is a reasonable default but
    // should never come up, since the presence of this interaction is already verified by the
    // interaction_requirement_met(...) call above.
    let used_ability = target
        .interaction_requires
        .get(&interaction)
        .unwrap_or(&ItemAbility::Use)
        .clone();

    let fired = check_triggers(
        world,
        view,
        &[
            TriggerCondition::UseItemOnItem {
                interaction,
                target_id,
                tool_id,
            },
            TriggerCondition::ActOnItem {
                action: interaction,
                target_id: target.id(),
            },
            TriggerCondition::UseItem {
                item_id: tool_id,
                ability: used_ability,
            },
        ],
    )?;
    // check to see if the ActOnItem trigger we just sent fired
    // (that's the one that will actually change world state --
    // an additional UseItemOnItem can provide flavor for a
    // particular item with the required ability.)
    let interaction_fired = triggers_contain_condition(&fired, |cond| match cond {
        TriggerCondition::ActOnItem { action, target_id } => {
            *action == sent_interaction && *target_id == sent_target_id
        },
        _ => false,
    });

    if !interaction_fired {
        view.push(ViewItem::ActionFailure(format!(
            "{}",
            world.spin_spinner(SpinnerType::NoEffect, "That appears to have had no effect, Captain.",)
        )));
        info!("No matching trigger for {interaction:?} {target_name} ({target_id}) with {tool_name} ({tool_id})");
    }
    if tool_is_consumable {
        let uses_left = consume(world, &tool_id, used_ability)?;
        if let Some(uses_left) = uses_left {
            if uses_left == 0 {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} has no more uses left.",
                    tool_name.item_style()
                )));
            } else {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} has {} use{} left",
                    tool_name.item_style(),
                    uses_left,
                    if uses_left == 1 { "" } else { "s" }
                )));
            }
        }
    }
    Ok(())
}
/// Turns something on, if it can be turned on
pub fn turn_on_handler(world: &mut AmbleWorld, view: &mut View, item_pattern: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    if let Some(entity) = find_world_object(&current_room.contents, &world.items, &world.npcs, item_pattern) {
        if let Some(item) = entity.item() {
            if item.abilities.contains(&ItemAbility::TurnOn) {
                info!("Player switched on {} ({})", item.name(), item.id());
                let sent_id = item.id();
                let fired_triggers = check_triggers(
                    world,
                    view,
                    &[TriggerCondition::UseItem {
                        item_id: sent_id,
                        ability: ItemAbility::TurnOn,
                    }],
                )?;
                let sent_trigger_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
                    TriggerCondition::UseItem { item_id, ability } => {
                        *item_id == sent_id && *ability == ItemAbility::TurnOn
                    },
                    _ => false,
                });
                if !sent_trigger_fired {
                    view.push(ViewItem::ActionFailure(format!(
                        "{}",
                        "You hear a clicking sound and then... nothing happens.".italic()
                    )));
                }
            } else {
                info!(
                    "Player tried to turn on unswitchable item {} ({})",
                    item.name(),
                    item.id()
                );
                view.push(ViewItem::ActionFailure(format!(
                    "The {} can't be turned on.",
                    item.name().item_style()
                )));
            }
        } else if let Some(npc) = entity.npc() {
            info!("Player tried to turn on an NPC {} ({})", npc.name(), npc.id());
            view.push(ViewItem::ActionFailure(format!(
                "{} is impervious to your attempt at seduction.",
                npc.name().npc_style()
            )));
        }
    } else {
        entity_not_found(world, view, item_pattern);
    }
    Ok(())
}

/// Opens an item if it is a closed, unlocked container.
pub fn open_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    // search player's location for an item matching search
    let room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = room.contents.union(&world.player.inventory).copied().collect();
    let (container_id, name) =
        if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, pattern) {
            if let Some(item) = entity.item() {
                (item.id(), item.name().to_string())
            } else {
                warn!("Player attempted to open a non-Item WorldEntity by searching ({pattern})");
                view.push(ViewItem::Error(format!(
                    "{} isn't an item. You can't open it.",
                    pattern.error_style()
                )));
                return Ok(());
            }
        } else {
            entity_not_found(world, view, pattern);
            return Ok(());
        };

    if let Some(target_item) = world.get_item_mut(container_id) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} can't be opened.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked) => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} is locked. You'll have to unlock it first.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already open.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Closed) => {
                target_item.container_state = Some(ContainerState::Open);
                view.push(ViewItem::ActionSuccess(format!(
                    "You opened the {}.\n",
                    target_item.name().item_style()
                )));
                info!("{} opened the {} ({})", world.player.name(), name, container_id,);
                check_triggers(world, view, &[TriggerCondition::Open(container_id)])?;
            },
        }
    }
    Ok(())
}

/// Closes a container item nearby.
pub fn close_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = room.contents.union(&world.player.inventory).copied().collect();
    let (uuid, name) = if let Some(entity) = find_world_object(&search_scope, &world.items, &world.npcs, pattern) {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Close({pattern}) matched a non-Item WorldEntity");
            view.push(ViewItem::Error(format!(
                "You do not see a {} to close.",
                pattern.error_style()
            )));
            return Ok(());
        }
    } else {
        entity_not_found(world, view, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(uuid) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} can't be closed.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Closed | ContainerState::Locked) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already closed.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open) => {
                target_item.container_state = Some(ContainerState::Closed);
                view.push(ViewItem::ActionSuccess(format!(
                    "You closed the {}.\n",
                    target_item.name().item_style()
                )));
                info!("{} closed the {} ({})", world.player.name(), name, uuid);
            },
        }
    }
    Ok(())
}

/// Locks a container item nearby.
pub fn lock_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let (uuid, name) = if let Some(entity) = find_world_object(&room.contents, &world.items, &world.npcs, pattern) {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Lock({pattern}) matched a non-Item WorldEntity");
            view.push(ViewItem::Error(format!(
                "You don't see a {} here to lock.",
                pattern.error_style()
            )));
            return Ok(());
        }
    } else {
        entity_not_found(world, view, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(uuid) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} isn't something that can be locked.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already locked.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open | ContainerState::Closed) => {
                target_item.container_state = Some(ContainerState::Locked);
                view.push(ViewItem::ActionSuccess(format!(
                    "You locked the {}.\n",
                    target_item.name().item_style()
                )));
                info!("{} locked the {} ({})", world.player.name(), name, uuid);
            },
        }
    }
    Ok(())
}

/// Unlocks and opens an item nearby.
pub fn unlock_handler(world: &mut AmbleWorld, view: &mut View, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let (container_id, container_name) =
        if let Some(entity) = find_world_object(&room.contents, &world.items, &world.npcs, pattern) {
            if let Some(item) = entity.item() {
                (item.id(), item.name().to_string())
            } else {
                warn!("Command:Unlock({pattern}) matched a non-Item (NPC) WorldEntity");
                view.push(ViewItem::Error(format!(
                    "You don't see a {} here to unlock.",
                    pattern.error_style()
                )));
                return Ok(());
            }
        } else {
            entity_not_found(world, view, pattern);
            return Ok(());
        };

    // Check player inventory for valid key
    let has_valid_key = world.player.inventory.iter().any(|id| {
        world.items.get(id).is_some_and(|i| {
            i.abilities.iter().any(|a| match a {
                ItemAbility::Unlock(Some(target)) => *target == container_id,
                ItemAbility::Unlock(None) => true, // universal key
                _ => false,
            })
        })
    });

    if let Some(target_item) = world.get_item_mut(container_id) {
        match target_item.container_state {
            None => {
                view.push(ViewItem::ActionFailure(format!(
                    "The {} doesn't have a lock.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Open | ContainerState::Closed) => {
                view.push(ViewItem::ActionSuccess(format!(
                    "The {} is already unlocked.",
                    target_item.name().item_style()
                )));
            },
            Some(ContainerState::Locked) => {
                if has_valid_key {
                    target_item.container_state = Some(ContainerState::Closed);
                    view.push(ViewItem::ActionSuccess(format!(
                        "You unlocked the {}.\n",
                        target_item.name().item_style()
                    )));
                    info!(
                        "{} unlocked the {} ({})",
                        world.player.name(),
                        container_name,
                        container_id
                    );
                    check_triggers(world, view, &[TriggerCondition::Unlock(container_id)])?;
                } else {
                    view.push(ViewItem::ActionFailure(format!(
                        "You don't have anything that can unlock the {}.",
                        target_item.name().item_style()
                    )));
                }
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        item::{ContainerState, Item, ItemAbility, ItemInteractionType},
        room::Room,
        trigger::{Trigger, TriggerAction, TriggerCondition},
        world::{AmbleWorld, Location},
    };
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    fn build_world() -> (AmbleWorld, View, Uuid, Uuid, Uuid, Uuid) {
        let mut world = AmbleWorld::new_empty();
        let room_id = Uuid::new_v4();
        let room = Room {
            id: room_id,
            symbol: "r".into(),
            name: "Room".into(),
            base_description: String::new(),
            overlays: Vec::new(),
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::new(),
            npcs: HashSet::new(),
        };
        world.rooms.insert(room_id, room);
        world.player.location = Location::Room(room_id);

        let container_id = Uuid::new_v4();
        let mut container = Item {
            id: container_id,
            symbol: "c".into(),
            name: "chest".into(),
            description: String::new(),
            location: Location::Room(room_id),
            portable: true,
            container_state: Some(ContainerState::Locked),
            restricted: false,
            contents: HashSet::new(),
            abilities: HashSet::new(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        container
            .interaction_requires
            .insert(ItemInteractionType::Open, ItemAbility::Pry);
        world.rooms.get_mut(&room_id).unwrap().contents.insert(container_id);
        world.items.insert(container_id, container);

        let tool_id = Uuid::new_v4();
        let tool = Item {
            id: tool_id,
            symbol: "t".into(),
            name: "crowbar".into(),
            description: String::new(),
            location: Location::Inventory,
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: [ItemAbility::Pry].into_iter().collect(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.player.inventory.insert(tool_id);
        world.items.insert(tool_id, tool);

        let lamp_id = Uuid::new_v4();
        let lamp = Item {
            id: lamp_id,
            symbol: "l".into(),
            name: "lamp".into(),
            description: String::new(),
            location: Location::Room(room_id),
            portable: false,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: [ItemAbility::TurnOn].into_iter().collect(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.rooms.get_mut(&room_id).unwrap().contents.insert(lamp_id);
        world.items.insert(lamp_id, lamp);

        let key_id = Uuid::new_v4();
        let key = Item {
            id: key_id,
            symbol: "k".into(),
            name: "key".into(),
            description: String::new(),
            location: Location::Inventory,
            portable: true,
            container_state: None,
            restricted: false,
            contents: HashSet::new(),
            abilities: [ItemAbility::Unlock(Some(container_id))].into_iter().collect(),
            interaction_requires: HashMap::new(),
            text: None,
            consumable: None,
        };
        world.player.inventory.insert(key_id);
        world.items.insert(key_id, key);
        let view = View::new();

        (world, view, container_id, tool_id, lamp_id, key_id)
    }

    #[test]
    fn use_item_on_handler_unlocks_container() {
        let (mut world, mut view, container_id, tool_id, _, _) = build_world();
        world.triggers.push(Trigger {
            name: "open".into(),
            conditions: vec![TriggerCondition::UseItemOnItem {
                interaction: ItemInteractionType::Open,
                target_id: container_id,
                tool_id,
            }],
            actions: vec![TriggerAction::UnlockItem(container_id)],
            only_once: false,
            fired: false,
        });
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
        use_item_on_handler(&mut world, &mut view, ItemInteractionType::Open, "crowbar", "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn use_item_on_handler_without_ability_does_nothing() {
        let (mut world, mut view, container_id, tool_id, _, _) = build_world();
        world.items.get_mut(&tool_id).unwrap().abilities.clear();
        world.triggers.push(Trigger {
            name: "open".into(),
            conditions: vec![TriggerCondition::UseItemOnItem {
                interaction: ItemInteractionType::Open,
                target_id: container_id,
                tool_id,
            }],
            actions: vec![TriggerAction::UnlockItem(container_id)],
            only_once: false,
            fired: false,
        });
        use_item_on_handler(&mut world, &mut view, ItemInteractionType::Open, "crowbar", "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }

    #[test]
    fn turn_on_handler_triggers_unlock() {
        let (mut world, mut view, container_id, _, lamp_id, _) = build_world();
        world.triggers.push(Trigger {
            name: "light".into(),
            conditions: vec![TriggerCondition::UseItem {
                item_id: lamp_id,
                ability: ItemAbility::TurnOn,
            }],
            actions: vec![TriggerAction::UnlockItem(container_id)],
            only_once: false,
            fired: false,
        });
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
        turn_on_handler(&mut world, &mut view, "lamp").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn open_handler_opens_closed_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Closed);
        open_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Open)
        );
    }

    #[test]
    fn open_handler_locked_container_stays_locked() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        open_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }

    #[test]
    fn close_handler_closes_open_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Open);
        close_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Closed)
        );
    }

    #[test]
    fn lock_handler_locks_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        world.items.get_mut(&container_id).unwrap().container_state = Some(ContainerState::Closed);
        lock_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }

    #[test]
    fn unlock_handler_with_key_unlocks_container() {
        let (mut world, mut view, container_id, _, _, _) = build_world();
        unlock_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Closed)
        );
    }

    #[test]
    fn unlock_handler_without_key_does_not_unlock() {
        let (mut world, mut view, container_id, _, _, key_id) = build_world();
        world.player.inventory.remove(&key_id);
        unlock_handler(&mut world, &mut view, "chest").unwrap();
        assert_eq!(
            world.items.get(&container_id).unwrap().container_state,
            Some(ContainerState::Locked)
        );
    }
}
