//! module Render Action
//!
//! This module contains the individual `ViewItem` renderers direct responses to action
//! commands -- successes, failures, and errors.

use colored::Colorize as _;
use textwrap::fill;

use crate::{
    View, ViewItem,
    style::{GameStyle as _, normal_block},
    view::icons::{ICON_ERROR, ICON_FAILURE, ICON_SUCCESS},
};

pub(super) fn action_success(view: &mut View) {
    let messages: Vec<_> = view
        .items
        .iter()
        .filter_map(|i| match &i.view_item {
            ViewItem::ActionSuccess(msg) => Some(msg),
            _ => None,
        })
        .collect();
    for msg in messages {
        println!(
            "{}",
            fill(
                format!("{} {}", ICON_SUCCESS.bright_green(), msg).as_str(),
                normal_block()
            )
        );
    }
}

pub(super) fn action_failure(view: &mut View) {
    let messages: Vec<_> = view
        .items
        .iter()
        .filter_map(|i| match &i.view_item {
            ViewItem::ActionFailure(msg) => Some(msg),
            _ => None,
        })
        .collect();
    for msg in messages {
        println!(
            "{}",
            fill(
                format!("{} {}", ICON_FAILURE.bright_red(), msg).as_str(),
                normal_block()
            )
        );
    }
}

pub(super) fn errors(view: &mut View) {
    let messages: Vec<_> = view
        .items
        .iter()
        .filter_map(|i| match &i.view_item {
            ViewItem::Error(msg) => Some(msg),
            _ => None,
        })
        .collect();
    for msg in messages {
        println!(
            "{}",
            fill(
                format!("{:<4}{}", ICON_ERROR.error_icon_style(), msg).as_str(),
                normal_block()
            )
        );
    }
}
