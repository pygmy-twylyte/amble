//! Player state representation and flag management.
//!
//! Defines the player struct plus helpers for manipulating inventory,
//! location history, and progression flags.
use crate::health::{HealthEffect, HealthState, LivingEntity};
use crate::{ItemHolder, Location, WorldObject};

use log::{info, warn};
use serde::de::{self, Deserializer, EnumAccess, VariantAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;
use variantly::Variantly;

/// The player-controlled character.
///
/// This struct tracks the player's state, such as inventory, score and flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub location: Location,
    pub location_history: Vec<Uuid>,
    pub inventory: HashSet<Uuid>,
    pub flags: HashSet<Flag>,
    pub score: usize,
    pub health: HealthState,
}
impl Player {
    /// Updates one of the existing flags. Emits a warning if the flag isn't found.
    pub fn update_flag<F>(&mut self, name: &str, updater: F)
    where
        F: FnOnce(&mut Flag),
    {
        let target = Flag::Simple {
            name: name.to_string(),
            turn_set: 0,
        };
        if let Some(mut flag) = self.flags.take(&target) {
            updater(&mut flag);
            info!("player flag updated: '{flag}'");
            self.flags.insert(flag);
        } else {
            warn!("update_flag: flag '{name}' not set");
        }
    }

    /// Advances a sequence flag to the next step.
    pub fn advance_flag(&mut self, name: &str) {
        self.update_flag(name, Flag::advance);
    }

    /// Reset a sequence flag to the first step.
    pub fn reset_flag(&mut self, name: &str) {
        self.update_flag(name, Flag::reset);
    }

    /// Returns list of applied status effects.
    ///
    /// Status effects are created by using a `Flag` with a name in the form "status:<`status_type`>",
    /// e.g. "status:nausea"
    pub fn status(&self) -> HashSet<&str> {
        self.flags
            .iter()
            .filter_map(|f| f.name().strip_prefix("status:"))
            .collect()
    }

    /// Adds current location to history and updates to new location.
    /// Maintains a maximum of 5 previous locations.
    pub fn move_to_room(&mut self, new_room_id: Uuid) {
        if let Location::Room(current_room) = self.location {
            self.location_history.push(current_room);
            // Keep only the last 5 locations
            if self.location_history.len() > 5 {
                self.location_history.remove(0);
            }
        }
        self.location = Location::Room(new_room_id);
    }

    /// Returns the most recent room in history, if any.
    pub fn previous_room(&self) -> Option<Uuid> {
        self.location_history.last().copied()
    }

    /// Moves back to the previous room, removing it from history.
    /// Returns the room ID moved to, or None if no history exists.
    pub fn go_back(&mut self) -> Option<Uuid> {
        if let Some(previous_room) = self.location_history.pop() {
            self.location = Location::Room(previous_room);
            Some(previous_room)
        } else {
            None
        }
    }
}
impl Default for Player {
    fn default() -> Player {
        Self {
            id: Uuid::new_v4(),
            symbol: "the_candidate".into(),
            name: "The Candidate".into(),
            description: "default".into(),
            location: Location::default(),
            location_history: Vec::new(),
            inventory: HashSet::<Uuid>::default(),
            flags: HashSet::<Flag>::default(),
            score: 1,
            health: HealthState::default(),
        }
    }
}
impl WorldObject for Player {
    fn id(&self) -> Uuid {
        self.id
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn location(&self) -> &Location {
        &self.location
    }
}
impl ItemHolder for Player {
    fn add_item(&mut self, item_id: Uuid) {
        self.inventory.insert(item_id);
    }

    fn remove_item(&mut self, item_id: Uuid) {
        self.inventory.remove(&item_id);
    }

    fn contains_item(&self, item_id: Uuid) -> bool {
        self.inventory.contains(&item_id)
    }
}
impl LivingEntity for Player {
    fn max_hp(&self) -> u32 {
        self.health.max_hp()
    }

    fn current_hp(&self) -> u32 {
        self.health.current_hp()
    }

    fn life_state(&self) -> crate::health::LifeState {
        self.health.life_state()
    }

    fn damage(&mut self, amount: u32) {
        self.health.damage(amount);
    }

    fn heal(&mut self, amount: u32) {
        self.health.heal(amount);
    }

    fn add_health_effect(&mut self, effect: HealthEffect) {
        self.health.add_effect(effect);
    }

    fn remove_health_effect(&mut self, cause: &str) -> Option<HealthEffect> {
        self.health.remove_effect(cause)
    }

    fn tick_health_effects(&mut self) -> crate::health::HealthTickResult {
        self.health.apply_effects(self.name.as_str())
    }
}

/// Flags that can be applied to the player
#[derive(Debug, Clone, Variantly)]
pub enum Flag {
    Simple {
        name: String,
        turn_set: usize,
    },
    Sequence {
        name: String,
        turn_set: usize,
        step: u8,
        end: Option<u8>,
    },
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum FlagKind {
    Simple,
    Sequence,
}

impl FlagKind {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            v if v.eq_ignore_ascii_case("simple") => Some(Self::Simple),
            v if v.eq_ignore_ascii_case("sequence") => Some(Self::Sequence),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for FlagKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlagKindVisitor;

        impl<'de> Visitor<'de> for FlagKindVisitor {
            type Value = FlagKind;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("flag type identifier 'simple' or 'sequence'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                FlagKind::from_str(value).ok_or_else(|| de::Error::unknown_variant(value, &["simple", "sequence"]))
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(value)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (variant, access) = data.variant::<String>()?;
                access.unit_variant()?;
                self.visit_str(&variant)
            }
        }

        deserializer.deserialize_any(FlagKindVisitor)
    }
}

impl Serialize for Flag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Flag::Simple { name, turn_set } => {
                let mut state = serializer.serialize_struct("Flag", 3)?;
                state.serialize_field("type", &FlagKind::Simple)?;
                state.serialize_field("name", name)?;
                state.serialize_field("turn_set", turn_set)?;
                state.end()
            },
            Flag::Sequence {
                name,
                turn_set,
                step,
                end,
            } => {
                let mut state = serializer.serialize_struct("Flag", 5)?;
                state.serialize_field("type", &FlagKind::Sequence)?;
                state.serialize_field("name", name)?;
                state.serialize_field("turn_set", turn_set)?;
                state.serialize_field("step", step)?;
                state.serialize_field("end", end)?;
                state.end()
            },
        }
    }
}

#[derive(Deserialize)]
struct FlagRepr {
    #[serde(rename = "type")]
    kind: FlagKind,
    name: String,
    #[serde(default)]
    turn_set: usize,
    #[serde(default)]
    step: u8,
    #[serde(default)]
    end: Option<u8>,
}

impl<'de> Deserialize<'de> for Flag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = FlagRepr::deserialize(deserializer)?;
        Ok(match repr.kind {
            FlagKind::Simple => Flag::Simple {
                name: repr.name,
                turn_set: repr.turn_set,
            },
            FlagKind::Sequence => Flag::Sequence {
                name: repr.name,
                turn_set: repr.turn_set,
                step: repr.step,
                end: repr.end,
            },
        })
    }
}
impl Flag {
    /// Return string value of the flag.
    /// For 'Simple' this is just "`flag_name`".
    /// For "Sequence" this is "`flag_name#N`" where N is the current sequence step number.
    pub fn value(&self) -> String {
        match self {
            Self::Simple { name, .. } => name.clone(),
            Self::Sequence { name, step, .. } => format_sequence_value(name, *step),
        }
    }

    /// Advances to next step of a sequence
    ///
    /// Logs a warning and does nothing if called on a simple flag.
    pub fn advance(&mut self) {
        match self {
            Flag::Simple { name, .. } => {
                warn!("advance() called on non-sequence flag '{name}'");
            },
            Flag::Sequence { name, step, end, .. } => {
                if let Some(final_step) = end {
                    *step = std::cmp::min(*step + 1, *final_step);
                } else {
                    *step += 1;
                }
                info!("sequence '{name}' advanced to step {step}");
            },
        }
    }

    /// Resets to beginning of sequence
    pub fn reset(&mut self) {
        match self {
            Flag::Simple { name, .. } => warn!("reset() called on non-sequence flag '{name}'"),
            Flag::Sequence { name, step, .. } => {
                *step = 0;
                info!("sequence '{name}' reset to step '{step}'");
            },
        }
    }

    /// Returns true if a sequence is complete, or if called on a simple flag.
    ///
    /// # Panics
    /// Cannot panic unless sunspots alter the value of `end` between evaluation of the last two match arms
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Simple { .. } => true,
            Self::Sequence { end, .. } if end.is_none() => false,
            Self::Sequence { step, end, .. } => *step == end.expect("end must be Some(u8) if we reach this arm"),
        }
    }

    /// Create a new simple flag
    pub fn simple(name: &str, turn_set: usize) -> Flag {
        Flag::Simple {
            name: name.to_string(),
            turn_set,
        }
    }

    /// Create a new sequence flag
    pub fn sequence(name: &str, end: Option<u8>, turn_set: usize) -> Flag {
        Flag::Sequence {
            name: name.to_string(),
            turn_set,
            step: 0u8,
            end,
        }
    }

    /// Get base name of the flag
    pub fn name(&self) -> &str {
        match self {
            Flag::Simple { name, .. } | Flag::Sequence { name, .. } => name,
        }
    }
}
impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flag::Simple { name, .. } => write!(f, "{name}"),
            Flag::Sequence { name, step, .. } => write!(f, "{name}#{step}"),
        }
    }
}
use std::hash::{Hash, Hasher};

impl PartialEq for Flag {
    /// Defines equality of two flags as based on name only (not step).
    ///
    /// This is crucial for `HashSet` operations - flags are considered equal
    /// if they have the same name, regardless of their current step in a sequence.
    /// This allows updating sequence flags by removing the old version and
    /// inserting the updated version.
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Flag {}

impl Hash for Flag {
    /// Hash implementation that matches the `PartialEq` implementation.
    ///
    /// Since equality is based only on the flag name, we hash only the name
    /// to maintain the invariant that equal items have equal hashes.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

/// Formats a sequence-type flag into a string value
///
/// Format is "name"#"step", e.g. "`hal_reboot#2`"
pub fn format_sequence_value(name: &str, step: u8) -> String {
    format!("{name}#{step}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use uuid::Uuid;

    fn create_test_player() -> Player {
        let mut player = Player::default();
        player.flags.insert(Flag::simple("test_flag", 0));
        player.flags.insert(Flag::sequence("test_seq", Some(3), 0));
        let item_id = Uuid::new_v4();
        player.inventory.insert(item_id);
        player
    }

    #[test]
    fn player_default_creates_valid_player() {
        let player = Player::default();
        assert_eq!(player.symbol, "the_candidate");
        assert_eq!(player.name, "The Candidate");
        assert_eq!(player.description, "default");
        assert_eq!(player.location, Location::default());
        assert_eq!(player.score, 1);
        assert!(player.inventory.is_empty());
        assert!(player.flags.is_empty());
        assert!(player.location_history.is_empty());
    }

    #[test]
    fn update_flag_modifies_existing_flag() {
        let mut player = create_test_player();

        player.update_flag("test_seq", |flag| {
            if let Flag::Sequence { step, .. } = flag {
                *step = 5;
            }
        });

        let updated_flag = player.flags.get(&Flag::simple("test_seq", 0)).unwrap();
        if let Flag::Sequence { step, .. } = updated_flag {
            assert_eq!(*step, 5);
        } else {
            panic!("Expected sequence flag");
        }
    }

    #[test]
    fn update_flag_does_nothing_for_nonexistent_flag() {
        let mut player = create_test_player();
        let original_flags = player.flags.clone();

        player.update_flag("nonexistent", |_flag| {
            // This should never be called
            panic!("Should not be called for nonexistent flag");
        });

        assert_eq!(player.flags, original_flags);
    }

    #[test]
    fn advance_flag_increments_sequence_step() {
        let mut player = create_test_player();

        player.advance_flag("test_seq");

        let flag = player.flags.get(&Flag::simple("test_seq", 0)).unwrap();
        if let Flag::Sequence { step, .. } = flag {
            assert_eq!(*step, 1);
        } else {
            panic!("Expected sequence flag");
        }
    }

    #[test]
    fn advance_flag_respects_end_limit() {
        let mut player = create_test_player();

        // Advance to end
        player.advance_flag("test_seq"); // step 1
        player.advance_flag("test_seq"); // step 2
        player.advance_flag("test_seq"); // step 3 (end)
        player.advance_flag("test_seq"); // should stay at 3

        let flag = player.flags.get(&Flag::simple("test_seq", 0)).unwrap();
        if let Flag::Sequence { step, end, .. } = flag {
            assert_eq!(*step, 3);
            assert_eq!(*end, Some(3));
        } else {
            panic!("Expected sequence flag");
        }
    }

    #[test]
    fn reset_flag_sets_sequence_to_zero() {
        let mut player = create_test_player();

        // Advance first
        player.advance_flag("test_seq");
        player.advance_flag("test_seq");

        // Then reset
        player.reset_flag("test_seq");

        let flag = player.flags.get(&Flag::simple("test_seq", 0)).unwrap();
        if let Flag::Sequence { step, .. } = flag {
            assert_eq!(*step, 0);
        } else {
            panic!("Expected sequence flag");
        }
    }

    #[test]
    fn world_object_trait_works() {
        let player = create_test_player();
        assert_eq!(player.symbol(), "the_candidate");
        assert_eq!(player.name(), "The Candidate");
        assert_eq!(player.description(), "default");
        assert_eq!(player.location(), &Location::default());
    }

    #[test]
    fn item_holder_add_item_works() {
        let mut player = Player::default();
        let item_id = Uuid::new_v4();

        player.add_item(item_id);
        assert!(player.inventory.contains(&item_id));
    }

    #[test]
    fn item_holder_remove_item_works() {
        let mut player = create_test_player();
        let item_id = *player.inventory.iter().next().unwrap();

        player.remove_item(item_id);
        assert!(!player.inventory.contains(&item_id));
    }

    #[test]
    fn item_holder_contains_item_works() {
        let player = create_test_player();
        let item_id = *player.inventory.iter().next().unwrap();

        assert!(player.contains_item(item_id));
        assert!(!player.contains_item(Uuid::new_v4()));
    }

    #[test]
    fn flag_simple_creates_simple_flag() {
        let flag = Flag::simple("test", 12);
        if let Flag::Simple { name, turn_set } = flag {
            assert_eq!(name, "test");
            assert_eq!(turn_set, 12);
        } else {
            panic!("Expected simple flag");
        }
    }

    #[test]
    fn flag_sequence_creates_sequence_flag() {
        let flag = Flag::sequence("test", Some(5), 12);
        if let Flag::Sequence {
            name,
            step,
            end,
            turn_set,
        } = flag
        {
            assert_eq!(name, "test");
            assert_eq!(step, 0);
            assert_eq!(end, Some(5));
            assert_eq!(turn_set, 12);
        } else {
            panic!("Expected sequence flag");
        }
    }

    #[test]
    fn flag_value_returns_correct_values() {
        let simple = Flag::simple("simple_flag", 0);
        assert_eq!(simple.value(), "simple_flag");

        let sequence = Flag::sequence("seq_flag", Some(2), 0);
        assert_eq!(sequence.value(), "seq_flag#0");

        let mut advanced_seq = Flag::sequence("advanced", Some(3), 0);
        advanced_seq.advance();
        assert_eq!(advanced_seq.value(), "advanced#1");
    }

    #[test]
    fn flag_advance_works_for_sequence() {
        let mut flag = Flag::sequence("test", Some(3), 0);

        flag.advance();
        if let Flag::Sequence { step, .. } = flag {
            assert_eq!(step, 1);
        }

        flag.advance();
        if let Flag::Sequence { step, .. } = flag {
            assert_eq!(step, 2);
        }
    }

    #[test]
    fn flag_advance_respects_end_limit() {
        let mut flag = Flag::sequence("test", Some(2), 0);

        flag.advance(); // step 1
        flag.advance(); // step 2 (end)
        flag.advance(); // should stay at 2

        if let Flag::Sequence { step, .. } = flag {
            assert_eq!(step, 2);
        }
    }

    #[test]
    fn flag_advance_unlimited_sequence() {
        let mut flag = Flag::sequence("test", None, 0);

        for i in 1..=10 {
            flag.advance();
            if let Flag::Sequence { step, .. } = flag {
                assert_eq!(step, i);
            }
        }
    }

    #[test]
    fn flag_reset_works_for_sequence() {
        let mut flag = Flag::sequence("test", Some(3), 0);
        flag.advance();
        flag.advance();

        flag.reset();
        if let Flag::Sequence { step, .. } = flag {
            assert_eq!(step, 0);
        }
    }

    #[test]
    fn flag_is_complete_works() {
        let simple = Flag::simple("test", 0);
        assert!(simple.is_complete());

        let incomplete_seq = Flag::sequence("test", Some(3), 0);
        assert!(!incomplete_seq.is_complete());

        let mut complete_seq = Flag::sequence("test", Some(2), 0);
        complete_seq.advance();
        complete_seq.advance();
        assert!(complete_seq.is_complete());

        let unlimited_seq = Flag::sequence("test", None, 0);
        assert!(!unlimited_seq.is_complete());
    }

    #[test]
    fn flag_name_returns_base_name() {
        let simple = Flag::simple("simple_name", 0);
        assert_eq!(simple.name(), "simple_name");

        let sequence = Flag::sequence("seq_name", Some(3), 0);
        assert_eq!(sequence.name(), "seq_name");
    }

    #[test]
    fn flag_display_works() {
        let simple = Flag::simple("test_flag", 0);
        assert_eq!(format!("{}", simple), "test_flag");

        let sequence = Flag::sequence("test_seq", Some(3), 0);
        assert_eq!(format!("{}", sequence), "test_seq#0");

        let mut advanced = Flag::sequence("advanced", Some(2), 0);
        advanced.advance();
        assert_eq!(format!("{}", advanced), "advanced#1");
    }

    #[test]
    fn flag_equality_based_on_name_only() {
        let flag1 = Flag::simple("test", 0);
        let mut flag2 = Flag::sequence("test", Some(3), 0);
        flag2.advance();

        // Different types but same name should be equal
        assert_eq!(flag1, flag2);

        let flag3 = Flag::simple("different", 0);
        assert_ne!(flag1, flag3);
    }

    #[test]
    fn flag_hash_based_on_name_only() {
        let flag1 = Flag::simple("test", 0);
        let mut flag2 = Flag::sequence("test", Some(3), 0);
        flag2.advance();

        let mut set = HashSet::new();
        set.insert(flag1);

        // Should find flag2 because it has same name as flag1
        assert!(set.contains(&flag2));

        let flag3 = Flag::simple("different", 0);
        assert!(!set.contains(&flag3));
    }

    #[test]
    fn format_sequence_value_works() {
        assert_eq!(format_sequence_value("test", 0), "test#0");
        assert_eq!(format_sequence_value("quest", 5), "quest#5");
        assert_eq!(format_sequence_value("special_name", 255), "special_name#255");
    }

    #[test]
    fn move_to_room_updates_location_and_history() {
        use crate::Location;
        let mut player = Player::default();
        let room1 = Uuid::new_v4();
        let room2 = Uuid::new_v4();
        let room3 = Uuid::new_v4();

        // Start with player in room1
        player.location = Location::Room(room1);

        // Move to room2
        player.move_to_room(room2);
        assert_eq!(player.location, Location::Room(room2));
        assert_eq!(player.location_history, vec![room1]);

        // Move to room3
        player.move_to_room(room3);
        assert_eq!(player.location, Location::Room(room3));
        assert_eq!(player.location_history, vec![room1, room2]);
    }

    #[test]
    fn move_to_room_limits_history_size() {
        use crate::Location;
        let mut player = Player::default();
        let rooms: Vec<Uuid> = (0..8).map(|_| Uuid::new_v4()).collect();

        // Start in room 0
        player.location = Location::Room(rooms[0]);

        // Move through all rooms
        for i in 1..8 {
            player.move_to_room(rooms[i]);
        }

        // Should only keep last 5 rooms in history
        assert_eq!(player.location_history.len(), 5);
        assert_eq!(player.location_history, rooms[2..7].to_vec());
    }

    #[test]
    fn previous_room_returns_last_room() {
        use crate::Location;
        let mut player = Player::default();
        let room1 = Uuid::new_v4();
        let room2 = Uuid::new_v4();

        // No history initially
        assert_eq!(player.previous_room(), None);

        // After one move
        player.location = Location::Room(room1);
        player.move_to_room(room2);
        assert_eq!(player.previous_room(), Some(room1));
    }

    #[test]
    fn go_back_returns_to_previous_room() {
        use crate::Location;
        let mut player = Player::default();
        let room1 = Uuid::new_v4();
        let room2 = Uuid::new_v4();
        let room3 = Uuid::new_v4();

        // Set up history: room1 -> room2 -> room3
        player.location = Location::Room(room1);
        player.move_to_room(room2);
        player.move_to_room(room3);

        // Go back to room2
        assert_eq!(player.go_back(), Some(room2));
        assert_eq!(player.location, Location::Room(room2));
        assert_eq!(player.location_history, vec![room1]);

        // Go back to room1
        assert_eq!(player.go_back(), Some(room1));
        assert_eq!(player.location, Location::Room(room1));
        assert_eq!(player.location_history.len(), 0);

        // No more history
        assert_eq!(player.go_back(), None);
        assert_eq!(player.location, Location::Room(room1));
    }

    #[test]
    fn go_back_with_no_history_returns_none() {
        let mut player = Player::default();
        assert_eq!(player.go_back(), None);
    }
}
