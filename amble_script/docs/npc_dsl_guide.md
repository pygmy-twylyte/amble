# NPC DSL Guide

This guide introduces the NPC subset of the amble_script DSL and how it maps to the engine's `npcs.toml`.

Highlights:
- Required fields: `name`, `desc`, `state`, and `location`.
- Optional inventory with `inventory (item1, item2, ...)`.
- `dialogue` block groups lines by state; use `custom(state)` for non‑named states.
- Optional `movement` block describing automatic movement (`movement_type`, `rooms`, `timing`, optional `active` and `loop_route`).

## Minimal NPC

```amble
npc receptionist {
  name "Receptionist"
  desc "A friendly face at the front desk."
  state idle
  location room lobby
  dialogue {
    idle { "Welcome!" }
  }
}
```

Emits:

```toml
[[npcs]]
id = "receptionist"
name = "Receptionist"
description = "A friendly face at the front desk."
state = "idle"

[npcs.location]
Room = "lobby"

[npcs.dialogue]
idle = ["Welcome!"]
```

## Dialogue

Additional states can provide distinct lines. Use `custom(state)` when referencing custom states in triggers:

```amble
dialogue {
  idle { "Welcome." }
  busy { "One moment." }
  custom(want-emitter) { "Have you found the emitter?" }
}
```

In the emitted TOML, custom states are keyed as `custom:<state>`.

## Inventory

Give an NPC starting items:

```amble
inventory (badge, coffee)
```

## Movement

NPCs may roam using a `movement` block:

```amble
movement {
  movement_type route        # or "random"
  rooms (lobby, break_room)
  timing every_2_turns       # token from the engine's movement scheduler
  active true                # optional; defaults to true
  loop_route false           # optional when using "route"
}
```

## Library Usage

```rust
use amble_script::{parse_npcs, compile_npcs_to_toml};
let src = std::fs::read_to_string("npcs.amble")?;
let npcs = parse_npcs(&src)?;
let toml = compile_npcs_to_toml(&npcs)?;
```

The resulting `toml` string matches `amble_engine/data/npcs.toml`.
