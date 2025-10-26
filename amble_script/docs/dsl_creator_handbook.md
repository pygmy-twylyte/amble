# Amble Script Creator Handbook

This handbook consolidates the practical information required to author Amble content with the `amble_script` DSL. It covers the end-to-end workflow, the CLI tooling, and the syntax for every entity the compiler understands: triggers, rooms, items, NPCs, spinners, and goals. Use it as your primary reference when designing new story content or migrating existing TOML definitions into the DSL.

If you only need a terse reminder of keywords and shapes, see the accompanying [DSL Cheat Sheet](./dsl_cheat_sheet.md).

---

## Authoring Workflow Overview

1. **Write DSL files** – author one or more `.amble` files that define triggers, rooms, items, NPCs, spinners, and goals. Files can be organized however you like - the .amble compiler recursively searches all directories under the root data directory.
2. **Compile TOML and Install** - using `cargo xtask content refresh` will compile, install, and lint all source .amble files using the amble_script CLI.
3. **Run the Engine** - There are a few different ways to do this, depending on whether you've installed the full repo or a pre-built package. If pre-built, you just run the program that you downloaded. If you're working from the cloned Amble repository, you can use `cargo run --bin amble_engine` from the workspace root. Or you can (re)build the engine with `cargo xtask build-engine --dev-mode enabled` which will build with developer game commands and debugging enabled. The executable will then be in ./target/debug/amble_engine (or amble_engine.exe if building for Windows).
4. **Iterate** - play with the additions and changes you've made to the game world, go back to the .amble sources and do more, refresh the content, build, and run again.


---

## CLI Tooling

(Note: the amble_script CLI can be used directly when more granular control over workflow  is needed, but most common build / install tasks are already automated and more easily called using the `cargo xtask` commands.)

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

### `compile-dir`

Batch-compile an entire directory tree of `.amble` files.

```bash
cargo run -p amble_script -- compile-dir content/ --out-dir amble_engine/data \
  [--only triggers,rooms,items,spinners,npcs,goals] [--verbose|-v]
```

What it does:

- Recursively scans the source directory for DSL files, parses them, and merges all matching entity definitions.
- Writes one TOML file per category (rooms.toml, items.toml, etc. - as expected by amble_engine) into the target `--out-dir`. When a category has no definitions, the tool still writes an empty skeleton (for example `triggers = []`) so that other generated files cannot go stale.
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

__Note: Because the linter uses the TOML files in the engine's data directory as a reference, the .amble source should be compiled and installed before linting, or you may get false reports on missing cross-references.__

---

## Triggers

Triggers drive the bulk of interactive logic. They listen for a game event or particular game state, optionally gate on additional conditions, and execute one or more actions.

### Skeleton

```amble
trigger "Friendly Greeter" only once note "First impressions"
when enter room lobby {
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
- Item-on-item interactions: `use item <tool_id> on item <target_id> interaction <interaction>`
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
- **Item/NPC/world state:**
  - Spawn/Despawn/Swap: `do spawn item keycard into room security-office`, `do spawn item keycard into container locker`, `do spawn item kit in inventory`, `do spawn item kit in current room`, `do spawn npc guard into room lobby`, `do despawn item vines`, `do despawn npc guard`, `do replace item glow-rod with drained-rod`, `do replace drop item badge with badge-fragments`
  - Exits/locks: `do reveal exit from lab to hallway direction east`, `do lock exit from lobby direction north`, `do unlock exit from lobby direction north`, `do lock item locker`, `do unlock item locker`, `do set barred message from lobby to vault "The door doesn’t budge."`
  - Items/NPCs: `do set item description statue "…"`, `do set container state locker locked`, `do npc says receptionist "We’re closed."`, `do npc says random receptionist`, `do npc refuse item receptionist "That’s not helpful."`, `do set npc state guard alert`, `do set npc active guard false`, `do give item badge to player from npc guard`
- **Player movement & restrictions:** `do push player to infirmary`, `do restrict item trophy`, `do deny read "It’s encrypted."`
- **Spinners (ambient random lines):** `do spinner message ambientInterior`, `do add wedge "Clanging pipes" width 2 spinner ambientInterior`
- **Scheduling follow-up actions:**
  - Unconditional: `do schedule in 2 { … }`, `do schedule on 15 { … }`
  - Conditional: `do schedule in 1 if player in room lobby onFalse retryNextTurn note "ambient-chime" { … }`
  - Conditional absolute: `do schedule on 30 if has flag finaleReady onFalse cancel { … }`
  - `onFalse` policies: `cancel`, `retryAfter <turns>`, `retryNextTurn`; `note "label"` tags the scheduled event for debugging.

You can also patch existing content with `do modify item|room|npc … { … }` blocks. They accept the same fields as the primary definitions—tweak names/descriptions, toggle `portable`/`restricted`, adjust container state (`container state off` clears it), add/remove abilities, exits, overlays, dialogue, or NPC movement (`route (…)`, `random rooms (…)`, `timing every 3 turns`, `timing on turn 5`, `active true|false`, `loop true|false`).

Need a deeper dive? See the [Trigger DSL Guide](./trigger_dsl_guide.md).

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

  exit "through the secret hall" -> lab-core {hidden}
  exit south -> atrium {
      locked
      barred "The security door is sealed."
      required_items(keycard)
      }

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
- `consumable { … }` configures limited-use items that despawn or transform after their charges are spent.

Consumable blocks support:

```amble
consumable {
  uses_left 2
  consume_on ability Use
  when_consumed replace inventory drained-portal-gun
}
```

- `uses_left <n>` sets the starting charge count (≥ 0).
- `consume_on ability <Ability> [<target>]` declares which abilities decrement the counter.
- `when_consumed …` chooses the depletion behaviour: `despawn`, `replace inventory <item>`, or `replace current room <item>`.

See the [Items DSL Guide](./items_dsl_guide.md) for exhaustive field coverage and emitted TOML structure.

---

## NPCs

NPC definitions describe characters, their starting location, state, optional movement, and dialogue banks.

```amble
npc receptionist {
  name "Receptionist"
  desc "Focused on a flickering terminal."
  location room lab-lobby
  state custom emergency

  movement random
    rooms (lab-lobby, atrium)
    timing every_3_turns
    active true
    loop false

  dialogue normal {
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
- Movement supports `route` (default) or `random` with a list of rooms. Optional `timing <schedule_id>` selects an engine-defined timing, `active true|false` toggles whether the routine starts running immediately, and `loop true|false` controls whether a route loops or stops at the final room.
- Dialogue blocks associate one or more lines with a state key. Use `dialogue custom panic { … }` for custom states; internally the compiler prefixes the key with `custom:` to match engine expectations.

For movement and dialogue patching examples, see the [NPCs DSL Guide](./npcs_dsl_guide.md).

---

## Spinners

Spinners are random text selectors. They are used by the engine to vary common messages (like command not found) to keep things fresh. There are defaults for each of those spinners, but they can be overridden by content creators simply by including a spinner with one of the core spinner identifiers. They can also be used to create ambient effects, status effects, bits of randomized dialogue, etc. Each spinner contains one or more wedges, each with text and an optional width (weight). Widths default to 1, so all wedges are equally likely to be selected if they are omitted (which is usually the desired behavior.)

```amble
spinner ambientLobby {
  wedge "The HVAC sighs." width 2
  wedge "Footsteps echo from deeper inside."
}
```

When referenced from triggers (`do spinner message ambientLobby`), the engine rolls a wedge according to its weight.

Check the [Spinners DSL Guide](./spinners_dsl_guide.md) for weighting tips and additional examples.

---

## Goals

Goals describe high-level objectives presented to the player.

```amble
goal stabilize-reactor {
  name "Stabilize the Reactor"
  desc "Restore power to the facility."
  group required
  start when has flag mission:assigned
  done when flag complete reactor:calibration
  fail when has flag mission:aborted
}
```

Components:

- `group` categorises the goal: `required`, `optional`, or `status-effect`.
- `activate when …` is optional; when omitted, the goal is active from the start. Conditions accept `has flag`, `missing flag`, `has item`, `reached room`, `goal complete <other_goal>`, `flag in progress`, and `flag complete`.
- `complete when …` is required and uses the same condition vocabulary.
- `fail when …` is optional and uses the same condition set to model failure states.

Goals compile into `goals.toml`, matching the engine schema for in-game goal tracking.

See the [Goals DSL Guide](./goals_dsl_guide.md) for condition details and compiler output samples.

---

## Debugging Toolkit

### Enable logging

The engine ships with structured logging you can enable without rebuilding:

```bash
AMBLE_LOG=info cargo run --bin amble_engine

# or if already built
AMBLE_LOG=info target/debug/amble_engine 
```

- Valid levels: `error`, `warn`, `info`, `debug`, `trace` (case-insensitive). Use `AMBLE_LOG=off` or unset the variable to disable logging.
- By default logs are written to `amble-VERSION.log` alongside the executable (for `cargo run`, this ends up in `target/debug/`).
- Override the destination with `AMBLE_LOG_OUTPUT`:
  - `AMBLE_LOG_OUTPUT=stderr` or `stdout` streams directly to the console.
  - Any other value (or unset) keeps the default file sink. Set `AMBLE_LOG_FILE=/custom/path/amble.log` to choose a specific file.

Logging is invaluable when tracing trigger evaluations, scheduler activity, and DEV commands (all DEV commands emit `warn`-level entries). `info`-level logging is most useful when trying to examine game flow in general. `warn`-level is best if you only want unusual (but recoverable) error states and any DEV command usage logged.

### Developer commands (`DEV_MODE`)

Interactive developer commands let you bend the world for fast testing. Build or run the engine with the `dev-mode` feature to enable them:

```bash
cargo run -p amble_engine --features dev-mode

# if using the xtask
cargo xtask build-engine --dev-mode enable
```

Once loaded, `:help dev` lists all commands; the most common are summarized below:

| Command | Purpose |
| --- | --- |
| `:teleport <room>` / `:port <room>` | Jump to any room by its symbol. |
| `:spawn <item>` / `:item <item>` | Move an item into the player inventory (spawns it if it is 'Nowhere'). |
| `:npcs` | List every NPC with current location and state. |
| `:flags` | Dump all flags on the player (`sequence#step` format). |
| `:sched` | Show scheduled events with due turn, note, and on-false policy. |
| `:schedule cancel <idx>` | Cancel a scheduled event by index (from `:sched`). |
| `:schedule delay <idx> <+turns>` | Push a scheduled event forward by N turns. |
| `:set-flag <flag>` | Create/set a simple flag immediately. |
| `:init-seq <flag> <limit|none>` | Create a sequence flag with an optional max step. |
| `:adv-seq <flag>` | Advance a sequence flag by one step. |
| `:reset-seq <flag>` | Reset a sequence flag to step 0. |

All developer actions log at `warn` level (see logging section above) so you can audit what changed during a test session.

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
