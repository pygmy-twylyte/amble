# Spinners DSL Guide

Spinners power ambient flavour text and other random selections. This guide explains the `spinner` syntax in the `amble_script` DSL and how it compiles into `WorldDef` (`world.ron`).

Highlights:
- A spinner is a named collection of text entries (`spinner <id> { "Text" … }`).
- Entries are unweighted; the engine draws from a refilling pool so every entry appears once before reshuffling.
- Referenced from triggers via `do spinner message <spinner_id>` or expanded with `do add entry … spinner <spinner_id>`.
- Compiles directly to the engine’s spinner schema inside `WorldDef`.

## Minimal Spinner

```amble
spinner ambientLobby {
  "The HVAC sighs."
  "Footsteps echo from deeper inside."
}
```

WorldDef excerpt (RON):

```ron
(
  id: "ambientLobby",
  entries: [
    "The HVAC sighs.",
    "Footsteps echo from deeper inside.",
  ],
)
```

The engine draws from a refilling pool whenever the spinner is triggered, which prevents repeats until every entry has appeared once.

## Entry Tips

- Keep entry text concise; use triggers to gate long-form narration.
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
