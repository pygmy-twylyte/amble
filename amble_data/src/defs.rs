use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Stable identifier used across WorldDef references.
pub type Id = String;

/// Top-level compiled world data loaded by the engine.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldDef {
    pub game: GameDef,
    #[serde(default)]
    pub rooms: Vec<RoomDef>,
    #[serde(default)]
    pub items: Vec<ItemDef>,
    #[serde(default)]
    pub npcs: Vec<NpcDef>,
    #[serde(default)]
    pub spinners: Vec<SpinnerDef>,
    #[serde(default)]
    pub triggers: Vec<TriggerDef>,
    #[serde(default)]
    pub goals: Vec<GoalDef>,
}

/// Game-level metadata and startup configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDef {
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub blurb: String,
    pub intro: String,
    pub player: PlayerDef,
    #[serde(default)]
    pub scoring: ScoringDef,
}

impl Default for GameDef {
    fn default() -> Self {
        Self {
            title: String::new(),
            slug: String::new(),
            author: String::new(),
            version: String::new(),
            blurb: String::new(),
            intro: String::new(),
            player: PlayerDef::default(),
            scoring: ScoringDef::default(),
        }
    }
}

/// Player definition emitted from DSL configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDef {
    pub name: String,
    pub description: String,
    pub start_room: Id,
    pub max_hp: u32,
}

impl Default for PlayerDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            start_room: String::new(),
            max_hp: 1,
        }
    }
}

/// Scoring configuration emitted from the DSL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringDef {
    #[serde(default = "default_report_title")]
    pub report_title: String,
    #[serde(default = "default_scoring_ranks")]
    pub ranks: Vec<ScoringRankDef>,
}

impl Default for ScoringDef {
    fn default() -> Self {
        Self {
            report_title: default_report_title(),
            ranks: default_scoring_ranks(),
        }
    }
}

/// Single scoring rank entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringRankDef {
    pub threshold: f32,
    pub name: String,
    pub description: String,
}

/// Room definition used by the engine at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDef {
    pub id: Id,
    pub name: String,
    pub desc: String,
    #[serde(default)]
    pub visited: bool,
    #[serde(default)]
    pub exits: Vec<ExitDef>,
    #[serde(default)]
    pub overlays: Vec<OverlayDef>,
}

/// Exit metadata for room navigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitDef {
    pub direction: String,
    pub to: Id,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub required_flags: Vec<String>,
    #[serde(default)]
    pub required_items: Vec<Id>,
    pub barred_message: Option<String>,
}

/// A room overlay with optional conditions and text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayDef {
    #[serde(default)]
    pub conditions: Vec<OverlayCondDef>,
    pub text: String,
}

/// Conditions that gate a room overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverlayCondDef {
    FlagSet { flag: String },
    FlagUnset { flag: String },
    FlagComplete { flag: String },
    ItemPresent { item: Id },
    ItemAbsent { item: Id },
    PlayerHasItem { item: Id },
    PlayerMissingItem { item: Id },
    NpcPresent { npc: Id },
    NpcAbsent { npc: Id },
    NpcInState { npc: Id, state: NpcState },
    ItemInRoom { item: Id, room: Id },
}

/// Spinner definition for ambient or random text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinnerDef {
    pub id: Id,
    #[serde(default)]
    pub wedges: Vec<SpinnerWedgeDef>,
}

/// Weighted spinner entry text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinnerWedgeDef {
    pub text: String,
    #[serde(default = "default_wedge_width")]
    pub width: usize,
}

fn default_wedge_width() -> usize {
    1
}

fn default_report_title() -> String {
    "Scorecard".to_string()
}

fn default_scoring_ranks() -> Vec<ScoringRankDef> {
    vec![
        ScoringRankDef {
            threshold: 99.0,
            name: "Quantum Overachiever".to_string(),
            description: "You saw the multiverse, understood it, then filed a bug report.".to_string(),
        },
        ScoringRankDef {
            threshold: 90.0,
            name: "Senior Field Operative".to_string(),
            description: "A nearly flawless run. Someone give this candidate a promotion.".to_string(),
        },
        ScoringRankDef {
            threshold: 75.0,
            name: "Licensed Reality Bender".to_string(),
            description: "Impressive grasp of nonlinear environments and cake-based paradoxes.".to_string(),
        },
        ScoringRankDef {
            threshold: 60.0,
            name: "Rogue Intern, Level II".to_string(),
            description: "You got the job done, and only melted one small pocket universe.".to_string(),
        },
        ScoringRankDef {
            threshold: 45.0,
            name: "Unpaid Research Assistant".to_string(),
            description: "Solid effort. Some concepts may have slipped through dimensional cracks.".to_string(),
        },
        ScoringRankDef {
            threshold: 30.0,
            name: "Junior Sandwich Technician".to_string(),
            description: "Good instincts, questionable execution. Especially with condiments.".to_string(),
        },
        ScoringRankDef {
            threshold: 15.0,
            name: "Volunteer Tour Guide".to_string(),
            description: "You wandered. You looked at stuff. It was something.".to_string(),
        },
        ScoringRankDef {
            threshold: 5.0,
            name: "Mailbox Stuffing Trainee".to_string(),
            description: "You opened a box, tripped on a rug, and called it a day.".to_string(),
        },
        ScoringRankDef {
            threshold: 1.0,
            name: "Accidental Hire".to_string(),
            description: "We're not sure how you got in. Please return your lanyard.".to_string(),
        },
        ScoringRankDef {
            threshold: 0.0,
            name: "Amnesiac Test Subject".to_string(),
            description: "Did you… play? Were you even awake?".to_string(),
        },
    ]
}

/// Item definition used by the engine at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: Id,
    pub name: String,
    pub desc: String,
    #[serde(default)]
    pub movability: Movability,
    pub container_state: Option<ContainerState>,
    pub location: LocationRef,
    #[serde(default)]
    pub abilities: Vec<ItemAbility>,
    #[serde(default)]
    pub interaction_requires: BTreeMap<ItemInteractionType, ItemAbility>,
    pub text: Option<String>,
    pub consumable: Option<ConsumableDef>,
}

/// Authoring-time reference to an object's starting location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationRef {
    Inventory,
    Nowhere,
    Room(Id),
    Item(Id),
    Npc(Id),
}

/// Consumable item metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableDef {
    pub uses_left: usize,
    #[serde(default)]
    pub consume_on: Vec<ItemAbility>,
    pub when_consumed: ConsumeTypeDef,
}

/// How consumables behave when used.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConsumeTypeDef {
    Despawn,
    ReplaceInventory { replacement: Id },
    ReplaceCurrentRoom { replacement: Id },
}

/// NPC definition used by the engine at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDef {
    pub id: Id,
    pub name: String,
    pub desc: String,
    pub max_hp: u32,
    pub location: LocationRef,
    pub state: NpcState,
    #[serde(default)]
    pub dialogue: BTreeMap<NpcState, Vec<String>>,
    pub movement: Option<NpcMovementDef>,
}

/// NPC movement configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcMovementDef {
    pub movement_type: NpcMovementType,
    #[serde(default)]
    pub rooms: Vec<Id>,
    pub timing: Option<NpcMovementTiming>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

/// Selects how an NPC chooses its next room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NpcMovementType {
    Route,
    RandomSet,
}

/// Timing rules for NPC movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NpcMovementTiming {
    EveryNTurns { turns: usize },
    OnTurn { turn: usize },
}

/// Trigger definition with event, conditions, and actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDef {
    pub name: String,
    pub note: Option<String>,
    #[serde(default)]
    pub only_once: bool,
    pub event: EventDef,
    #[serde(default)]
    pub conditions: ConditionExpr,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
}

/// Trigger event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventDef {
    Always,
    EnterRoom {
        room: Id,
    },
    LeaveRoom {
        room: Id,
    },
    TakeItem {
        item: Id,
    },
    DropItem {
        item: Id,
    },
    LookAtItem {
        item: Id,
    },
    OpenItem {
        item: Id,
    },
    UnlockItem {
        item: Id,
    },
    TouchItem {
        item: Id,
    },
    TalkToNpc {
        npc: Id,
    },
    UseItem {
        item: Id,
        ability: ItemAbility,
    },
    UseItemOnItem {
        tool: Id,
        target: Id,
        interaction: ItemInteractionType,
    },
    ActOnItem {
        target: Id,
        action: ItemInteractionType,
    },
    GiveToNpc {
        item: Id,
        npc: Id,
    },
    TakeFromNpc {
        item: Id,
        npc: Id,
    },
    InsertItemInto {
        item: Id,
        container: Id,
    },
    Ingest {
        item: Id,
        mode: IngestMode,
    },
    PlayerDeath,
    NpcDeath {
        npc: Id,
    },
}

/// Boolean expression tree used by triggers and conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionExpr {
    All(Vec<ConditionExpr>),
    Any(Vec<ConditionExpr>),
    Pred(ConditionDef),
}

impl Default for ConditionExpr {
    fn default() -> Self {
        ConditionExpr::All(Vec::new())
    }
}

/// Leaf predicates used by ConditionExpr.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionDef {
    HasFlag { flag: String },
    MissingFlag { flag: String },
    FlagInProgress { flag: String },
    FlagComplete { flag: String },
    HasItem { item: Id },
    MissingItem { item: Id },
    HasVisited { room: Id },
    PlayerInRoom { room: Id },
    WithNpc { npc: Id },
    NpcHasItem { npc: Id, item: Id },
    NpcInState { npc: Id, state: NpcState },
    ContainerHasItem { container: Id, item: Id },
    ChancePercent { percent: f64 },
    Ambient { spinner: Id, rooms: Option<Vec<Id>> },
}

/// Action entry with optional scheduling priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub action: ActionKind,
    #[serde(default)]
    pub priority: Option<isize>,
}

/// Actions executed by triggers or schedules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionKind {
    ShowMessage {
        text: String,
    },
    AddFlag {
        flag: FlagDef,
    },
    AdvanceFlag {
        name: String,
    },
    RemoveFlag {
        name: String,
    },
    ResetFlag {
        name: String,
    },
    AwardPoints {
        amount: isize,
        reason: String,
    },
    DamagePlayer {
        amount: u32,
        cause: String,
    },
    DamagePlayerOT {
        amount: u32,
        turns: u32,
        cause: String,
    },
    HealPlayer {
        amount: u32,
        cause: String,
    },
    HealPlayerOT {
        amount: u32,
        turns: u32,
        cause: String,
    },
    RemovePlayerEffect {
        cause: String,
    },
    DamageNpc {
        npc: Id,
        amount: u32,
        cause: String,
    },
    DamageNpcOT {
        npc: Id,
        amount: u32,
        turns: u32,
        cause: String,
    },
    HealNpc {
        npc: Id,
        amount: u32,
        cause: String,
    },
    HealNpcOT {
        npc: Id,
        amount: u32,
        turns: u32,
        cause: String,
    },
    RemoveNpcEffect {
        npc: Id,
        cause: String,
    },
    SetNpcActive {
        npc: Id,
        active: bool,
    },
    SetNpcState {
        npc: Id,
        state: NpcState,
    },
    NpcSays {
        npc: Id,
        quote: String,
    },
    NpcSaysRandom {
        npc: Id,
    },
    NpcRefuseItem {
        npc: Id,
        reason: String,
    },
    GiveItemToPlayer {
        npc: Id,
        item: Id,
    },
    PushPlayerTo {
        room: Id,
    },
    AddSpinnerWedge {
        spinner: Id,
        text: String,
        width: usize,
    },
    SpinnerMessage {
        spinner: Id,
    },
    DenyRead {
        reason: String,
    },
    SpawnItemCurrentRoom {
        item: Id,
    },
    SpawnItemInRoom {
        item: Id,
        room: Id,
    },
    SpawnItemInInventory {
        item: Id,
    },
    SpawnItemInContainer {
        item: Id,
        container: Id,
    },
    SpawnNpcInRoom {
        npc: Id,
        room: Id,
    },
    DespawnItem {
        item: Id,
    },
    DespawnNpc {
        npc: Id,
    },
    ReplaceItem {
        old_item: Id,
        new_item: Id,
    },
    ReplaceDropItem {
        old_item: Id,
        new_item: Id,
    },
    LockItem {
        item: Id,
    },
    UnlockItem {
        item: Id,
    },
    SetContainerState {
        item: Id,
        state: Option<ContainerState>,
    },
    SetItemDescription {
        item: Id,
        text: String,
    },
    SetItemMovability {
        item: Id,
        movability: Movability,
    },
    LockExit {
        from_room: Id,
        direction: String,
    },
    UnlockExit {
        from_room: Id,
        direction: String,
    },
    RevealExit {
        exit_from: Id,
        exit_to: Id,
        direction: String,
    },
    SetBarredMessage {
        exit_from: Id,
        exit_to: Id,
        msg: String,
    },
    ModifyItem {
        item: Id,
        patch: ItemPatchDef,
    },
    ModifyRoom {
        room: Id,
        patch: RoomPatchDef,
    },
    ModifyNpc {
        npc: Id,
        patch: NpcPatchDef,
    },
    Conditional {
        condition: ConditionExpr,
        actions: Vec<ActionDef>,
    },
    ScheduleIn {
        turns_ahead: usize,
        actions: Vec<ActionDef>,
        note: Option<String>,
    },
    ScheduleOn {
        on_turn: usize,
        actions: Vec<ActionDef>,
        note: Option<String>,
    },
    ScheduleInIf {
        turns_ahead: usize,
        condition: ConditionExpr,
        #[serde(default)]
        on_false: OnFalsePolicy,
        actions: Vec<ActionDef>,
        note: Option<String>,
    },
    ScheduleOnIf {
        on_turn: usize,
        condition: ConditionExpr,
        #[serde(default)]
        on_false: OnFalsePolicy,
        actions: Vec<ActionDef>,
        note: Option<String>,
    },
}

/// Flag definition used by authoring and trigger actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FlagDef {
    Simple { name: String },
    Sequence { name: String, end: Option<u8> },
}

/// Patch data applied to an item at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemPatchDef {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub text: Option<String>,
    pub movability: Option<Movability>,
    pub container_state: Option<ContainerState>,
    #[serde(default)]
    pub remove_container_state: bool,
    #[serde(default)]
    pub add_abilities: Vec<ItemAbility>,
    #[serde(default)]
    pub remove_abilities: Vec<ItemAbility>,
}

/// Patch data applied to a room at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoomPatchDef {
    pub name: Option<String>,
    pub desc: Option<String>,
    #[serde(default)]
    pub remove_exits: Vec<Id>,
    #[serde(default)]
    pub add_exits: Vec<RoomExitPatchDef>,
}

/// Exit data used by room patching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomExitPatchDef {
    pub direction: String,
    pub to: Id,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub required_flags: Vec<String>,
    #[serde(default)]
    pub required_items: Vec<Id>,
    pub barred_message: Option<String>,
}

/// Single dialogue line to add to an NPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDialoguePatchDef {
    pub state: NpcState,
    pub line: String,
}

/// Timing change for NPC movement patches.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NpcTimingPatchDef {
    EveryNTurns { turns: usize },
    OnTurn { turn: usize },
}

/// Patch data applied to NPC movement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcMovementPatchDef {
    pub route: Option<Vec<Id>>,
    pub random_rooms: Option<Vec<Id>>,
    pub timing: Option<NpcTimingPatchDef>,
    pub active: Option<bool>,
    pub loop_route: Option<bool>,
}

/// Patch data applied to an NPC at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NpcPatchDef {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub state: Option<NpcState>,
    #[serde(default)]
    pub add_lines: Vec<NpcDialoguePatchDef>,
    pub movement: Option<NpcMovementPatchDef>,
}

/// Policy for scheduled actions when a condition evaluates false.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum OnFalsePolicy {
    #[default]
    Cancel,
    RetryAfter {
        turns: usize,
    },
    RetryNextTurn,
}

/// Goal definition for player progress tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub group: GoalGroup,
    pub activate_when: Option<GoalCondition>,
    pub finished_when: GoalCondition,
    pub failed_when: Option<GoalCondition>,
}

/// Groups goals for display and tracking purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalGroup {
    Required,
    Optional,
    StatusEffect,
}

/// Predicate used for goal activation/completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GoalCondition {
    FlagComplete { flag: String },
    FlagInProgress { flag: String },
    GoalComplete { goal_id: String },
    HasItem { item: Id },
    HasFlag { flag: String },
    MissingFlag { flag: String },
    ReachedRoom { room: Id },
}

/// Ingest action mode for consumables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IngestMode {
    Eat,
    Drink,
    Inhale,
}

/// Ability tags used to gate item interactions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemAbility {
    Attach,
    Clean,
    Cut,
    CutWood,
    Drink,
    Eat,
    Extinguish,
    Ignite,
    Inhale,
    Insulate,
    Magnify,
    Pluck,
    Pry,
    Read,
    Repair,
    Sharpen,
    Smash,
    TurnOn,
    TurnOff,
    Unlock(Option<Id>),
    Use,
}

/// Interaction verbs used when applying one item to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemInteractionType {
    Attach,
    Break,
    Burn,
    Extinguish,
    Clean,
    Cover,
    Cut,
    Handle,
    Move,
    Open,
    Repair,
    Sharpen,
    Turn,
    Unlock,
}

/// Container visibility and lock state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContainerState {
    Open,
    Closed,
    Locked,
    TransparentOpen,
    TransparentClosed,
    TransparentLocked,
}

/// Movability constraints for items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Movability {
    Fixed {
        reason: String,
    },
    Restricted {
        reason: String,
    },
    #[default]
    Free,
}

/// NPC mood/state tags used for dialogue and conditions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NpcState {
    Bored,
    Happy,
    Mad,
    Normal,
    Sad,
    Tired,
    Custom(String),
}
