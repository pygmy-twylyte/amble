use anyhow::{Context, Result, bail};
use gametools::{Spinner, Wedge};
use log::info;
use std::collections::HashMap;
use std::hash::BuildHasher;

use crate::spinners::SpinnerType;
use crate::style::GameStyle;
use crate::view::{View, ViewItem};
use crate::world::AmbleWorld;

/// Adds a weighted text option ("wedge") to a random text spinner.
///
/// # Errors
/// Returns an error if the specified spinner type doesn't exist.
pub fn add_spinner_wedge<S: BuildHasher>(
    spinners: &mut HashMap<SpinnerType, Spinner<String>, S>,
    spin_type: &SpinnerType,
    text: &str,
    width: usize,
) -> Result<()> {
    let wedge = Wedge::new_weighted(text.to_string(), width);
    let spinref = spinners
        .get_mut(spin_type)
        .with_context(|| format!("add_spinner_wedge(_, {spin_type:?}, _, _): spinner not found"))?;
    *spinref = spinref.add_wedge(wedge);
    info!("└─ action: AddSpinnerWedge({spin_type:?}, \"{text}\"");
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
        let msg = spinner.spin().unwrap_or_default();
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
