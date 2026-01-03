use ae::style::GameStyle;
use ae::*;
use amble_engine as ae;
use amble_engine::health::{HealthState, LivingEntity};

#[test]
fn test_command_parse() {
    use ae::View;
    use ae::command::*;
    let mut view = View::new();
    assert!(matches!(parse_command("look", &mut view), Command::Look));
}

#[test]
fn test_goal_condition_flag() {
    use ae::goal::GoalCondition;
    use ae::player::Flag;
    let mut world = world::AmbleWorld::new_empty();
    assert!(!GoalCondition::HasFlag { flag: "a".into() }.satisfied(&world));
    world.player.flags.insert(Flag::simple("a", usize::MAX));
    assert!(GoalCondition::HasFlag { flag: "a".into() }.satisfied(&world));
}

#[test]
fn test_idgen_uuid_deterministic() {
    let u1 = idgen::uuid_from_token(idgen::NAMESPACE_ROOM, "test");
    let u2 = idgen::uuid_from_token(idgen::NAMESPACE_ROOM, "test");
    assert_eq!(u1, u2);
}

#[test]
fn test_item_accessible() {
    use ae::item::{ContainerState, Item};
    let item = Item {
        id: ae::idgen::new_id(),
        symbol: "i".into(),
        name: "Box".into(),
        description: String::new(),
        location: world::Location::Nowhere,
        container_state: Some(ContainerState::Open),
        movability: item::Movability::Free,
        contents: Default::default(),
        abilities: Default::default(),
        interaction_requires: Default::default(),
        text: None,
        consumable: None,
    };
    assert!(item.is_accessible());
}

#[test]
fn test_lib_version() {
    assert!(!ae::AMBLE_VERSION.is_empty());
}

#[test]
fn test_resolve_location_inventory() {
    let symbols = loader::SymbolTable::default();
    let mut table = std::collections::HashMap::new();
    table.insert("Inventory".to_string(), String::new());
    assert!(matches!(
        loader::resolve_location(&table, &symbols).unwrap(),
        world::Location::Inventory
    ));
}

#[test]
fn test_npc_state_keys() {
    use ae::npc::NpcState;
    let custom = NpcState::from_key("custom:foo");
    assert_eq!(custom.as_key(), "custom:foo");
}

#[test]
fn test_player_flag_sequence() {
    use ae::player::Flag;
    let mut flag = Flag::sequence("quest", Some(2), usize::MAX);
    flag.advance();
    assert_eq!(flag.value(), ae::player::format_sequence_value("quest", 1));
}

#[test]
fn test_find_world_object() {
    use std::collections::HashMap;
    let id = ae::idgen::new_id();
    let item = ae::item::Item {
        id: id.clone(),
        symbol: "i".into(),
        name: "Foo".into(),
        description: String::new(),
        location: world::Location::Inventory,
        container_state: None,
        movability: item::Movability::Free,
        contents: Default::default(),
        abilities: Default::default(),
        interaction_requires: Default::default(),
        text: None,
        consumable: None,
    };
    let mut items = HashMap::new();
    items.insert(id.clone(), item);
    let npcs = HashMap::new();
    let res = ae::repl::find_world_object(std::iter::once(&id), &items, &npcs, "foo");
    assert!(res.is_some());
}

#[test]
fn test_room_overlay_applies_flag() {
    use ae::player::Flag;
    use ae::room::{OverlayCondition, RoomOverlay};
    let mut world = world::AmbleWorld::new_empty();
    world.player.flags.insert(Flag::simple("x", usize::MAX));
    let overlay = RoomOverlay {
        conditions: vec![OverlayCondition::FlagSet { flag: "x".into() }],
        text: String::new(),
    };
    let room_id = ae::idgen::new_id();
    assert!(overlay.applies(&room_id, &world));
}

#[test]
fn test_spinner_type_serde() {
    use ae::spinners::{CoreSpinnerType, SpinnerType};
    let ty = SpinnerType::Core(CoreSpinnerType::Movement);
    let s = serde_json::to_string(&ty).unwrap();
    let back: SpinnerType = serde_json::from_str(&s).unwrap();
    assert_eq!(ty, back);
}

#[test]
fn test_style_item() {
    colored::control::set_override(true);
    let styled = "hi".item_style();
    let out = styled.to_string();
    assert!(out.contains("\u{1b}"));
}

#[test]
fn test_trigger_award_points() {
    let mut world = world::AmbleWorld::new_empty();
    let mut view = View::new();
    trigger::award_points(&mut world, &mut view, 5, "test harness bonus");
    assert_eq!(world.player.score, 6); // default 1 + 5
}

#[test]
fn test_world_new_empty_version() {
    let world = world::AmbleWorld::new_empty();
    assert_eq!(world.version, ae::AMBLE_VERSION);
}

#[test]
fn test_loader_goals_to_goal() {
    use ae::loader::goals::{RawGoal, RawGoalCondition};
    let symbols = loader::SymbolTable::default();
    let goal_id = "g".to_string();
    let raw = RawGoal {
        id: goal_id.clone(),
        name: "name".into(),
        description: String::new(),
        group: ae::goal::GoalGroup::Required,
        activate_when: None,
        finished_when: RawGoalCondition::HasFlag { flag: "f".into() },
        failed_when: None,
    };
    let goal = raw.to_goal(&symbols).unwrap();
    assert_eq!(goal.id, goal_id);
}

#[test]
fn test_interaction_requirement_met() {
    use ae::item::{Item, ItemAbility, ItemInteractionType};
    let tool = Item {
        id: ae::idgen::new_id(),
        symbol: "t".into(),
        name: "tool".into(),
        description: String::new(),
        location: world::Location::Nowhere,
        container_state: None,
        contents: Default::default(),
        movability: item::Movability::Free,
        abilities: [ItemAbility::Clean].into_iter().collect(),
        interaction_requires: Default::default(),
        text: None,
        consumable: None,
    };
    let target = Item {
        id: ae::idgen::new_id(),
        symbol: "x".into(),
        name: "target".into(),
        description: String::new(),
        location: world::Location::Nowhere,
        container_state: None,
        movability: item::Movability::Free,
        contents: Default::default(),
        abilities: Default::default(),
        interaction_requires: std::iter::once((ItemInteractionType::Clean, ItemAbility::Clean)).collect(),
        text: None,
        consumable: None,
    };
    assert!(ae::loader::items::interaction_requirement_met(
        ItemInteractionType::Clean,
        &target,
        &tool
    ));
}

#[test]
fn test_raw_npc_to_npc() {
    use ae::loader::npcs::RawNpc;
    use std::collections::{HashMap, HashSet};
    let mut symbols = loader::SymbolTable::default();
    ae::loader::rooms::register_npc(&mut symbols, "npc");
    let raw = RawNpc {
        id: "npc".into(),
        name: "Npc".into(),
        description: String::new(),
        location: HashMap::from([("Nowhere".to_string(), "".to_string())]),
        inventory: HashSet::new(),
        dialogue: HashMap::new(),
        state: ae::npc::NpcState::Normal,
        movement: None,
        max_hp: 10,
    };

    let npc = raw.to_npc(&symbols).unwrap();
    assert_eq!(npc.name, "Npc");
}

#[test]
fn test_raw_player_to_player() {
    use ae::loader::player::{RawPlayer, build_player};
    use std::collections::{HashMap, HashSet};
    let mut symbols = loader::SymbolTable::default();
    let raw = RawPlayer {
        id: "player".into(),
        name: "P".into(),
        description: String::new(),
        location: {
            let mut m = HashMap::new();
            m.insert("Inventory".into(), String::new());
            m
        },
        inventory: HashMap::new(),
        flags: HashSet::new(),
        score: 0,
        max_hp: 10,
    };
    let player = build_player(&raw, &mut symbols).unwrap();
    assert_eq!(player.name, "P");
    assert_eq!(player.max_hp(), 10);
    assert_eq!(player.current_hp(), 10);
}

#[test]
fn test_register_item() {
    let mut symbols = loader::SymbolTable::default();
    let id = ae::loader::rooms::register_item(&mut symbols, "item");
    let again = ae::loader::rooms::register_item(&mut symbols, "item");
    assert_eq!(id, again);
}

#[test]
fn test_spinner_file_to_map() {
    use ae::loader::spinners::{RawSpinnerData, SpinnerFile};
    use ae::spinners::{CoreSpinnerType, SpinnerType};
    let file = SpinnerFile {
        entries: vec![RawSpinnerData {
            spinner_type_key: "movement".into(),
            values: vec!["go".into()],
            widths: vec![1],
        }],
    };
    let map = file.to_spinner_map();
    assert!(map.contains_key(&SpinnerType::Core(CoreSpinnerType::Movement)));
}

#[test]
fn test_build_triggers_empty() {
    let triggers = ae::loader::triggers::build_triggers(&[], &loader::SymbolTable::default()).unwrap();
    assert!(triggers.is_empty());
}

#[test]
fn test_inventory_vessel_type() {
    use ae::repl::inventory::VesselType;
    assert!(matches!(VesselType::Item, VesselType::Item));
}

#[test]
fn test_move_to_handler_simple() {
    use ae::View;
    use ae::room::{Exit, Room};
    use std::collections::{HashMap, HashSet};
    let mut world = world::AmbleWorld::new_empty();
    let r1 = ae::idgen::new_id();
    let r2 = ae::idgen::new_id();
    let mut room1 = Room {
        id: r1.clone(),
        symbol: "r1".into(),
        name: "R1".into(),
        base_description: String::new(),
        overlays: vec![],
        location: world::Location::Nowhere,
        visited: true,
        exits: HashMap::new(),
        contents: HashSet::new(),
        npcs: HashSet::new(),
    };
    room1.exits.insert(
        "north".into(),
        Exit {
            to: r2.clone(),
            hidden: false,
            locked: false,
            required_flags: HashSet::new(),
            required_items: HashSet::new(),
            barred_message: None,
        },
    );
    let room2 = Room {
        id: r2.clone(),
        symbol: "r2".into(),
        name: "R2".into(),
        base_description: String::new(),
        overlays: vec![],
        location: world::Location::Nowhere,
        visited: false,
        exits: HashMap::new(),
        contents: HashSet::new(),
        npcs: HashSet::new(),
    };
    world.rooms.insert(r1.clone(), room1);
    world.rooms.insert(r2.clone(), room2);
    world.player.location = world::Location::Room(r1.clone());
    let mut view = View::new();
    assert!(ae::repl::movement::move_to_handler(&mut world, &mut view, "north").is_ok());
    assert!(matches!(world.player.location, world::Location::Room(id) if id == r2));
}

#[test]
fn test_filtered_goals_empty() {
    let world = world::AmbleWorld::new_empty();
    let list = ae::repl::system::filtered_goals(&world, goal::GoalStatus::Active);
    assert!(list.is_empty());
}

#[test]
fn test_check_scheduled_events() {
    use ae::trigger::{ScriptedAction, TriggerAction};
    let mut world = world::AmbleWorld::new_empty();
    let mut view = View::new();
    world.scheduler.schedule_on(
        1,
        vec![ScriptedAction::new(TriggerAction::ShowMessage("test".to_string()))],
        None,
    );
    world.turn_count = 1;
    ae::repl::check_scheduled_events(&mut world, &mut view).unwrap();
    assert!(view.items.iter().any(|entry| {
        matches!(
            &entry.view_item,
            view::ViewItem::TriggeredEvent(msg) if msg.contains("test")
        )
    }));
}

#[test]
fn test_check_npc_movement() {
    use ae::npc::{MovementTiming, MovementType, Npc, NpcMovement};
    use ae::room::Room;
    use std::collections::{HashMap, HashSet};
    let mut world = world::AmbleWorld::new_empty();
    let mut view = View::new();
    let r1 = ae::idgen::new_id();
    let r2 = ae::idgen::new_id();
    let room1 = Room {
        id: r1.clone(),
        symbol: "r1".into(),
        name: "R1".into(),
        base_description: String::new(),
        overlays: vec![],
        location: world::Location::Nowhere,
        visited: true,
        exits: HashMap::new(),
        contents: HashSet::new(),
        npcs: HashSet::new(),
    };
    let room2 = Room {
        id: r2.clone(),
        symbol: "r2".into(),
        name: "R2".into(),
        base_description: String::new(),
        overlays: vec![],
        location: world::Location::Nowhere,
        visited: false,
        exits: HashMap::new(),
        contents: HashSet::new(),
        npcs: HashSet::new(),
    };
    world.rooms.insert(r1.clone(), room1);
    world.rooms.insert(r2.clone(), room2);
    let npc_id = ae::idgen::new_id();
    let npc = Npc {
        id: npc_id.clone(),
        symbol: "npc".into(),
        name: "NPC".into(),
        description: String::new(),
        location: world::Location::Room(r1.clone()),
        inventory: HashSet::new(),
        dialogue: HashMap::new(),
        state: npc::NpcState::Normal,
        health: HealthState::new_at_max(10),
        movement: Some(NpcMovement {
            movement_type: MovementType::Route {
                rooms: vec![r1.clone(), r2.clone()],
                current_idx: 0,
                loop_route: false,
            },
            timing: MovementTiming::EveryNTurns { turns: 1 },
            active: true,
            last_moved_turn: 0,
            paused_until: None,
        }),
    };
    world.npcs.insert(npc_id.clone(), npc);
    world.player.location = world::Location::Room(r1.clone());
    world.turn_count = 1;
    ae::repl::check_npc_movement(&mut world, &mut view).unwrap();
    let npc = world.npcs.get(&npc_id).unwrap();
    assert!(matches!(&npc.location, world::Location::Room(id) if *id == r2));
}

#[test]
fn test_check_ambient_triggers() {
    use ae::scheduler::EventCondition;
    use ae::spinners::SpinnerType;
    use ae::trigger::{Trigger, TriggerCondition};
    use gametools::{Spinner, Wedge};
    let mut world = world::AmbleWorld::new_empty();
    let mut view = View::new();
    let r1 = ae::idgen::new_id();
    let room1 = room::Room {
        id: r1.clone(),
        symbol: "r1".into(),
        name: "R1".into(),
        base_description: String::new(),
        overlays: vec![],
        location: world::Location::Nowhere,
        visited: true,
        exits: std::collections::HashMap::new(),
        contents: std::collections::HashSet::new(),
        npcs: std::collections::HashSet::new(),
    };
    world.rooms.insert(r1.clone(), room1);
    world.player.location = world::Location::Room(r1.clone());
    let spinner_type = SpinnerType::Custom("test_spinner".to_string());
    let spinner = Spinner::new(vec![Wedge::new("test message".to_string())]);
    world.spinners.insert(spinner_type.clone(), spinner);
    let trigger = Trigger {
        name: "ambient".into(),
        conditions: EventCondition::Trigger(TriggerCondition::Ambient {
            room_ids: [r1.clone()].into_iter().collect(),
            spinner: spinner_type,
        }),
        actions: vec![],
        only_once: false,
        fired: false,
    };
    world.triggers.push(trigger);
    ae::repl::check_ambient_triggers(&mut world, &mut view).unwrap();
    assert!(view.items.iter().any(|entry| {
        matches!(
            &entry.view_item,
            view::ViewItem::AmbientEvent(msg) if msg.contains("test message")
        )
    }));
}

#[test]
fn test_entity_not_found() {
    let world = world::AmbleWorld::new_empty();
    let mut view = View::new();
    ae::repl::entity_not_found(&world, &mut view, "test_entity");
    assert!(view.items.iter().any(|entry| {
        matches!(
            &entry.view_item,
            view::ViewItem::Error(msg) if msg.contains("test_entity")
        )
    }));
}
