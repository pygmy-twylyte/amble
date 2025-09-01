# amble_script

This crate defines a DSL for Amble triggers. In this first version, it supports only a single trigger with a missing-flag condition and a few simple actions. Later versions will expand this grammar.

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
cargo run -p amble_script -- compile amble_script/examples/first.amble
# or with output file
cargo run -p amble_script -- compile amble_script/examples/third.amble --out /tmp/triggers.toml
```

- Output: Pretty TOML containing a single `[[triggers]]` entry compatible with the Amble engine’s RawTrigger schema.

## Testing

```
cargo test -p amble_script
```

This runs a unit test that parses the DSL and validates the constructed AST and emitted TOML.
