use amble_script::{compile_rooms_to_toml, parse_rooms};

#[test]
fn minimal_room_multiline_desc_golden() {
    let src = r#"room observation-deck {
    name "Observation Deck"
    desc "A narrow platform overlooking the valley.\n\nTelescopes and weather instruments line the railing."
}
"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms.len(), 1);
    let actual = compile_rooms_to_toml(&rooms).expect("compile ok");
    let expected = include_str!("fixtures/rooms_observation_deck.toml");
    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn overlay_flag_pair_golden() {
    let src = r#"room test {
    name "Test"
    desc "Desc"
    overlay if flag foo {
        set "Foo Is Set"
        unset "Foo Is Not Set"
    }
}"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].overlays.len(), 2);
    let actual = compile_rooms_to_toml(&rooms).expect("compile ok");
    let expected = include_str!("fixtures/rooms_overlay_flag_pair.toml");
    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn overlay_item_pair_parses() {
    let src = r#"room test {
    name "Test"
    desc "Desc"
    overlay if item widget {
        present "Widget here"
        absent "Widget missing"
    }
}"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms[0].overlays.len(), 2);
    assert!(matches!(
        rooms[0].overlays[0].conditions[0],
        amble_script::OverlayCondAst::ItemPresent(_)
    ));
    assert!(matches!(
        rooms[0].overlays[1].conditions[0],
        amble_script::OverlayCondAst::ItemAbsent(_)
    ));
}

#[test]
fn overlay_npc_pair_parses() {
    let src = r#"room test {
    name "Test"
    desc "Desc"
    overlay if npc bob {
        present "Bob waves"
        absent "Bob is gone"
    }
}"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms[0].overlays.len(), 2);
    assert!(matches!(
        rooms[0].overlays[0].conditions[0],
        amble_script::OverlayCondAst::NpcPresent(_)
    ));
    assert!(matches!(
        rooms[0].overlays[1].conditions[0],
        amble_script::OverlayCondAst::NpcAbsent(_)
    ));
}
