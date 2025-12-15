//! View module.
//! This contains the view to the game world / messages.
//! Rather than printing to the console from each handler, we'll aggregate needed information and messages
//! to be organized and displayed at the end of the turn.
use std::fmt::Write;

use colored::Colorize;
use log::info;
use textwrap::{fill, termwidth};
use variantly::Variantly;

use crate::health::HealthState;
use crate::helpers::plural_s;
use crate::loader::help::HelpCommand;
use crate::markup::{StyleKind, StyleMods, WrapMode, render_inline, render_wrapped};
use crate::npc::NpcState;
use crate::save_files::{SaveFileEntry, SaveFileStatus, format_modified};
use crate::style::{GameStyle, indented_block, normal_block};

const ICON_SUCCESS: &str = "\u{2611}"; // ✔
const ICON_FAILURE: &str = "\u{274C}"; // ✖
const ICON_ERROR: &str = "⚠︎"; // U+26A0 U+FE0E
const ICON_TRIGGER: &str = "⚡︎"; // U+26A1 U+FE0E
const ICON_AMBIENT: &str = "⌘";
const ICON_NEGATIVE: &str = "➖";
const ICON_POSITIVE: &str = "➕";
const ICON_CELEBRATE: &str = "🎉"; // U+1F389
const ICON_ENGINE: &str = "⚙";
const ICON_STATUS: &str = "⚕";
const ICON_NPC_ENTER: &str = "→"; // U+2192
const ICON_NPC_LEAVE: &str = "←"; // U+2190
const ICON_HARMED: &str = "\u{2623}"; // biohazard sign
const ICON_HEALED: &str = "\u{2624}"; // caduceus
const ICON_DEATH: &str = "☠";

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
            println!("{:.>width$}\n", "situation".section_style(), width = self.width);
            self.ambience();
        }
        // Second Section: Immediate/ direct results of player command
        if !direct.is_empty() {
            println!("{:.>width$}\n", "results".section_style(), width = self.width);
            self.direct_results();
        }
        // Third Section: Triggered World / NPC reaction to Command
        if !world.is_empty() {
            println!("{:.>width$}\n", "responses".section_style(), width = self.width);
            self.world_reaction();
        }
        // Fifth Section: System Commands (load/save, help, quit etc)
        if !system.is_empty() {
            println!("{:.>width$}\n", "game".section_style(), width = self.width);
            self.system();
        }

        // clear the buffer for the next turn
        self.items.clear();

        // create a little space before the next prompt
        println!("\n");
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
        Self::triggered_event(entries);
        Self::npc_events_sorted(entries);
        Self::character_harmed(entries);
        Self::status_change(entries);
        Self::character_healed(entries);
        Self::points_awarded(entries);
        Self::character_death(entries);
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
        self.ambient_event();
    }

    fn system(&mut self) {
        self.show_help();
        self.saved_games();
        self.load_or_save();
        self.engine_message();
        self.quit_summary();
    }

    // INDIVIDUAL VIEW ITEM HANDLERS START HERE -------------------------------
    fn status_change(entries: &[&ViewEntry]) {
        let status_msgs: Vec<_> = entries
            .iter()
            .copied()
            .filter(|entry| entry.view_item.is_status_change())
            .collect();
        for msg in &status_msgs {
            if let ViewItem::StatusChange { action, status } = &msg.view_item {
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
        let engine_msgs = self.items.iter().filter(|i| i.view_item.is_engine_message());
        for msg in engine_msgs {
            println!(
                "{}",
                fill(
                    format!("{ICON_ENGINE:<4}{}", msg.view_item.clone().unwrap_engine_message()).as_str(),
                    normal_block()
                )
            );
        }
        println!();
    }

    fn points_awarded(entries: &[&ViewEntry]) {
        let point_msgs = entries.iter().copied().filter(|i| i.view_item.is_points_awarded());
        for msg in point_msgs {
            if let ViewItem::PointsAwarded { amount, reason } = &msg.view_item {
                if amount.is_negative() {
                    let text = format!("{} (-{} point{})", reason, amount.abs(), plural_s(amount.abs())).bright_red();
                    println!("{:<4}{}", ICON_NEGATIVE.bright_red(), text);
                } else if *amount > 15 {
                    let text = format!("{} (+{} point{}!)", reason, amount, plural_s(*amount)).bright_blue();
                    println!("{:<4}{}", ICON_CELEBRATE.bright_blue(), text);
                } else {
                    let text = format!("{} (+{} point{})", reason, amount, plural_s(*amount)).bright_green();
                    println!("{:<4}{}", ICON_POSITIVE.bright_green(), text);
                }
            }
        }
    }

    fn ambient_event(&mut self) {
        let trig_messages = self
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
    fn triggered_event(entries: &[&ViewEntry]) {
        let trig_messages = entries
            .iter()
            .copied()
            .filter(|i| matches!(i.view_item, ViewItem::TriggeredEvent(_)));
        for msg in trig_messages {
            let formatted = format!(
                "{:<4}{}",
                ICON_TRIGGER.trig_icon_style(),
                msg.view_item.clone().unwrap_triggered_event().triggered_style()
            );
            println!("{}", fill(formatted.as_str(), normal_block()));
            println!();
        }
    }

    fn npc_events_sorted(entries: &[&ViewEntry]) {
        // Collect all NPC-related events
        let mut npc_enters: Vec<_> = entries
            .iter()
            .copied()
            .filter(|i| i.view_item.is_npc_entered())
            .collect();
        let mut npc_leaves: Vec<_> = entries.iter().copied().filter(|i| i.view_item.is_npc_left()).collect();
        let speech_msgs: Vec<_> = entries
            .iter()
            .copied()
            .filter(|i| i.view_item.is_npc_speech())
            .collect();

        // Sort by NPC name for consistent ordering
        npc_enters.sort_by(|a, b| a.view_item.npc_name().cmp(b.view_item.npc_name()));
        npc_leaves.sort_by(|a, b| a.view_item.npc_name().cmp(b.view_item.npc_name()));

        let has_events = !npc_enters.is_empty() || !npc_leaves.is_empty() || !speech_msgs.is_empty();

        // Display entered events first
        for msg in npc_enters {
            if let ViewItem::NpcEntered { npc_name, spin_msg } = &msg.view_item {
                let formatted = format!(
                    "{:<4}{}",
                    ICON_NPC_ENTER.trig_icon_style(),
                    format!("{} {spin_msg}", npc_name.npc_style()).npc_movement_style()
                );
                println!("{}", fill(formatted.as_str(), normal_block()));
            }
        }

        // Then display speech events
        for quote in speech_msgs {
            if let ViewItem::NpcSpeech { speaker, quote } = &quote.view_item {
                println!("{} says:", speaker.npc_style());
                println!("{}", fill(quote.as_str(), indented_block()).clone().npc_quote_style());
            }
        }

        // Finally display left events
        for msg in npc_leaves {
            if let ViewItem::NpcLeft { npc_name, spin_msg } = &msg.view_item {
                let formatted = format!(
                    "{:<4}{}",
                    ICON_NPC_LEAVE.trig_icon_style(),
                    format!("{} {spin_msg}", npc_name.npc_style()).npc_movement_style()
                );
                println!("{}", fill(formatted.as_str(), normal_block()));
            }
        }

        // Add spacing if any NPC events were displayed
        if has_events {
            println!();
        }
    }

    fn saved_games(&mut self) {
        let Some((directory, entries)) = self.items.iter().find_map(|entry| match &entry.view_item {
            ViewItem::SavedGamesList { directory, entries } => Some((directory, entries)),
            _ => None,
        }) else {
            return;
        };

        println!("{}", format!("Saved games in {directory}/").subheading_style());
        if entries.is_empty() {
            println!(
                "    {}",
                "No saved games found. Use `save <slot>` to create one.".italic()
            );
            println!();
            return;
        }

        for entry in entries {
            let slot_label = entry.slot.highlight();
            let version_label = format!("[v{}]", entry.version).dimmed();
            let header = if let Some(modified) = entry.modified {
                format!(
                    "  • {} {} — saved {}",
                    slot_label,
                    version_label,
                    format_modified(modified).dimmed()
                )
            } else {
                format!("  • {slot_label} {version_label}")
            };
            println!("{header}");

            if let Some(summary) = &entry.summary {
                let location = summary.player_location.as_deref().unwrap_or("Unknown location");
                println!(
                    "    Player: {} | Turn {} | Score {} | Location: {}",
                    summary.player_name.as_str().highlight(),
                    summary.turn_count,
                    summary.score,
                    location
                );
            } else {
                println!("    {}", "Metadata unavailable for this save.".denied_style());
            }

            println!(
                "    {}",
                format!("load {}    [{directory}/{}]", entry.slot, entry.file_name).dimmed()
            );

            match &entry.status {
                SaveFileStatus::Ready => {},
                SaveFileStatus::VersionMismatch {
                    save_version,
                    current_version,
                } => println!(
                    "    {} {}",
                    "Warning:".bold().yellow(),
                    format!("saved with v{save_version}, current engine v{current_version}.").yellow()
                ),
                SaveFileStatus::Corrupted { message } => println!("    {} {}", "Error:".bold().red(), message.red()),
            }
            println!();
        }
    }

    fn load_or_save(&mut self) {
        if let Some(entry) = self
            .items
            .iter()
            .find(|i| matches!(i.view_item, ViewItem::GameSaved { .. }))
            && let ViewItem::GameSaved { save_slot, save_file } = &entry.view_item
        {
            println!("{}: \"{}\" ({})", "Game Saved".green().bold(), save_slot, save_file);
            println!("{}", format!("Type \"load {save_slot}\" to reload it.").italic());
            println!();
        }
        if let Some(entry) = self
            .items
            .iter()
            .find(|i| matches!(i.view_item, ViewItem::GameLoaded { .. }))
            && let ViewItem::GameLoaded { save_slot, save_file } = &entry.view_item
        {
            println!("{}: \"{}\" ({})", "Game Loaded".green().bold(), save_slot, save_file);
            println!();
        }
    }

    fn goals(&mut self) {
        let active: Vec<_> = self
            .items
            .iter()
            .filter(|i| matches!(i.view_item, ViewItem::ActiveGoal { .. }))
            .collect();

        let complete: Vec<_> = self
            .items
            .iter()
            .filter(|i| matches!(i.view_item, ViewItem::CompleteGoal { .. }))
            .collect();
        if !active.is_empty() || !complete.is_empty() {
            println!("{}:", "Active Goals".subheading_style());
            if active.is_empty() {
                println!("   {}", "Nothing here - explore more!".italic().dimmed());
            } else {
                for goal in active {
                    if let ViewItem::ActiveGoal { name, description } = &goal.view_item {
                        println!("{}", name.goal_active_style());
                        println!(
                            "{}",
                            render_wrapped(
                                description,
                                self.width,
                                WrapMode::Indented,
                                StyleKind::Description,
                                StyleMods::default(),
                            )
                        );
                    }
                }
            }
            println!();

            if !complete.is_empty() {
                println!("{}:", "Completed Goals".subheading_style());
                for goal in complete {
                    if let ViewItem::CompleteGoal { name, .. } = &goal.view_item {
                        println!("{}", name.goal_complete_style());
                    }
                }
            }
        }
    }

    fn show_help(&mut self) {
        if let Some(entry) = self
            .items
            .iter()
            .find(|item| matches!(&item.view_item, ViewItem::Help { .. }))
            && let ViewItem::Help { basic_text, commands } = &entry.view_item
        {
            // Print the basic help text with proper text wrapping
            println!("{}", fill(basic_text, normal_block()).italic().cyan());
            println!();

            // Partition commands into normal vs DEV (':'-prefixed)
            let (dev_cmds, normal_cmds): (Vec<_>, Vec<_>) =
                commands.iter().cloned().partition(|c| c.command.starts_with(':'));

            // Print normal commands section
            println!("{}", "Some Common Commands:".bold().yellow());
            println!();
            for command in &normal_cmds {
                let formatted_line = format!("{} - {}", command.command.bold().green(), command.description.italic());
                println!("{}", fill(&formatted_line, normal_block()));
            }

            // Print developer commands section if present and DEV_MODE
            if crate::DEV_MODE && !dev_cmds.is_empty() {
                println!();
                println!("{}", "Developer Commands (DEV_MODE):".bold().yellow());
                println!();
                for command in &dev_cmds {
                    let desc = command
                        .description
                        .strip_prefix("DEV: ")
                        .unwrap_or(&command.description)
                        .to_string();
                    let formatted_line = format!("{} - {}", command.command.bold().green(), desc.italic());
                    println!("{}", fill(&formatted_line, normal_block()));
                }
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn quit_summary(&mut self) {
        if let Some(entry) = self
            .items
            .iter()
            .find(|entry| matches!(entry.view_item, ViewItem::QuitSummary { .. }))
            && let ViewItem::QuitSummary {
                title,
                rank,
                notes,
                score,
                max_score,
                visited,
                max_visited,
            } = &entry.view_item
        {
            let score_pct = 100.0 * (*score as f32 / *max_score as f32);
            let visit_pct = 100.0 * (*visited as f32 / *max_visited as f32);
            println!("{:^width$}", title.as_str().black().on_yellow(), width = termwidth());
            println!("{:10} {}", "Rank:", rank.bright_cyan());
            println!("{:10} {}", "Notes:", notes.description_style());
            println!("{:10} {}/{} ({:.1}%)", "Score:", score, max_score, score_pct);
            println!("{:10} {}/{} ({:.1}%)", "Visited:", visited, max_visited, visit_pct);
        }
    }

    fn inventory(&mut self) {
        if let Some(entry) = self
            .items
            .iter()
            .find(|i| matches!(i.view_item, ViewItem::Inventory(..)))
            && let ViewItem::Inventory(item_lines) = &entry.view_item
        {
            println!("{}:", "Inventory".subheading_style());
            if item_lines.is_empty() {
                println!("   {}", "You have... nothing at all.".italic().dimmed());
            } else {
                for line in item_lines {
                    println!("   {}", line.item_name.item_style());
                }
            }
        }
    }

    fn action_success(&mut self) {
        let messages: Vec<_> = self
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

    fn action_failure(&mut self) {
        let messages: Vec<_> = self
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

    fn character_harmed(entries: &[&ViewEntry]) {
        let messages: Vec<_> = entries
            .iter()
            .filter_map(|i| match &i.view_item {
                ViewItem::CharacterHarmed { name, cause, amount } => Some((name, cause, amount)),
                _ => None,
            })
            .collect();
        for (name, cause, amount) in messages {
            println!(
                "{}",
                fill(
                    format!(
                        "{:<4}{} injured by {}! (-{} hp)",
                        ICON_HARMED.bright_yellow(),
                        name.npc_style(),
                        cause.underline(),
                        amount.to_string().bright_red()
                    )
                    .as_str(),
                    normal_block()
                )
            );
            println!();
        }
    }

    fn character_death(entries: &[&ViewEntry]) {
        let messages: Vec<_> = entries
            .iter()
            .filter_map(|i| match &i.view_item {
                ViewItem::CharacterDeath { name, cause, is_player } => Some((name, cause, is_player)),
                _ => None,
            })
            .collect();
        for (name, cause, is_player) in messages {
            let base = format!("{:<4}{}", ICON_DEATH.red(), name.npc_style());
            let cause_text = cause
                .as_ref()
                .filter(|c| !c.is_empty())
                .map(|c| format!(" ({c})"))
                .unwrap_or_default();
            let suffix = if *is_player {
                " has fallen.".to_string()
            } else {
                " dies.".to_string()
            };
            println!(
                "{}",
                fill(format!("{base}{cause_text}{suffix}").as_str(), normal_block())
            );
            println!();
        }
    }

    fn character_healed(entries: &[&ViewEntry]) {
        let messages: Vec<_> = entries
            .iter()
            .filter_map(|i| match &i.view_item {
                ViewItem::CharacterHealed { name, cause, amount } => Some((name, cause, amount)),
                _ => None,
            })
            .collect();
        for (name, cause, amount) in messages {
            println!(
                "{}",
                fill(
                    format!(
                        "{:<4}{} healed by {}! (+{} hp)",
                        ICON_HEALED.bright_blue(),
                        name.npc_style(),
                        cause.underline(),
                        amount.to_string().bright_green()
                    )
                    .as_str(),
                    normal_block()
                )
            );
            println!();
        }
    }

    fn errors(&mut self) {
        let messages: Vec<_> = self
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

    fn item_text(&mut self) {
        if let Some(entry) = self.items.iter().find(|i| matches!(i.view_item, ViewItem::ItemText(_)))
            && let ViewItem::ItemText(text) = &entry.view_item
        {
            println!("{}:\n", "Upon closer inspection, you see".subheading_style());
            println!(
                "{}",
                render_wrapped(
                    text,
                    self.width,
                    WrapMode::Indented,
                    StyleKind::ItemText,
                    StyleMods::default(),
                )
            );
            println!();
        }
    }

    fn npc_detail(&mut self) {
        if let Some(entry) = self
            .items
            .iter()
            .find(|i| matches!(i.view_item, ViewItem::NpcDescription { .. }))
            && let ViewItem::NpcDescription {
                name,
                description,
                health,
                state,
            } = &entry.view_item
        {
            println!("{}", name.npc_style().underline());
            let formatted_state = if let NpcState::Custom(custom_state) = state {
                custom_state.highlight()
            } else {
                state.to_string().highlight()
            };
            println!(
                "{}",
                fill(
                    format!(
                        "Health: {}/{} | State: {}",
                        health.current_hp().to_string().highlight(),
                        health.max_hp().to_string().highlight(),
                        formatted_state
                    )
                    .as_str(),
                    indented_block()
                )
            );
            // if description has a multiple lines, bold the first as a tagline - otherwise
            // use the whole thing as the tagline + description.
            if let Some((tagline, rest)) = description.split_once('\n') {
                println!(
                    "{}",
                    render_wrapped(
                        tagline,
                        self.width,
                        WrapMode::Indented,
                        StyleKind::Description,
                        StyleMods {
                            bold: true,
                            ..StyleMods::default()
                        },
                    )
                );
                println!(
                    "{}",
                    render_wrapped(
                        rest,
                        self.width,
                        WrapMode::Indented,
                        StyleKind::Description,
                        StyleMods::default(),
                    )
                );
            } else {
                println!(
                    "{}",
                    render_wrapped(
                        description,
                        self.width,
                        WrapMode::Indented,
                        StyleKind::Description,
                        StyleMods {
                            bold: true,
                            ..StyleMods::default()
                        },
                    )
                );
            }
            println!();
        }
        if let Some(ViewItem::NpcInventory(content_lines)) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::NpcInventory(_) => Some(&i.view_item),
            _ => None,
        }) {
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
        if let Some(ViewItem::ItemDescription { name, description }) =
            self.items.iter().find_map(|i| match i.view_item {
                ViewItem::ItemDescription { .. } => Some(&i.view_item),
                _ => None,
            })
        {
            println!("{}", name.item_style().underline());
            println!(
                "{}",
                render_wrapped(
                    description,
                    self.width,
                    WrapMode::Indented,
                    StyleKind::Description,
                    StyleMods::default(),
                )
            );
            println!();
        }

        if let Some(ViewItem::ItemConsumableStatus(status_line)) = self.items.iter().find_map(|i| {
            if i.view_item.is_item_consumable_status() {
                Some(&i.view_item)
            } else {
                None
            }
        }) {
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

        if let Some(ViewItem::ItemContents(content_lines)) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::ItemContents(_) => Some(&i.view_item),
            _ => None,
        }) {
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
        if let Some(ViewItem::RoomNpcs(npcs)) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::RoomNpcs(_) => Some(&i.view_item),
            _ => None,
        }) {
            println!("{}:", "Others".subheading_style());
            for npc_line in npcs {
                println!(
                    "    {} - {}",
                    npc_line.name.npc_style(),
                    render_inline(&npc_line.description, StyleKind::Description, StyleMods::default())
                );
            }
        }
    }

    fn room_exit_list(&mut self) {
        if let Some(ViewItem::RoomExits(exit_lines)) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::RoomExits(_) => Some(&i.view_item),
            _ => None,
        }) {
            println!("{}:", "Exits".subheading_style());
            for exit in exit_lines {
                print!("    > ");
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
        if let Some(ViewItem::RoomItems(names)) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::RoomItems(_) => Some(&i.view_item),
            _ => None,
        }) {
            println!("{}:", "Items".subheading_style());
            for name in names {
                println!("    * {}", name.item_style());
            }
        }
    }

    fn room_overlays(&mut self) {
        // Note: force_mode is passed with a RoomOverlay item but currently unused
        // (overlays are displayed regardless of view mode)
        if let Some(ViewItem::RoomOverlays { text, .. }) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::RoomOverlays { .. } => Some(&i.view_item),
            _ => None,
        }) {
            let mut full_ovl = String::new();
            for ovl in text {
                let _ = write!(full_ovl, "{ovl} ");
            }
            println!(
                "{}\n",
                render_wrapped(
                    &full_ovl,
                    self.width,
                    WrapMode::Normal,
                    StyleKind::Overlay,
                    StyleMods::default(),
                )
            );
        }
    }

    /// Used by `flush()` to show base room description
    fn room_description(&mut self) {
        if let Some(ViewItem::RoomDescription {
            name,
            description,
            visited,
            force_mode,
        }) = self.items.iter().find_map(|i| match i.view_item {
            ViewItem::RoomDescription { .. } => Some(&i.view_item),
            _ => None,
        }) {
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
                    render_wrapped(
                        description,
                        self.width,
                        WrapMode::Normal,
                        StyleKind::Description,
                        StyleMods::default(),
                    )
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
    CharacterHarmed {
        name: String,
        cause: String,
        amount: u32,
    },
    CharacterHealed {
        name: String,
        cause: String,
        amount: u32,
    },
    CharacterDeath {
        name: String,
        cause: Option<String>,
        is_player: bool,
    },
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
    SavedGamesList {
        directory: String,
        entries: Vec<SaveFileEntry>,
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
        health: HealthState,
        state: NpcState,
    },
    NpcInventory(Vec<ContentLine>),
    NpcSpeech {
        speaker: String,
        quote: String,
    },
    NpcEntered {
        npc_name: String,
        spin_msg: String,
    },
    NpcLeft {
        npc_name: String,
        spin_msg: String,
    },
    PointsAwarded {
        amount: isize,
        reason: String,
    },
    QuitSummary {
        title: String,
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
    /// Classify a view item into a top-level output section.
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
            ViewItem::CharacterHarmed { .. }
            | ViewItem::CharacterDeath { .. }
            | ViewItem::CharacterHealed { .. }
            | ViewItem::NpcSpeech { .. }
            | ViewItem::NpcEntered { .. }
            | ViewItem::NpcLeft { .. }
            | ViewItem::TriggeredEvent(_)
            | ViewItem::PointsAwarded { .. }
            | ViewItem::StatusChange { .. } => Section::WorldResponse,
            ViewItem::AmbientEvent(_) => Section::Ambient,
            ViewItem::QuitSummary { .. }
            | ViewItem::EngineMessage(_)
            | ViewItem::Help { .. }
            | ViewItem::GameLoaded { .. }
            | ViewItem::GameSaved { .. }
            | ViewItem::SavedGamesList { .. } => Section::System,
            ViewItem::TransitionMessage(_) => Section::Transition,
        }
    }

    pub fn default_priority(&self) -> isize {
        match &self {
            ViewItem::TriggeredEvent(_) => -30,
            ViewItem::CharacterHarmed { .. } => -20,
            ViewItem::CharacterHealed { .. } => -10,
            ViewItem::NpcEntered { .. } => 5,
            ViewItem::NpcSpeech { .. } => 10,
            ViewItem::NpcLeft { .. } => 15,
            ViewItem::CharacterDeath { .. } => 100,
            _ => 0,
        }
    }

    /// Extract NPC name from NPC transit items.
    pub fn npc_name(&self) -> &str {
        match self {
            ViewItem::NpcEntered { npc_name, .. } | ViewItem::NpcLeft { npc_name, .. } => npc_name,
            _ => {
                info!("Called npc_name on ViewItem that doesn't have npc_name field");
                ""
            },
        }
    }
}
/// Indicates whether a status effect is being applied or removed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StatusAction {
    Apply,
    Remove,
}

/// Row data for listing container contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentLine {
    pub item_name: String,
    pub restricted: bool,
}

/// Row data for the exit listing portion of the view.
#[derive(Debug, Clone, PartialEq, Eq)]
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
