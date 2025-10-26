# Spinners DSL Guide

Spinners power ambient flavour text and other weighted random selections. This guide explains the `spinner` syntax in the `amble_script` DSL and how it compiles into `spinners.toml`.

Highlights:
- A spinner is a named collection of wedges (`spinner <id> { wedge "Text" [width <n>] … }`).
- Each wedge carries an optional weight; omit `width` to default to 1.
- Referenced from triggers via `do spinner message <spinner_id>` or expanded with `do add wedge … spinner <spinner_id>`.
- Compiles directly to the engine’s spinner schema with source comments for provenance.

## Minimal Spinner

```amble
spinner ambientLobby {
  wedge "The HVAC sighs."
  wedge "Footsteps echo from deeper inside." width 2
}
```

Emits:

```toml
[[spinners]]
# spinner ambientLobby (source line N)
id = "ambientLobby"

[[spinners.wedges]]
text = "The HVAC sighs."
width = 1

[[spinners.wedges]]
text = "Footsteps echo from deeper inside."
width = 2
```

The engine rolls a weighted random selection whenever the spinner is triggered. In this example, the second line is twice as likely as the first.

## Wedge Tips

- Keep wedge text concise; use triggers to gate long-form narration.
- Combine with `schedule` triggers for recurring ambience (`do schedule in 3 { do spinner message ambientLobby }`).
- Use multiple spinners for themed areas (e.g. `ambientLab`, `ambientAtrium`) and swap between them via `do spinner message …` actions.

## Library Usage

```rust
use amble_script::{parse_spinners, compile_spinners_to_toml};
let src = std::fs::read_to_string("spinners.amble")?;
let spinners = parse_spinners(&src)?;
let toml = compile_spinners_to_toml(&spinners)?;
```

The resulting TOML matches `amble_engine/data/spinners.toml`, ready to drop into the engine’s data directory or ship with compiled content.
