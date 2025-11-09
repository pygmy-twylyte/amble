# amble_engine

`amble_engine` is a data-first interactive fiction engine and REPL that loads worlds compiled from the Amble DSL. It ingests TOML (or saved RON snapshots), cross-validates references, and drives a text adventure loop with helpful developer tooling.

## Features

- Layered room descriptions with conditional overlays, locks, hidden exits, and remapped connections.
- Items with capabilities (ignite, smash, turn on, etc.), arbitrary nesting, trade hooks, and scheduler-driven triggers.
- NPCs with dialogue trees, trade, movement, and stateful behavior.
- Goals, scoring, and configurable status effects to keep players oriented.
- Built-in help system, logging, save/load support, and developer commands for rapid iteration.

## Usage

```bash
# Run the demo content bundled with the repository
cargo run -p amble_engine

# Point to a custom data directory
cargo run -p amble_engine -- --data-dir path/to/toml
```

The engine expects category TOML files (`rooms`, `items`, `triggers`, `spinners`, `npcs`, `goals`, etc.) that can be produced by `amble_script`. Save files are written as RON archives under `saved_games/`.

## Developing Content

1. Author `.amble` sources in `amble_script/data/` (or any directory).
2. Compile to TOML with `cargo run -p amble_script -- compile-dir <src> --out-dir amble_engine/data`.
3. Launch the engine to explore and iterate.

## License

MIT License – see the repository root `LICENSE`.
