//! Styling helpers for terminal output.
//!
//! The [`GameStyle`] trait provides a set of convenience methods for applying
//! ANSI styling via the `colored` crate. Implementations for `&str` and
//! `String` are provided so string literals can be styled directly.

use colored::{ColoredString, Colorize};
use textwrap::{Options, wrap_algorithms::Penalties};

use crate::theme::current_theme_colors;

/// Returns `textwrap::Options` for an indented, wrapped block of text.
pub fn indented_block() -> Options<'static> {
    let indent = "    ";
    Options::with_termwidth()
        .initial_indent(indent)
        .subsequent_indent(indent)
        .wrap_algorithm(textwrap::WrapAlgorithm::OptimalFit(Penalties::new()))
}

/// Returns `textwrap::Options` for an unindented, wrapped block of text.
pub fn normal_block() -> Options<'static> {
    Options::with_termwidth().wrap_algorithm(textwrap::WrapAlgorithm::OptimalFit(Penalties::new()))
}

/// Convenience trait for applying color and style to text output.
pub trait GameStyle {
    fn prompt_style(&self) -> ColoredString;
    fn status_style(&self) -> ColoredString;
    fn highlight(&self) -> ColoredString;
    fn transition_style(&self) -> ColoredString;
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
    fn npc_movement_style(&self) -> ColoredString;
}

impl GameStyle for &str {
    fn prompt_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.prompt.to_color())
            .on_color(colors.prompt_bg.to_color())
    }

    fn status_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.to_uppercase().color(colors.status.to_color())
    }

    fn transition_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.transition.to_color())
    }

    fn item_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.item.to_color())
    }

    fn item_text_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.item_text.to_color())
    }

    fn npc_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.npc.to_color()).underline()
    }

    fn room_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.room.to_color())
    }

    fn room_titlebar_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.room_titlebar.to_color()).underline()
    }

    fn description_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.description.to_color())
    }

    fn triggered_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.triggered.to_color())
    }

    fn trig_icon_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.bold().color(colors.trig_icon.to_color())
    }

    fn ambient_icon_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.dimmed().color(colors.ambient_icon.to_color())
    }

    fn ambient_trig_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.ambient_trig.to_color()).dimmed()
    }

    fn exit_visited_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.exit_visited.to_color())
    }

    fn exit_locked_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.exit_locked.to_color())
    }

    fn exit_unvisited_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.exit_unvisited.to_color())
    }

    fn error_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.error.to_color())
    }

    fn error_icon_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.error_icon.to_color())
    }

    fn subheading_style(&self) -> ColoredString {
        self.bold()
    }

    fn goal_active_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.goal_active.to_color())
    }

    fn goal_complete_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.goal_complete.to_color()).strikethrough()
    }

    fn denied_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.denied.to_color())
    }

    fn overlay_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.overlay.to_color())
    }

    fn section_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        let bracketed = format!("[{self}]");
        bracketed.color(colors.section.to_color())
    }

    fn npc_quote_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.npc_quote.to_color())
    }

    fn highlight(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.color(colors.highlight.to_color())
    }

    fn npc_movement_style(&self) -> ColoredString {
        let colors = current_theme_colors();
        self.italic().color(colors.npc_movement.to_color())
    }
}

impl GameStyle for String {
    fn status_style(&self) -> ColoredString {
        self.as_str().status_style()
    }
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

    fn transition_style(&self) -> ColoredString {
        self.as_str().transition_style()
    }

    fn highlight(&self) -> ColoredString {
        self.as_str().highlight()
    }

    fn npc_movement_style(&self) -> ColoredString {
        self.as_str().npc_movement_style()
    }

    fn prompt_style(&self) -> ColoredString {
        self.as_str().prompt_style()
    }
}
