use anyhow::{Context, Result, bail};
use log::info;
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::spinners::{SpinnerType, TextPool};
use crate::style::GameStyle;
use crate::view::{View, ViewItem};
use crate::world::AmbleWorld;

/// Adds a text entry to a random text spinner.
///
/// # Errors
/// Returns an error if the specified spinner type doesn't exist.
pub fn add_spinner_entry<S: BuildHasher>(
    spinners: &mut HashMap<SpinnerType, TextPool, S>,
    spin_type: &SpinnerType,
    text: &str,
) -> Result<()> {
    let pool = spinners
        .get_mut(spin_type)
        .with_context(|| format!("add_spinner_entry(_, {spin_type:?}, _): spinner not found"))?;
    pool.add(text.to_string());
    info!("└─ action: AddSpinnerEntry({spin_type:?}, \"{text}\"");
    Ok(())
}

/// Displays a random message from a world spinner.
///
/// # Errors
/// Returns an error if the requested spinner type doesn't exist in the world.
pub fn spinner_message(
    world: &mut AmbleWorld,
    view: &mut View,
    spinner_type: &SpinnerType,
    priority: Option<isize>,
) -> Result<()> {
    if let Some(spinner) = world.spinners.get(spinner_type) {
        let msg = spinner.draw();
        if !msg.is_empty() {
            view.push_with_custom_priority(
                ViewItem::AmbientEvent(format!("{}", msg.ambient_trig_style())),
                priority,
            );
        }
        info!("└─ action: SpinnerMessage(\"{msg}\")");
        Ok(())
    } else {
        bail!("action SpinnerMessage({spinner_type:?}): no spinner found for type");
    }
}

/// Displays a message to the player as a triggered event.
pub fn show_message(view: &mut View, text: &str) {
    show_message_with_priority(view, text, None);
}

pub(super) fn show_message_with_priority(view: &mut View, text: &str, priority: Option<isize>) {
    view.push_with_custom_priority(ViewItem::TriggeredEvent(text.to_owned()), priority);
    info!(
        "└─ action: ShowMessage(\"{}...\")",
        &text[..std::cmp::min(text.len(), 50)]
    );
}

/// Prevents a player from reading an item and displays a custom denial message.
pub fn deny_read(view: &mut View, reason: &String) {
    view.push(ViewItem::ActionFailure(format!("{}", reason.denied_style())));
    info!("└─ action: DenyRead(\"{reason}\")");
}
