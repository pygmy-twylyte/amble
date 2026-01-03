# Goals DSL Guide

Goals communicate progress and objectives to the player. This guide explains the goal syntax in the `amble_script` DSL and how it compiles into `WorldDef` (`world.ron`).

Highlights:
- Required fields: `name`, `desc`, `group`, and `complete when …`.
- Optional `activate when …` gates when the goal becomes visible/active.
- Optional `fail when …` marks failure states.
- Conditions can reference flags, items, rooms, other goals, and sequence progress.
- Output matches the engine’s goal schema for traceability.

## Minimal Goal

```amble
goal get-out {
  name "Escape the Facility"
  desc "Find a path to the surface."
  group required
  done when reached room surface
}
```

This produces a `GoalDef` with the corresponding `id`, `name`, `desc`, `group`, and condition fields.

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
use amble_script::{GameAst, PlayerAst, parse_goals, worlddef_from_asts};
use ron::ser::PrettyConfig;
let src = std::fs::read_to_string("goals.amble")?;
let goals = parse_goals(&src)?;
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
let worlddef = worlddef_from_asts(Some(&game), &[], &[], &[], &[], &[], &goals)?;
let ron = ron::ser::to_string_pretty(&worlddef, PrettyConfig::default())?;
```

The generated `ron` string can be written to `world.ron` for engine loading.
