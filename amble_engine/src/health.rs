//! Health Module
//!
//! Handles health and health-related effects for living entities.
use std::cmp;

use log::info;
use serde::de::{self, Deserializer, EnumAccess, VariantAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::{ViewItem, WorldObject};

/// Outcome of ticking queued health effects for an entity.
pub struct HealthTickResult {
    pub view_items: Vec<ViewItem>,
    pub death_cause: Option<String>,
}

/// Represents the state of a living entity's health and related effects.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct HealthState {
    max_hp: u32,
    current_hp: u32,
    pub(crate) effects: Vec<HealthEffect>,
}
impl HealthState {
    /// Creates a new, empty `HealthState`
    pub fn new() -> HealthState {
        HealthState::default()
    }

    /// Create a clean `HealthState` with specified maximum health
    pub fn new_at_max(max_hp: u32) -> HealthState {
        HealthState {
            max_hp,
            current_hp: max_hp,
            effects: Vec::new(),
        }
    }

    /// Get the maximum HP for this entity
    pub fn max_hp(&self) -> u32 {
        self.max_hp
    }

    /// Get the current HP for this entity
    pub fn current_hp(&self) -> u32 {
        self.current_hp
    }

    /// Return whether this entity is alive or dead.
    /// In the future, there may be additional states -- so not using a boolean here.
    pub fn life_state(&self) -> LifeState {
        if self.current_hp > 0 {
            LifeState::Alive
        } else {
            LifeState::Dead
        }
    }

    /// Do damage to health. Saturates at zero.
    pub fn damage(&mut self, amount: u32) {
        self.current_hp = self.current_hp.saturating_sub(amount);
    }

    /// Heal the character. Saturates at max health.
    pub fn heal(&mut self, amount: u32) {
        self.current_hp = cmp::min(self.max_hp, self.current_hp.saturating_add(amount));
    }

    /// Add a `HealthEffect` to the queue
    pub fn add_effect(&mut self, fx: HealthEffect) {
        self.effects.push(fx);
    }

    /// Add a damage over time effect.
    pub fn add_dot_effect(&mut self, cause: &str, damage: u32, times: u32) {
        self.effects.push(HealthEffect::DamageOverTime {
            cause: cause.to_string(),
            amount: damage,
            times,
        });
    }
    /// Take an effect out of the queue
    pub fn remove_effect(&mut self, cause: &str) -> Option<HealthEffect> {
        if let Some(idx) = self.effects.iter().position(|fx| fx.cause_matches(cause)) {
            Some(self.effects.remove(idx))
        } else {
            None
        }
    }

    /// Iterate through pending health effects, applying each one.
    pub fn apply_effects(&mut self, display_name: &str) -> HealthTickResult {
        let mut hp_tally = self.current_hp;
        let mut unexpired_effects = Vec::new();
        let mut health_messages = Vec::new();
        let mut death_cause: Option<String> = None;

        for fx in &self.effects {
            // log and display each effect
            match &fx {
                HealthEffect::InstantDamage { cause, amount } => {
                    info!("{display_name} damaged by '{cause}' (-{amount} hp)");
                    health_messages.push(ViewItem::CharacterHarmed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    });
                },
                HealthEffect::InstantHeal { cause, amount } => {
                    info!("{display_name} healed by '{cause}' (+{amount} hp)");
                    health_messages.push(ViewItem::CharacterHealed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    });
                },
                HealthEffect::DamageOverTime { cause, amount, times } => {
                    info!(
                        "{display_name} damaged by '{cause}' d.o.t. (-{amount} hp, {} left)",
                        times - 1
                    );
                    health_messages.push(ViewItem::CharacterHarmed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    });
                },
                HealthEffect::HealOverTime { cause, amount, times } => {
                    info!(
                        "{display_name} healed by '{cause}' h.o.t. (-{amount} hp, {} left)",
                        times - 1
                    );
                    health_messages.push(ViewItem::CharacterHealed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    });
                },
            }
            let effect_outcome = fx.apply(hp_tally, self.max_hp);
            hp_tally = effect_outcome.remaining_hp;
            // break out and return if character is dead!
            if effect_outcome.remaining_hp == 0 {
                self.current_hp = 0;
                death_cause = Some(fx.cause_string());
                break;
            }
            if let Some(effect) = effect_outcome.residual_effect {
                unexpired_effects.push(effect);
            }
        }
        self.current_hp = hp_tally;
        self.effects = unexpired_effects;
        HealthTickResult {
            view_items: health_messages,
            death_cause,
        }
    }
}

/// Holds the result of one 'tick' of a particular health effect.
pub struct EffectResult {
    remaining_hp: u32,
    residual_effect: Option<HealthEffect>,
}
impl From<(u32, Option<HealthEffect>)> for EffectResult {
    fn from(value: (u32, Option<HealthEffect>)) -> Self {
        EffectResult {
            remaining_hp: value.0,
            residual_effect: value.1,
        }
    }
}

/// Functionality common to all game entities that are alive
pub trait LivingEntity: WorldObject {
    /// The amount of health this entity has when fully healed.
    fn max_hp(&self) -> u32;

    /// The current level of this entity's health.
    fn current_hp(&self) -> u32;

    /// Reduce this entity's health by `amount`.
    fn damage(&mut self, amount: u32);

    /// Increase this entity's health by `amount` (up to defined maximum)
    fn heal(&mut self, amount: u32);

    /// Determine the life status of this entity.
    ///
    /// For now, the means `Alive` or `Dead` but could later include other
    /// variants like "Revived" or "Stasis" or "Undead".
    fn life_state(&self) -> LifeState;

    /// Add a health effect to start processing on the next turn.
    fn add_health_effect(&mut self, effect: HealthEffect);

    /// Remove a pending health effect from the queue.
    fn remove_health_effect(&mut self, cause: &str) -> Option<HealthEffect>;

    /// Apply and update queued health effects when advancing a turn.
    #[must_use] /* life or death notifications may be returned! */
    fn tick_health_effects(&mut self) -> HealthTickResult;
}

/// Possible life states for living entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LifeState {
    Alive,
    Dead,
}

/// Types of health effects that can be applied to living game entities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HealthEffect {
    InstantDamage { cause: String, amount: u32 },
    InstantHeal { cause: String, amount: u32 },
    DamageOverTime { cause: String, amount: u32, times: u32 },
    HealOverTime { cause: String, amount: u32, times: u32 },
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum HealthEffectKind {
    InstantDamage,
    InstantHeal,
    DamageOverTime,
    HealOverTime,
}

impl HealthEffectKind {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            v if v.eq_ignore_ascii_case("instantDamage") => Some(Self::InstantDamage),
            v if v.eq_ignore_ascii_case("instantHeal") => Some(Self::InstantHeal),
            v if v.eq_ignore_ascii_case("damageOverTime") => Some(Self::DamageOverTime),
            v if v.eq_ignore_ascii_case("healOverTime") => Some(Self::HealOverTime),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for HealthEffectKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = HealthEffectKind;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("health effect type identifier")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                HealthEffectKind::from_str(value).ok_or_else(|| {
                    de::Error::unknown_variant(
                        value,
                        &["instantDamage", "instantHeal", "damageOverTime", "healOverTime"],
                    )
                })
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

        deserializer.deserialize_any(KindVisitor)
    }
}

#[derive(Deserialize)]
struct HealthEffectRepr {
    #[serde(rename = "type")]
    kind: HealthEffectKind,
    cause: String,
    #[serde(default)]
    amount: u32,
    #[serde(default)]
    times: u32,
}

impl<'de> Deserialize<'de> for HealthEffect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let repr = HealthEffectRepr::deserialize(deserializer)?;
        Ok(match repr.kind {
            HealthEffectKind::InstantDamage => HealthEffect::InstantDamage {
                cause: repr.cause,
                amount: repr.amount,
            },
            HealthEffectKind::InstantHeal => HealthEffect::InstantHeal {
                cause: repr.cause,
                amount: repr.amount,
            },
            HealthEffectKind::DamageOverTime => HealthEffect::DamageOverTime {
                cause: repr.cause,
                amount: repr.amount,
                times: repr.times,
            },
            HealthEffectKind::HealOverTime => HealthEffect::HealOverTime {
                cause: repr.cause,
                amount: repr.amount,
                times: repr.times,
            },
        })
    }
}

impl HealthEffect {
    /// Returns `true` if the `cause` matches the supplied string
    pub fn cause_matches(&self, pattern: &str) -> bool {
        match &self {
            Self::DamageOverTime { cause, .. }
            | Self::HealOverTime { cause, .. }
            | Self::InstantDamage { cause, .. }
            | Self::InstantHeal { cause, .. } => cause == pattern,
        }
    }

    /// Returns a string describing what caused the health change (e.g. "drank potion", "played with fire")
    pub fn cause_string(&self) -> String {
        match &self {
            Self::DamageOverTime { cause, .. }
            | Self::HealOverTime { cause, .. }
            | Self::InstantDamage { cause, .. }
            | Self::InstantHeal { cause, .. } => cause.clone(),
        }
    }

    /// Applies this effect to the supplied `HealthState`
    ///
    /// The current and max hp are passed in. The result is a tuple containing updated hp
    /// after processing the effect, and an optional follow up effect (if any) for
    /// over-time effects.
    pub fn apply(&self, current_hp: u32, max_hp: u32) -> EffectResult {
        match self {
            Self::InstantDamage { amount, .. } => (current_hp.saturating_sub(*amount), None).into(),
            Self::InstantHeal { amount, .. } => (cmp::min(max_hp, current_hp.saturating_add(*amount)), None).into(),
            Self::DamageOverTime { cause, amount, times } => {
                let times_left: u32 = times.saturating_sub(1);
                let hp_left = current_hp.saturating_sub(*amount);
                let follow_up = if times_left > 0 {
                    Some(Self::DamageOverTime {
                        cause: cause.clone(),
                        amount: *amount,
                        times: times_left,
                    })
                } else {
                    None
                };
                (hp_left, follow_up).into()
            },
            Self::HealOverTime { cause, amount, times } => {
                let times_left: u32 = times.saturating_sub(1);
                let healed_hp = cmp::min(current_hp.saturating_add(*amount), max_hp);
                let follow_up = if times_left > 0 {
                    Some(Self::HealOverTime {
                        cause: cause.clone(),
                        amount: *amount,
                        times: times_left,
                    })
                } else {
                    None
                };
                (healed_hp, follow_up).into()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heal_saturates_at_max_hp() {
        let mut state = HealthState::new_at_max(10);
        state.damage(5);
        state.heal(3);
        assert_eq!(state.current_hp, 8);

        state.heal(10);
        assert_eq!(state.current_hp, 10);
    }

    #[test]
    fn instant_heal_effect_respects_max_hp() {
        let effect = HealthEffect::InstantHeal {
            cause: "potion".into(),
            amount: 5,
        };

        let outcome = effect.apply(8, 10);
        assert_eq!(outcome.remaining_hp, 10);
        assert!(outcome.residual_effect.is_none());
    }

    #[test]
    fn apply_effects_enqueues_follow_up_for_overtime_healing() {
        let mut state = HealthState {
            max_hp: 10,
            current_hp: 6,
            effects: vec![HealthEffect::HealOverTime {
                cause: "campfire".into(),
                amount: 3,
                times: 2,
            }],
        };

        state.apply_effects("test");
        assert_eq!(state.current_hp, 9);
        assert_eq!(state.effects.len(), 1);

        match &state.effects[0] {
            HealthEffect::HealOverTime { amount, times, .. } => {
                assert_eq!(*amount, 3);
                assert_eq!(*times, 1);
            },
            unexpected => panic!("unexpected effect remaining: {unexpected:?}"),
        }

        state.apply_effects("test");
        assert_eq!(state.current_hp, 10);
        assert!(state.effects.is_empty());
    }

    #[test]
    fn lethal_effects_stop_processing_remaining_queue() {
        let mut state = HealthState {
            max_hp: 10,
            current_hp: 4,
            effects: vec![
                HealthEffect::DamageOverTime {
                    cause: "poison cloud".into(),
                    amount: 5,
                    times: 1,
                },
                HealthEffect::InstantHeal {
                    cause: "healing potion".into(),
                    amount: 5,
                },
            ],
        };

        state.apply_effects("test");
        assert_eq!(state.current_hp, 0);
        assert!(state.effects.is_empty());
    }
}
