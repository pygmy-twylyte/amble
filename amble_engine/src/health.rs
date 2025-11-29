//! Health Module
//!
//! Handles health and health-related effects for living entities.
use std::cmp;

use log::info;
use serde::{Deserialize, Serialize};

use crate::{ViewItem, WorldObject};

/// Outcome of ticking queued health effects for an entity.
pub struct HealthTickResult {
    pub view_items: Vec<ViewItem>,
    pub death_cause: Option<String>,
}

/// Represents the state of a living entity's health and related effects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
        })
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
        // runny tally of character's hp as effects are processed
        let mut running_hp = self.current_hp;
        // list of decremented versions of over-time effects to retain for next turn
        let mut ongoing_fx = Vec::new();
        // an effect after processing -- None if one-off or expired, Some decremented version otherwise
        let mut updated_fx: Option<HealthEffect>;
        // collection of items to be pushed to the View by the caller
        let mut view_items = Vec::new();
        let mut death_cause: Option<String> = None;
        for fx in &self.effects {
            // apply the effect, keeping a running tally of the character's HP
            // and updating over-time effects (tick down)
            (running_hp, updated_fx) = fx.apply(running_hp, self.max_hp);
            // log and display each effect
            match &fx {
                HealthEffect::InstantDamage { cause, amount } => {
                    info!("{display_name} damaged by '{cause}' (-{amount} hp)");
                    view_items.push(ViewItem::CharacterHarmed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    })
                },
                HealthEffect::InstantHeal { cause, amount } => {
                    info!("{display_name} healed by '{cause}' (+{amount} hp)");
                    view_items.push(ViewItem::CharacterHealed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    })
                },
                HealthEffect::DamageOverTime { cause, amount, times } => {
                    info!(
                        "{display_name} damaged by '{cause}' d.o.t. (-{amount} hp, {} left)",
                        times - 1
                    );
                    view_items.push(ViewItem::CharacterHarmed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    })
                },
                HealthEffect::HealOverTime { cause, amount, times } => {
                    info!(
                        "{display_name} healed by '{cause}' h.o.t. (-{amount} hp, {} left)",
                        times - 1
                    );
                    view_items.push(ViewItem::CharacterHealed {
                        name: display_name.into(),
                        cause: cause.into(),
                        amount: *amount,
                    })
                },
            }
            // break out and return if character is dead!
            if running_hp == 0 {
                self.current_hp = 0;
                death_cause = Some(fx.cause_string());
                break;
            }
            if let Some(contd_fx) = updated_fx {
                ongoing_fx.push(contd_fx);
            }
        }
        self.current_hp = running_hp;
        self.effects = ongoing_fx;
        HealthTickResult {
            view_items,
            death_cause,
        }
    }
}

/// Abilities common to game entities that are alive
pub trait LivingEntity: WorldObject {
    fn max_hp(&self) -> u32;
    fn current_hp(&self) -> u32;
    fn damage(&mut self, amount: u32);
    fn heal(&mut self, amount: u32);
    fn life_state(&self) -> LifeState;
    fn add_health_effect(&mut self, effect: HealthEffect);
    fn remove_health_effect(&mut self, cause: &str) -> Option<HealthEffect>;
    fn tick_health_effects(&mut self) -> HealthTickResult;
}

/// Possible life states for living entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LifeState {
    Alive,
    Dead,
}

/// Types of health effects that can be applied to living game entities.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HealthEffect {
    InstantDamage { cause: String, amount: u32 },
    InstantHeal { cause: String, amount: u32 },
    DamageOverTime { cause: String, amount: u32, times: u32 },
    HealOverTime { cause: String, amount: u32, times: u32 },
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
    pub fn apply(&self, current_hp: u32, max_hp: u32) -> (u32, Option<HealthEffect>) {
        match self {
            Self::InstantDamage { amount, .. } => (current_hp.saturating_sub(*amount), None),
            Self::InstantHeal { amount, .. } => (cmp::min(max_hp, current_hp.saturating_add(*amount)), None),
            Self::DamageOverTime { cause, amount, times } => {
                let times_left: u32 = times.saturating_sub(1);
                let hp_left = current_hp.saturating_sub(*amount);
                let follow_up = if times_left > 0 {
                    Some(Self::DamageOverTime {
                        cause: cause.to_string(),
                        amount: *amount,
                        times: times_left,
                    })
                } else {
                    None
                };
                (hp_left, follow_up)
            },
            Self::HealOverTime { cause, amount, times } => {
                let times_left: u32 = times.saturating_sub(1);
                let healed_hp = cmp::min(current_hp.saturating_add(*amount), max_hp);
                let follow_up = if times_left > 0 {
                    Some(Self::HealOverTime {
                        cause: cause.to_string(),
                        amount: *amount,
                        times: times_left,
                    })
                } else {
                    None
                };
                (healed_hp, follow_up)
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

        let (hp, follow_up) = effect.apply(8, 10);
        assert_eq!(hp, 10);
        assert!(follow_up.is_none());
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
