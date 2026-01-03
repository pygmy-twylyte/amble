# Spinners DSL Guide

Spinners power ambient flavour text and other weighted random selections. This guide explains the `spinner` syntax in the `amble_script` DSL and how it compiles into `WorldDef` (`world.ron`).

Highlights:
- A spinner is a named collection of wedges (`spinner <id> { wedge "Text" [width <n>] … }`).
- Each wedge carries an optional weight; omit `width` to default to 1.
- Referenced from triggers via `do spinner message <spinner_id>` or expanded with `do add wedge … spinner <spinner_id>`.
- Compiles directly to the engine’s spinner schema inside `WorldDef`.

## Minimal Spinner

```amble
spinner ambientLobby {
  wedge "The HVAC sighs."
  wedge "Footsteps echo from deeper inside." width 2
}
```

WorldDef excerpt (RON):

```ron
(
  id: "ambientLobby",
  wedges: [
    (text: "The HVAC sighs.", width: 1),
    (text: "Footsteps echo from deeper inside.", width: 2),
  ],
)
```

The engine rolls a weighted random selection whenever the spinner is triggered. In this example, the second line is twice as likely as the first.

## Wedge Tips

- Keep wedge text concise; use triggers to gate long-form narration.
- Combine with `schedule` triggers for recurring ambience (`do schedule in 3 { do spinner message ambientLobby }`).
- Use multiple spinners for themed areas (e.g. `ambientLab`, `ambientAtrium`) and swap between them via `do spinner message …` actions.

## Library Usage

```rust
use amble_script::{GameAst, PlayerAst, parse_spinners, worlddef_from_asts};
use ron::ser::PrettyConfig;
let src = std::fs::read_to_string("spinners.amble")?;
let spinners = parse_spinners(&src)?;
let game = GameAst {
    title: "Demo".into(),
    intro: "Intro".into(),
    player: PlayerAst {
        name: "The Candidate".into(),
        description: "An adventurer.".into(),
        max_hp: 20,
        start_room: "foyer".into(),
    },
    scoring: None,
};
let worlddef = worlddef_from_asts(Some(&game), &[], &[], &[], &spinners, &[], &[])?;
let ron = ron::ser::to_string_pretty(&worlddef, PrettyConfig::default())?;
```

The resulting `ron` string can be written to `world.ron` for engine loading.
