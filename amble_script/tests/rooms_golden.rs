use amble_script::{compile_rooms_to_toml, parse_rooms};

#[test]
fn minimal_room_multiline_desc_golden() {
    let src = r#"room high-ridge {
    name "High Isolated Ridge"
    desc "A small, flat ridge in the midst of a steeply sloped wooded area. Probably west of something, depending on how you're oriented.\n\nSome rough stairs carved into the slope curve upward into the trees."
}
"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms.len(), 1);
    let actual = compile_rooms_to_toml(&rooms).expect("compile ok");
    let expected = include_str!("fixtures/rooms_minimal_high_ridge.toml");
    assert_eq!(actual.trim(), expected.trim());
}
