use amble_script::{compile_rooms_to_toml, parse_rooms};

#[test]
fn exits_with_options_golden() {
    let src = r#"room two-sheds-landing {
  name "Jackson's Landing"
  desc "A quiet landing tucked along the slope..."

  exit up   -> guard-post { locked, barred "Need to clear the tree.", required_flags(cleared-fallen-tree, seq other-flag limit 3), required_items(machete, gasoline) }
  exit down -> parish-landing
  exit east -> two-sheds { required_items(machete) }
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
    let src = r#"room front-entrance {
  name "Front Entrance"
  desc "..."

  overlay if npc present cmot_dibbler, npc in state cmot_dibbler happy {
    text "Dibbler beams and offers a celebratory sausage-inna-bun."
  }

  overlay if npc in state emh custom "want-emitter" {
    text "The EMH fidgets restlessly, craving a mobile emitter."
  }

  overlay if item in room margarine st-alfonzo-parish {
    text "On the pedestal sits a tub of margarine."
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
