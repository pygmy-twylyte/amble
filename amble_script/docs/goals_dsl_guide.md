# Goals DSL Guide

Goals communicate progress and objectives to the player. This guide explains the goal syntax in the `amble_script` DSL and how it compiles into `goals.toml`.

Highlights:
- Required fields: `name`, `desc`, `group`, and `complete when …`.
- Optional `activate when …` gates when the goal becomes visible/active.
- Optional `fail when …` marks failure states.
- Conditions can reference flags, items, rooms, other goals, and sequence progress.
- Output matches the engine’s goal schema with source comments for traceability.

## Minimal Goal

```amble
goal get-out {
  name "Escape the Facility"
  desc "Find a path to the surface."
  group required
  done when reached room surface
}
```

Emits:

```toml
[[goals]]
# goal get-out (source line N)
id = "get-out"
name = "Escape the Facility"
description = "Find a path to the surface."
group = "required"
complete_when = { type = "reachedRoom", room = "surface" }
```

## Groups

The group determines how the engine presents and tallies the goal:

- `required` — must be completed to win.
- `optional` — side quest or bonus objective.
- `status-effect` — shown for ongoing states (e.g. timed ailments, debuffs).

## Conditions

Every `… when …` clause accepts a single condition from the following vocabulary:

- `has flag <flag>` | `missing flag <flag>`
- `flag in progress <flag>` | `flag complete <flag>`
- `has item <item>`
- `reached room <room>`
- `goal complete <other_goal>`

You could use supporting triggers to synthesise additional flags if you need compound logic.

```amble
goal stabilize-reactor {
  name "Stabilise the Reactor"
  desc "Restore power to critical systems."
  group required
  start when has flag reactor-destabilized
  done when flag complete reactor-recalibrated
  fail when has flag catastrophic-meltdown
}
```

`activate when …` is optional; omit it to make the goal visible from the start. Likewise, `fail when …` is optional—leave it off if the goal cannot fail.

## Library Usage

```rust
use amble_script::{parse_goals, compile_goals_to_toml};
let src = std::fs::read_to_string("goals.amble")?;
let goals = parse_goals(&src)?;
let toml = compile_goals_to_toml(&goals)?;
```

The generated string matches `amble_engine/data/goals.toml` so it can be copied directly into the engine’s data directory.
