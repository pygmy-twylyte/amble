//! `repl::inventory` module
//!
//! Contains repl loop handlers for commands that affect player inventory

use std::collections::HashMap;

use crate::{
    AmbleWorld, ItemHolder, Location, WorldObject,
    npc::Npc,
    repl::{entity_not_found, find_world_object},
    spinners::SpinnerType,
    style::GameStyle,
    trigger::{TriggerAction, TriggerCondition, check_triggers, triggers_contain_condition},
};

use anyhow::{Context, Result};
use colored::Colorize;
use log::{info, warn};
use uuid::Uuid;

/// Selects an NPC in given location by first partial name match.
fn select_npc<'a>(location: &Location, world_npcs: &'a HashMap<Uuid, Npc>, query: &str) -> Option<&'a Npc> {
    let npcs_in_room = world_npcs
        .values()
        .filter(|npc| npc.location() == location)
        .collect::<Vec<_>>();
    let query = query.to_lowercase();
    npcs_in_room
        .into_iter()
        .find(|&npc| npc.name().to_lowercase().contains(&query))
}
/// Handles TalkTo(npc) commands
pub fn talk_to_handler(world: &mut AmbleWorld, npc_name: &str) -> Result<()> {
    // find one that matches npc_name in present room
    let sent_id = if let Some(npc) = select_npc(world.player.location(), &world.npcs, npc_name) {
        npc.id()
    } else {
        entity_not_found(world, npc_name);
        return Ok(());
    };

    // check for any condition-specific dialogue
    let fired_triggers = check_triggers(world, &[TriggerCondition::TalkToNpc(sent_id)])?;
    let dialogue_fired = triggers_contain_condition(&fired_triggers, |cond| match cond {
        TriggerCondition::TalkToNpc(npc_id) => sent_id == *npc_id,
        _ => false,
    });

    // if no dialogue was triggered, fire random response according to Npc's mood
    if !dialogue_fired && let Some(npc) = world.npcs.get(&sent_id) {
        let stem = format!("{}", npc.name().npc_style());
        if let Some(ignore_spinner) = world.spinners.get(&SpinnerType::NpcIgnore) {
            let dialogue = npc.random_dialogue(ignore_spinner);
            println!("{stem}:\n{dialogue}");
            info!("NPC \"{}\" ({}) said \"{}\"", npc.name(), npc.id(), dialogue);
        }
    }
    Ok(())
}

/// Gives an inventory item to an NPC.
///
/// Transfer only occurs if there is a specific trigger causing the NPC to accept the item
/// from the player. Otherwise, player is informed that the NPC won't accept it.
///
/// # Errors
/// - on failed uuid lookups
pub fn give_to_npc_handler(world: &mut AmbleWorld, item: &str, npc: &str) -> Result<()> {
    // find the target npc in the current room and collect metadata
    let current_room = world.player_room_ref()?;
    let (npc_id, npc_name) = if let Some(entity) = find_world_object(&current_room.npcs, &world.items, &world.npcs, npc)
    {
        if let Some(npc) = entity.npc() {
            (npc.id(), npc.name.to_string())
        } else {
            println!(
                "{} matches an item. Did you mean 'put {} in {}'?",
                npc.error_style(),
                item.italic(),
                npc.italic()
            );
            return Ok(());
        }
    } else {
        entity_not_found(world, npc);
        return Ok(());
    };

    // find the target item in inventory, ensure it's portable, collect metadata
    let (item_id, item_name) =
        if let Some(entity) = find_world_object(&world.player.inventory, &world.items, &world.npcs, item) {
            if let Some(item) = entity.item() {
                if !item.portable {
                    info!("player tried to move fixed item {} ({})", item.name(), item.id());
                    println!("Sorry, the {} isn't portable.", item.name().error_style());
                    return Ok(());
                }
                (item.id(), item.name().to_string())
            } else {
                warn!("non-Item entity matching '{item}' found in inventory");
                println!(
                    "{} matched an entity that shouldn't exist in inventory. Let's pretend this never happened.",
                    item.error_style()
                );
                return Ok(());
            }
        } else {
            entity_not_found(world, item);
            return Ok(());
        };

    let fired_triggers = check_triggers(world, &[TriggerCondition::GiveToNpc { item_id, npc_id }])?;
    let fired = fired_triggers.iter().any(|&trigger| {
        trigger
            .conditions
            .iter()
            .any(|cond| matches!(cond, TriggerCondition::GiveToNpc { .. }))
    });

    let refused = fired_triggers.iter().any(|t| {
        t.actions
            .iter()
            .any(|a| matches!(a, TriggerAction::NpcRefuseItem { .. }))
    });

    // the trigger fired -- proceed with item transfer if it wasn't a refusal
    if fired && !refused {
        // set new location in NPC on world item
        world
            .get_item_mut(item_id)
            .with_context(|| format!("looking up item {item_id}"))?
            .set_location_npc(npc_id);

        // add to npc inventory
        world
            .npcs
            .get_mut(&npc_id)
            .with_context(|| format!("looking up NPC {npc_id}"))?
            .add_item(item_id);

        // remove from player inventory
        world.player.remove_item(item_id);
        check_triggers(world, &[TriggerCondition::Drop(item_id)])?;

        // report and log success
        println!("You gave the {} to {}.\n", item_name.item_style(), npc_name.npc_style());
        info!(
            "{} gave {} ({}) to {} ({})",
            world.player.name(),
            item_name,
            item_id,
            npc_name,
            npc_id
        );
    // trigger didn't fire, so NPC refuses the item
    } else {
        // show a default message if there wasn't a specific refusal trigger fired to do it
        if !fired {
            println!(
                "{} has no use for {}, and won't hold it for you.",
                npc_name.npc_style(),
                item_name.item_style()
            );
        }
        info!("{npc_name} ({npc_id}) refused a gift of {item_name} ({item_id})");
    }
    Ok(())
}
