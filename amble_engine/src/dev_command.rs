//! Developer-mode command parsing.
//!
//! Recognizes colon-prefixed commands that expose debugging / play-testing tools when
//! `DEV_MODE` is enabled.

use log::warn;

use crate::{
    DEV_MODE,
    command::Command,
    style::GameStyle,
    view::{View, ViewItem},
};

/// Parse developer-only commands if '`DEV_MODE`' is true.
///
/// Developer commands are prefixed with ':' and are only available when
/// the engine is built with the "dev-mode" feature enabled. These commands
/// provide debugging and testing functionality not intended for normal gameplay.
///
/// # Arguments
/// * `input` - The raw command input from the user
/// * `view` - Mutable reference to the view for displaying error messages
///
/// # Returns
/// `Some(Command)` if a valid developer command is parsed, `None` otherwise
pub fn parse_dev_command(input: &str, view: &mut View) -> Option<Command> {
    if !input.starts_with(':') {
        return None;
    }
    if !DEV_MODE {
        view.push(ViewItem::Error(
            "Developer commands are disabled in this build."
                .error_style()
                .to_string(),
        ));
        warn!("possible attempt to use developer command in non-dev-mode build ({input})");
        return None;
    }

    let trimmed = input.trim();
    // ":note" handled separately before splitting into words because we must
    // consume and retain all chars (including whitespace) after ":note"
    // for the note text.
    if let Some(note_text) = parse_note_command(trimmed) {
        return Some(Command::DevNote(note_text));
    }

    let words: Vec<&str> = trimmed.trim_start_matches(':').split_whitespace().collect();
    let maybe_command = match words.as_slice() {
        ["help", "dev"] => Some(Command::HelpDev),
        ["npcs"] => Some(Command::ListNpcs),
        ["flags"] => Some(Command::ListFlags),
        ["sched" | "schedule"] => Some(Command::ListSched),
        ["sched" | "schedule", "cancel", idx_str] => {
            if let Ok(idx) = idx_str.parse::<usize>() {
                Some(Command::SchedCancel(idx))
            } else {
                view.push(ViewItem::ActionFailure(format!(
                    "Invalid index '{}' for :schedule cancel.",
                    idx_str.error_style()
                )));
                None
            }
        },
        ["sched" | "schedule", "delay", idx_str, turns_str] => {
            if let (Ok(idx), Ok(turns)) = (idx_str.parse::<usize>(), turns_str.parse::<usize>()) {
                Some(Command::SchedDelay { idx, turns })
            } else {
                view.push(ViewItem::ActionFailure(
                    "Usage: :schedule delay <idx> <+turns>".to_string(),
                ));
                None
            }
        },
        ["teleport" | "port", room_symbol] => Some(Command::Teleport((*room_symbol).into())),
        ["spawn" | "item", item_symbol] => Some(Command::SpawnItem((*item_symbol).into())),
        ["adv-seq", seq_name] => Some(Command::AdvanceSeq((*seq_name).into())),
        ["init-seq", seq_name, end_opt] => Some(Command::StartSeq {
            seq_name: (*seq_name).into(),
            end: (*end_opt).into(),
        }),
        ["reset-seq", seq_name] => Some(Command::ResetSeq((*seq_name).into())),
        ["set-flag", flag_name] => Some(Command::SetFlag((*flag_name).into())),

        _ => None,
    };
    maybe_command
}

fn parse_note_command(input: &str) -> Option<String> {
    let rest = input.strip_prefix(":note")?;
    if let Some(next) = rest.chars().next()
        && !next.is_whitespace()
    {
        return None;
    }
    Some(rest.trim().to_string())
}
