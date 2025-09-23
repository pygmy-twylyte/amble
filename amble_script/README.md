# amble_script

This crate defines the Amble authoring DSL and compiles it to the engine’s TOML data files. It covers triggers, rooms, items, NPCs, spinners, and goals, and ships with a CLI for linting and compilation.

## Status

- Grammar: Minimal, Pest-based
- Supported forms:
  - `trigger "name" when enter room <ident> { ... }`
  - Inside `{}`:
    - `if missing flag <ident> { ... }`
    - `do show "..."`
    - `do add flag <ident>`
    - `do award points <number>`

## Build & Run

- Compile and run the CLI:

```
# Compile triggers only (stdout)
cargo run -p amble_script -- compile amble_script/examples/first.amble

# Compile triggers only to a file
cargo run -p amble_script -- compile amble_script/examples/third.amble --out /tmp/triggers.toml

# Compile rooms only to a file
cargo run -p amble_script -- compile amble_script/examples/rooms_demo.amble --out-rooms /tmp/rooms.toml

# Compile mixed file (triggers + rooms) to two outputs
cargo run -p amble_script -- compile amble_script/examples/rooms_demo.amble \
  --out /tmp/triggers.toml \
  --out-rooms /tmp/rooms.toml
```

Output:
- Triggers: Pretty TOML with `[[triggers]]` entries, each prefixed by `# trigger <name> (source line N)`.
- Rooms: Pretty TOML with `[[rooms]]` entries (`location = "Nowhere"` implicit), each prefixed by `# room <id> (source line N)`.
- File header comment indicates the source `.amble` path and that the file is generated (do not edit).

Lint cross-references (items/rooms/npcs/spinners):

```
cargo run -p amble_script -- lint amble_script/examples/rooms_demo.amble --data-dir amble_engine/data --deny-missing
```

The linter covers triggers and rooms (exit targets, overlay refs), suggests likely IDs for unknown references, and shows line/column with a caret.

## Testing

```
cargo test -p amble_script
```

Runs unit tests that parse the DSL and validate emitted TOML for triggers and rooms.

## Documentation

- **Creator Handbook:** [`docs/dsl_creator_handbook.md`](docs/dsl_creator_handbook.md) – complete walkthrough of the DSL, entity types, and CLI workflow.
- **DSL Cheat Sheet:** [`docs/dsl_cheat_sheet.md`](docs/dsl_cheat_sheet.md) – quick reference for keywords, shapes, and commands.
- Deep dives (legacy references):
  - Triggers: [`docs/trigger_dsl_guide.md`](docs/trigger_dsl_guide.md)
  - Rooms: [`docs/rooms_dsl_guide.md`](docs/rooms_dsl_guide.md)
  - Items: [`docs/items_dsl_guide.md`](docs/items_dsl_guide.md)
