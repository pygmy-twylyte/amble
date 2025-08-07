//! View module.
//! This contains the view to the game world / messages.
//! Rather than printing to the console from each handler, we'll aggregate needed information and messages
//! to be organized and displayed at the end of the turn.

use textwrap::fill;
use variantly::Variantly;

use crate::style::GameStyle;

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
            mode: ViewMode::Brief,
            items: Vec::new(),
        }
    }

    /// Add something to be displayed in the next frame.
    pub fn push(&mut self, item: ViewItem) {
        self.items.push(item);
    }

    /// Compose and diplay all message contents in the current frame / turn.
    pub fn flush(&mut self) {
        // Section 1A: Room Description
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

        // Section 1B: Room Overlays
        if let Some(ViewItem::RoomOverlays { text, force_mode }) =
            self.items.iter().find(|i| matches!(i, ViewItem::RoomOverlays { .. }))
        {
            let display_mode = force_mode.unwrap_or(self.mode);
            if display_mode != ViewMode::Brief {
                text.iter()
                    .for_each(|line| println!("{}\n", fill(line, self.width).overlay_style()));
            }
        }

        // Section 1C: Room Items
        if let Some(ViewItem::RoomItems(names)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomItems(_))) {
            println!("{}", "Items:".subheading_style());
            names.iter().for_each(|name| println!("   {}", name.item_style()));
            println!();
        }

        // Section 1D: Exits
        if let Some(ViewItem::RoomExits(exit_lines)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomExits(_))) {
            println!("{}", "Exits:".subheading_style());
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

        // Section 1E: Room NPC list
        if let Some(ViewItem::RoomNpcs(npcs)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomNpcs(_))) {
            println!("{}", "Others:".subheading_style());
            npcs.iter()
                .for_each(|npc| println!("   {} - {}", npc.name.npc_style(), npc.description.description_style()));
            println!();
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
        descripton: String,
    },
    ItemText(String),
    ItemContents(Vec<String>),
    NpcDescription {
        name: String,
        description: String,
    },
    NpcInventory(Vec<String>),
    NpcSpeech {
        speaker: String,
        quote: String,
    },
    AmbientEvent(String),
    TriggeredEvent(String),
    ActionResult(String),
    ActionDenied(String),
    Error(String),
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
