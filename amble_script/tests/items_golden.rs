use amble_script::{compile_items_to_toml, parse_items};

#[test]
fn items_basic_golden() {
    let src = r#"item no_cake {
    name "No Cake"
    desc "Absence of cake."
    portable true
    location inventory player
}

item portal_gun {
    name "Portal Gun"
    desc "A device."
    portable false
    container state closed
    location room portal-room
    ability TurnOn
}

item lost_and_found_key {
    name "Lost and Found Key"
    desc "A small brass key."
    portable true
    location npc clerk
    ability Unlock box
}
"#;
    let items = parse_items(src).expect("parse items ok");
    assert_eq!(items.len(), 3);
    let actual = compile_items_to_toml(&items).expect("compile ok");
    let expected = include_str!("fixtures/items_basic_three.toml");
    assert_eq!(actual.trim(), expected.trim());
}
