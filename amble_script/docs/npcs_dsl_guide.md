# NPCs DSL Guide

This guide covers the NPC portion of the `amble_script` DSL and how it maps into the engine’s `WorldDef` (`world.ron`). Non-player characters can be static flavour, mobile actors that roam rooms, or storytellers with branching dialogue.

Highlights:
- Required fields: `name`, `desc`, `max_hp`, and `location`.
- Optional initial `state` (`normal` by default) or `state custom <id>` for bespoke variants.
- Movement controls: `movement route rooms (…)` or `movement random rooms (…)` with optional `timing`, `active`, and `loop` modifiers.
- Dialogue banks keyed by state (`dialogue normal { … }`, `dialogue custom panic { … }`).
- Compiles into `WorldDef` data the engine reads.

## Minimal NPC

```amble
npc receptionist {
  name "Receptionist"
  desc "Focused on a flickering terminal."
  max_hp 10
  location room lab-lobby
}
```

This produces an `NpcDef` with the corresponding `id`, `name`, `desc`, `max_hp`, `location`, and `state` fields. If no explicit state is provided, the compiler emits `state = normal`.

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
use amble_script::{parse_npcs, worlddef_from_asts};
use ron::ser::PrettyConfig;
let src = std::fs::read_to_string("npcs.amble")?;
let npcs = parse_npcs(&src)?;
let worlddef = worlddef_from_asts(&[], &[], &[], &[], &npcs, &[])?;
let ron = ron::ser::to_string_pretty(&worlddef, PrettyConfig::default())?;
```

The resulting `ron` string can be written to `world.ron` for engine loading.
