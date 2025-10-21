use amble_script::{compile_rooms_to_toml, parse_rooms};

#[test]
fn overlay_npc_states_block_expands_to_multiple_overlays() {
    let src = r#"room triage-center {
  name "Triage Center"
  desc "Bright lights and organized supplies."

  overlay if npc holo_medic here {
    normal "The holographic medic waits for instructions."
    cheerful "The holographic medic hums a tune while checking the scanners."
    custom(needs-calibration) "The holographic medic paces, requesting a recalibration module."
  }
}
"#;
    let rooms = parse_rooms(src).expect("parse rooms ok");
    assert_eq!(rooms.len(), 1);
    // Should expand to three overlays
    assert_eq!(rooms[0].overlays.len(), 3);
    let actual = compile_rooms_to_toml(&rooms).expect("compile ok");
    let expected = include_str!("fixtures/rooms_overlay_npc_states_block.toml");
    assert_eq!(actual.trim(), expected.trim());
}
