//! View module.
//! This contains the view to the game world / messages.
//! Rather than printing to the console from each handler, we'll aggregate needed information and messages
//! to be organized and displayed at the end of the turn.

use crate::style::{GameStyle, normal_block};

pub mod icons;
pub mod view_item;
use textwrap::{fill, termwidth};
pub use view_item::ViewItem;

mod render_action;
mod render_env;
mod render_health;
mod render_item;
mod render_npc;
mod render_player;
mod render_system;
mod render_trig;

/// View aggregates information to be displayed on each pass through the REPL and then organizes
/// and displays the result.
#[derive(Debug, Clone)]
pub struct View {
    pub width: usize,
    pub mode: ViewMode,
    pub items: Vec<ViewEntry>,
    pub sequence: usize,
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
            sequence: 0,
        }
    }

    pub fn push(&mut self, item: ViewItem) {
        self.push_with_custom_priority(item, None);
    }

    /// Push a `ViewItem` for the next frame with a custom set priority
    pub fn push_with_priority(&mut self, item: ViewItem, priority: isize) {
        self.push_with_custom_priority(item, Some(priority));
    }

    /// Push a `ViewItem` honoring an optional custom priority override.
    pub fn push_with_custom_priority(&mut self, item: ViewItem, priority: Option<isize>) {
        self.items.push(ViewEntry {
            section: item.section(),
            priority: item.default_priority(),
            custom_priority: priority,
            view_item: item,
            sequence: self.sequence,
        });
        self.sequence += 1;
    }

    /// Compose and diplay all message contents in the current frame / turn.
    pub fn flush(&mut self) {
        // re-check terminal width in case it's been resized
        self.width = termwidth();

        // Bin each item by section so we only iterate once.
        let mut transitions = Vec::new();
        let mut environment = Vec::new();
        let mut direct = Vec::new();
        let mut world = Vec::new();
        let mut ambient = Vec::new();
        let mut system = Vec::new();
        for item in &self.items {
            match item.view_item.section() {
                Section::Transition => transitions.push(item.clone()),
                Section::Environment => environment.push(item.clone()),
                Section::DirectResult => direct.push(item.clone()),
                Section::WorldResponse => world.push(item.clone()),
                Section::Ambient => ambient.push(item.clone()),
                Section::System => system.push(item.clone()),
            }
        }

        // Section Zero: Movement transition message, if any
        if let Some(msg) = transitions.iter().find_map(|i| match &i.view_item {
            ViewItem::TransitionMessage(msg) => Some(msg),
            _ => None,
        }) {
            println!("\n{}", fill(msg.as_str(), normal_block()).transition_style());
        }

        // First Section: Environment / Frame of Reference
        if !environment.is_empty() {
            println!("{:.>width$}\n", "scene".section_style(), width = self.width);
            self.environment();
        }
        // Fourth Section: Messages not related to last command / action (ambients, goals, etc.)
        if !ambient.is_empty() {
            println!("{:.>width$}\n", "surroundings".section_style(), width = self.width);
            self.ambience();
        }
        // Second Section: Immediate/ direct results of player command
        if !direct.is_empty() {
            println!("{:.>width$}\n", "results".section_style(), width = self.width);
            self.direct_results();
        }
        // Third Section: Triggered World / NPC reaction to Command
        if !world.is_empty() {
            println!("{:.>width$}\n", "reactions".section_style(), width = self.width);
            self.world_reaction();
        }
        // Fifth Section: System Commands (load/save, help, quit etc)
        if !system.is_empty() {
            println!("{:.>width$}\n", "game".section_style(), width = self.width);
            self.system();
        }

        // clear the buffer for the next turn
        self.items.clear();
    }

    // SECTION AGGREGATORS START HERE --------------------

    fn environment(&mut self) {
        // Show overview of room/area
        render_env::room_description(self);
        render_env::room_overlays(self);
        render_env::room_item_list(self);
        render_env::room_exit_list(self);
        render_env::room_npc_list(self);
    }

    fn direct_results(&mut self) {
        // direct inspection (read, look_at) results
        render_item::item_detail(self);
        render_item::item_text(self);
        render_npc::npc_detail(self);
        render_player::inventory(self);
        render_player::goals(self);

        // successes / failures
        render_action::action_success(self);
        render_action::action_failure(self);
        render_action::errors(self);
    }

    /// Collect world reaction-type entries, sort, and display them in batches (`bucket`) according to priority view order.
    fn world_reaction(&mut self) {
        let world_entries = self.world_entries_sorted();
        if world_entries.is_empty() {
            return;
        }

        let mut current_priority: Option<isize> = None;
        let mut bucket: Vec<&ViewEntry> = Vec::new();
        for entry in world_entries {
            let priority = entry.effective_priority();
            if current_priority.is_some_and(|p| p != priority) {
                Self::render_world_bucket(&bucket);
                bucket.clear();
            }
            bucket.push(entry);
            current_priority = Some(priority);
        }
        if !bucket.is_empty() {
            Self::render_world_bucket(&bucket);
        }
    }

    /// Display a collection of view entries that have the same effective priority.
    fn render_world_bucket(entries: &[&ViewEntry]) {
        if entries.is_empty() {
            return;
        }
        render_trig::triggered_event(entries);
        render_npc::npc_events_sorted(entries);
        render_health::character_harmed(entries);
        render_health::status_change(entries);
        render_health::character_healed(entries);
        render_trig::points_awarded(entries);
        render_health::character_death(entries);
    }

    /// Filter all `ViewEntry`s for this frame, retaining only those in the `WorldResponse` section and sort them
    /// by effective priority (lowest priority value shows first, e.g. 1 goes before 10, -10 goes before 1).
    fn world_entries_sorted(&self) -> Vec<&ViewEntry> {
        let mut world_entries: Vec<&ViewEntry> = self
            .items
            .iter()
            .filter(|entry| entry.section == Section::WorldResponse)
            .collect();
        world_entries.sort_by(|a, b| {
            a.effective_priority()
                .cmp(&b.effective_priority())
                .then_with(|| a.sequence.cmp(&b.sequence))
        });
        world_entries
    }

    fn ambience(&mut self) {
        render_trig::ambient_event(self);
    }

    fn system(&mut self) {
        render_system::show_help(self);
        render_system::saved_games(self);
        render_system::load_or_save(self);
        render_system::engine_message(self);
        render_system::quit_summary(self);
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
    /// Transitional text/log lines between turns.
    Transition,
    /// Room description, exits, and ambient context.
    Environment,
    /// Direct results of the player's command.
    DirectResult,
    /// Follow-up reactions from the world or NPCs.
    WorldResponse,
    /// Ambient chatter and scheduled flavour text.
    Ambient,
    /// Meta/game-system feedback (saves, help, etc.).
    System,
}

/// `ViewMode` alters the way that each "frame" is rendered.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ViewMode {
    /// Always render full descriptions and clear before each frame.
    ClearVerbose,
    /// Always render full descriptions without clearing between turns.
    Verbose,
    /// Render brief descriptions after the first visit to a room.
    Brief,
}

/// Wrapper for a `ViewItem` to allow flexible ordering of display items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewEntry {
    pub section: Section,
    pub priority: isize,
    pub custom_priority: Option<isize>,
    pub view_item: ViewItem,
    pub sequence: usize,
}

impl ViewEntry {
    /// Returns an overriding custom display priority if one is set, otherwise the base value.
    fn effective_priority(&self) -> isize {
        self.custom_priority.unwrap_or(self.priority)
    }
}

/// Indicates whether a status effect is being applied or removed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusAction {
    Apply,
    Remove,
}

/// Row data for listing container contents.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentLine {
    pub item_name: String,
    pub restricted: bool,
}

/// Row data for the exit listing portion of the view.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct ExitLine {
    pub direction: String,
    pub destination: String,
    pub exit_locked: bool,
    pub dest_visited: bool,
}

/// Row data for the NPC list within room descriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcLine {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_entries_sorted_respects_custom_priority() {
        let mut view = View::new();
        view.push(ViewItem::ActionSuccess("ignored".into()));
        view.push(ViewItem::NpcSpeech {
            speaker: "NPC".into(),
            quote: "Hello".into(),
        });
        view.push_with_priority(ViewItem::TriggeredEvent("Radio hums".into()), -25);
        view.push_with_priority(
            ViewItem::StatusChange {
                action: StatusAction::Apply,
                status: "Poisoned".into(),
            },
            25,
        );

        let ordered: Vec<&str> = view
            .world_entries_sorted()
            .iter()
            .map(|entry| match &entry.view_item {
                ViewItem::TriggeredEvent(_) => "triggered",
                ViewItem::NpcSpeech { .. } => "speech",
                ViewItem::StatusChange { .. } => "status",
                other => panic!("Unexpected ViewItem in results: {other:?}"),
            })
            .collect();

        assert_eq!(ordered, vec!["triggered", "speech", "status"]);
    }

    #[test]
    fn world_entries_sorted_excludes_other_sections() {
        let mut view = View::new();
        view.push(ViewItem::ActionSuccess("direct result".into()));
        view.push(ViewItem::AmbientEvent("ambient".into()));
        view.push(ViewItem::NpcSpeech {
            speaker: "NPC".into(),
            quote: "Priority".into(),
        });

        let entries = view.world_entries_sorted();
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0].view_item, ViewItem::NpcSpeech { .. }));
    }
}
