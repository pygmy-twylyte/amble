# Amble
![Amble Logo](amble_logo.png)

## How It Started

I'm a long-term hobbyist developer since the Apple II+ was peak technology, and enjoy learning different languages, over the years including BASIC, 6502 Asm, Pascal, Perl, C++, Python... and I decided I wanted to learn Rust. So, I decided I'd create an 80's style parser game (think Zork) in Rust as a way to start building some Rust chops, and thought it would be fun to pack in as many references to things I like as I could along the way.

## What It Is Now

What started as a simple little game blossomed into a fully data-driven game engine that could be used for building any variety of adventures and a full set of developer tools for game creation, as well as a reasonably sized fully playable demo. It seemed a shame for me to be the only one to ever play or use it.

## Crates in this Repository
- `amble_engine` - loads game data either from TOML files or a saved state (in RON format) and runs the game
- `amble_script` - an intuitive, English-like language (DSL) for defining the game world, which is compiled into the TOML used by `amble_engine`
- `xtask` - commands to simplify linting / compiling and installing / packaging from .amble sources

## Optional (but nice!) External Repositories for Developers
- `tree-sitter-amble` - a tree-sitter parser / syntax highlighter for the amble_script DSL
- `zed-amble-ext` - a full-featured extension for the Zed editor with not only syntax highlighting but a language server that supports outlining, references / go to definition, symbol renaming, formatting, diagnostics, autocomplete -- the works.

## Engine Features

- Data-first design so stories live entirely in TOML, not code
- Rooms with conditional description overlays that can adapt to world state and connections that can be conditional, hidden, locked, or remapped entirely during play
- Items support a variety of capabilities (like "ignite" or "smash" or "turn on") and interaction types and can be consumable; items can also be containers and nested indefinitely
- NPCs supported with dialogue, trade options (via triggers), moods/states, and movement on either prescribed routes or randomly through a defined area
- Goals / Achievement system to help guide players to important objectives and mark progress
- Configurable point scoring system
- Customizable status effects
- In-game help system for players with built-in help for commands but customizable general help text.
- 2-stage loader that verifies and cross-references all symbols during world building
- Thorough logging of game and engine events enabled througout
- REPL-style parser with natural language verbs, synonyms, and DEV tools
- Powerful trigger/scheduler system for conditional, delayed, or repeating events
- Flexible flag model with sequence counters and derived logic helpers
- Themeable terminal UI with multiple palettes and optional styling
- Save system (RON full game state snapshots) for restoring worlds mid-adventure
- Comprehensive test suite and CLI harness for fast iteration

## Engine Development / Contributions
