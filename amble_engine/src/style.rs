//! Styling helpers for terminal output.
//!
//! The [`GameStyle`] trait provides a set of convenience methods for applying
//! ANSI styling via the `colored` crate. Implementations for `&str` and
//! `String` are provided so string literals can be styled directly.

use colored::{ColoredString, Colorize};
use textwrap::Options;

/// Returns textwrap::ptions for an indented, wrapped block of text.
pub fn indented_block(display_width: usize) -> Options<'static> {
    let indent = "    ";
    Options::new(display_width)
        .initial_indent(indent)
        .subsequent_indent(indent)
}

/// Returns textwrap::Options for an unindented, wrapped block of text.
pub fn normal_block(display_width: usize) -> Options<'static> {
    Options::new(display_width)
}

/// Convenience trait for applying color and style to text output.
pub trait GameStyle {
    fn item_style(&self) -> ColoredString;
    fn item_text_style(&self) -> ColoredString;
    fn npc_style(&self) -> ColoredString;
    fn npc_quote_style(&self) -> ColoredString;
    fn room_style(&self) -> ColoredString;
    fn room_titlebar_style(&self) -> ColoredString;
    fn description_style(&self) -> ColoredString;
    fn triggered_style(&self) -> ColoredString;
    fn trig_icon_style(&self) -> ColoredString;
    fn ambient_icon_style(&self) -> ColoredString;
    fn ambient_trig_style(&self) -> ColoredString;
    fn exit_visited_style(&self) -> ColoredString;
    fn exit_locked_style(&self) -> ColoredString;
    fn exit_unvisited_style(&self) -> ColoredString;
    fn error_style(&self) -> ColoredString;
    fn error_icon_style(&self) -> ColoredString;
    fn subheading_style(&self) -> ColoredString;
    fn goal_active_style(&self) -> ColoredString;
    fn goal_complete_style(&self) -> ColoredString;
    fn denied_style(&self) -> ColoredString;
    fn overlay_style(&self) -> ColoredString;
    fn section_style(&self) -> ColoredString;
}

impl GameStyle for &str {
    fn item_style(&self) -> ColoredString {
        self.truecolor(220, 180, 40)
    }
    fn item_text_style(&self) -> ColoredString {
        self.truecolor(40, 180, 40)
    }
    fn npc_style(&self) -> ColoredString {
        self.truecolor(13, 130, 60).underline()
    }
    fn room_style(&self) -> ColoredString {
        self.truecolor(223, 77, 10)
    }
    fn room_titlebar_style(&self) -> ColoredString {
        self.truecolor(223, 77, 10).underline()
    }
    fn description_style(&self) -> ColoredString {
        self.italic().truecolor(102, 208, 250)
    }
    fn triggered_style(&self) -> ColoredString {
        self.italic().truecolor(230, 230, 30)
    }
    fn trig_icon_style(&self) -> ColoredString {
        self.bold().truecolor(230, 80, 80)
    }
    fn ambient_icon_style(&self) -> ColoredString {
        self.dimmed().truecolor(80, 80, 230)
    }
    fn ambient_trig_style(&self) -> ColoredString {
        self.truecolor(150, 230, 30).dimmed()
    }
    fn exit_visited_style(&self) -> ColoredString {
        self.italic().truecolor(110, 220, 110)
    }
    fn exit_locked_style(&self) -> ColoredString {
        self.italic().truecolor(200, 50, 50)
    }
    fn exit_unvisited_style(&self) -> ColoredString {
        self.italic().truecolor(220, 180, 40)
    }
    fn error_style(&self) -> ColoredString {
        self.truecolor(230, 30, 30)
    }
    fn error_icon_style(&self) -> ColoredString {
        self.bright_red()
    }
    fn subheading_style(&self) -> ColoredString {
        self.bold()
    }
    fn goal_active_style(&self) -> ColoredString {
        self.truecolor(220, 40, 220)
    }
    fn goal_complete_style(&self) -> ColoredString {
        self.truecolor(220, 40, 220).strikethrough()
    }
    fn denied_style(&self) -> ColoredString {
        self.italic().truecolor(230, 30, 30)
    }
    fn overlay_style(&self) -> ColoredString {
        self.italic().truecolor(75, 180, 255)
    }

    fn section_style(&self) -> ColoredString {
        let bracketed = format!("[{}]", self);
        bracketed.truecolor(75, 80, 75)
    }

    fn npc_quote_style(&self) -> ColoredString {
        self.italic().truecolor(40, 180, 40)
    }
}

impl GameStyle for String {
    fn item_style(&self) -> ColoredString {
        self.as_str().item_style()
    }
    fn npc_style(&self) -> ColoredString {
        self.as_str().npc_style()
    }
    fn room_style(&self) -> ColoredString {
        self.as_str().room_style()
    }
    fn room_titlebar_style(&self) -> ColoredString {
        self.as_str().room_titlebar_style()
    }
    fn description_style(&self) -> ColoredString {
        self.as_str().description_style()
    }
    fn triggered_style(&self) -> ColoredString {
        self.as_str().triggered_style()
    }
    fn trig_icon_style(&self) -> ColoredString {
        self.as_str().trig_icon_style()
    }
    fn ambient_icon_style(&self) -> ColoredString {
        self.as_str().ambient_icon_style()
    }
    fn ambient_trig_style(&self) -> ColoredString {
        self.as_str().ambient_trig_style()
    }
    fn exit_visited_style(&self) -> ColoredString {
        self.as_str().exit_visited_style()
    }
    fn exit_locked_style(&self) -> ColoredString {
        self.as_str().exit_locked_style()
    }
    fn exit_unvisited_style(&self) -> ColoredString {
        self.as_str().exit_unvisited_style()
    }
    fn error_style(&self) -> ColoredString {
        self.as_str().error_style()
    }
    fn error_icon_style(&self) -> ColoredString {
        self.as_str().error_icon_style()
    }
    fn subheading_style(&self) -> ColoredString {
        self.as_str().subheading_style()
    }
    fn goal_active_style(&self) -> ColoredString {
        self.as_str().goal_active_style()
    }
    fn goal_complete_style(&self) -> ColoredString {
        self.as_str().goal_complete_style()
    }
    fn denied_style(&self) -> ColoredString {
        self.as_str().denied_style()
    }
    fn overlay_style(&self) -> ColoredString {
        self.as_str().overlay_style()
    }

    fn section_style(&self) -> ColoredString {
        self.as_str().section_style()
    }

    fn item_text_style(&self) -> ColoredString {
        self.as_str().item_text_style()
    }

    fn npc_quote_style(&self) -> ColoredString {
        self.as_str().npc_quote_style()
    }
}
