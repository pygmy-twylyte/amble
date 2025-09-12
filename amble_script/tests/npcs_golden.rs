use amble_script::{compile_npcs_to_toml, parse_npcs};

#[test]
fn npcs_basic_golden() {
    let src = r#"npc emh {
  name "Emergency Medical Hologram"
  desc "A holographic doctor."
  location room med-bay
  state custom want-emitter
  movement random rooms (med-bay, lounge) timing every_3_turns active true
  dialogue normal {
    "Please state the nature of the medical emergency."
  }
  dialogue custom want-emitter {
    "I need my mobile emitter."
  }
}

npc gonk_droid {
  name "Gonk Droid"
  desc "A walking battery charger."
  location room main-lobby
  state normal
  movement route rooms (main-lobby, lounge) timing every_2_turns
  dialogue normal { "GONK!" }
}
"#;
    let npcs = parse_npcs(src).expect("parse npcs ok");
    let actual = compile_npcs_to_toml(&npcs).expect("compile ok");

    // Compare as TOML values to ignore formatting and comments
    let expected = include_str!("fixtures/npcs_basic.toml");
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
