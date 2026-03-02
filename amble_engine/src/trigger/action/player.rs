use crate::RoomId;
use anyhow::{Result, bail};
use log::{info, warn};

use crate::player::{Flag, Player};
use crate::view::{StatusAction, View, ViewItem};
use crate::world::{AmbleWorld, Location};

/// Resets a sequence flag back to its initial step (0).
pub fn reset_flag(player: &mut Player, flag_name: &str) {
    info!("└─ action: ResetFlag(\"{flag_name}\")");
    player.reset_flag(flag_name);
}

/// Advances a sequence flag to the next step in the sequence.
pub fn advance_flag(player: &mut Player, flag_name: &str) {
    info!("└─ action: AdvanceFlag(\"{flag_name}\")");
    player.advance_flag(flag_name);
}

/// Removes a flag from the player.
pub fn remove_flag(world: &mut AmbleWorld, view: &mut View, flag: &str) {
    remove_flag_with_priority(world, view, flag, None);
}

pub(super) fn remove_flag_with_priority(world: &mut AmbleWorld, view: &mut View, flag: &str, priority: Option<isize>) {
    let target = Flag::simple(flag, 0);
    if world.player.flags.remove(&target) {
        info!("└─ action: RemoveFlag(\"{flag}\")");
        if let Some(status) = flag.strip_prefix("status:") {
            view.push_with_custom_priority(
                ViewItem::StatusChange {
                    action: StatusAction::Remove,
                    status: status.to_string(),
                },
                priority,
            );
        }
    } else {
        warn!("└─ action: RemoveFlag(\"{flag}\") - flag was not set");
    }
}

/// Awards points to the player or penalizes them if the amount is negative.
pub fn award_points(world: &mut AmbleWorld, view: &mut View, amount: isize, reason: &str) {
    award_points_with_priority(world, view, amount, reason, None);
}

pub(super) fn award_points_with_priority(
    world: &mut AmbleWorld,
    view: &mut View,
    amount: isize,
    reason: &str,
    priority: Option<isize>,
) {
    world.player.score = world.player.score.saturating_add_signed(amount);
    info!("└─ action: AwardPoints({amount}, reason: {reason})");
    view.push_with_custom_priority(
        ViewItem::PointsAwarded {
            amount,
            reason: reason.to_string(),
        },
        priority,
    );
}

/// Adds a status flag to the player.
pub fn add_flag(world: &mut AmbleWorld, view: &mut View, flag: &Flag) {
    add_flag_with_priority(world, view, flag, None);
}

pub(super) fn add_flag_with_priority(world: &mut AmbleWorld, view: &mut View, flag: &Flag, priority: Option<isize>) {
    if let Some(status) = flag.name().strip_prefix("status:") {
        view.push_with_custom_priority(
            ViewItem::StatusChange {
                action: StatusAction::Apply,
                status: status.to_string(),
            },
            priority,
        );
    }
    world.player.flags.insert(flag.clone());
    info!("└─ action: AddFlag(\"{flag}\")");
}

/// Instantly moves the player to a different room.
///
/// # Errors
/// Returns an error if the destination room doesn't exist in the world.
pub fn push_player(world: &mut AmbleWorld, room_id: &RoomId) -> Result<()> {
    if world.rooms.contains_key(room_id) {
        world.player.location = Location::Room(room_id.clone());
        info!("└─ action: PushPlayerTo({room_id})");
        Ok(())
    } else {
        bail!("tried to push player to unknown room ({room_id})");
    }
}
