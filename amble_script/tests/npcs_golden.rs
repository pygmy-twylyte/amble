use amble_script::{compile_npcs_to_toml, parse_npcs};

#[test]
fn npcs_basic_golden() {
    let src = r#"npc black_knight {
    name "The Black Knight"
    desc "A dark, glowering presence in a black suit of armor with a longsword."
    state normal
    location room sublevel-1-entrance
    dialogue {
        mad {
            "Have at you!"
            "I'm invincible!"
            "'tis but a scratch."
            "It's just a flesh wound!"
            "Come on, you pansy!"
            "Chicken! Bawk-bawk! Chicken!"
            "Gimme that sword back, or I'll bite your legs off!"
            "I won that sword in the office gift game. Give it back!"
        }
        normal {
            "None shall pass."
            "I move -- for no man."
            "(He stands silently, blocking your path.)"
        }
        happy {
            "None shall pa-- oh, no, you can go."
            "YOU may pass, Candidate; I will be sure to dice anyone who dares to follow."
            "You may pass, but beware and steer clear of Room AA-3B, should it reappear."
            "Behold! (slices an aluminum can and then a tomato, laughing maniacally)"
        }
    }
}

npc gonk_droid {
    name "Gonk Droid"
    desc "A walking battery charger that looks like a trash bin."
    state normal
    location room main-lobby
    movement {
        movement_type route
        rooms (main-lobby, lift-bank-main, lounge)
        timing every_2_turns
    }
    dialogue {
        normal {
            "GONK!"
            "(gonk)"
            "GONK gonk."
            "Gonk... Gonk..."
        }
        happy {
            "(You hear a transformer humming a happy tune.)"
        }
    }
}
"#;
    let npcs = parse_npcs(src).expect("parse npcs ok");
    assert_eq!(npcs.len(), 2);
    let actual = compile_npcs_to_toml(&npcs).expect("compile ok");
    let expected = include_str!("fixtures/npcs_roundtrip.toml");
    assert_eq!(actual.trim(), expected.trim());
}
