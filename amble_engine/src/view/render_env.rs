//! # Render Env(ironment) Module
//!
//! This module contains the individual `ViewItem` renderers for the "environment" section
//! of an output frame.

use crate::{
    View, ViewItem,
    markup::{StyleKind, StyleMods, WrapMode, render_inline, render_wrapped},
    style::GameStyle as _,
    view::ViewMode,
};

/// Used by `flush()` to show base room description
pub(super) fn room_description(view: &mut View) {
    if let Some(ViewItem::RoomDescription {
        name,
        description,
        visited,
        force_mode,
    }) = view.items.iter().find_map(|i| match i.view_item {
        ViewItem::RoomDescription { .. } => Some(&i.view_item),
        _ => None,
    }) {
        // Use the forced display mode if there is one, otherwise use current setting
        let display_mode = force_mode.unwrap_or(view.mode);
        if display_mode == ViewMode::ClearVerbose {
            // clear the screen
            print!("\x1B[2J\x1B[H");
        }
        println!("{:^width$}", name.room_titlebar_style(), width = view.width);
        if display_mode != ViewMode::Brief || !visited {
            println!(
                "{}",
                render_wrapped(
                    description,
                    view.width,
                    WrapMode::Normal,
                    StyleKind::Description,
                    StyleMods::default(),
                )
            );

            println!();
        }
    }
}

pub(super) fn room_overlays(view: &mut View) {
    // Note: force_mode is passed with a RoomOverlay item but currently unused
    // (overlays are displayed regardless of view mode)
    if let Some(ViewItem::RoomOverlays { text, .. }) = view.items.iter().find_map(|i| match i.view_item {
        ViewItem::RoomOverlays { .. } => Some(&i.view_item),
        _ => None,
    }) {
        let bullet_prefix = "• ";
        let indent_prefix = "  ";
        let wrap_width = view.width.saturating_sub(indent_prefix.len()).max(1);

        for overlay in text {
            let wrapped = render_wrapped(
                overlay,
                wrap_width,
                WrapMode::Normal,
                StyleKind::Overlay,
                StyleMods::default(),
            );
            let mut lines = wrapped.lines();
            if let Some(first_line) = lines.next() {
                if first_line.is_empty() {
                    println!("{bullet_prefix}");
                } else {
                    println!("{bullet_prefix}{first_line}");
                }
            } else {
                println!("{bullet_prefix}");
            }
            for line in lines {
                if line.is_empty() {
                    println!("{indent_prefix}");
                } else {
                    println!("{indent_prefix}{line}");
                }
            }
        }
        println!();
    }
}

pub(super) fn room_item_list(view: &mut View) {
    if let Some(ViewItem::RoomItems(names)) = view.items.iter_mut().find_map(|i| match i.view_item {
        ViewItem::RoomItems(_) => Some(&mut i.view_item),
        _ => None,
    }) {
        println!("{}:", "Items".subheading_style());
        names.sort();
        for name in names {
            println!("    * {}", name.item_style());
        }
    }
}

pub(super) fn room_exit_list(view: &mut View) {
    if let Some(ViewItem::RoomExits(exit_lines)) = view.items.iter_mut().find_map(|i| match i.view_item {
        ViewItem::RoomExits(_) => Some(&mut i.view_item),
        _ => None,
    }) {
        println!("{}:", "Exits".subheading_style());
        exit_lines.sort();
        for exit in exit_lines {
            print!("    > ");
            match (exit.dest_visited, exit.exit_locked) {
                (true, false) => println!(
                    "{} (to {})",
                    exit.direction.exit_visited_style(),
                    exit.destination.room_style()
                ),
                (true, true) => println!(
                    "{} (to {})",
                    exit.direction.exit_locked_style(),
                    exit.destination.room_style()
                ),
                (false, true) => println!("{}", exit.direction.exit_locked_style()),
                (false, false) => println!("{}", exit.direction.exit_unvisited_style()),
            }
        }
        println!();
    }
}

/// Render a `ViewItem::RoomNpcs` into a list of NPCs and their descriptions.
pub(super) fn room_npc_list(view: &mut View) {
    if let Some(ViewItem::RoomNpcs(npcs)) = view.items.iter().find_map(|i| match i.view_item {
        ViewItem::RoomNpcs(_) => Some(&i.view_item),
        _ => None,
    }) {
        println!("{}:", "Others".subheading_style());
        for npc_line in npcs {
            println!(
                "    {} - {}",
                npc_line.name.npc_style(),
                render_inline(&npc_line.description, StyleKind::Description, StyleMods::default())
            );
        }
    }
}
