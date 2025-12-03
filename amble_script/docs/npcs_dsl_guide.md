# NPCs DSL Guide

This guide covers the NPC portion of the `amble_script` DSL and how it maps to the engine’s `npcs.toml`. Non-player characters can be static flavour, mobile actors that roam rooms, or storytellers with branching dialogue.

Highlights:
- Required fields: `name`, `desc`, `max_hp`, and `location`.
- Optional initial `state` (`normal` by default) or `state custom <id>` for bespoke variants.
- Movement controls: `movement route rooms (…)` or `movement random rooms (…)` with optional `timing`, `active`, and `loop` modifiers.
- Dialogue banks keyed by state (`dialogue normal { … }`, `dialogue custom panic { … }`).
- Compiles to the same TOML the engine reads, including source annotations for easy debugging.

## Minimal NPC

```amble
npc receptionist {
  name "Receptionist"
  desc "Focused on a flickering terminal."
  max_hp 10
  location room lab-lobby
}
```

Emits:

```toml
[[npcs]]
# npc receptionist (source line N)
id = "receptionist"
name = "Receptionist"
description = "Focused on a flickering terminal."
max_hp = 10
location = { Room = "lab-lobby" }
state = "normal"
```

If no explicit state is provided, the compiler emits `state = "normal"`.

## Locations

Use `location room <room_id>` to place the NPC in a room at start, or `location nowhere "note"` to keep them off-stage until spawned by a trigger.

```amble
location room lobby         # immediately present
location nowhere "In the wings"  # available for later spawn
```

## States

States gate dialogue sets and trigger conditions. Two forms are supported:

- Named states (`state alert`) map to the engine’s built-in variants.
- Custom states (`state custom emergency`) let you invent new labels without editing engine enums.

Either form is valid in triggers (`npc in state guard alert`) and overlays.

## Movement

Add a movement routine to describe patrols or ambient wanderers:

```amble
movement route rooms (atrium-north, atrium-east, atrium-south)
  timing every_3_turns
  active true
  loop true
```

Options:

- `movement route rooms (…)` walks through the list in order.
- `movement random rooms (…)` chooses a random destination from the list.
- `timing <ident>` with the timing string in the form of "every_N_turns" or "on_turn_N".
- `active true|false` decides whether the routine starts immediately (`true` by default).
- `loop true|false` controls whether route patrols wrap to the first room or stop after one lap.

Movement is optional; omit it for static characters.

## Dialogue

Dialogue blocks collect one or more lines keyed by state:

```amble
dialogue normal {
  "Welcome to the lab."
  "Please sign in."
}

dialogue custom emergency {
  "Please evacuate immediately!"
}
```

- Multiple dialogue blocks are allowed; add new blocks for each state you support.
- The compiler prefixes custom dialogue states with `custom:` internally to match engine expectations.
- Triggers can use `do npc says random guard` to pull from these banks.

## Library Usage

```rust
use amble_script::{parse_npcs, compile_npcs_to_toml};
let src = std::fs::read_to_string("npcs.amble")?;
let npcs = parse_npcs(&src)?;
let toml = compile_npcs_to_toml(&npcs)?;
```

The resulting TOML mirrors `amble_engine/data/npcs.toml`, complete with provenance comments for troubleshooting.
