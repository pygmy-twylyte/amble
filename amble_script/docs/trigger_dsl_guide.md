# Amble Trigger DSL – Creator’s Guide

This guide introduces Amble’s trigger DSL for content creators. It explains the main concepts, shows common patterns, and provides copy‑paste examples you can adapt. It aims for a practical middle‑ground: enough detail to be productive without being exhaustive.

The DSL compiles to the engine’s `triggers.toml`. If you can express it here, you can ship it in your game.

## Core Concepts

- Trigger: A named block with a firing event (`when …`), optional gate conditions (`if …`), and a sequence of actions (`do …`).
- Event vs Conditions:
  - Event is the thing the player (or world) does that raises the trigger (e.g., `enter room lab`).
  - Conditions are AND/OR logic that must also be true for the trigger to run.
- Actions: What happens when the trigger fires (e.g., show text, give points, spawn items, schedule future actions).
- Sets: Reusable named lists for ambient room lists (see “Sets for Ambients”).

## Quick Start

```amble
trigger "First time in the Lab" when enter room lab {
  if missing flag visited:lab {
    do show "The ozone smell hints at recent experiments."
    do add flag visited:lab
    do award points 1
  }
}
```

## Events (`when …`)

- `enter room <room>`
- `leave room <room>`
- `look at item <item>`
- `open item <item>`
- `use item <item> ability <ability>`
- `use item <tool> on item <target> interaction <interaction>`
- `act <interaction> on item <target>`
- `take item <item>`
- `drop item <item>`
- `unlock item <item>`
- `insert item <item> into item <container>`
- `take item <item> from npc <npc>`
- `give item <item> to npc <npc>`
- `talk to npc <npc>`
- `always` — eventless; trigger is evaluated each turn against its conditions (useful for ambients/status).

## Conditions (`if …`)

Simple conditions:
- `has flag <name>` | `missing flag <name>`
- `has item <item>` | `missing item <item>`
- `player in room <room>` | `has visited room <room>`
- `container <container> has item <item>`
- `with npc <npc>` | `npc has item <npc> <item>` | `npc in state <npc> <state>`
- `flag in progress <name>` | `flag complete <name>`
- `chance <percent>%` (re‑rolls on each evaluation)
- `in rooms <room1,room2,…>` (preferred for ambients; see “Sets for Ambients”)

Grouping:
- `all(cond1, cond2, …)` — AND
- `any(cond1, cond2, …)` — OR
- You can nest groups, e.g. `all(has flag a, any(has item key, with npc guard))`

Examples:

```amble
# Gate on flags
if all(has flag quest-started, missing flag quest-finished) { … }

# Gate on location
if player in room sublevel-1-entrance { … }

# OR logic
if any(has item keycard, with npc receptionist) { … }

# Ambient condition (see sets below)
if in rooms lobby,restaurant { … }
```

## Actions (`do …`)

Player feedback and flags:
- `do show "…"`
- `do add flag <name>` — simple boolean flag
- `do add seq flag <name>` — sequence flag (unbounded)
- `do add seq flag <name> limit <n>` — sequence flag with a final step
- `do remove flag <name>` | `do reset flag <name>` | `do advance flag <name>`
- `do award points <number>` (negative allowed)

Item/NPC/world state:
- Spawn/Despawn:
  - `do spawn item <item> into room <room>`
  - `do spawn item <item> into container <container>`
  - `do spawn item <item> in inventory`
  - `do spawn item <item> in current room`
  - `do despawn item <item>`
- Exits/Locks:
  - `do reveal exit from <from_room> to <to_room> direction <dir>`
  - `do lock exit from <from_room> direction <dir>`
  - `do unlock exit from <from_room> direction <dir>`
  - `do lock item <item>` | `do unlock item <item>`
  - `do set barred message from <from_room> to <to_room> "Message…"`
- Items/NPCs:
  - `do set item description <item_sym> "New description…"`
  - `do npc says <npc> "Quote…"` | `do npc says random <npc>`
  - `do set npc state <npc> <state>`
  - `do npc refuse item <npc> "Reason…"`
  - `do give item <item> to player from npc <npc>`
- Player movement/restrictions:
  - `do push player to <room>`
  - `do restrict item <item>` (item can’t be taken right now)
  - `do deny read "Reason…"`
- Spinners (styled random lines):
  - `do add wedge "Text…" width <n> spinner <spinner>`
  - `do spinner message <spinner>`

Scheduling:
- Fire in N turns (no condition):
  - `do schedule in <n> { … }`
- Fire on an absolute turn (no condition):
  - `do schedule on <turn> { … }`
- Conditionally fire later, with retry policy:
  - `do schedule in <n> if <condition> onFalse <policy> note "<str>" { … }`
  - `do schedule on <turn> if <condition> onFalse <policy> note "<str>" { … }`
- `onFalse` policy options:
  - `cancel` — drop the event if condition is false at fire time
  - `retryAfter <n>` — reschedule n turns later
  - `retryNextTurn` — try again next turn
- `note` is optional and appears in `:sched` developer output.

Examples:

```amble
# Unconditional: in 2 turns
do schedule in 2 {
  do show "A faint hum grows louder."
}

# Conditional with retries
do schedule in 1 if player in room lobby onFalse retryNextTurn note "ambient-lobby-chime" {
  do show "A distant chime rings."
}

# Absolute turn with cancel if not met
do schedule on 20 if any(player in room hall, player in room kitchen) onFalse cancel {
  do award points 5
}
```

**NOTE:** The turn counter is advanced immediately after events are scheduled, so an event scheduled in 1 turn will appear to fire with no delay. Use 2 or more for something to fire after an apparent delay.

## Sets for Ambients

Sets let you name and reuse room lists in ambient conditions.

- Declare a set:
  - `let set outside_house = (front-lawn, side-yard, back-yard)`
- Use it with `in rooms …`:
  - `if in rooms outside_house` — expands to the full list at compile time
  - You can mix sets and literal rooms: `outside_house,lobby`
- Currently, sets are scoped to ambient room lists.

Example (preferred syntax):

```amble
let set outside_house = (front-lawn, side-yard, back-yard)

trigger "Outside ambience" when always {
  if all(chance 20%, in rooms outside_house,lobby) {
    do spinner message ambientInterior
  }
}
```

Legacy: `ambient <spinner> in rooms …` remains supported for backward compatibility, but is redundant with the spinner action and may be deprecated.

## Patterns and Tips

- Use `when always` for ambient or status effects that aren’t tied to a specific player action.
- Prefer `all(…)`/`any(…)` for combining conditions; nest as needed.
- String literals:
  - Regular: `"..."` (supports escapes `\n`, `\t`, `\r`, `\"`, `\\`)
  - Single-quoted: `'...'` (same escapes as regular)
  - Raw: `r"..."` (no escapes)
  - Hashed raw: `r#"..."#`, `r##"..."##`, up to 5 `#`s for easy embedding of quotes
  - Triple-quoted: `"""..."""` (multiline; supports escapes)
- Identifiers exclude reserved words (e.g., `trigger`, `when`, `do`, `if`, `npc`, etc.). If you need a name that starts with a keyword, add more letters: `readable` is fine but `read` is reserved.
- Sequence flags:
  - Initialize with `do add seq flag quest limit 3` (creates `quest#0` with an end of 3)
  - Increment in actions with `do advance flag quest`
  - Reset with `do reset flag quest`
- “Container has” reads naturally: `if container toolbox has item wrench`
- Scheduling:
  - Start simple with `do schedule in 2 { … }`
  - Add `if` + `onFalse` when you need conditional delivery.
- Errors show line and column with a caret; the offending line is quoted to speed up fixing typos.

## CLI Usage

From the repo root:

```bash
# Compile a DSL file to TOML (stdout)
cargo run -p amble_script -- compile amble_script/examples/first.amble

# Compile to a file
cargo run -p amble_script -- compile amble_script/examples/third.amble --out /tmp/triggers.toml

# Run tests for the DSL crate
cargo test -p amble_script
```

## Reference Cheat‑Sheet

Trigger skeletons:

```amble
trigger "Name" when enter room <room> { if <condition> { <actions> } }
trigger "Name" when always { if <condition> { <actions> } }
```

Condition atoms:
- Flags: `has flag <name>` | `missing flag <name>` | `flag in progress <name>` | `flag complete <name>`
- Items: `has item <item>` | `missing item <item>` | `container <container> has item <item>`
- Location: `player in room <room>` | `has visited room <room>`
- NPC: `with npc <npc>` | `npc has item <npc> <item>` | `npc in state <npc> <state>`
- Random/ambient: `chance <n>%` | `in rooms <r1,r2,…>`
- Groups: `all(… , …)` | `any(… , …)` (nestable)

Action atoms:
- Feedback/flags/score: `do show …`, `do add flag …`, `do add seq flag … [limit n]`, `do remove/reset/advance flag …`, `do award points n`
- Spawns: `do spawn item … into room …`, `… into container …`, `… in inventory`, `… in current room`, `do despawn item …`
- Exits/locks: `do reveal/lock/unlock exit …`, `do lock/unlock item …`, `do set barred message from … to … "msg"`
- Items/NPCs: `do set item description … "…"`, `do npc says … "…"`, `do npc says random …`, `do set npc state … state`, `do npc refuse item … "…"`
- Player/world: `do push player to …`, `do restrict item …`, `do deny read "…"`
- Spinners: `do add wedge "…" width n spinner <spinner>`, `do spinner message <spinner>`
- Schedules: `do schedule in n { … }`, `do schedule on t { … }`, and `do schedule in/on … if <cond> onFalse <policy> note "…" { … }`

—

If you find yourself repeating a list of rooms for ambients, declare a `set` once and reuse it. If you need a recipe that isn’t here, we can extend the DSL — the intent is to make authoring fast, readable, and safe.

## Multiple If Blocks

You can have multiple top‑level `if { … }` blocks in a single trigger, plus plain `do …` lines not wrapped by an `if`.

- Each `if` block compiles to its own trigger sharing the same `when` event; it carries only that block’s conditions and actions.
- Plain `do …` lines outside any `if` compile to an additional unconditional trigger for the same event.
- If several blocks’ conditions are true on the same turn, each corresponding trigger can fire.
- `only once` applies to all triggers produced from the parent.

Example:

```amble
trigger "Arrival" when enter room lab {
  if has flag quest { do show "Welcome back" }
  if chance 30% { do show "A light flickers" }
  do spinner message ambientInterior
}
```

Compiles to three triggers with the same name and event: one for each `if` and one unconditional.
