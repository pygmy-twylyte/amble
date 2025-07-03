//! `repl::item` module
//!
//! Contains repl loop handlers for commands that affect item state

use std::collections::HashSet;

use crate::{
    AmbleWorld, WorldObject,
    item::{ItemAbility, ItemInteractionType},
    loader::items::interaction_requirement_met,
    repl::{entity_not_found, find_world_object},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::Result;
use colored::Colorize;
use log::{info, warn};
use uuid::Uuid;

/// Use one item on another item in a specific way
pub fn use_item_on_handler(
    world: &mut AmbleWorld,
    interaction: ItemInteractionType,
    tool: &str,
    target: &str,
) -> Result<()> {
    // make sure we can find valid matches for tool and target items and notify player if not
    let items_nearby = &world.player_room_ref()?.contents;
    let target_scope: HashSet<_> = items_nearby.union(&world.player.inventory).collect();
    let maybe_target =
        find_world_object(target_scope, &world.items, &world.npcs, target).and_then(|e| e.item());
    let maybe_tool = find_world_object(&world.player.inventory, &world.items, &world.npcs, tool)
        .and_then(|e| e.item());
    if maybe_target.is_none() {
        println!("You don't see any {} nearby.", target.error_style());
        return Ok(());
    }
    if maybe_tool.is_none() {
        println!("You don't have any {} in inventory.", tool.error_style());
        return Ok(());
    }
    // unwrap OK here because we just checked for None above
    let target = maybe_target.unwrap();
    let tool = maybe_tool.unwrap();
    let target_name = target.name().to_string();
    let target_id = target.id().clone();
    let tool_name = tool.name().to_string();
    let tool_id = tool.id().clone();

    // check if these items can interact in this way
    if !interaction_requirement_met(interaction, target, tool) {
        println!("You can't do that with a {}!", tool.name().item_style(),);
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
    let sent_tool_id = tool.id();
    let fired = check_triggers(
        world,
        &[TriggerCondition::UseItemOnItem {
            interaction,
            target_id: target.id(),
            tool_id: tool.id(),
        }],
    )?;
    // check to see if the trigger we just sent fired
    let reaction_fired = triggers_contain_condition(&fired, |cond| match cond {
        TriggerCondition::UseItemOnItem {
            interaction,
            target_id,
            tool_id,
        } => {
            *interaction == sent_interaction
                && *target_id == sent_target_id
                && *tool_id == sent_tool_id
        }
        _ => false,
    });

    if !reaction_fired {
        println!(
            "{}",
            world.spin_spinner(
                SpinnerType::NoEffect,
                "That appears to have had no effect, Captain.",
            )
        );
        warn!(
            "No matching trigger for {interaction:?} {target_name} ({target_id}) with {tool_name} ({tool_id})"
        );
    }
    Ok(())
}
/// Turns something on, if it can be turned on
pub fn turn_on_handler(world: &mut AmbleWorld, item_pattern: &str) -> Result<()> {
    let current_room = world.player_room_ref()?;
    if let Some(entity) = find_world_object(
        &current_room.contents,
        &world.items,
        &world.npcs,
        item_pattern,
    ) {
        if let Some(item) = entity.clone().item() {
            if item.abilities.contains(&ItemAbility::TurnOn) {
                info!("Player switched on {} ({})", item.name(), item.id());
                let fired = check_triggers(
                    world,
                    &[TriggerCondition::UseItem {
                        item_id: item.id(),
                        ability: ItemAbility::TurnOn,
                    }],
                )?;
                if fired.is_empty() {
                    println!(
                        "{}",
                        "You hear a clicking sound and then... nothing happens.".italic()
                    );
                }
            } else {
                info!(
                    "Player tried to turn on unswitchable item {} ({})",
                    item.name(),
                    item.id()
                );
                println!("The {} can't be turned on.", item.name().item_style());
            }
        } else if let Some(npc) = entity.npc() {
            info!(
                "Player tried to turn on an NPC {} ({})",
                npc.name(),
                npc.id()
            );
            println!(
                "{} is impervious to your attempt at seduction.",
                npc.name().npc_style()
            );
        }
    } else {
        entity_not_found(world, item_pattern);
    }
    Ok(())
}

/// Opens an item if it is a closed, unlocked container.
pub fn open_handler(world: &mut AmbleWorld, pattern: &str) -> Result<()> {
    // search player's location for an item matching search
    let room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = room
        .contents
        .union(&world.player.inventory)
        .copied()
        .collect();
    let (container_id, name) = if let Some(entity) =
        find_world_object(&search_scope, &world.items, &world.npcs, pattern)
    {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Player attempted to open a non-Item WorldEntity by searching ({pattern})");
            println!(
                "{} isn't an item. You can't open it.",
                pattern.error_style()
            );
            return Ok(());
        }
    } else {
        entity_not_found(world, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(container_id) {
        match target_item {
            item if !item.container => {
                println!("The {} can't be opened.", item.name().item_style());
            }
            item if item.locked => println!(
                "The {} is locked. You'll have to unlock it first.",
                item.name().item_style()
            ),
            item if item.open => println!("The {} is already open.", item.name().item_style()),
            item => {
                item.open = true;
                println!("You opened the {}.\n", item.name().item_style());
                info!(
                    "{} opened the {} ({})",
                    world.player.name(),
                    name,
                    container_id,
                );
                check_triggers(world, &[TriggerCondition::Open(container_id)])?;
            }
        }
    }
    Ok(())
}

/// Closes a container item nearby.
pub fn close_handler(world: &mut AmbleWorld, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let search_scope: HashSet<Uuid> = room
        .contents
        .union(&world.player.inventory)
        .copied()
        .collect();
    let (uuid, name) = if let Some(entity) =
        find_world_object(&search_scope, &world.items, &world.npcs, pattern)
    {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Close({pattern}) matched a non-Item WorldEntity");
            println!("You do not see a {} to close.", pattern.error_style());
            return Ok(());
        }
    } else {
        entity_not_found(world, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(uuid) {
        match target_item {
            item if !item.container => {
                println!(
                    "The {} isn't something that can be closed.",
                    item.name().item_style()
                );
            }
            item if !item.open => {
                println!("The {} is already closed.", item.name().item_style());
            }
            item => {
                item.open = false;
                println!("You closed the {}.\n", item.name().item_style());
                info!("{} closed the {} ({})", world.player.name(), name, uuid);
            }
        }
    }
    Ok(())
}

/// Locks a container item nearby.
pub fn lock_handler(world: &mut AmbleWorld, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let (uuid, name) = if let Some(entity) =
        find_world_object(&room.contents, &world.items, &world.npcs, pattern)
    {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Lock({pattern}) matched a non-Item WorldEntity");
            println!("You don't see a {} here to lock.", pattern.error_style());
            return Ok(());
        }
    } else {
        entity_not_found(world, pattern);
        return Ok(());
    };

    if let Some(target_item) = world.get_item_mut(uuid) {
        match target_item {
            item if !item.container => {
                println!(
                    "The {} isn't something that can be locked.",
                    item.name().item_style()
                );
            }
            item if item.locked => {
                println!("The {} is already locked.", item.name().item_style());
            }
            item => {
                item.locked = true;
                println!("You locked the {}.\n", item.name().item_style());
                info!("{} locked the {} ({})", world.player.name(), name, uuid);
            }
        }
    }
    Ok(())
}

/// Unlocks and opens an item nearby.
pub fn unlock_handler(world: &mut AmbleWorld, pattern: &str) -> Result<()> {
    let room = world.player_room_ref()?;
    let (container_id, container_name) = if let Some(entity) =
        find_world_object(&room.contents, &world.items, &world.npcs, pattern)
    {
        if let Some(item) = entity.item() {
            (item.id(), item.name().to_string())
        } else {
            warn!("Command:Unlock({pattern}) matched a non-Item WorldEntity");
            println!("You don't see a {} here to unlock.", pattern.error_style());
            return Ok(());
        }
    } else {
        entity_not_found(world, pattern);
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
        match target_item {
            item if !item.container => {
                println!(
                    "The {} isn't something that can be unlocked.",
                    item.name().item_style()
                );
            }
            item if !item.locked => {
                println!("The {} is already unlocked.", item.name().item_style());
            }
            item => {
                if has_valid_key {
                    item.locked = false;
                    item.open = true;
                    println!("You unlocked the {}.\n", item.name().item_style());
                    info!(
                        "{} unlocked the {} ({})",
                        world.player.name(),
                        container_name,
                        container_id
                    );
                    check_triggers(world, &[TriggerCondition::Unlock(container_id)])?;
                } else {
                    println!(
                        "You don't have anything that can unlock the {}.",
                        item.name().item_style()
                    );
                }
            }
        }
    }
    Ok(())
}
