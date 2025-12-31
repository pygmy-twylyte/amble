//! module Render System
//!
//! This module contains the `ViewItem` renderers for system/engine messages,
//! such as display of saved games, help, summary upon quitting, or other items
//! related more to the system than to the game content.

use colored::Colorize as _;
use textwrap::{fill, termwidth};

use crate::{
    View, ViewItem,
    save_files::{SaveFileStatus, format_modified},
    style::{GameStyle as _, normal_block},
    view::icons::ICON_ENGINE,
};

/// Used for generic messages from the engine -- rare.
pub(super) fn engine_message(view: &mut View) {
    let engine_msgs = view.items.iter().filter(|i| i.view_item.is_engine_message());
    for msg in engine_msgs {
        println!(
            "{}",
            fill(
                format!("{ICON_ENGINE:<4}{}", msg.view_item.clone().unwrap_engine_message()).as_str(),
                normal_block()
            )
        );
    }
    println!();
}

/// Displays a list of available saved games.
pub(super) fn saved_games(view: &mut View) {
    let Some((directory, entries)) = view.items.iter().find_map(|entry| match &entry.view_item {
        ViewItem::SavedGamesList { directory, entries } => Some((directory, entries)),
        _ => None,
    }) else {
        return;
    };

    println!("{}", format!("Saved games in {directory}/").subheading_style());
    if entries.is_empty() {
        println!(
            "    {}",
            "No saved games found. Use `save <slot>` to create one.".italic()
        );
        println!();
        return;
    }

    for entry in entries {
        let slot_label = entry.slot.highlight();
        let version_label = format!("[v{}]", entry.version).dimmed();
        let header = if let Some(modified) = entry.modified {
            format!(
                "  • {} {} — saved {}",
                slot_label,
                version_label,
                format_modified(modified).dimmed()
            )
        } else {
            format!("  • {slot_label} {version_label}")
        };
        println!("{header}");

        if let Some(summary) = &entry.summary {
            let location = summary.player_location.as_deref().unwrap_or("Unknown location");
            println!(
                "    Player: {} | Turn {} | Score {} | Location: {}",
                summary.player_name.as_str().highlight(),
                summary.turn_count,
                summary.score,
                location
            );
        } else {
            println!("    {}", "Metadata unavailable for this save.".denied_style());
        }

        println!(
            "    {}",
            format!("load {}    [{directory}/{}]", entry.slot, entry.file_name).dimmed()
        );

        match &entry.status {
            SaveFileStatus::Ready => {},
            SaveFileStatus::VersionMismatch {
                save_version,
                current_version,
            } => println!(
                "    {} {}",
                "Warning:".bold().yellow(),
                format!("saved with v{save_version}, current engine v{current_version}.").yellow()
            ),
            SaveFileStatus::Corrupted { message } => println!("    {} {}", "Error:".bold().red(), message.red()),
        }
        println!();
    }
}

/// Displays confirmation message when a game is loaded or saved.
pub(super) fn load_or_save(view: &mut View) {
    if let Some(entry) = view
        .items
        .iter()
        .find(|i| matches!(i.view_item, ViewItem::GameSaved { .. }))
        && let ViewItem::GameSaved { save_slot, save_file } = &entry.view_item
    {
        println!("{}: \"{}\" ({})", "Game Saved".green().bold(), save_slot, save_file);
        println!("{}", format!("Type \"load {save_slot}\" to reload it.").italic());
        println!();
    }
    if let Some(entry) = view
        .items
        .iter()
        .find(|i| matches!(i.view_item, ViewItem::GameLoaded { .. }))
        && let ViewItem::GameLoaded { save_slot, save_file } = &entry.view_item
    {
        println!("{}: \"{}\" ({})", "Game Loaded".green().bold(), save_slot, save_file);
        println!();
    }
}

/// Displays the general help message and command guide.
pub(super) fn show_help(view: &mut View) {
    if let Some(entry) = view
        .items
        .iter()
        .find(|item| matches!(&item.view_item, ViewItem::Help { .. }))
        && let ViewItem::Help { basic_text, commands } = &entry.view_item
    {
        // Print the basic help text with proper text wrapping
        println!("{}", fill(basic_text, normal_block()).italic().cyan());
        println!();

        // Partition commands into normal vs DEV (':'-prefixed)
        let (dev_cmds, normal_cmds): (Vec<_>, Vec<_>) =
            commands.iter().cloned().partition(|c| c.command.starts_with(':'));

        // Print normal commands section
        println!("{}", "Some Common Commands:".bold().yellow());
        println!();
        for command in &normal_cmds {
            let formatted_line = format!("{} - {}", command.command.bold().green(), command.description.italic());
            println!("{}", fill(&formatted_line, normal_block()));
        }

        // Print developer commands section if present and DEV_MODE
        if crate::DEV_MODE && !dev_cmds.is_empty() {
            println!();
            println!("{}", "Developer Commands (DEV_MODE):".bold().yellow());
            println!();
            for command in &dev_cmds {
                let desc = command
                    .description
                    .strip_prefix("DEV: ")
                    .unwrap_or(&command.description)
                    .to_string();
                let formatted_line = format!("{} - {}", command.command.bold().green(), desc.italic());
                println!("{}", fill(&formatted_line, normal_block()));
            }
        }
    }
}

/// Displays the game summary when the player quits.
#[allow(clippy::cast_precision_loss)]
pub(super) fn quit_summary(view: &mut View) {
    if let Some(entry) = view
        .items
        .iter()
        .find(|entry| matches!(entry.view_item, ViewItem::QuitSummary { .. }))
        && let ViewItem::QuitSummary {
            title,
            rank,
            notes,
            score,
            max_score,
            visited,
            max_visited,
        } = &entry.view_item
    {
        let score_pct = 100.0 * (*score as f32 / *max_score as f32);
        let visit_pct = 100.0 * (*visited as f32 / *max_visited as f32);
        println!("{:^width$}", title.as_str().black().on_yellow(), width = termwidth());
        println!("{:10} {}", "Rank:", rank.bright_cyan());
        println!("{:10} {}", "Notes:", notes.description_style());
        println!("{:10} {}/{} ({:.1}%)", "Score:", score, max_score, score_pct);
        println!("{:10} {}/{} ({:.1}%)", "Visited:", visited, max_visited, visit_pct);
    }
}
