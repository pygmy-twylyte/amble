use amble_script::{compile_npcs_to_toml, parse_npcs};

#[test]
fn npcs_basic_golden() {
    let src = r#"npc holo_medic {
  name "Holographic Medic"
  desc "A simulated emergency physician."
  location room triage-center
  max_hp 12
  state custom needs-calibration
  movement random rooms (triage-center, commons) timing every_3_turns active true
  dialogue normal {
    "Please describe the nature of your emergency."
  }
  dialogue custom needs-calibration {
    "I require a calibration module to resume field operations."
  }
}

npc courier_bot {
  name "Courier Bot"
  desc "A walking delivery crate."
  location room transit-hall
  max_hp 8
  state normal
  movement route rooms (transit-hall, commons) timing every_2_turns
  dialogue normal { "Package routed." }
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
