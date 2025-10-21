use amble_script::{compile_rooms_to_toml, parse_rooms};

#[test]
fn exits_with_options_golden() {
    let src = r#"room plaza-gate {
  name "Plaza Gate"
  desc "A broad gateway leading into the central plaza."

  exit north -> security-post { locked, barred "Access badge required.", required_flags(clearance-granted, seq maintenance-inspection limit 2), required_items(keycard, access-form) }
  exit south -> shuttle-platform
  exit east -> gallery-corridor { required_items(keycard) }
}
"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms.len(), 1);
    let actual = compile_rooms_to_toml(&rooms).expect("compile ok");
    let expected = include_str!("fixtures/rooms_exits_with_options.toml");
    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn overlays_multi_condition_and_custom_state_golden() {
    let src = r#"room plaza-hub {
  name "Plaza Hub"
  desc "..."

  overlay if npc present vendor, npc in state vendor cheerful {
    text "The vendor beams and offers you a complimentary sample."
  }

  overlay if npc in state guide custom "off-duty" {
    text "The tour guide fidgets, clearly eager for their next assignment."
  }

  overlay if item in room brochure info-kiosk {
    text "A stack of brochures waits patiently on the kiosk."
  }
}
"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms.len(), 1);
    assert_eq!(rooms[0].overlays.len(), 3);
    let actual = compile_rooms_to_toml(&rooms).expect("compile ok");
    let expected = include_str!("fixtures/rooms_overlays_multi.toml");
    assert_eq!(actual.trim(), expected.trim());
}
