# Amble

<picture>
  <source srcset="amble_logo.png" />
  <img src="amble_logo.png" alt="Amble Logo" width="260" />
</picture>

Data‑first interactive fiction engine and authoring DSL in Rust.

## How It Started

I wanted to learn Rust and thought an 80s‑style parser adventure (think Zork) would be the perfect project. A small game prototype quickly snowballed into a data‑driven engine, a friendly DSL, and tooling to build real adventures.

## What It Is Now

Amble is a data‑first engine that loads worlds from compiled `WorldDef` data (`world.ron` or `worlds/*.ron`), plus a DSL and compiler that make worldbuilding fast and readable. The repo includes developer tools and a fully playable demo you can run immediately.

## DSL Spotlight

The Amble DSL is designed to read like plain English while staying precise and composable.

<img src="amble_zed_dsl_screenshot.png" alt="Amble DSL in Zed with syntax highlighting" width="950" />

- Start with `amble_script/docs/dsl_creator_handbook.md` for the language tour.
- For rich editing, see the [Zed Amble extension](./zed_extension.md) - powered by [tree-sitter-amble](https://github.com/pygmy-twylyte/tree-sitter-amble)

## Quickstart

### Play Now (prebuilt ZIP)
- Download the appropriate ZIP from Releases.
- Extract and run `amble_engine` (use `amble_engine.exe` on Windows).
- Type `help` in the game REPL for commands. Saves go to `saved_games/<world>/`.

### Build & Tinker (Rust toolchain)
1. Install the latest stable Rust toolchain.
2. Clone this repository and `cd` into it.
3. Run the engine with the bundled content:
   `cargo run -p amble_engine`
4. Use `help` in the REPL; saves land in `saved_games/<world>/`.

### Author New Content
1. Explore the DSL guides in `amble_script/docs/`—start with `dsl_creator_handbook.md`.
2. Compile the sample DSL to `world.ron`:
   `cargo run -p amble_script -- compile-dir amble_script/data/Amble --out-dir amble_engine/data`
   (Use `--out-world amble_engine/data/worlds/<slug>.ron` to add multiple worlds.)
3. Launch the engine to test your changes:
   `cargo run -p amble_engine`
4. Iterate with `amble_script lint …` to catch missing references early.

## Crates in this Repository
- `amble_engine` - loads world data from `world.ron` or `worlds/*.ron` (plus TOML configs for themes/scoring/help) or a saved state, then runs the game
- `amble_script` - an intuitive, English-like language (DSL) for defining the game world, which compiles into `WorldDef` (RON) used by `amble_engine`
- `amble_data` - world data model, shared between `amble_engine` and `amble_script`.
- [`xtask`](../xtask/README.md) - automation helpers for builds, packaging, and the content pipeline

## Optional (but nice!) External Repositories for Developers
- `tree-sitter-amble` - a tree-sitter parser / syntax highlighter for the amble_script DSL
- `zed-amble-ext` - a full-featured extension for the Zed editor with syntax highlighting and a language server (supports outlining, references / go-to-definition, symbol renaming, formatting, diagnostics, autocomplete -- development still ongoing.)

## Engine Features

- Data-first design so stories live entirely in `.amble` -> `world.ron` (or `worlds/*.ron`) data, not code
- Rooms with conditional description overlays that can adapt to world state and connections that can be conditional, hidden, locked, or remapped entirely during play
- Items support a variety of capabilities (like "ignite" or "smash" or "turn on") and interactions, and can be consumable; items can also be containers and nested to an arbitrary depth.
- NPCs supported with dialogue, trade options (via triggers), moods/states, and movement on either predetermined routes or randomly through a defined area
- Goals / Achievement system to help guide players to important objectives and mark progress
- Configurable point scoring system
- Customizable status effects
- In-game help system for players (built-in help for commands with customizable general help text.)
- WorldDef validation plus a placement pass that seeds initial room/NPC/item locations
- Thorough logging of game and engine events enabled throughout
- REPL-style parser with natural language verbs, synonyms, and DEV tools
- Powerful trigger/scheduler system for conditional, delayed, or repeating events
- Flexible flag model with sequence counters and derived logic helpers
- Themeable terminal UI with multiple palettes and optional styling
- Simple terminal markup allows additional styling for emphasis on top of the active theme (but also theme-aware)
- Save system (RON full game state snapshots) for restoring worlds mid-adventure
- Comprehensive test suite and CLI for fast iteration

## Engine Development / Contributions
- Any ideas / comments or contributions welcome!
- I check the repo Issues and Discussions regularly.
