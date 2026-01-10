//! module Render NPC
//!
//! This module contains the individual `ViewItem` renderers for `Npc`-related feedback,
//! as well as the display of NPC entrance/exit and speech messages.

use colored::Colorize as _;
use textwrap::fill;

use crate::{
    View, ViewItem,
    markup::{StyleKind, StyleMods, WrapMode, render_wrapped},
    npc::NpcState,
    style::{GameStyle as _, indented_block, normal_block},
    view::ViewEntry,
    view::icons::{ICON_NPC_ENTER, ICON_NPC_LEAVE},
};

pub(super) fn npc_detail(view: &mut View) {
    if let Some(entry) = view
        .items
        .iter()
        .find(|i| matches!(i.view_item, ViewItem::NpcDescription { .. }))
        && let ViewItem::NpcDescription {
            name,
            description,
            health,
            state,
        } = &entry.view_item
    {
        println!("{}", name.npc_style().underline());
        let formatted_state = if let NpcState::Custom(custom_state) = state {
            custom_state.highlight()
        } else {
            state.to_string().highlight()
        };
        println!(
            "{}",
            fill(
                format!(
                    "Health: {}/{} | State: {}",
                    health.current_hp().to_string().highlight(),
                    health.max_hp().to_string().highlight(),
                    formatted_state
                )
                .as_str(),
                indented_block()
            )
        );
        // if description has a multiple lines, bold the first as a tagline - otherwise
        // use the whole thing as the tagline + description.
        if let Some((tagline, rest)) = description.split_once('\n') {
            println!(
                "{}",
                render_wrapped(
                    tagline,
                    view.width,
                    WrapMode::Indented,
                    StyleKind::Description,
                    StyleMods {
                        bold: true,
                        ..StyleMods::default()
                    },
                )
            );
            println!(
                "{}",
                render_wrapped(
                    rest,
                    view.width,
                    WrapMode::Indented,
                    StyleKind::Description,
                    StyleMods::default(),
                )
            );
        } else {
            println!(
                "{}",
                render_wrapped(
                    description,
                    view.width,
                    WrapMode::Indented,
                    StyleKind::Description,
                    StyleMods {
                        bold: true,
                        ..StyleMods::default()
                    },
                )
            );
        }
        println!();
    }
    if let Some(ViewItem::NpcInventory(content_lines)) = view.items.iter().find_map(|i| match i.view_item {
        ViewItem::NpcInventory(_) => Some(&i.view_item),
        _ => None,
    }) {
        println!("{}:", "Inventory".subheading_style());
        if content_lines.is_empty() {
            println!("   {}", "(Empty)".dimmed().italic());
        } else {
            for line in content_lines {
                println!(
                    "   {} {}",
                    line.item_name.item_style(),
                    if line.restricted { "[R]" } else { "" }
                );
            }
        }
    }
}

/// Collects and sorts, and then displays events related to NPC activities.
pub(super) fn npc_events_sorted(entries: &[&ViewEntry]) {
    // Collect all NPC-related events
    let mut npc_enters: Vec<_> = entries
        .iter()
        .cloned()
        .filter(|i| i.view_item.is_npc_entered())
        .collect();
    let mut npc_leaves: Vec<_> = entries.iter().cloned().filter(|i| i.view_item.is_npc_left()).collect();
    let speech_msgs: Vec<_> = entries
        .iter()
        .cloned()
        .filter(|i| i.view_item.is_npc_speech())
        .collect();

    // Sort by NPC name for consistent ordering
    npc_enters.sort_by(|a, b| a.view_item.npc_name().cmp(b.view_item.npc_name()));
    npc_leaves.sort_by(|a, b| a.view_item.npc_name().cmp(b.view_item.npc_name()));

    let has_events = !npc_enters.is_empty() || !npc_leaves.is_empty() || !speech_msgs.is_empty();

    // Display entered events first
    for msg in npc_enters {
        if let ViewItem::NpcEntered { npc_name, spin_msg } = &msg.view_item {
            let formatted = format!(
                "{:<4}{}",
                ICON_NPC_ENTER.trig_icon_style(),
                format!("{} {spin_msg}", npc_name.npc_style()).npc_movement_style()
            );
            println!("{}", fill(formatted.as_str(), normal_block()));
        }
    }

    // Then display speech events
    for quote in speech_msgs {
        if let ViewItem::NpcSpeech { speaker, quote } = &quote.view_item {
            println!("{}:", speaker.npc_style());
            println!(
                "{}",
                fill((String::from("\"") + quote.as_str() + "\"").as_str(), indented_block())
                    .clone()
                    .npc_quote_style()
            );
        }
    }

    // Finally display left events
    for msg in npc_leaves {
        if let ViewItem::NpcLeft { npc_name, spin_msg } = &msg.view_item {
            let formatted = format!(
                "{:<4}{}",
                ICON_NPC_LEAVE.trig_icon_style(),
                format!("{} {spin_msg}", npc_name.npc_style()).npc_movement_style()
            );
            println!("{}", fill(formatted.as_str(), normal_block()));
        }
    }

    // Add spacing if any NPC events were displayed
    if has_events {
        println!();
    }
}
