//! module Render Item
//!
//! This module contains the individual `ViewItem` renderers for `Item`-related feedback.

use colored::Colorize as _;
use textwrap::fill;

use crate::{
    View, ViewItem,
    markup::{StyleKind, StyleMods, WrapMode, render_wrapped},
    style::{GameStyle as _, indented_block},
};

/// Aggregates `ViewItem::ItemDescription`, `ViewItem::ItemConsumableStatus`, and `ViewItem:ItemContents`
/// and renders them into a full item description.
pub(super) fn item_detail(view: &mut View) {
    // display item description
    if let Some(ViewItem::ItemDescription { name, description }) = view.items.iter().find_map(|i| match i.view_item {
        ViewItem::ItemDescription { .. } => Some(&i.view_item),
        _ => None,
    }) {
        println!("{}", name.item_style().underline());
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
        println!();
    }

    // display consumable status, if item is consumable
    if let Some(ViewItem::ItemConsumableStatus(status_line)) = view.items.iter().find_map(|i| {
        if i.view_item.is_item_consumable_status() {
            Some(&i.view_item)
        } else {
            None
        }
    }) {
        println!(
            "{}",
            fill(
                format!("({} {})", "Consumable:".yellow(), status_line).as_str(),
                indented_block()
            )
            .italic()
            .dimmed()
        );
        println!();
    }

    // display list of contained items
    if let Some(ViewItem::ItemContents(content_lines)) = view.items.iter().find_map(|i| match i.view_item {
        ViewItem::ItemContents(_) => Some(&i.view_item),
        _ => None,
    }) {
        println!("{}:", "Contents".subheading_style());
        if content_lines.is_empty() {
            println!("   {}", "Empty".italic().dimmed());
        } else {
            for line in content_lines {
                println!(
                    "   {} {}",
                    line.item_name.item_style(),
                    if line.restricted { "[R]" } else { "" }
                );
            }
            println!();
        }
    }
}

/// Render the text / detail field for an `Item` to the display.
pub(super) fn item_text(view: &mut View) {
    if let Some(entry) = view.items.iter().find(|i| matches!(i.view_item, ViewItem::ItemText(_)))
        && let ViewItem::ItemText(text) = &entry.view_item
    {
        println!("{}:\n", "Upon closer inspection, you see".subheading_style());
        println!(
            "{}",
            render_wrapped(
                text,
                view.width,
                WrapMode::Indented,
                StyleKind::ItemText,
                StyleMods::default(),
            )
        );
        println!();
    }
}
