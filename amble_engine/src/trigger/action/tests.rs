use super::*;
use crate::{
    health::{HealthEffect, HealthState, LivingEntity},
    item::{ContainerState, Item},
    npc::{MovementTiming, MovementType, Npc, NpcMovement, NpcState},
    player::Flag,
    room::{Exit, Room},
    view::{View, ViewItem},
    world::{AmbleWorld, Location},
};
use gametools::{Spinner, Wedge};
use std::collections::{HashMap, HashSet};

fn build_test_world() -> (AmbleWorld, RoomId, RoomId) {
    let mut world = AmbleWorld::new_empty();
    let room1_id = crate::idgen::new_room_id();
    let room2_id = crate::idgen::new_room_id();

    let room1 = Room {
        id: room1_id.clone(),
        symbol: "r1".into(),
        name: "Room1".into(),
        base_description: "Room1".into(),
        overlays: Vec::new(),
        scenery: Vec::new(),
        scenery_default: None,
        location: Location::Nowhere,
        visited: false,
        exits: HashMap::new(),
        contents: HashSet::new(),
        npcs: HashSet::new(),
    };
    let room2 = Room {
        id: room2_id.clone(),
        symbol: "r2".into(),
        name: "Room2".into(),
        base_description: "Room2".into(),
        overlays: Vec::new(),
        scenery: Vec::new(),
        scenery_default: None,
        location: Location::Nowhere,
        visited: false,
        exits: HashMap::new(),
        contents: HashSet::new(),
        npcs: HashSet::new(),
    };
    world.rooms.insert(room1_id.clone(), room1);
    world.rooms.insert(room2_id.clone(), room2);
    world.player.location = Location::Room(room1_id.clone());
    (world, room1_id, room2_id)
}

fn make_item(id: ItemId, location: Location, container_state: Option<ContainerState>) -> Item {
    Item {
        id,
        symbol: "it".into(),
        name: "Item".into(),
        description: String::new(),
        location,
        visibility: crate::item::ItemVisibility::Listed,
        visible_when: None,
        aliases: Vec::new(),
        movability: crate::item::Movability::Free,
        container_state,
        contents: HashSet::new(),
        abilities: HashSet::new(),
        interaction_requires: HashMap::new(),
        text: None,
        consumable: None,
    }
}

fn make_npc(id: NpcId, location: Location, state: NpcState) -> Npc {
    Npc {
        id,
        symbol: "n".into(),
        name: "Npc".into(),
        description: String::new(),
        location,
        inventory: HashSet::new(),
        dialogue: HashMap::new(),
        state,
        movement: None,
        health: HealthState::new_at_max(10),
    }
}

#[test]
fn modify_room_updates_name_desc_and_exits() {
    let mut world = AmbleWorld::new_empty();
    let room_id = crate::idgen::new_room_id();
    let dest_id = crate::idgen::new_room_id();
    let target_id = crate::idgen::new_room_id();
    world.rooms.insert(
        room_id.clone(),
        Room {
            id: room_id.clone(),
            symbol: "lab".into(),
            name: "Lab".into(),
            base_description: "Original description".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::from([("north".into(), Exit::new(dest_id.clone()))]),
            contents: HashSet::default(),
            npcs: HashSet::default(),
        },
    );
    world.rooms.insert(
        dest_id.clone(),
        Room {
            id: dest_id.clone(),
            symbol: "hall".into(),
            name: "Hall".into(),
            base_description: "Hall".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::default(),
            npcs: HashSet::default(),
        },
    );
    world.rooms.insert(
        target_id.clone(),
        Room {
            id: target_id.clone(),
            symbol: "vault".into(),
            name: "Vault".into(),
            base_description: "Vault".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::default(),
            npcs: HashSet::default(),
        },
    );

    let patch = RoomPatch {
        name: Some("Ruined Lab".into()),
        desc: Some("Destroyed lab".into()),
        remove_exits: vec![dest_id.clone()],
        add_exits: vec![RoomExitPatch {
            direction: "through the vault door".into(),
            to: target_id.clone(),
            hidden: false,
            locked: true,
            required_flags: HashSet::from([Flag::simple("opened-vault", 0)]),
            required_items: HashSet::new(),
            barred_message: Some("You can't go that way yet.".into()),
        }],
    };

    modify_room(&mut world, &room_id, &patch).expect("modify room");
    let room = world.rooms.get(&room_id).expect("room present");
    assert_eq!(room.name, "Ruined Lab");
    assert_eq!(room.base_description, "Destroyed lab");
    assert!(!room.exits.contains_key("north"));
    let exit = room.exits.get("through the vault door").expect("new exit present");
    assert_eq!(exit.to, target_id);
    assert!(exit.locked);
    assert_eq!(exit.barred_message.as_deref(), Some("You can't go that way yet."));
    assert!(exit.required_flags.contains(&Flag::simple("opened-vault", 0)));
}

#[test]
fn modify_room_missing_exit_ok() {
    let mut world = AmbleWorld::new_empty();
    let room_id = crate::idgen::new_room_id();
    world.rooms.insert(
        room_id.clone(),
        Room {
            id: room_id.clone(),
            symbol: "lab".into(),
            name: "Lab".into(),
            base_description: "Original description".into(),
            overlays: Vec::new(),
            scenery: Vec::new(),
            scenery_default: None,
            location: Location::Nowhere,
            visited: false,
            exits: HashMap::new(),
            contents: HashSet::default(),
            npcs: HashSet::default(),
        },
    );
    let missing_exit_target = crate::idgen::new_room_id();
    let patch = RoomPatch {
        remove_exits: vec![missing_exit_target],
        ..Default::default()
    };
    assert!(modify_room(&mut world, &room_id, &patch).is_ok());
}

#[test]
fn modify_npc_updates_identity_and_dialogue() {
    let (mut world, room_id, _) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let mut npc = make_npc(npc_id.clone(), Location::Room(room_id), NpcState::Normal);
    npc.dialogue.insert(NpcState::Normal, vec!["Hello there.".into()]);
    world.npcs.insert(npc_id.clone(), npc);

    let patch = NpcPatch {
        name: Some("Professor Whistles".into()),
        desc: Some("An eccentric inventor with wild hair.".into()),
        state: Some(NpcState::Happy),
        add_lines: vec![NpcDialoguePatch {
            state: NpcState::Happy,
            line: "Have you met my clockwork ferret?".into(),
        }],
        movement: None,
    };

    modify_npc(&mut world, &npc_id, &patch).expect("modify npc succeeds");

    let npc = world.npcs.get(&npc_id).expect("npc present");
    assert_eq!(npc.name, "Professor Whistles");
    assert_eq!(npc.description, "An eccentric inventor with wild hair.");
    assert!(matches!(npc.state, NpcState::Happy));
    let happy_lines = npc.dialogue.get(&NpcState::Happy).expect("happy dialogue");
    assert!(
        happy_lines
            .iter()
            .any(|line| line == "Have you met my clockwork ferret?")
    );
}

#[test]
fn modify_npc_updates_movement_and_flags() {
    let (mut world, room_a, room_b) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let mut npc = make_npc(npc_id.clone(), Location::Room(room_a.clone()), NpcState::Normal);
    npc.movement = Some(NpcMovement {
        movement_type: MovementType::Route {
            rooms: vec![room_a.clone()],
            current_idx: 0,
            loop_route: true,
        },
        timing: MovementTiming::EveryNTurns { turns: 2 },
        active: true,
        last_moved_turn: 0,
        paused_until: None,
    });
    world.npcs.insert(npc_id.clone(), npc);

    let patch = NpcPatch {
        movement: Some(NpcMovementPatch {
            route: Some(vec![room_a.clone(), room_b.clone()]),
            random_rooms: None,
            timing: Some(MovementTiming::EveryNTurns { turns: 5 }),
            active: Some(false),
            loop_route: Some(false),
        }),
        ..Default::default()
    };

    modify_npc(&mut world, &npc_id, &patch).expect("modify npc succeeds");

    let npc = world.npcs.get(&npc_id).expect("npc present");
    let movement = npc.movement.as_ref().expect("movement present");
    match &movement.movement_type {
        MovementType::Route {
            rooms,
            current_idx,
            loop_route,
        } => {
            assert_eq!(rooms, &vec![room_a, room_b]);
            assert_eq!(*current_idx, 0);
            assert!(!loop_route);
        },
        other @ MovementType::RandomSet { .. } => panic!("expected route movement, got {other:?}"),
    }
    assert!(matches!(movement.timing, MovementTiming::EveryNTurns { turns: 5 }));
    assert!(!movement.active);
    assert_eq!(movement.last_moved_turn, world.turn_count);
}

#[test]
fn modify_npc_creates_movement_if_missing() {
    let (mut world, room_id, other_room) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let npc = make_npc(npc_id.clone(), Location::Room(room_id.clone()), NpcState::Normal);
    world.npcs.insert(npc_id.clone(), npc);

    let patch = NpcPatch {
        movement: Some(NpcMovementPatch {
            route: Some(vec![room_id.clone(), other_room.clone()]),
            random_rooms: None,
            timing: Some(MovementTiming::OnTurn { turn: 7 }),
            active: Some(true),
            loop_route: Some(true),
        }),
        ..Default::default()
    };

    modify_npc(&mut world, &npc_id, &patch).expect("modify npc succeeds");

    let npc = world.npcs.get(&npc_id).expect("npc present");
    let movement = npc.movement.as_ref().expect("movement present");
    assert!(matches!(movement.movement_type, MovementType::Route { .. }));
    assert!(matches!(movement.timing, MovementTiming::OnTurn { turn: 7 }));
    assert!(movement.active);
}

#[test]
fn damage_player_action_applies_on_next_tick() {
    let mut world = AmbleWorld::new_empty();
    world.player.name = "Tester".into();
    world.player.health = HealthState::new_at_max(10);
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::DamagePlayer {
        cause: "trap".into(),
        amount: 3,
    });
    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert_eq!(world.player.current_hp(), 10);

    let applied = world.player.tick_health_effects();
    assert_eq!(world.player.current_hp(), 7);
    assert!(applied.view_items.contains(&ViewItem::CharacterHarmed {
        name: "Tester".into(),
        cause: "trap".into(),
        amount: 3,
    }));
}

#[test]
fn heal_player_over_time_saturates_and_expires() {
    let mut world = AmbleWorld::new_empty();
    world.player.name = "Tester".into();
    world.player.health = HealthState::new_at_max(10);
    world.player.damage(5);
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::HealPlayerOT {
        cause: "regen".into(),
        amount: 3,
        turns: 2,
    });
    dispatch_action(&mut world, &mut view, &action).unwrap();
    assert_eq!(world.player.current_hp(), 5);

    let first_tick = world.player.tick_health_effects();
    assert_eq!(world.player.current_hp(), 8);
    assert_eq!(world.player.health.effects.len(), 1);
    assert!(first_tick.view_items.contains(&ViewItem::CharacterHealed {
        name: "Tester".into(),
        cause: "regen".into(),
        amount: 3,
    }));

    world.player.damage(1);
    let second_tick = world.player.tick_health_effects();
    assert_eq!(world.player.current_hp(), 10);
    assert!(world.player.health.effects.is_empty());
    assert!(second_tick.view_items.contains(&ViewItem::CharacterHealed {
        name: "Tester".into(),
        cause: "regen".into(),
        amount: 3,
    }));
}

#[test]
fn damage_npc_over_time_ticks_each_turn() {
    let (mut world, room_id, _) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    world.npcs.insert(
        npc_id.clone(),
        make_npc(npc_id.clone(), Location::Room(room_id.clone()), NpcState::Normal),
    );
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::DamageNpcOT {
        npc_id: npc_id.clone(),
        cause: "acid".into(),
        amount: 2,
        turns: 2,
    });
    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert_eq!(world.npcs.get(&npc_id).unwrap().current_hp(), 10);

    let first_tick = world.npcs.get_mut(&npc_id).unwrap().tick_health_effects();
    assert_eq!(world.npcs.get(&npc_id).unwrap().current_hp(), 8);
    assert_eq!(world.npcs.get(&npc_id).unwrap().health.effects.len(), 1);
    assert!(first_tick.view_items.iter().any(|item| {
        matches!(
            item,
            ViewItem::CharacterHarmed { cause, amount, .. }
            if cause == "acid" && *amount == 2
        )
    }));

    let second_tick = world.npcs.get_mut(&npc_id).unwrap().tick_health_effects();
    assert_eq!(world.npcs.get(&npc_id).unwrap().current_hp(), 6);
    assert!(world.npcs.get(&npc_id).unwrap().health.effects.is_empty());
    assert!(second_tick.view_items.iter().any(|item| {
        matches!(
            item,
            ViewItem::CharacterHarmed { cause, amount, .. }
            if cause == "acid" && *amount == 2
        )
    }));
}

#[test]
fn remove_player_effect_action_removes_matching_effect() {
    let mut world = AmbleWorld::new_empty();
    world.player.name = "Tester".into();
    world.player.health = HealthState::new_at_max(10);
    world.player.health.effects = vec![
        HealthEffect::DamageOverTime {
            cause: "poison".into(),
            amount: 1,
            times: 3,
        },
        HealthEffect::InstantHeal {
            cause: "bandage".into(),
            amount: 1,
        },
    ];
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::RemovePlayerEffect { cause: "poison".into() });
    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert_eq!(world.player.health.effects.len(), 1);
    assert!(matches!(
        world.player.health.effects[0],
        HealthEffect::InstantHeal { .. }
    ));
}

#[test]
fn remove_npc_effect_action_clears_effect_and_is_idempotent() {
    let (mut world, room_id, _) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let mut npc = make_npc(npc_id.clone(), Location::Room(room_id.clone()), NpcState::Normal);
    npc.health.effects.push(HealthEffect::DamageOverTime {
        cause: "burn".into(),
        amount: 2,
        times: 2,
    });
    world.npcs.insert(npc_id.clone(), npc);
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::RemoveNpcEffect {
        npc_id: npc_id.clone(),
        cause: "burn".into(),
    });
    dispatch_action(&mut world, &mut view, &action).unwrap();
    assert!(world.npcs.get(&npc_id).unwrap().health.effects.is_empty());

    let repeat = ScriptedAction::new(TriggerAction::RemoveNpcEffect {
        npc_id: npc_id.clone(),
        cause: "burn".into(),
    });
    dispatch_action(&mut world, &mut view, &repeat).unwrap();
    assert!(world.npcs.get(&npc_id).unwrap().health.effects.is_empty());
}

#[test]
fn push_player_moves_player_to_room() {
    let (mut world, _start, dest) = build_test_world();
    assert!(push_player(&mut world, &dest).is_ok());
    assert_eq!(world.player.location, Location::Room(dest));
}

#[test]
fn push_player_errors_with_invalid_room() {
    let (mut world, _, _) = build_test_world();
    let bad_room = crate::idgen::new_room_id();
    assert!(push_player(&mut world, &bad_room).is_err());
}

#[test]
fn add_and_remove_flag_updates_player_flags() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();
    let flag = Flag::simple("test", 0);
    add_flag(&mut world, &mut view, &flag);
    assert!(world.player.flags.contains(&flag));
    remove_flag(&mut world, &mut view, "test");
    assert!(!world.player.flags.contains(&flag));
}

#[test]
fn reset_and_advance_flag_modifies_sequence() {
    let (mut world, _, _) = build_test_world();
    let flag = Flag::sequence("quest", Some(2), 0);
    world.player.flags.insert(flag);
    advance_flag(&mut world.player, "quest");
    assert!(
        world
            .player
            .flags
            .iter()
            .any(|f| matches!(f, Flag::Sequence { name, step, .. } if name == "quest" && *step == 1))
    );
    reset_flag(&mut world.player, "quest");
    assert!(
        world
            .player
            .flags
            .iter()
            .any(|f| matches!(f, Flag::Sequence { name, step, .. } if name == "quest" && *step == 0))
    );
}

#[test]
fn award_points_modifies_player_score() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();
    award_points(&mut world, &mut view, 5, "test gain");
    assert_eq!(world.player.score, 6);
    award_points(&mut world, &mut view, -3, "test loss");
    assert_eq!(world.player.score, 3);
}

#[test]
fn lock_and_unlock_item_changes_state() {
    let (mut world, room_id, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let item = make_item(
        item_id.clone(),
        Location::Room(room_id.clone()),
        Some(ContainerState::Open),
    );
    world.items.insert(item_id.clone(), item);
    lock_item(&mut world, &item_id).unwrap();
    assert_eq!(
        world.items.get(&item_id).unwrap().container_state,
        Some(ContainerState::Locked)
    );
    unlock_item(&mut world, &item_id).unwrap();
    assert_eq!(
        world.items.get(&item_id).unwrap().container_state,
        Some(ContainerState::Open)
    );
}

#[test]
fn lock_and_unlock_exit_changes_state() {
    let (mut world, room1_id, room2_id) = build_test_world();
    world
        .rooms
        .get_mut(&room1_id)
        .unwrap()
        .exits
        .insert("north".into(), Exit::new(room2_id));
    lock_exit(&mut world, &room1_id, &"north".into()).unwrap();
    assert!(world.rooms[&room1_id].exits["north"].locked);
    unlock_exit(&mut world, &room1_id, &"north".into()).unwrap();
    assert!(!world.rooms[&room1_id].exits["north"].locked);
}

#[test]
fn modify_item_updates_scalar_fields() {
    let (mut world, room_id, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let mut item = make_item(
        item_id.clone(),
        Location::Room(room_id.clone()),
        Some(ContainerState::Closed),
    );
    item.text = Some("old text".to_string());
    world.items.insert(item_id.clone(), item);

    let patch = ItemPatch {
        name: Some("Renamed Widget".to_string()),
        desc: Some("Updated description".to_string()),
        text: Some("new dynamic text".to_string()),
        movability: Some(crate::item::Movability::Fixed {
            reason: "It is nailed to the floor.".to_string(),
        }),
        container_state: Some(ContainerState::Open),
        ..Default::default()
    };

    modify_item(&mut world, item_id.clone(), &patch).unwrap();

    let updated = world.items.get(&item_id).unwrap();
    assert_eq!(updated.name, "Renamed Widget");
    assert_eq!(updated.description, "Updated description");
    assert_eq!(updated.text.as_deref(), Some("new dynamic text"));
    assert_eq!(updated.container_state, Some(ContainerState::Open));
    assert_eq!(
        updated.movability,
        crate::item::Movability::Fixed {
            reason: "It is nailed to the floor.".to_string(),
        }
    );
}

#[test]
fn modify_item_leaves_container_state_when_unset() {
    let (mut world, room_id, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let item = make_item(
        item_id.clone(),
        Location::Room(room_id.clone()),
        Some(ContainerState::Locked),
    );
    world.items.insert(item_id.clone(), item);

    let patch = ItemPatch {
        name: Some("Still Locked".to_string()),
        ..Default::default()
    };

    modify_item(&mut world, item_id.clone(), &patch).unwrap();

    let updated = world.items.get(&item_id).unwrap();
    assert_eq!(updated.container_state, Some(ContainerState::Locked));
}

#[test]
fn modify_item_updates_abilities() {
    let (mut world, room_id, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let mut item = make_item(item_id.clone(), Location::Room(room_id.clone()), None);
    item.abilities.insert(crate::item::ItemAbility::Ignite);
    world.items.insert(item_id.clone(), item);

    let patch = ItemPatch {
        add_abilities: vec![crate::item::ItemAbility::Read],
        remove_abilities: vec![crate::item::ItemAbility::Ignite],
        ..Default::default()
    };

    modify_item(&mut world, item_id.clone(), &patch).unwrap();

    let updated = world.items.get(&item_id).unwrap();
    assert!(updated.abilities.contains(&crate::item::ItemAbility::Read));
    assert!(!updated.abilities.contains(&crate::item::ItemAbility::Ignite));
}

#[test]
fn modify_item_removes_container_state_when_empty() {
    let (mut world, room_id, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let container = make_item(
        item_id.clone(),
        Location::Room(room_id.clone()),
        Some(ContainerState::Open),
    );
    world.items.insert(item_id.clone(), container);

    let patch = ItemPatch {
        remove_container_state: true,
        ..Default::default()
    };

    modify_item(&mut world, item_id.clone(), &patch).unwrap();

    let updated = world.items.get(&item_id).unwrap();
    assert_eq!(updated.container_state, None);
}

#[test]
fn modify_item_errors_when_removing_container_state_with_contents() {
    let (mut world, room_id, _) = build_test_world();
    let container_id: ItemId = crate::idgen::new_id().into();
    let container = make_item(
        container_id.clone(),
        Location::Room(room_id.clone()),
        Some(ContainerState::Open),
    );
    world.items.insert(container_id.clone(), container);

    let child_id: ItemId = crate::idgen::new_id().into();
    let child_item = make_item(child_id.clone(), Location::Item(container_id.clone()), None);
    world.items.insert(child_id.clone(), child_item);
    world
        .items
        .get_mut(&container_id)
        .unwrap()
        .contents
        .insert(child_id.clone());

    let patch = ItemPatch {
        remove_container_state: true,
        ..Default::default()
    };

    let result = modify_item(&mut world, container_id.clone(), &patch);
    assert!(result.is_ok());
    assert_eq!(
        world.items.get(&container_id).unwrap().container_state,
        Some(ContainerState::Open)
    );
}

#[test]
fn spawn_item_in_specific_room_places_item() {
    let (mut world, _room1, room2) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let item = make_item(item_id.clone(), Location::Nowhere, None);
    world.items.insert(item_id.clone(), item);
    spawn_item_in_specific_room(&mut world, &item_id, &room2).unwrap();
    assert_eq!(world.items[&item_id].location, Location::Room(room2.clone()));
    assert!(world.rooms[&room2].contents.contains(&item_id));
}

#[test]
fn spawn_item_in_current_room_places_item() {
    let (mut world, room1, _room2) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    world
        .items
        .insert(item_id.clone(), make_item(item_id.clone(), Location::Nowhere, None));
    spawn_item_in_current_room(&mut world, &item_id).unwrap();
    assert_eq!(world.items[&item_id].location, Location::Room(room1.clone()));
    assert!(world.rooms[&room1].contents.contains(&item_id));
}

#[test]
fn spawn_item_in_inventory_adds_to_player_and_removes_restriction() {
    let (mut world, _, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    let mut item = make_item(item_id.clone(), Location::Nowhere, None);
    item.movability = crate::item::Movability::Restricted {
        reason: "some reason".to_string(),
    };
    world.items.insert(item_id.clone(), item);
    spawn_item_in_inventory(&mut world, &item_id).unwrap();
    assert_eq!(world.items[&item_id].location, Location::Inventory);
    assert!(world.player.inventory.contains(&item_id));
    assert!(matches!(
        world.items[&item_id].movability,
        crate::item::Movability::Free
    ));
}

#[test]
fn spawn_item_in_container_places_item_inside() {
    let (mut world, room1, _) = build_test_world();
    let container_id: ItemId = crate::idgen::new_id().into();
    let container = make_item(
        container_id.clone(),
        Location::Room(room1.clone()),
        Some(ContainerState::Open),
    );
    world.items.insert(container_id.clone(), container);
    world
        .rooms
        .get_mut(&room1)
        .unwrap()
        .contents
        .insert(container_id.clone());
    let item_id: ItemId = crate::idgen::new_id().into();
    world
        .items
        .insert(item_id.clone(), make_item(item_id.clone(), Location::Nowhere, None));
    spawn_item_in_container(&mut world, &item_id, &container_id).unwrap();
    assert_eq!(world.items[&item_id].location, Location::Item(container_id.clone()));
    assert!(world.items[&container_id].contents.contains(&item_id));
}

#[test]
fn despawn_item_removes_item_from_world() {
    let (mut world, room1, _) = build_test_world();
    let item_id: ItemId = crate::idgen::new_id().into();
    world.items.insert(
        item_id.clone(),
        make_item(item_id.clone(), Location::Room(room1.clone()), None),
    );
    world.rooms.get_mut(&room1).unwrap().contents.insert(item_id.clone());
    despawn_item(&mut world, &item_id).unwrap();
    assert_eq!(world.items[&item_id].location, Location::Nowhere);
    assert!(!world.rooms[&room1].contents.contains(&item_id));
}

#[test]
fn give_to_player_transfers_item_from_npc() {
    let (mut world, room1, _) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let npc = make_npc(npc_id.clone(), Location::Room(room1.clone()), NpcState::Normal);
    world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id.clone());
    world.npcs.insert(npc_id.clone(), npc);
    let item_id: ItemId = crate::idgen::new_id().into();
    world.items.insert(
        item_id.clone(),
        make_item(item_id.clone(), Location::Npc(npc_id.clone()), None),
    );
    world.npcs.get_mut(&npc_id).unwrap().inventory.insert(item_id.clone());
    give_to_player(&mut world, &npc_id, &item_id).unwrap();
    assert_eq!(world.items[&item_id].location, Location::Inventory);
    assert!(world.player.inventory.contains(&item_id));
    assert!(!world.npcs[&npc_id].inventory.contains(&item_id));
}

#[test]
fn schedule_in_adds_event_to_scheduler() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();
    let actions = vec![ScriptedAction::new(TriggerAction::ShowMessage(
        "Test message".to_string(),
    ))];

    schedule_in(&mut world, &mut view, 5, &actions, Some("Test event".to_string())).unwrap();

    assert_eq!(world.scheduler.events.len(), 1);
    assert_eq!(world.scheduler.heap.len(), 1);

    let event = &world.scheduler.events[0];
    assert_eq!(event.on_turn, world.turn_count + 6);
    assert_eq!(event.actions.len(), 1);
    assert_eq!(event.note, Some("Test event".to_string()));
}

#[test]
fn schedule_on_adds_event_to_scheduler() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();
    let actions = vec![ScriptedAction::new(TriggerAction::ShowMessage(
        "Test message".to_string(),
    ))];

    schedule_on(
        &mut world,
        &mut view,
        42,
        &actions,
        Some("Exact turn event".to_string()),
    )
    .unwrap();

    assert_eq!(world.scheduler.events.len(), 1);
    assert_eq!(world.scheduler.heap.len(), 1);

    let event = &world.scheduler.events[0];
    assert_eq!(event.on_turn, 42);
    assert_eq!(event.actions.len(), 1);
    assert_eq!(event.note, Some("Exact turn event".to_string()));
}

#[test]
fn schedule_in_with_multiple_actions() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();
    let actions = vec![
        ScriptedAction::new(TriggerAction::ShowMessage("First message".to_string())),
        ScriptedAction::new(TriggerAction::AwardPoints {
            amount: 10,
            reason: "scheduler bonus".into(),
        }),
        ScriptedAction::new(TriggerAction::ShowMessage("Second message".to_string())),
    ];

    schedule_in(&mut world, &mut view, 3, &actions, None).unwrap();

    let event = &world.scheduler.events[0];
    assert_eq!(event.actions.len(), 3);
    assert_eq!(event.note, None);
}

#[test]
fn schedule_on_with_no_note() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();
    let actions = vec![ScriptedAction::new(TriggerAction::AwardPoints {
        amount: 5,
        reason: "scheduled payout".into(),
    })];

    schedule_on(&mut world, &mut view, 100, &actions, None).unwrap();

    let event = &world.scheduler.events[0];
    assert_eq!(event.note, None);
    assert_eq!(event.on_turn, 100);
}

#[test]
fn dispatch_action_schedule_in_works() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();

    let nested_actions = vec![ScriptedAction::new(TriggerAction::ShowMessage(
        "Delayed message".to_string(),
    ))];
    let action = ScriptedAction::new(TriggerAction::ScheduleIn {
        turns_ahead: 7,
        actions: nested_actions,
        note: Some("Integration test".to_string()),
    });

    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert_eq!(world.scheduler.events.len(), 1);
    let event = &world.scheduler.events[0];
    assert_eq!(event.on_turn, world.turn_count + 8);
    assert_eq!(event.note, Some("Integration test".to_string()));
}

#[test]
fn dispatch_action_schedule_on_works() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();

    let nested_actions = vec![
        ScriptedAction::new(TriggerAction::AwardPoints {
            amount: 25,
            reason: "exact timing".into(),
        }),
        ScriptedAction::new(TriggerAction::ShowMessage("Exact timing!".to_string())),
    ];
    let action = ScriptedAction::new(TriggerAction::ScheduleOn {
        on_turn: 50,
        actions: nested_actions,
        note: None,
    });

    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert_eq!(world.scheduler.events.len(), 1);
    let event = &world.scheduler.events[0];
    assert_eq!(event.on_turn, 50);
    assert_eq!(event.actions.len(), 2);
    assert_eq!(event.note, None);
}

#[test]
fn dispatch_action_conditional_executes_nested_actions_when_condition_true() {
    let (mut world, _, _) = build_test_world();
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::Conditional {
        condition: crate::scheduler::EventCondition::Trigger(crate::trigger::TriggerCondition::MissingFlag(
            "hint".into(),
        )),
        actions: vec![ScriptedAction::new(TriggerAction::ShowMessage(
            "Conditional fired".into(),
        ))],
        false_actions: None,
    });

    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert!(
        view.items.iter().any(
            |entry| matches!(&entry.view_item, ViewItem::TriggeredEvent(msg) if msg.contains("Conditional fired"))
        )
    );
}

#[test]
fn dispatch_action_conditional_skips_actions_when_condition_false() {
    let (mut world, _, _) = build_test_world();
    world.player.flags.insert(Flag::simple("hint", world.turn_count));
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::Conditional {
        condition: crate::scheduler::EventCondition::Trigger(crate::trigger::TriggerCondition::MissingFlag(
            "hint".into(),
        )),
        actions: vec![ScriptedAction::new(TriggerAction::ShowMessage(
            "Should not appear".into(),
        ))],
        false_actions: None,
    });

    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert!(view.items.iter().all(|entry| {
        !matches!(
            &entry.view_item,
            ViewItem::TriggeredEvent(msg) if msg.contains("Should not appear")
        )
    }));
}

#[test]
fn dispatch_action_conditional_executes_false_branch_when_condition_false() {
    let (mut world, _, _) = build_test_world();
    world.player.flags.insert(Flag::simple("hint", world.turn_count));
    let mut view = View::new();

    let action = ScriptedAction::new(TriggerAction::Conditional {
        condition: crate::scheduler::EventCondition::Trigger(crate::trigger::TriggerCondition::MissingFlag(
            "hint".into(),
        )),
        actions: vec![ScriptedAction::new(TriggerAction::ShowMessage(
            "Should not appear".into(),
        ))],
        false_actions: Some(vec![ScriptedAction::new(TriggerAction::ShowMessage(
            "False branch fired".into(),
        ))]),
    });

    dispatch_action(&mut world, &mut view, &action).unwrap();

    assert!(
        view.items.iter().any(
            |entry| matches!(&entry.view_item, ViewItem::TriggeredEvent(msg) if msg.contains("False branch fired"))
        )
    );
    assert!(view.items.iter().all(|entry| {
        !matches!(
            &entry.view_item,
            ViewItem::TriggeredEvent(msg) if msg.contains("Should not appear")
        )
    }));
}

#[test]
fn replace_item_swaps_items_preserving_location() {
    let (mut world, room1, _) = build_test_world();
    let old_id: ItemId = crate::idgen::new_id().into();
    let new_id: ItemId = crate::idgen::new_id().into();
    world.items.insert(
        old_id.clone(),
        make_item(old_id.clone(), Location::Room(room1.clone()), None),
    );
    world.rooms.get_mut(&room1).unwrap().contents.insert(old_id.clone());
    world
        .items
        .insert(new_id.clone(), make_item(new_id.clone(), Location::Nowhere, None));

    replace_item(&mut world, &old_id, &new_id).unwrap();

    assert_eq!(world.items[&old_id].location, Location::Nowhere);
    assert_eq!(world.items[&new_id].location, Location::Room(room1.clone()));
    assert!(world.rooms[&room1].contents.contains(&new_id));
    assert!(!world.rooms[&room1].contents.contains(&old_id));
}

#[test]
fn set_barred_message_updates_exit() {
    let (mut world, room1, room2) = build_test_world();
    world
        .rooms
        .get_mut(&room1)
        .unwrap()
        .exits
        .insert("north".into(), Exit::new(room2.clone()));

    set_barred_message(&mut world, &room1, &room2, "No entry").unwrap();

    let exit = world.rooms[&room1].exits.get("north").unwrap();
    assert_eq!(exit.barred_message, Some("No entry".to_string()));
}

#[test]
fn npc_says_adds_dialogue_to_view() {
    let (mut world, room1, _) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let npc = make_npc(npc_id.clone(), Location::Room(room1.clone()), NpcState::Normal);
    world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id.clone());
    world.npcs.insert(npc_id.clone(), npc);

    let mut view = View::new();
    npc_says(&world, &mut view, &npc_id, "Hello there", None).unwrap();

    assert!(matches!(
        view.items.last().map(|entry| &entry.view_item),
        Some(ViewItem::NpcSpeech { quote, .. }) if quote == "Hello there"
    ));
}

#[test]
fn npc_says_random_uses_npc_dialogue() {
    let (mut world, room1, _) = build_test_world();
    world.spinners.insert(
        crate::spinners::SpinnerType::Core(crate::spinners::CoreSpinnerType::NpcIgnore),
        Spinner::new(vec![Wedge::new("Ignores you.".into())]),
    );
    let npc_id: NpcId = crate::idgen::new_id().into();
    let mut npc = make_npc(npc_id.clone(), Location::Room(room1.clone()), NpcState::Normal);
    npc.dialogue.insert(NpcState::Normal, vec!["Howdy".to_string()]);
    world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id.clone());
    world.npcs.insert(npc_id.clone(), npc);

    let mut view = View::new();
    npc_says_random(&world, &mut view, &npc_id, None).unwrap();

    assert!(matches!(
        view.items.last().map(|entry| &entry.view_item),
        Some(ViewItem::NpcSpeech { quote, .. }) if quote == "Howdy"
    ));
}

#[test]
fn set_npc_state_changes_state() {
    let (mut world, room1, _) = build_test_world();
    let npc_id: NpcId = crate::idgen::new_id().into();
    let npc = make_npc(npc_id.clone(), Location::Room(room1.clone()), NpcState::Normal);
    world.rooms.get_mut(&room1).unwrap().npcs.insert(npc_id.clone());
    world.npcs.insert(npc_id.clone(), npc);

    set_npc_state(&mut world, &npc_id, &NpcState::Mad).unwrap();

    assert_eq!(world.npcs[&npc_id].state, NpcState::Mad);
}
