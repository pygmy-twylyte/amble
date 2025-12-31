//! module Render Player
//!
//! This module contains the individual `ViewItem` renderers for player-related
//! feedback about the current game, such as inventory, goals status

use colored::Colorize as _;

use crate::{
    View, ViewItem,
    markup::{StyleKind, StyleMods, WrapMode, render_wrapped},
    style::GameStyle as _,
};

/// Displays the player's inventory list from `ViewItem::Inventory`
pub(super) fn inventory(view: &mut View) {
    if let Some(entry) = view
        .items
        .iter_mut()
        .find(|i| matches!(i.view_item, ViewItem::Inventory(..)))
        && let ViewItem::Inventory(item_lines) = &mut entry.view_item
    {
        println!("{}:", "Inventory".subheading_style());
        if item_lines.is_empty() {
            println!("   {}", "You have... nothing at all.".italic().dimmed());
        } else {
            item_lines.sort();
            for line in item_lines {
                println!("   {}", line.item_name.item_style());
            }
        }
    }
}

/// Displays the current lists of active goals (with descriptions) and a list of
/// (crossed-out) completed goals.
pub(super) fn goals(view: &mut View) {
    let active: Vec<_> = view
        .items
        .iter()
        .filter(|i| matches!(i.view_item, ViewItem::ActiveGoal { .. }))
        .collect();

    let complete: Vec<_> = view
        .items
        .iter()
        .filter(|i| matches!(i.view_item, ViewItem::CompleteGoal { .. }))
        .collect();

    if active.is_empty() && complete.is_empty() {
        return;
    }

    println!("{}:", "Active Goals".subheading_style());
    if active.is_empty() {
        println!("   {}", "All goals met - explore to find more!\n".italic().dimmed());
    } else {
        for goal in active {
            if let ViewItem::ActiveGoal { name, description } = &goal.view_item {
                println!("{}", name.goal_active_style());
                println!(
                    "{}",
                    render_wrapped(
                        description,
                        view.width,
                        WrapMode::Indented,
                        StyleKind::Description,
                        StyleMods::default(),
                    )
                );
            }
        }
        println!();
    }

    if !complete.is_empty() {
        println!("{}:", "Completed Goals".subheading_style());
        for goal in complete {
            if let ViewItem::CompleteGoal { name, .. } = &goal.view_item {
                println!("{}", name.goal_complete_style());
            }
        }
    }
}
