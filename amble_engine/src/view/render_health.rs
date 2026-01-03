//! module Render Health
//!
//! This module contains the individual `ViewItem` renderers for health related
//! `ViewItem`s -- damage / healing / death.

use colored::Colorize as _;
use textwrap::fill;

use crate::{
    ViewItem,
    style::{GameStyle as _, normal_block},
    view::{StatusAction, ViewEntry},
    view::icons::{ICON_DEATH, ICON_HARMED, ICON_HEALED, ICON_STATUS, },
};

/// Renders messages indicating status effects being applied to or removed
/// from the player.
pub(super) fn status_change(entries: &[&ViewEntry]) {
    let status_msgs: Vec<_> = entries
        .iter()
        .cloned()
        .filter(|entry| entry.view_item.is_status_change())
        .collect();
    for msg in &status_msgs {
        if let ViewItem::StatusChange { action, status } = &msg.view_item {
            println!(
                "{:<4}Status {}: {}",
                ICON_STATUS.yellow(),
                status.status_style(),
                match action {
                    StatusAction::Apply => "applied",
                    StatusAction::Remove => "removed",
                }
            );
        }
    }
    if !status_msgs.is_empty() {
        println!();
    }
}

/// Renders messages displayed when a character is harmed.
pub(super) fn character_harmed(entries: &[&ViewEntry]) {
    let messages: Vec<_> = entries
        .iter()
        .filter_map(|i| match &i.view_item {
            ViewItem::CharacterHarmed { name, cause, amount } => Some((name, cause, amount)),
            _ => None,
        })
        .collect();
    for (name, cause, amount) in messages {
        println!(
            "{}",
            fill(
                format!(
                    "{:<4}{} injured by {}! (-{} hp)",
                    ICON_HARMED.bright_yellow(),
                    name.npc_style(),
                    cause.underline(),
                    amount.to_string().bright_red()
                )
                .as_str(),
                normal_block()
            )
        );
        println!();
    }
}

/// Renders messages announcing the death of a character.
pub(super) fn character_death(entries: &[&ViewEntry]) {
    let messages: Vec<_> = entries
        .iter()
        .filter_map(|i| match &i.view_item {
            ViewItem::CharacterDeath { name, cause, is_player } => Some((name, cause, is_player)),
            _ => None,
        })
        .collect();
    for (name, cause, is_player) in messages {
        let base = format!("{:<4}{}", ICON_DEATH.red(), name.npc_style());
        let cause_text = cause
            .as_ref()
            .filter(|c| !c.is_empty())
            .map(|c| format!(" ({c})"))
            .unwrap_or_default();
        let suffix = if *is_player {
            " has fallen.".to_string()
        } else {
            " dies.".to_string()
        };
        println!(
            "{}",
            fill(format!("{base}{cause_text}{suffix}").as_str(), normal_block())
        );
        println!();
    }
}

/// Renders the message when a character is healed.
pub(super) fn character_healed(entries: &[&ViewEntry]) {
    let messages: Vec<_> = entries
        .iter()
        .filter_map(|i| match &i.view_item {
            ViewItem::CharacterHealed { name, cause, amount } => Some((name, cause, amount)),
            _ => None,
        })
        .collect();
    for (name, cause, amount) in messages {
        println!(
            "{}",
            fill(
                format!(
                    "{:<4}{} healed by {}! (+{} hp)",
                    ICON_HEALED.bright_blue(),
                    name.npc_style(),
                    cause.underline(),
                    amount.to_string().bright_green()
                )
                .as_str(),
                normal_block()
            )
        );
        println!();
    }
}
