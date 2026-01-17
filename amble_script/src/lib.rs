//! `amble_script` – authoring-focused DSL, parser, and compiler for building
//! Amble worlds.
//!
//! The crate powers the `amble_script` CLI but is fully usable as a library.
//! Major capabilities:
//! - Parse and lint `.amble` sources for game config, rooms, items, triggers,
//!   spinners, NPCs, and goals, catching unresolved references early.
//! - Compile the DSL into `WorldDef` (RON) that the `amble_engine` crate consumes.
//! - Provide AST types so tooling can analyze or transform worlds before
//!   serialization.
//!
//! ```ignore
//! use amble_script::{parse_program_full, worlddef_from_asts};
//!
//! let src = r#"
//! game {
//!   title "Demo"
//!   intro "Welcome to the demo."
//!   player {
//!     name "The Candidate"
//!     desc "a seasoned adventurer."
//!     max_hp 20
//!     start room foyer
//!   }
//! }
//!
//! room foyer {
//!   name "Foyer"
//!   desc "An inviting entryway."
//! }
//! "#;
//! let (game, triggers, rooms, items, spinners, npcs, goals) = parse_program_full(src).expect("valid DSL");
//! let worlddef = worlddef_from_asts(game.as_ref(), &triggers, &rooms, &items, &spinners, &npcs, &goals)
//!     .expect("compiles");
//! let ron = ron::ser::to_string_pretty(&worlddef, ron::ser::PrettyConfig::default()).expect("serializes");
//! println!("{ron}");
//! ```
//!
//! For a full language tour see `amble_script/docs/dsl_creator_handbook.md` in
//! the repository.

mod parser;
mod worlddef;
pub use parser::{AstError, parse_program, parse_trigger};
pub use parser::{parse_goals, parse_items, parse_npcs, parse_program_full, parse_rooms, parse_spinners};
pub use worlddef::{WorldDefError, worlddef_from_asts};

/// Game-level configuration AST.
#[derive(Debug, Clone, PartialEq)]
pub struct GameAst {
    pub title: String,
    pub slug: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub blurb: Option<String>,
    pub intro: String,
    pub player: PlayerAst,
    pub scoring: Option<ScoringAst>,
}

/// Player definition contained inside the game block.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerAst {
    pub name: String,
    pub description: String,
    pub max_hp: u32,
    pub start_room: String,
}

/// Scoring definition emitted from the DSL.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoringAst {
    pub report_title: Option<String>,
    pub ranks: Vec<ScoringRankAst>,
}

/// Single scoring rank entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoringRankAst {
    pub threshold: f32,
    pub name: String,
    pub description: String,
}

/// A minimal AST for a single trigger.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerAst {
    /// Human-readable trigger name.
    pub name: String,
    /// Optional developer note for this trigger.
    pub note: Option<String>,
    /// 1-based line number in the source file where this trigger starts.
    pub src_line: usize,
    /// The event condition that triggers this (e.g., enter room, take item, talk to npc).
    pub event: ConditionAst,
    /// List of conditions (currently only missing-flag).
    pub conditions: Vec<ConditionAst>,
    /// List of actions supported in this minimal version.
    pub actions: Vec<ActionStmt>,
    /// If true, the trigger should only fire once.
    pub only_once: bool,
}

/// Trigger condition variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionAst {
    /// Event: trigger has no event; conditions only.
    Always,
    /// Event: player enters a room.
    EnterRoom(String),
    /// Event: player takes an item.
    TakeItem(String),
    /// Event: player touches / presses an item
    TouchItem(String),
    /// Event: player talks to an NPC.
    TalkToNpc(String),
    /// Event: player opens an item.
    OpenItem(String),
    /// Event: player leaves a room.
    LeaveRoom(String),
    /// Event: player looks at an item.
    LookAtItem(String),
    /// Event: player uses an item with an ability.
    UseItem {
        item: String,
        ability: String,
    },
    /// Event: player gives an item to an NPC.
    GiveToNpc {
        item: String,
        npc: String,
    },
    /// Event: player uses one item on another item with an interaction.
    UseItemOnItem {
        tool: String,
        target: String,
        interaction: String,
    },
    /// Event: player ingests an item using a specific mode (eat, drink, inhale).
    Ingest {
        item: String,
        mode: IngestModeAst,
    },
    /// Event: player dies.
    PlayerDeath,
    /// Event: an NPC dies.
    NpcDeath(String),
    /// Event: player performs an interaction on an item (tool-agnostic).
    ActOnItem {
        target: String,
        action: String,
    },
    /// Event: player takes an item from an NPC.
    TakeFromNpc {
        item: String,
        npc: String,
    },
    /// Event: player inserts an item into a container item.
    InsertItemInto {
        item: String,
        container: String,
    },
    /// Event: player drops an item.
    DropItem(String),
    /// Event: player unlocks an item.
    UnlockItem(String),
    /// Require that a flag is missing (by name).
    MissingFlag(String),
    /// Require that a flag is present (by name).
    HasFlag(String),
    /// Require that the player has an item (by symbol id).
    HasItem(String),
    /// Require that the player is currently in a room (by symbol id).
    PlayerInRoom(String),
    HasVisited(String),
    MissingItem(String),
    FlagInProgress(String),
    FlagComplete(String),
    WithNpc(String),
    NpcHasItem {
        npc: String,
        item: String,
    },
    NpcInState {
        npc: String,
        state: NpcStateValue,
    },
    ContainerHasItem {
        container: String,
        item: String,
    },
    Ambient {
        spinner: String,
        rooms: Option<Vec<String>>,
    },
    /// Random chance in percent (0-100).
    ChancePercent(f64),
    /// All of the nested conditions must hold.
    All(Vec<ConditionAst>),
    /// Any of the nested conditions may hold.
    Any(Vec<ConditionAst>),
}

/// Ingestion modes supported by the DSL; mirrors engine `IngestMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestModeAst {
    Eat,
    Drink,
    Inhale,
}

/// Minimal action variants.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionAst {
    /// Show a message to the player.
    Show(String),
    /// Add a weighted wedge to a spinner
    AddSpinnerWedge {
        spinner: String,
        width: usize,
        text: String,
    },
    /// Add a simple flag by name.
    AddFlag(String),
    /// Add a sequence flag by name with optional limit (end)
    AddSeqFlag {
        name: String,
        end: Option<u8>,
    },
    /// Award points to the player's score with a reason string.
    AwardPoints {
        amount: i64,
        reason: String,
    },
    /// Damage the player once or over multiple turns.
    DamagePlayer {
        amount: u32,
        turns: Option<usize>,
        cause: String,
    },
    /// Heal the player once or over multiple turns.
    HealPlayer {
        amount: u32,
        turns: Option<usize>,
        cause: String,
    },
    /// Remove a queued health effect from the player by cause.
    RemovePlayerEffect {
        cause: String,
    },
    /// Damage an NPC once or over multiple turns.
    DamageNpc {
        npc: String,
        amount: u32,
        turns: Option<usize>,
        cause: String,
    },
    /// Heal an NPC once or over multiple turns.
    HealNpc {
        npc: String,
        amount: u32,
        turns: Option<usize>,
        cause: String,
    },
    /// Remove a queued health effect from an NPC by cause.
    RemoveNpcEffect {
        npc: String,
        cause: String,
    },
    /// Remove a flag by name.
    RemoveFlag(String),
    /// Replace an item instance by symbol with another
    ReplaceItem {
        old_sym: String,
        new_sym: String,
    },
    /// Replace an item when dropped with another symbol
    ReplaceDropItem {
        old_sym: String,
        new_sym: String,
    },
    /// Apply an item patch to mutate fields atomically.
    ModifyItem {
        item: String,
        patch: ItemPatchAst,
    },
    /// Apply a room patch to mutate room fields atomically.
    ModifyRoom {
        room: String,
        patch: RoomPatchAst,
    },
    /// Apply an NPC patch to mutate npc fields atomically.
    ModifyNpc {
        npc: String,
        patch: NpcPatchAst,
    },
    /// Spawn an item into a room.
    SpawnItemIntoRoom {
        item: String,
        room: String,
    },
    /// Despawn an item.
    DespawnItem(String),
    /// Despawn an NPC
    DespawnNpc(String),
    /// Reset a sequence flag to step 0.
    ResetFlag(String),
    /// Advance a sequence flag by one step.
    AdvanceFlag(String),
    /// Set a barred message for an exit between rooms
    SetBarredMessage {
        exit_from: String,
        exit_to: String,
        msg: String,
    },
    /// Reveal an exit in a room in a direction to another room.
    RevealExit {
        exit_from: String,
        exit_to: String,
        direction: String,
    },
    /// Lock or unlock exits and items
    LockExit {
        from_room: String,
        direction: String,
    },
    UnlockExit {
        from_room: String,
        direction: String,
    },
    LockItem(String),
    UnlockItemAction(String),
    /// Push player to a room.
    PushPlayerTo(String),
    /// NPC gives an item to the player
    GiveItemToPlayer {
        npc: String,
        item: String,
    },
    /// Spawns
    SpawnItemInInventory(String),
    SpawnItemCurrentRoom(String),
    SpawnItemInContainer {
        item: String,
        container: String,
    },
    SpawnNpcIntoRoom {
        npc: String,
        room: String,
    },
    /// Set description for an item by symbol.
    SetItemDescription {
        item: String,
        text: String,
    },
    /// Set movability for an item by symbol.
    SetItemMovability {
        item: String,
        movability: MovabilityAst,
    },
    NpcSays {
        npc: String,
        quote: String,
    },
    NpcSaysRandom {
        npc: String,
    },
    /// NPC refuses an item with a reason
    NpcRefuseItem {
        npc: String,
        reason: String,
    },
    /// Set NPC active/inactive for movement
    SetNpcActive {
        npc: String,
        active: bool,
    },
    SetNpcState {
        npc: String,
        state: String,
    },
    DenyRead(String),
    /// Set container state for an item by symbol; omit state to clear
    SetContainerState {
        item: String,
        state: Option<String>,
    },
    /// Show a random message from a spinner
    SpinnerMessage {
        spinner: String,
    },
    /// Schedules without conditions
    ScheduleIn {
        turns_ahead: usize,
        actions: Vec<ActionStmt>,
        note: Option<String>,
    },
    ScheduleOn {
        on_turn: usize,
        actions: Vec<ActionStmt>,
        note: Option<String>,
    },
    /// Schedule actions at turns ahead if a condition holds.
    ScheduleInIf {
        turns_ahead: usize,
        condition: Box<ConditionAst>,
        on_false: Option<OnFalseAst>,
        actions: Vec<ActionStmt>,
        note: Option<String>,
    },
    /// Schedule actions on an absolute turn if a condition holds.
    ScheduleOnIf {
        on_turn: usize,
        condition: Box<ConditionAst>,
        on_false: Option<OnFalseAst>,
        actions: Vec<ActionStmt>,
        note: Option<String>,
    },
    /// Conditionally execute nested actions when the condition evaluates true at runtime.
    Conditional {
        condition: Box<ConditionAst>,
        actions: Vec<ActionStmt>,
    },
}

/// Top-level action statement with optional priority metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionStmt {
    /// Optional priority assigned via `do priority <n>`.
    pub priority: Option<isize>,
    /// The underlying action emitted by the DSL.
    pub action: ActionAst,
}
impl ActionStmt {
    /// Construct an action statement without priority metadata.
    pub fn new(action: ActionAst) -> Self {
        Self { priority: None, action }
    }

    /// Construct an action statement with an explicit priority.
    pub fn with_priority(action: ActionAst, priority: isize) -> Self {
        Self {
            priority: Some(priority),
            action,
        }
    }
}

/// Data patch applied to an item when executing a `modify item` action.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemPatchAst {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub text: Option<String>,
    pub movability: Option<MovabilityAst>,
    pub container_state: Option<ContainerStateAst>,
    pub remove_container_state: bool,
    pub add_abilities: Vec<ItemAbilityAst>,
    pub remove_abilities: Vec<ItemAbilityAst>,
}

/// Data patch applied to a room when executing a `modify room` action.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoomPatchAst {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub remove_exits: Vec<String>,
    pub add_exits: Vec<RoomExitPatchAst>,
}

/// Exit data emitted inside a `modify room` action patch.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoomExitPatchAst {
    pub direction: String,
    pub to: String,
    pub hidden: bool,
    pub locked: bool,
    pub barred_message: Option<String>,
    pub required_flags: Vec<String>,
    pub required_items: Vec<String>,
}

/// NPC dialogue line update used inside a `modify npc` action.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcDialoguePatchAst {
    pub state: NpcStateValue,
    pub line: String,
}

/// Movement timing update for an NPC.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcTimingPatchAst {
    /// NPC moves every _n_ turns.
    EveryNTurns(usize),
    /// NPC moves on the specified absolute turn.
    OnTurn(usize),
}

/// Movement configuration updates for an NPC.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NpcMovementPatchAst {
    pub route: Option<Vec<String>>,
    pub random_rooms: Option<Vec<String>>,
    pub timing: Option<NpcTimingPatchAst>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

/// Data patch applied to an NPC when executing a `modify npc` action.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NpcPatchAst {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub state: Option<NpcStateValue>,
    pub add_lines: Vec<NpcDialoguePatchAst>,
    pub movement: Option<NpcMovementPatchAst>,
}

/// Policy to apply when a scheduled condition evaluates to false at fire time.
#[derive(Debug, Clone, PartialEq)]
pub enum OnFalseAst {
    /// Drop the scheduled event entirely.
    Cancel,
    /// Reschedule the event the specified number of turns ahead.
    RetryAfter { turns: usize },
    /// Reschedule the event for the very next turn.
    RetryNextTurn,
}

// -----------------
// Rooms (minimal)
// -----------------

/// Minimal AST for a room definition.
/// AST node describing a compiled room definition.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomAst {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub visited: bool, // defaults to false when omitted in DSL
    pub exits: Vec<(String, ExitAst)>,
    pub overlays: Vec<OverlayAst>,
    pub src_line: usize,
}

/// Connection between rooms emitted within a room AST.
#[derive(Debug, Clone, PartialEq)]
pub struct ExitAst {
    pub to: String,
    pub hidden: bool,
    pub locked: bool,
    pub barred_message: Option<String>,
    pub required_flags: Vec<String>,
    pub required_items: Vec<String>,
}

/// Conditional overlay text applied to a room.
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayAst {
    pub conditions: Vec<OverlayCondAst>,
    pub text: String,
}

/// Overlay predicate used when computing room description variants.
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayCondAst {
    FlagSet(String),
    FlagUnset(String),
    FlagComplete(String),
    ItemPresent(String),
    ItemAbsent(String),
    PlayerHasItem(String),
    PlayerMissingItem(String),
    NpcPresent(String),
    NpcAbsent(String),
    NpcInState { npc: String, state: NpcStateValue },
    ItemInRoom { item: String, room: String },
}

/// NPC state reference used in overlays and patches.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcStateValue {
    Named(String),
    Custom(String),
}

/// AST node describing an item definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemAst {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub movability: MovabilityAst,
    pub location: ItemLocationAst,
    pub container_state: Option<ContainerStateAst>,
    pub abilities: Vec<ItemAbilityAst>,
    pub text: Option<String>,
    pub interaction_requires: Vec<(String, String)>,
    pub consumable: Option<ConsumableAst>,
    pub src_line: usize,
}

/// Possible item locations in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemLocationAst {
    Inventory(String),
    Room(String),
    Npc(String),
    Chest(String),
    Nowhere(String),
}

/// Container states expressible in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStateAst {
    Open,
    Closed,
    Locked,
    TransparentClosed,
    TransparentLocked,
}

/// Movability options for items, mirroring the engine `Movability`.
#[derive(Debug, Clone, PartialEq)]
pub enum MovabilityAst {
    Free,
    Fixed { reason: String },
    Restricted { reason: String },
}

/// Single item ability entry declared within an item.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemAbilityAst {
    pub ability: String,
    pub target: Option<String>,
}

/// Consumable configuration attached to an item.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumableAst {
    pub uses_left: usize,
    pub consume_on: Vec<ItemAbilityAst>,
    pub when_consumed: ConsumableWhenAst,
}

/// Behavior when a consumable item is depleted.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsumableWhenAst {
    Despawn,
    ReplaceInventory { replacement: String },
    ReplaceCurrentRoom { replacement: String },
}

// -----------------
// Spinners
// -----------------

/// Spinner definition containing weighted text wedges.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinnerAst {
    pub id: String,
    pub wedges: Vec<SpinnerWedgeAst>,
    pub src_line: usize,
}

/// Individual wedge (value + weight) inside a spinner.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinnerWedgeAst {
    pub text: String,
    pub width: usize,
}

// -----------------
// NPCs
// -----------------

/// Movement types supported for NPC definitions.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcMovementTypeAst {
    Route,
    Random,
}

/// Movement configuration emitted for NPCs.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcMovementAst {
    pub movement_type: NpcMovementTypeAst,
    pub rooms: Vec<String>,
    pub timing: Option<String>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

/// AST node describing an NPC definition.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcAst {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub max_hp: u32,
    pub location: NpcLocationAst,
    pub state: NpcStateValue,
    pub movement: Option<NpcMovementAst>,
    pub dialogue: Vec<(String, Vec<String>)>,
    pub src_line: usize,
}

/// Location specifier used for NPC placement.
#[derive(Debug, Clone, PartialEq)]
pub enum NpcLocationAst {
    Room(String),
    Nowhere(String),
}

// -----------------
// Goals
// -----------------

/// Logical grouping for goals used when rendering score breakdowns.
#[derive(Debug, Clone, PartialEq)]
pub enum GoalGroupAst {
    /// Mandatory goals that count toward completion.
    Required,
    /// Optional side objectives.
    Optional,
    /// Status effects or temporary conditions.
    StatusEffect,
}

/// Conditions that can activate, complete, or fail a goal.
#[derive(Debug, Clone, PartialEq)]
pub enum GoalCondAst {
    /// Goal requires the player to have a flag.
    HasFlag(String),
    /// Goal requires the player to be missing a flag.
    MissingFlag(String),
    /// Goal requires the player to possess an item.
    HasItem(String),
    /// Goal requires the player to reach a room.
    ReachedRoom(String),
    /// Goal requires another goal to be complete.
    GoalComplete(String),
    /// Goal depends on a sequence flag progressing but not finishing.
    FlagInProgress(String),
    /// Goal requires a sequence flag to reach its terminal step.
    FlagComplete(String),
}

/// High-level representation of a single goal definition in the DSL.
#[derive(Debug, Clone, PartialEq)]
pub struct GoalAst {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group: GoalGroupAst,
    pub activate_when: Option<GoalCondAst>,
    pub failed_when: Option<GoalCondAst>,
    pub finished_when: GoalCondAst,
    pub src_line: usize,
}
