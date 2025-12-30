//! ViewItem module
//!
//! A `ViewItem` is an enum variant sent to the `View`, which aggregates them, styles them,
//! organizes them, and displays them before moving to the next turn. Variants contain different
//! payloads, depending the type of information that needs to be displayed.

use log::info;
use variantly::Variantly;

use crate::health::HealthState;
use crate::loader::help::HelpCommand;
use crate::npc::NpcState;
use crate::save_files::SaveFileEntry;
use crate::view::{ExitLine, NpcLine, Section, StatusAction, ViewMode};

use super::ContentLine;

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
