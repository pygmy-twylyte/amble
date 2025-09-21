# Amble Script Creator Handbook

This handbook consolidates the practical information required to author Amble content with the `amble_script` DSL. It covers the end-to-end workflow, the CLI tooling, and the syntax for every entity the compiler understands: triggers, rooms, items, NPCs, spinners, and goals. Use it as your primary reference when designing new story content or migrating existing TOML definitions into the DSL.

If you only need a terse reminder of keywords and shapes, see the accompanying [DSL Cheat Sheet](./dsl_cheat_sheet.md).

---

## Authoring Workflow Overview

1. **Write DSL files** – author one or more `.amble` (or legacy `.able`) files that define triggers, rooms, items, NPCs, spinners, and goals.
2. **Validate with `lint`** – catch missing cross-references by running `amble_script lint` against the file or directory you are editing.
3. **Compile** – translate the DSL into engine-ready TOML via `amble_script compile` (single file) or `amble_script compile-dir` (directory tree).
4. **Load in the engine** – copy the generated TOML into `amble_engine/data/` or point the engine at the output directory.
5. **Iterate** – repeat the lint/compile cycle as you expand the world.

The compiler writes source provenance comments (file paths, line numbers, and content hashes) into every generated TOML table so that you can confidently trace runtime behaviour back to the DSL source.

---

## CLI Tooling

The `amble_script` binary ships inside this repository and can be run via `cargo run -p amble_script -- <command> …` or directly after `cargo install --path amble_script`.

### `compile`

Translate a single DSL file into one or more TOML outputs.

```bash
cargo run -p amble_script -- compile path/to/content.amble \
  [--out-triggers triggers.toml] \
  [--out-rooms rooms.toml] \
  [--out-items items.toml] \
  [--out-spinners spinners.toml] \
  [--out-npcs npcs.toml] \
  [--out-goals goals.toml]
```

Key details:

- When only triggers are present and no explicit `--out-triggers` path is provided, compiled triggers are printed to stdout for quick inspection. The same behaviour applies to rooms, spinners, NPCs, and goals when they are the only category present and no other outputs were written.
- `--out` still works as a deprecated alias for `--out-triggers` and prints a warning so you know to update scripts.
- The emitted TOML is prefixed with a generated header containing the source file path and an FNV-64 hash of the DSL to detect stale copies.

Typical uses:

- Convert an isolated prototype trigger file into `triggers.toml` before copying it into `amble_engine/data/`.
- Split a combined `.amble` file into separate outputs (`--out-triggers`, `--out-rooms`, …) to avoid manual editing.

### `compile-dir`

Batch-compile an entire directory tree of `.amble`/`.able` files.

```bash
cargo run -p amble_script -- compile-dir content/ --out-dir amble_engine/data \
  [--only triggers,rooms,items,spinners,npcs,goals] [--verbose|-v]
```

What it does:

- Recursively scans the source directory for DSL files, parses them, and merges all matching entity definitions.
- Writes one TOML file per category into the target `--out-dir`. When a category has no definitions, the tool still writes an empty skeleton (for example `triggers = []`) so that other generated files cannot go stale.
- Accepts `--only` to restrict which categories are written. Provide a comma-separated list (`--only triggers,items`) to leave other TOML files untouched.
- `--verbose` (or `-v`) prints per-file and summary counts, which is useful while refactoring a larger project.

Use `compile-dir` for day-to-day development once you maintain more than a handful of DSL files. It guarantees that every engine data file is regenerated together from the same source snapshot (the header includes the aggregated hash across all compiled files).

### `lint`

Validate references from DSL files against the engine data directory.

```bash
cargo run -p amble_script -- lint path/to/file.amble \
  [--data-dir amble_engine/data] [--deny-missing]
```

Highlights:

- Accepts either a single file or a directory; directories are walked recursively.
- Loads identifiers from the target `--data-dir` (defaults to `amble_engine/data`) so it can verify that exits point at existing rooms, trigger references mention valid items/NPCs, spinner IDs exist, etc.
- Reports each missing reference with file, line/column, and a caret indicator. The command exits with code 1 when `--deny-missing` is supplied and at least one issue was found—perfect for CI pipelines.

Run the linter before compiling to catch typos and outdated IDs early.

---

## Triggers

Triggers drive the bulk of interactive logic. They listen for a game event, optionally gate on additional conditions, and execute one or more actions.

### Skeleton

```amble
trigger "Friendly Greeter" when enter room lobby {
  note "First impressions"
  only once

  if missing flag greeted:lobby {
    do show "A concierge smiles warmly."
    do add flag greeted:lobby
  }

  if chance 20% {
    do spinner message ambientLobby
  }
}
```

- `note` is optional and copied into generated comments to help debugging.
- `only once` prevents the trigger (and any lowered clones—see below) from firing more than a single time.
- Each top-level `if { … }` block compiles into its own trigger entry; standalone `do …` lines outside of `if` become an unconditional variant.

### Events (`when …`)

The DSL supports a wide range of trigger events. A trigger fires when the player (or world) performs the described action:

- Room transitions: `enter room <room_id>`, `leave room <room_id>`
- Item interactions: `take item <item_id>`, `drop item <item_id>`, `look at item <item_id>`, `open item <item_id>`, `unlock item <item_id>`, `use item <item_id> ability <ability>`, `act <verb> on item <item_id>`, `insert item <item_id> into item <container_id>`, `take item <item_id> from npc <npc_id>`, `give item <item_id> to npc <npc_id>`
- NPC interactions: `talk to npc <npc_id>`
- Ambient/status: `always` (evaluated every turn against conditions)

### Conditions (`if …`)

Conditions refine when actions run. You can mix and nest logical groups:

- Flag tests: `has flag quest:started`, `missing flag door:open`, `flag in progress quest`, `flag complete quest`
- Inventory/world checks: `has item badge`, `missing item badge`, `container toolbox has item wrench`, `player in room lab`, `has visited room museum`
- NPC checks: `with npc guard`, `npc has item guard badge`, `npc in state guard alert`
- Randomised ambience: `chance 40%`, `in rooms lobby,atrium` (supports comma-separated lists and declared sets)
- Grouping: `all(cond1, cond2, …)` (AND), `any(cond1, cond2, …)` (OR). Nested groups are allowed.

Each condition group inside an `if` compiles into a flat list of engine conditions. `any(…)` groups are lowered into multiple triggers under the hood so you can use them freely.

### Actions (`do …`)

Actions describe the outcomes once all conditions pass. Common categories include:

- **Player feedback and flags:** `do show "…"`, `do award points 5`, `do add flag …`, `do add seq flag goal limit 3`, `do advance flag goal`, `do reset flag goal`, `do remove flag goal`
- **Item and NPC manipulation:** `do spawn item keycard into room security-office`, `do spawn item keycard into container locker`, `do spawn item kit in inventory`, `do despawn item vines`, `do set item description statue "…"`, `do set npc state guard alert`, `do npc says receptionist "We’re closed."`, `do npc says random receptionist`, `do npc refuse item receptionist "That’s not helpful."`, `do give item badge to player from npc guard`
- **World structure:** `do reveal exit from lab to hallway direction east`, `do lock exit from lobby direction north`, `do unlock exit from lobby direction north`, `do set barred message from lobby to vault "The door doesn’t budge."`, `do lock item locker`, `do unlock item locker`
- **Player movement & restrictions:** `do push player to infirmary`, `do restrict item trophy`, `do deny read "It’s encrypted."`
- **Spinners (ambient random lines):** `do spinner message ambientInterior`, `do add wedge "Clanging pipes" width 2 spinner ambientInterior`
- **Scheduling follow-up actions:**
  - Unconditional: `do schedule in 2 { … }`, `do schedule on 15 { … }`
  - Conditional: `do schedule in 1 if player in room lobby onFalse retryNextTurn note "ambient-chime" { … }`
  - Conditional absolute: `do schedule on 30 if has flag finaleReady onFalse cancel { … }`
  - `onFalse` policies: `cancel`, `retryAfter <turns>`, `retryNextTurn`

### Sets for Ambient Conditions

Reuse room lists in ambience triggers by declaring sets:

```amble
let set mezzanine = (lobby-balcony, mezzanine-west, mezzanine-east)

trigger "Ambient: creaking beams" when always {
  if all(chance 25%, in rooms mezzanine) {
    do spinner message ambientCreaks
  }
}
```

### Tips

- Use `when always` for periodic checks (status text, background events) instead of event-specific triggers.
- Remember that scheduling “in 1” turn fires almost immediately because the engine advances the turn counter right after evaluating triggers; use 2 or more for visible delays.
- Combine `note` fields with `:sched` developer commands in the engine to debug timed events.

---

## Rooms

Rooms provide the backdrop of the world. A room definition names the location, supplies a base description, and enumerates exits and overlays.

```amble
room lab-lobby {
  name "Research Lobby"
  desc "A crisp lobby hums with low machinery."
  visited false

  exit north lab-core hidden
  exit south atrium locked barred "The security door is sealed." required_items(keycard)

  overlay if flag set power:offline {
    text "Emergency lights bathe the lobby in red."
  }
}
```

Highlights:

- `visited` defaults to `false`; set it to `true` for starting rooms.
- `exit <direction> <room_id>` supports optional modifiers: `hidden`, `locked`, `barred "…"`, `required_items(item_a,item_b)`, and `required_flags(flag_a,flag_b#3)` (steps are normalised to the base flag name).
- Overlays let you swap or append flavour text when conditions hold. Supported overlay conditions mirror the engine’s room overlay system: flag set/unset/complete, item present/absent, player has/missing item, NPC present/absent/in state, and item-in-room checks.

---

## Items

Items represent objects the player can interact with, carry, or read.

```amble
item portal_gun {
  name "Portal Gun"
  desc "A compact device humming with potential."
  portable true
  location nowhere "Appears after calibrating the emitter"
  container state closed
  restricted false

  ability TurnOn
  ability Fire portal_emitter

  text "The housing still smells of ozone."
  requires insulate to handle
}
```

Key fields:

- `name`, `desc`, and `portable` are required.
- `location` accepts `inventory <owner>`, `room <room_id>`, `npc <npc_id>`, `chest <container_id>`, or `nowhere "note"` for items that spawn later.
- Optional container states: `open`, `closed`, `locked`, `transparentClosed`, `transparentLocked`.
- `restricted true` marks an item as non-droppable until explicitly allowed.
- Each `ability` entry becomes a `[[items.abilities]]` table with optional target (`ability Unlock vault_door`).
- `text` attaches readable flavour.
- `requires <ability> to <interaction>` gates interactions (e.g., require an item ability `cut` to perform the `open` interaction on this item).

---

## NPCs

NPC definitions describe characters, their starting location, state, optional movement, and dialogue banks.

```amble
npc receptionist {
  name "Receptionist"
  desc "Focused on a flickering terminal."
  location room lab-lobby
  state attentive

  movement random rooms (lab-lobby, atrium) timing slow active true

  dialogue attentive {
    "Welcome to the lab."
    "Please sign in."
  }

  dialogue custom emergency {
    "Please evacuate immediately!"
  }
}
```

Highlights:

- `location` accepts either a room ID or `nowhere "note"` for off-stage characters.
- `state` defaults to `normal` when omitted. Use `state custom <id>` for bespoke states that do not map to predefined engine enums.
- Movement supports `route` (default) or `random` with a list of rooms. Optional `timing <schedule_id>` selects an engine-defined timing, and `active true|false` toggles whether the routine starts running immediately.
- Dialogue blocks associate one or more lines with a state key. Use `dialogue custom panic { … }` for custom states; internally the compiler prefixes the key with `custom:` to match engine expectations.

---

## Spinners

Spinners are ambient random text selectors. Each spinner contains one or more wedges, each with text and an optional width (weight).

```amble
spinner ambientLobby {
  wedge "The HVAC sighs." width 2
  wedge "Footsteps echo from deeper inside."
}
```

When referenced from triggers (`do spinner message ambientLobby`), the engine rolls a wedge according to its weight.

---

## Goals

Goals describe high-level objectives presented to the player.

```amble
goal stabilize-reactor {
  name "Stabilize the Reactor"
  desc "Restore power to the facility."
  group required
  activate when has flag mission:assigned
  complete when flag complete reactor:calibration
}
```

Components:

- `group` categorises the goal: `required`, `optional`, or `status-effect`.
- `activate when …` is optional; when omitted, the goal is active from the start. Conditions accept `has flag`, `missing flag`, `has item`, `reached room`, `goal complete <other_goal>`, `flag in progress`, and `flag complete`.
- `complete when …` is required and uses the same condition vocabulary.

Goals compile into `goals.toml`, matching the engine schema for in-game goal tracking.

---

## Putting It Together

A typical content pack keeps all these entities side-by-side in one or more `.amble` files:

```amble
let set atrium_ring = (atrium-north, atrium-east, atrium-south, atrium-west)

# Rooms, items, NPCs, and triggers can live together in the same source file.
room atrium-north { … }
item security_badge { … }
npc guard { … }
spinner ambientAtrium { … }
goal restore-atrium { … }
trigger "Atrium ambience" when always { … }
```

Run `amble_script lint ./content --deny-missing` to ensure every reference is valid, then `amble_script compile-dir ./content --out-dir amble_engine/data` to regenerate the TOML the engine consumes.

For a fast reminder of syntax across all entities, keep the [DSL Cheat Sheet](./dsl_cheat_sheet.md) open while you work.
