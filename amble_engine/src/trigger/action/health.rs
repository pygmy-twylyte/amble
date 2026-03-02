use crate::health::{HealthEffect, LivingEntity};
use log::info;

/// Cause a specified amount of damage to a character, once, when the turn advances.
pub fn damage_character(target: &mut impl LivingEntity, cause: &str, amount: u32) {
    target.add_health_effect(HealthEffect::InstantDamage {
        cause: cause.to_string(),
        amount,
    });
}

/// Cause a specified amount of damage to a character each turn for a specified number of turns.
pub fn damage_character_ot(target: &mut impl LivingEntity, cause: &str, amount: u32, turns: u32) {
    target.add_health_effect(HealthEffect::DamageOverTime {
        cause: cause.into(),
        amount,
        times: turns,
    });
}

/// Heal a character a specified amount, once, on the next turn taken.
pub fn heal_character(target: &mut impl LivingEntity, cause: &str, amount: u32) {
    target.add_health_effect(HealthEffect::InstantHeal {
        cause: cause.to_string(),
        amount,
    });
}

/// Heal a character a certain amount each turn for a specified number of turns.
pub fn heal_character_ot(target: &mut impl LivingEntity, cause: &str, amount: u32, turns: u32) {
    target.add_health_effect(HealthEffect::HealOverTime {
        cause: cause.into(),
        amount,
        times: turns,
    });
}

/// Remove a queued health effect from a character by cause string.
pub fn remove_health_effect(target: &mut impl LivingEntity, cause: &str, target_label: &str) {
    let removed = target.remove_health_effect(cause);
    let name = target.name();
    match removed {
        Some(_) => info!("└─ action: removed health effect '{cause}' from {target_label} {name}"),
        None => info!("└─ action: no health effect '{cause}' found on {target_label} {name}"),
    }
}
