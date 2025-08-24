//! View module.
//! This contains the view to the game world / messages.
//! Rather than printing to the console from each handler, we'll aggregate needed information and messages
//! to be organized and displayed at the end of the turn.

use colored::Colorize;
use log::info;
use textwrap::{fill, termwidth};
use variantly::Variantly;

use crate::loader::help::HelpCommand;
use crate::style::{GameStyle, indented_block, normal_block};

const ICON_SUCCESS: &str = "\u{2611}"; // ✔
const ICON_FAILURE: &str = "\u{274C}"; // ✖
const ICON_ERROR: &str = "⚠︎"; // U+26A0 U+FE0E
const ICON_TRIGGER: &str = "⚡︎"; // U+26A1 U+FE0E
const ICON_AMBIENT: &str = "⌘";
const ICON_NEGATIVE: &str = "➖";
const ICON_POSITIVE: &str = "➕";
const ICON_ENGINE: &str = "⚙";
const ICON_STATUS: &str = "⚕";
const ICON_NPC_ENTER: &str = "→"; // U+2192
const ICON_NPC_LEAVE: &str = "←"; // U+2190

/// View aggregates information to be displayed on each pass through the REPL and then organizes
/// and displays the result.
#[derive(Debug, Clone)]
pub struct View {
    pub width: usize,
    pub mode: ViewMode,
    pub items: Vec<ViewItem>,
}
impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl View {
    /// Create a new empty view.
    /// Defaults to Verbose behavior.
    pub fn new() -> Self {
        Self {
            width: termwidth(),
            mode: ViewMode::Verbose,
            items: Vec::new(),
        }
    }

    /// Add something to be displayed in the next frame.
    pub fn push(&mut self, item: ViewItem) {
        self.items.push(item);
    }

    /// Compose and diplay all message contents in the current frame / turn.
    pub fn flush(&mut self) {
        // re-check terminal width in case it's been resized
        self.width = termwidth();

        // Optimization: We could bin sections here and then iterate over them separately,
        // but considering we're typically dealing with a maximum of about 12 items to display,
        // there would be no tangible benefit.

        // Section Zero: Movement transition message, if any
        if let Some(ViewItem::TransitionMessage(msg)) = self.items.iter().find(|i| i.is_transition_message()) {
            println!("\n{}", fill(msg, normal_block()).transition_style());
        }

        // First Section: Environment / Frame of Reference
        if self.items.iter().any(|item| item.section() == Section::Environment) {
            println!("{:.>width$}\n", "scene".section_style(), width = self.width);
            self.environment();
        }
        // Second Section: Immediate/ direct results of player command
        if self.items.iter().any(|item| item.section() == Section::DirectResult) {
            println!("{:.>width$}\n", "results".section_style(), width = self.width);
            self.direct_results();
        }
        // Third Section: Triggered World / NPC reaction to Command
        if self.items.iter().any(|item| item.section() == Section::WorldResponse) {
            println!("{:.>width$}\n", "responses".section_style(), width = self.width);
            self.world_reaction();
        }
        // Fourth Section: Messages not related to last command / action (ambients, goals, etc.)
        if self.items.iter().any(|item| item.section() == Section::Ambient) {
            println!("{:.>width$}\n", "situation".section_style(), width = self.width);
            self.ambience();
        }
        // Fifth Section: System Commands (load/save, help, quit etc)
        if self.items.iter().any(|item| item.section() == Section::System) {
            println!("{:.>width$}\n", "game".section_style(), width = self.width);
            self.system();
        }

        // clear the buffer for the next turn
        self.items.clear();
    }

    // SECTION AGGREGATORS START HERE --------------------

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
        self.item_text();
        self.npc_detail();
        self.inventory();
        self.goals();

        // successes / failures
        self.action_success();
        self.action_failure();
        self.errors();
    }

    fn world_reaction(&mut self) {
        self.npc_events_sorted();
        self.triggered_event();
        self.status_change();
        self.points_awarded();
    }

    fn ambience(&mut self) {
        self.ambient_event();
    }

    fn system(&mut self) {
        self.show_help();
        self.load_or_save();
        self.engine_message();
        self.quit_summary();
    }

    // INDIVIDUAL VIEW ITEM HANDLERS START HERE -------------------------------
    fn status_change(&mut self) {
        let status_msgs: Vec<_> = self.items.iter().filter(|item| item.is_status_change()).collect();
        for msg in &status_msgs {
            if let ViewItem::StatusChange { action, status } = msg {
                println!(
                    "{:<4}Status {}: {}",
                    ICON_STATUS.yellow(),
                    status.status_style(),
                    match action {
                        StatusAction::Apply => "applied",
                        StatusAction::Remove => "removed",
                    }
                );
            }
        }
        if !status_msgs.is_empty() {
            println!();
        }
    }

    fn engine_message(&mut self) {
        let engine_msgs = self.items.iter().filter(|i| i.is_engine_message());
        for msg in engine_msgs {
            println!(
                "{}",
                fill(
                    format!("{ICON_ENGINE:<4}{}", msg.clone().unwrap_engine_message()).as_str(),
                    normal_block()
                )
            );
        }
        println!();
    }

    fn points_awarded(&mut self) {
        let point_msgs = self.items.iter().filter(|i| i.is_points_awarded());
        for msg in point_msgs {
            if let ViewItem::PointsAwarded(amount) = msg {
                if amount.is_negative() {
                    println!(
                        "{:<4}You were penalized {} point{}.",
                        ICON_NEGATIVE.bright_red(),
                        amount.abs(),
                        if *amount == 1 { "" } else { "s" }
                    );
                } else {
                    println!(
                        "{:<4}You were awarded {} point{}.",
                        ICON_POSITIVE.bright_green(),
                        amount,
                        if *amount == 1 { "" } else { "s" }
                    );
                }
            }
        }
    }

    fn ambient_event(&mut self) {
        let trig_messages = self.items.iter().filter(|i| matches!(i, ViewItem::AmbientEvent(_)));
        for msg in trig_messages {
            let formatted = format!(
                "{:<4}{}",
                ICON_AMBIENT.ambient_icon_style(),
                msg.clone().unwrap_ambient_event().ambient_trig_style()
            );
            println!("{}", fill(formatted.as_str(), normal_block()));
            println!();
        }
    }
    fn triggered_event(&mut self) {
        let trig_messages = self.items.iter().filter(|i| matches!(i, ViewItem::TriggeredEvent(_)));
        for msg in trig_messages {
            let formatted = format!(
                "{:<4}{}",
                ICON_TRIGGER.trig_icon_style(),
                msg.clone().unwrap_triggered_event().triggered_style()
            );
            println!("{}", fill(formatted.as_str(), normal_block()));
            println!();
        }
    }

    fn npc_events_sorted(&mut self) {
        // Collect all NPC-related events
        let mut npc_enters: Vec<_> = self.items.iter().filter(|i| i.is_npc_entered()).collect();
        let mut npc_leaves: Vec<_> = self.items.iter().filter(|i| i.is_npc_left()).collect();
        let speech_msgs: Vec<_> = self.items.iter().filter(|i| i.is_npc_speech()).collect();

        // Sort by NPC name for consistent ordering
        npc_enters.sort_by(|a, b| a.npc_name().cmp(b.npc_name()));
        npc_leaves.sort_by(|a, b| a.npc_name().cmp(b.npc_name()));

        let has_events = !npc_enters.is_empty() || !npc_leaves.is_empty() || !speech_msgs.is_empty();

        // Display entered events first
        for msg in npc_enters {
            if let ViewItem::NpcEntered { npc_name } = msg {
                let formatted = format!(
                    "{:<4}{}",
                    ICON_NPC_ENTER.trig_icon_style(),
                    format!("{} entered.", npc_name.npc_style()).npc_movement_style()
                );
                println!("{}", fill(formatted.as_str(), normal_block()));
            }
        }

        // Then display speech events
        for quote in speech_msgs {
            if let ViewItem::NpcSpeech { speaker, quote } = quote {
                println!("{} says:", speaker.npc_style());
                println!(
                    "{}",
                    fill(quote.as_str(), indented_block()).to_string().npc_quote_style()
                );
            }
        }

        // Finally display left events
        for msg in npc_leaves {
            if let ViewItem::NpcLeft { npc_name } = msg {
                let formatted = format!(
                    "{:<4}{}",
                    ICON_NPC_LEAVE.trig_icon_style(),
                    format!("{} left.", npc_name.npc_style()).npc_movement_style()
                );
                println!("{}", fill(formatted.as_str(), normal_block()));
            }
        }

        // Add spacing if any NPC events were displayed
        if has_events {
            println!();
        }
    }

    fn load_or_save(&mut self) {
        if let Some(ViewItem::GameSaved { save_slot, save_file }) =
            self.items.iter().find(|i| matches!(i, ViewItem::GameSaved { .. }))
        {
            println!("{}: \"{}\" ({})", "Game Saved".green().bold(), save_slot, save_file);
            println!("{}", format!("Type \"load {save_slot}\" to reload it.").italic());
            println!();
        }
        if let Some(ViewItem::GameLoaded { save_slot, save_file }) =
            self.items.iter().find(|i| matches!(i, ViewItem::GameLoaded { .. }))
        {
            println!("{}: \"{}\" ({})", "Game Loaded".green().bold(), save_slot, save_file);
            println!();
        }
    }

    fn goals(&mut self) {
        let active: Vec<_> = self
            .items
            .iter()
            .filter(|i| matches!(i, ViewItem::ActiveGoal { .. }))
            .collect();

        let complete: Vec<_> = self
            .items
            .iter()
            .filter(|i| matches!(i, ViewItem::CompleteGoal { .. }))
            .collect();
        if !active.is_empty() || !complete.is_empty() {
            println!("{}:", "Active Goals".subheading_style());
            if active.is_empty() {
                println!("   {}", "Nothing here - explore more!".italic().dimmed());
            } else {
                for goal in active {
                    if let ViewItem::ActiveGoal { name, description } = goal {
                        println!("{}", name.goal_active_style());
                        println!(
                            "{}",
                            fill(description.as_str(), indented_block())
                                .to_string()
                                .description_style()
                        );
                    }
                }
            }
            println!();

            if !complete.is_empty() {
                println!("{}:", "Completed Goals".subheading_style());
                for goal in complete {
                    if let ViewItem::CompleteGoal { name, .. } = goal {
                        println!("{}", name.goal_complete_style());
                    }
                }
            }
        }
    }

    fn show_help(&mut self) {
        if let Some(ViewItem::Help { basic_text, commands }) =
            self.items.iter().find(|item| matches!(item, ViewItem::Help { .. }))
        {
            // Print the basic help text with proper text wrapping
            println!("{}", fill(basic_text, normal_block()).italic().cyan());
            println!();

            // Print "Some Common Commands:" header
            println!("{}", "Some Common Commands:".bold().yellow());
            println!();

            // Print each command with formatting and proper text wrapping
            for command in commands {
                let formatted_line = format!("{} - {}", command.command.bold().green(), command.description.italic());
                println!("{}", fill(&formatted_line, normal_block()));
            }
        }
    }

    fn quit_summary(&mut self) {
        if let Some(ViewItem::QuitSummary {
            rank,
            notes,
            score,
            max_score,
            visited,
            max_visited,
        }) = self
            .items
            .iter()
            .find(|item| matches!(item, ViewItem::QuitSummary { .. }))
        {
            let score_pct = 100.0 * (*score as f32 / *max_score as f32);
            let visit_pct = 100.0 * (*visited as f32 / *max_visited as f32);
            println!(
                "{:^width$}",
                "CANDIDATE EVALUATION REPORT".black().on_yellow(),
                width = termwidth()
            );
            println!("{:10} {}", "Rank:", rank.bright_cyan());
            println!("{:10} {}", "Notes:", notes.description_style());
            println!("{:10} {}/{} ({:.1}%)", "Score:", score, max_score, score_pct);
            println!("{:10} {}/{} ({:.1}%)", "Visited:", visited, max_visited, visit_pct);
        }
    }

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

    fn action_failure(&mut self) {
        let messages: Vec<_> = self
            .items
            .iter()
            .filter_map(|i| match i {
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

    fn errors(&mut self) {
        let messages: Vec<_> = self
            .items
            .iter()
            .filter_map(|i| match i {
                ViewItem::Error(msg) => Some(msg),
                _ => None,
            })
            .collect();
        for msg in messages {
            println!(
                "{}",
                fill(
                    format!("{} {}", ICON_ERROR.error_icon_style(), msg).as_str(),
                    normal_block()
                )
            );
        }
    }

    fn item_text(&mut self) {
        if let Some(ViewItem::ItemText(text)) = self.items.iter().find(|i| matches!(i, ViewItem::ItemText(_))) {
            println!("{}:\n", "You read".subheading_style());
            println!("{}", fill(text, indented_block()).item_text_style());
            println!();
        }
    }

    fn npc_detail(&mut self) {
        if let Some(ViewItem::NpcDescription { name, description }) =
            self.items.iter().find(|i| matches!(i, ViewItem::NpcDescription { .. }))
        {
            println!("{}", name.npc_style().underline());
            println!(
                "{}",
                fill(description.as_str(), indented_block())
                    .to_string()
                    .description_style()
            );
            println!();
        }
        if let Some(ViewItem::NpcInventory(content_lines)) =
            self.items.iter().find(|i| matches!(i, ViewItem::NpcInventory(_)))
        {
            println!("{}:", "Inventory".subheading_style());
            if content_lines.is_empty() {
                println!("   {}", "(Empty)".dimmed().italic());
            } else {
                for line in content_lines {
                    println!(
                        "   {} {}",
                        line.item_name.item_style(),
                        if line.restricted { "[R]" } else { "" }
                    );
                }
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
            println!(
                "{}",
                fill(description, indented_block()).to_string().description_style()
            );
            println!();
        }

        if let Some(ViewItem::ItemConsumableStatus(status_line)) =
            self.items.iter().find(|i| i.is_item_consumable_status())
        {
            println!(
                "{}",
                fill(
                    format!("({} {})", "Consumable:".yellow(), status_line).as_str(),
                    indented_block()
                )
                .italic()
                .dimmed()
            );
            println!();
        }

        if let Some(ViewItem::ItemContents(content_lines)) =
            self.items.iter().find(|i| matches!(i, ViewItem::ItemContents(_)))
        {
            println!("{}:", "Contents".subheading_style());
            if content_lines.is_empty() {
                println!("   {}", "Empty".italic().dimmed());
            } else {
                for line in content_lines {
                    println!(
                        "   {} {}",
                        line.item_name.item_style(),
                        if line.restricted { "[R]" } else { "" }
                    );
                }
                println!();
            }
        }
    }

    fn room_npc_list(&mut self) {
        if let Some(ViewItem::RoomNpcs(npcs)) = self.items.iter().find(|i| matches!(i, ViewItem::RoomNpcs(_))) {
            println!("{}:", "Others".subheading_style());
            npcs.iter().for_each(|npc| println!("   {}", npc.name.npc_style()));
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
        // Note: force_mode is passed with a RoomOverlay item but currently unused
        // (overlays are displayed regardless of view mode)
        if let Some(ViewItem::RoomOverlays { text, .. }) =
            self.items.iter().find(|i| matches!(i, ViewItem::RoomOverlays { .. }))
        {
            for ovl in text {
                println!("{}", fill(ovl, normal_block()).to_string().overlay_style());
                println!();
            }
        }
    }

    /// Used by `flush()` to show base room description
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
            println!("{:^width$}", name.room_titlebar_style(), width = self.width);
            if display_mode != ViewMode::Brief || !visited {
                println!(
                    "{}",
                    fill(description.as_str(), normal_block())
                        .to_string()
                        .description_style()
                );

                println!();
            }
        }
    }

    /// Clears the View's buffer but does not reset the mode.
    pub fn reset(&mut self) {
        self.items.clear();
    }

    /// Sets a `ViewMode` and returns the previously set mode.
    pub fn set_mode(&mut self, mode: ViewMode) -> ViewMode {
        std::mem::replace(&mut self.mode, mode)
    }
}

/// Subsections of the output.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Section {
    Transition,
    Environment,
    DirectResult,
    WorldResponse,
    Ambient,
    System,
}

/// `ViewMode` alters the way that each "frame" is rendered.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ViewMode {
    ClearVerbose, // clears before every room description
    Verbose,      // always shows full room description
    Brief,        // only show full description on first entry
}

/// `ViewItems` are each of the various types of information / messages that may be displayed to the player.
#[derive(Debug, Clone, PartialEq, Eq, Variantly)]
pub enum ViewItem {
    ActionFailure(String),
    ActionSuccess(String),
    ActiveGoal {
        name: String,
        description: String,
    },
    AmbientEvent(String),
    CompleteGoal {
        name: String,
        description: String,
    },
    EngineMessage(String),
    Error(String),
    GameLoaded {
        save_slot: String,
        save_file: String,
    },
    GameSaved {
        save_slot: String,
        save_file: String,
    },
    Help {
        basic_text: String,
        commands: Vec<HelpCommand>,
    },
    Inventory(Vec<ContentLine>),
    ItemConsumableStatus(String),
    ItemContents(Vec<ContentLine>),
    ItemDescription {
        name: String,
        description: String,
    },
    ItemText(String),
    NpcDescription {
        name: String,
        description: String,
    },
    NpcInventory(Vec<ContentLine>),
    NpcSpeech {
        speaker: String,
        quote: String,
    },
    NpcEntered {
        npc_name: String,
    },
    NpcLeft {
        npc_name: String,
    },
    PointsAwarded(isize),
    QuitSummary {
        rank: String,
        notes: String,
        score: usize,
        max_score: usize,
        visited: usize,
        max_visited: usize,
    },
    RoomDescription {
        name: String,
        description: String,
        visited: bool,
        force_mode: Option<ViewMode>,
    },
    RoomExits(Vec<ExitLine>),
    RoomItems(Vec<String>),
    RoomNpcs(Vec<NpcLine>),
    RoomOverlays {
        text: Vec<String>,
        force_mode: Option<ViewMode>,
    },
    StatusChange {
        action: StatusAction,
        status: String,
    },
    TransitionMessage(String),
    TriggeredEvent(String),
}
impl ViewItem {
    pub fn section(&self) -> Section {
        match self {
            ViewItem::RoomDescription { .. }
            | ViewItem::RoomOverlays { .. }
            | ViewItem::RoomItems(_)
            | ViewItem::RoomExits(_)
            | ViewItem::RoomNpcs(_) => Section::Environment,
            ViewItem::ActionSuccess(_)
            | ViewItem::ActionFailure(_)
            | ViewItem::Error(_)
            | ViewItem::ItemDescription { .. }
            | ViewItem::ItemText(_)
            | ViewItem::ItemConsumableStatus(_)
            | ViewItem::ItemContents(_)
            | ViewItem::NpcDescription { .. }
            | ViewItem::NpcInventory(_)
            | ViewItem::Inventory(_)
            | ViewItem::ActiveGoal { .. }
            | ViewItem::CompleteGoal { .. } => Section::DirectResult,
            ViewItem::NpcSpeech { .. }
            | ViewItem::NpcEntered { .. }
            | ViewItem::NpcLeft { .. }
            | ViewItem::TriggeredEvent(_)
            | ViewItem::PointsAwarded(_)
            | ViewItem::StatusChange { .. } => Section::WorldResponse,
            ViewItem::AmbientEvent(_) => Section::Ambient,
            ViewItem::QuitSummary { .. }
            | ViewItem::EngineMessage(_)
            | ViewItem::Help { .. }
            | ViewItem::GameLoaded { .. }
            | ViewItem::GameSaved { .. } => Section::System,
            ViewItem::TransitionMessage(_) => Section::Transition,
        }
    }

    /// Extract NPC name from NPC transit items.
    pub fn npc_name(&self) -> &str {
        match self {
            ViewItem::NpcEntered { npc_name } | ViewItem::NpcLeft { npc_name } => npc_name,
            _ => {
                info!("Called npc_name on ViewItem that doesn't have npc_name field");
                ""
            },
        }
    }
}
/// Different actions that can be applied to status effects.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusAction {
    Apply,
    Remove,
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
