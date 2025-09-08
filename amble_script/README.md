# amble_script

This crate defines a DSL for Amble content (triggers and rooms) and compiles it to the engine’s TOML files.

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

- Trigger DSL guide: `docs/trigger_dsl_guide.md`
- Rooms DSL guide: `docs/rooms_dsl_guide.md`
