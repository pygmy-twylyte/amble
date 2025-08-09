//! View module.
//! This contains the view to the game world / messages.
//! Rather than printing to the console from each handler, we'll aggregate needed information and messages
//! to be organized and displayed at the end of the turn.

use colored::Colorize;
use textwrap::fill;
use variantly::Variantly;

use crate::style::GameStyle;

const ICON_SUCCESS: &str = "\u{2714}"; // ✔
const ICON_FAILURE: &str = "\u{2716}"; // ✖

/// View aggregates information to be displayed on each pass through the REPL and then organizes
/// and displays the result.
#[derive(Debug, Clone)]
pub struct View {
    width: usize,
    mode: ViewMode,
    items: Vec<ViewItem>,
}
impl View {
    /// Create a new empty view.
    /// Defaults to Verbose behavior.
    pub fn new() -> Self {
        Self {
            width: 80,
            mode: ViewMode::ClearVerbose,
            items: Vec::new(),
        }
    }

    /// Add something to be displayed in the next frame.
    pub fn push(&mut self, item: ViewItem) {
        self.items.push(item);
    }

    /// Compose and diplay all message contents in the current frame / turn.
    pub fn flush(&mut self) {
        // First Section: Environment / Frame of Reference
        self.environment();
        // Second Section: Immediate/ direct results of player command
        self.direct_results();
        // Third Section: Triggered World / NPC reaction to Command
        self.world_reaction();
        // Fourth Section: Messages not related to last command / action (ambients, etc.)
        self.ambience();

        // clear the buffer for the next turn
        self.items.clear();
    }

    fn environment(&mut self) {
        // Show overview of room/area
        self.room_description();
        self.room_overlays();
        self.room_item_list();
        self.room_exit_list();
        self.room_npc_list();
    }

    fn direct_results(&mut self) {
        // direct inspection (read, look_at) results
        self.item_detail();
        self.npc_detail();
        self.item_text();
        self.inventory();

        // successes / failures
        self.action_success();
        self.action_failure();
        self.errors();
    }

    fn world_reaction(&mut self) {}
    fn ambience(&mut self) {}

    fn inventory(&mut self) {
        if let Some(ViewItem::Inventory(item_lines)) = self.items.iter().find(|i| matches!(i, ViewItem::Inventory(..)))
        {
            println!("{}:", "Inventory".subheading_style());
            if item_lines.is_empty() {
                println!("   {}", "You have... nothing at all.".italic().dimmed());
            } else {
                item_lines
                    .iter()
                    .for_each(|line| println!("   {}", line.item_name.item_style()));
            }
        }
    }

    fn action_success(&mut self) {
        let messages: Vec<_> = self
            .items
            .iter()
            .filter_map(|i| match i {
                ViewItem::ActionSuccess(msg) => Some(msg),
                _ => None,
            })
            .collect();
        messages
            .iter()
            .for_each(|msg| println!("{} {}", ICON_SUCCESS.bright_green(), *msg));
    }

    fn action_failure(&mut self) {
        let messages: Vec<_> = self
            .items
            .iter()
            .filter_map(|i| match i {
                ViewItem::ActionFailure(msg) => Some(msg),
                _ => None,
            })
            .collect();
        messages
            .iter()
            .for_each(|msg| println!("{} {}", ICON_FAILURE.bright_red(), *msg));
    }

    fn errors(&mut self) {
        let messages: Vec<_> = self
            .items
            .iter()
            .filter_map(|i| match i {
                ViewItem::Error(msg) => Some(msg),
                _ => None,
            })
            .collect();
        messages.iter().for_each(|msg| println!("{}", (*msg).error_style()));
    }

    fn item_text(&mut self) {
        if let Some(ViewItem::ItemText(text)) = self.items.iter().find(|i| matches!(i, ViewItem::ItemText(_))) {
            println!("{}:", "You can read".subheading_style());
            println!("{}", text.italic());
            println!();
        }
    }

    fn npc_detail(&mut self) {
        if let Some(ViewItem::NpcDescription { name, description }) =
            self.items.iter().find(|i| matches!(i, ViewItem::NpcDescription { .. }))
        {
            println!("{}", name.npc_style().underline());
            println!("{}", fill(description, self.width).description_style());
            println!();
        }
        if let Some(ViewItem::NpcInventory(content_lines)) =
            self.items.iter().find(|i| matches!(i, ViewItem::NpcInventory(_)))
        {
            println!("{}:", "Inventory".subheading_style());
            if content_lines.is_empty() {
                println!("   {}", "(Empty)".dimmed().italic());
            } else {
                content_lines.iter().for_each(|line| {
                    println!(
                        "   {} {}",
                        line.item_name.item_style(),
                        if line.restricted { "[R]" } else { "" }
                    )
                });
            }
        }
    }

    fn item_detail(&mut self) {
        if let Some(ViewItem::ItemDescription { name, description }) = self
            .items
            .iter()
            .find(|i| matches!(i, ViewItem::ItemDescription { .. }))
        {
            println!("{}", name.item_style().underline());
            println!("{}", fill(description, self.width).description_style());
            println!();
        }

        if let Some(ViewItem::ItemContents(content_lines)) =
            self.items.iter().find(|i| matches!(i, ViewItem::ItemContents(_)))
        {
            println!("{}:", "Contents".subheading_style());
            if content_lines.is_empty() {
                println!("   {}", "Empty".italic().dimmed());
            } else {
                content_lines.iter().for_each(|line| {
                    println!(
                        "   {} {}",
                        line.item_name.item_style(),
                        if line.restricted { "[R]" } else { "" }
                    );
                });
                println!();
            }
        }
    }

    fn room_npc_list(&mut self) {
        if let Some(ViewItem::RoomNpcs(npcs)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomNpcs(_))) {
            println!("{}:", "Others".subheading_style());
            npcs.iter()
                .for_each(|npc| println!("   {} - {}", npc.name.npc_style(), npc.description.description_style()));
            println!();
        }
    }

    fn room_exit_list(&mut self) {
        if let Some(ViewItem::RoomExits(exit_lines)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomExits(_))) {
            println!("{}:", "Exits".subheading_style());
            for exit in exit_lines {
                print!("   > ");
                match (exit.dest_visited, exit.exit_locked) {
                    (true, false) => println!(
                        "{} (to {})",
                        exit.direction.exit_visited_style(),
                        exit.destination.room_style()
                    ),
                    (true, true) => println!(
                        "{} (to {})",
                        exit.direction.exit_locked_style(),
                        exit.destination.room_style()
                    ),
                    (false, true) => println!("{}", exit.direction.exit_locked_style()),
                    (false, false) => println!("{}", exit.direction.exit_unvisited_style()),
                }
            }
            println!();
        }
    }

    fn room_item_list(&mut self) {
        if let Some(ViewItem::RoomItems(names)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomItems(_))) {
            println!("{}:", "Items".subheading_style());
            names.iter().for_each(|name| println!("   {}", name.item_style()));
            println!();
        }
    }

    fn room_overlays(&mut self) {
        if let Some(ViewItem::RoomOverlays { text, force_mode }) =
            self.items.iter().find(|i| matches!(i, ViewItem::RoomOverlays { .. }))
        {
            let display_mode = force_mode.unwrap_or(self.mode);
            if display_mode != ViewMode::Brief {
                text.iter()
                    .for_each(|line| println!("{}\n", fill(line, self.width).overlay_style()));
            }
        }
    }

    /// Used by flush() to show base room description
    fn room_description(&mut self) {
        if let Some(ViewItem::RoomDescription {
            name,
            description,
            visited,
            force_mode,
        }) = self
            .items
            .iter()
            .find(|i| matches!(i, ViewItem::RoomDescription { .. }))
        {
            // Use the forced display mode if there is one, otherwise use current setting
            let display_mode = force_mode.unwrap_or(self.mode);
            if display_mode == ViewMode::ClearVerbose {
                // clear the screen
                print!("\x1B[2J\x1B[H");
            }
            println!("{:^80}", name.room_titlebar_style());
            if display_mode != ViewMode::Brief || !visited {
                println!("{}", fill(description, self.width).description_style());
                println!();
            }
        }
    }

    /// Clears the View's buffer but does not reset the mode.
    pub fn reset(&mut self) {
        self.items.clear();
    }

    /// Sets a ViewMode and returns the previously set mode.
    pub fn set_mode(&mut self, mode: ViewMode) -> ViewMode {
        std::mem::replace(&mut self.mode, mode)
    }
}

/// ViewMode alters the way that each "frame" is rendered.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ViewMode {
    ClearVerbose, // clears before every room description
    Verbose,      // always shows full room description
    Brief,        // only show full description on first entry
}

/// ViewItems are each of the various types of information / messages that may be displayed to the player.
#[derive(Debug, Clone, PartialEq, Eq, Variantly)]
pub enum ViewItem {
    RoomDescription {
        name: String,
        description: String,
        visited: bool,
        force_mode: Option<ViewMode>,
    },
    RoomOverlays {
        text: Vec<String>,
        force_mode: Option<ViewMode>,
    },
    RoomItems(Vec<String>),
    RoomExits(Vec<ExitLine>),
    RoomNpcs(Vec<NpcLine>),
    ItemDescription {
        name: String,
        description: String,
    },
    ItemText(String),
    ItemContents(Vec<ContentLine>),
    NpcDescription {
        name: String,
        description: String,
    },
    NpcInventory(Vec<ContentLine>),
    NpcSpeech {
        speaker: String,
        quote: String,
    },
    AmbientEvent(String),
    TriggeredEvent(String),
    ActionSuccess(String),
    ActionFailure(String),
    Error(String),
    Inventory(Vec<ContentLine>),
}
/// Information needed to display a contents list for an item correctly
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentLine {
    pub item_name: String,
    pub restricted: bool,
}

/// Information needed to display a line in the room exit list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitLine {
    pub direction: String,
    pub destination: String,
    pub exit_locked: bool,
    pub dest_visited: bool,
}

/// Information needed to display a line in the room NPC list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcLine {
    pub name: String,
    pub description: String,
}
