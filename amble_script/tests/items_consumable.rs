use amble_script::{compile_items_to_toml, parse_items};

#[test]
fn items_consumable_golden() {
    let src = r#"
item energy_bar {
  name "Energy Bar"
  desc "Restores energy."
  portable true
  location inventory player
  consumable {
    uses_left 2
    consume_on ability Use
    when_consumed replace inventory wrapper
  }
}
"#;
    let items = parse_items(src).expect("parse items ok");
    assert_eq!(items.len(), 1);
    let actual = compile_items_to_toml(&items).expect("compile ok");
    let expected = include_str!("fixtures/items_consumable.toml");

    let expected_clean = expected
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let actual_clean = actual
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    let expected_val: toml::Value = toml::from_str(&expected_clean).expect("parse expected");
    let actual_val: toml::Value = toml::from_str(&actual_clean).expect("parse actual");
    assert_eq!(actual_val, expected_val);
}
