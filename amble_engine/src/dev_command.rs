//! `dev_command.rs`
//!
//! Implements commands only to be used in developer mode.

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
/// the `DEV_MODE` constant is set to true. These commands provide debugging
/// and testing functionality not intended for normal gameplay.
///
/// # Arguments
/// * `input` - The raw command input from the user
/// * `view` - Mutable reference to the view for displaying error messages
///
/// # Returns
/// `Some(Command)` if a valid developer command is parsed, `None` otherwise
pub fn parse_dev_command(input: &str, view: &mut View) -> Option<Command> {
    if input.starts_with(':') {
        let words: Vec<&str> = input.trim_start_matches(':').split_whitespace().collect();
        let maybe_command = match words.as_slice() {
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
        if maybe_command.is_some() && !DEV_MODE {
            view.push(ViewItem::Error(
                "Developer commands are disabled in this build."
                    .error_style()
                    .to_string(),
            ));
            warn!(
                "player attempted to use developer command '{:?}' with DEV_MODE = false",
                maybe_command.unwrap()
            );
            return None;
        }
        maybe_command
    } else {
        None
    }
}
