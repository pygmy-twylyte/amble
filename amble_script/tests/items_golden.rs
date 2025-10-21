use amble_script::{compile_items_to_toml, parse_items};

#[test]
fn items_basic_golden() {
    let src = r#"item sample_widget {
    name "Sample Widget"
    desc "Prototype component."
    portable true
    location inventory player
}

item control_panel {
    name "Control Panel"
    desc "A sealed control panel."
    portable false
    container state closed
    location room control-room
    ability TurnOn
}

item maintenance_key {
    name "Maintenance Key"
    desc "A small bronze key."
    portable true
    location npc attendant
    ability Unlock cabinet
}
"#;
    let items = parse_items(src).expect("parse items ok");
    assert_eq!(items.len(), 3);
    let actual = compile_items_to_toml(&items).expect("compile ok");
    let expected = include_str!("fixtures/items_basic_three.toml");
    assert_eq!(actual.trim(), expected.trim());
}
