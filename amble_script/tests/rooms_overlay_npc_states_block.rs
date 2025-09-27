use amble_script::{compile_rooms_to_toml, parse_rooms};

#[test]
fn overlay_npc_states_block_expands_to_multiple_overlays() {
    let src = r#"room med-bay {
  name "Med Bay"
  desc "Clinical and spotless."

  overlay if npc emh here {
    normal "The EMH stands with professional detachment, awaiting your symptoms."
    happy "The EMH smiles and hums a bright tune while adjusting the diagnostic displays."
    custom(want-emitter) "The EMH fidgets restlessly, casting longing glances toward the door as if craving a mobile emitter."
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
