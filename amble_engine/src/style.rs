use colored::{ColoredString, Colorize};

pub trait GameStyle {
    fn item_style(&self) -> ColoredString;
    fn npc_style(&self) -> ColoredString;
    fn room_style(&self) -> ColoredString;
    fn room_titlebar_style(&self) -> ColoredString;
    fn description_style(&self) -> ColoredString;
    fn triggered_style(&self) -> ColoredString;
    fn trig_icon_style(&self) -> ColoredString;
    fn exit_visited_style(&self) -> ColoredString;
    fn exit_locked_style(&self) -> ColoredString;
    fn exit_unvisited_style(&self) -> ColoredString;
    fn error_style(&self) -> ColoredString;
}

impl GameStyle for &str {
    fn item_style(&self) -> ColoredString {
        self.truecolor(220, 180, 40)
    }
    fn npc_style(&self) -> ColoredString {
        self.truecolor(13, 130, 60).underline()
    }
    fn room_style(&self) -> ColoredString {
        self.truecolor(223, 77, 10).bold()
    }
    fn room_titlebar_style(&self) -> ColoredString {
        self.truecolor(223, 77, 10).on_black()
    }
    fn description_style(&self) -> ColoredString {
        self.italic().truecolor(102, 208, 250)
    }
    fn triggered_style(&self) -> ColoredString {
        self.italic().truecolor(230, 230, 30)
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
    fn trig_icon_style(&self) -> ColoredString {
        self.bold().truecolor(230, 80, 80)
    }
    fn error_style(&self) -> ColoredString {
        self.underline().truecolor(230, 30, 30)
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
    fn exit_visited_style(&self) -> ColoredString {
        self.as_str().exit_visited_style()
    }
    fn exit_locked_style(&self) -> ColoredString {
        self.as_str().exit_locked_style()
    }
    fn exit_unvisited_style(&self) -> ColoredString {
        self.as_str().exit_unvisited_style()
    }

    fn trig_icon_style(&self) -> ColoredString {
        self.as_str().trig_icon_style()
    }
    fn error_style(&self) -> ColoredString {
        self.as_str().error_style()
    }
}
