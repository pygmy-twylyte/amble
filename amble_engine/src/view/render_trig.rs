//! module Render Trig(gered Events)
//!
//! This module contains the `ViewItem` renderers for generic triggered events
//! that are not (necessarily) in response to a player command, such as ambient
//! events, point awards and messages from triggers (`do show` commands).

use colored::Colorize as _;
use textwrap::{fill, termwidth};

use crate::{
    View, ViewItem,
    helpers::plural_s,
    markup::{StyleKind, StyleMods, WrapMode, render_wrapped},
    style::{GameStyle as _, normal_block},
    view::ViewEntry,
    view::icons::{ICON_AMBIENT, ICON_CELEBRATE, ICON_NEGATIVE, ICON_POSITIVE, ICON_TRIGGER},
};

pub(super) fn points_awarded(entries: &[&ViewEntry]) {
    let point_msgs = entries.iter().copied().filter(|i| i.view_item.is_points_awarded());
    for msg in point_msgs {
        if let ViewItem::PointsAwarded { amount, reason } = &msg.view_item {
            if amount.is_negative() {
                let text = format!("{} (-{} point{})\n", reason, amount.abs(), plural_s(amount.abs())).bright_red();
                println!("{:<4}{}", ICON_NEGATIVE.bright_red(), text);
            } else if *amount > 15 {
                let text = format!("{} (+{} point{}!)\n", reason, amount, plural_s(*amount)).bright_blue();
                println!("{:<4}{}", ICON_CELEBRATE.bright_blue(), text);
            } else {
                let text = format!("{} (+{} point{})\n", reason, amount, plural_s(*amount)).bright_green();
                println!("{:<4}{}", ICON_POSITIVE.bright_green(), text);
            }
        }
    }
}

pub(super) fn ambient_event(view: &mut View) {
    let trig_messages = view
        .items
        .iter()
        .filter(|i| matches!(i.view_item, ViewItem::AmbientEvent(_)));
    for msg in trig_messages {
        let formatted = format!(
            "{:<4}{}",
            ICON_AMBIENT.ambient_icon_style(),
            msg.view_item.clone().unwrap_ambient_event().ambient_trig_style()
        );
        println!("{}", fill(formatted.as_str(), normal_block()));
        println!();
    }
}

pub(super) fn triggered_event(entries: &[&ViewEntry]) {
    let trig_messages = entries.iter().filter_map(|entry| match &entry.view_item {
        ViewItem::TriggeredEvent(text) => Some(text),
        _ => None,
    });
    for text in trig_messages {
        let rendered = render_wrapped(
            text,
            termwidth(),
            WrapMode::Normal,
            StyleKind::Triggered,
            StyleMods {
                italic: true,
                ..StyleMods::default()
            },
        );
        let prefix = format!("{:<4}", ICON_TRIGGER.trig_icon_style());
        let mut lines = rendered.lines();
        let mut formatted = String::new();
        if let Some(first) = lines.next() {
            formatted.push_str(&prefix);
            formatted.push_str(first);
            for line in lines {
                formatted.push('\n');
                formatted.push_str(line);
            }
        } else {
            formatted.push_str(&prefix);
        }
        println!("{formatted}");
        println!();
    }
}
