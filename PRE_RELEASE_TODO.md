# Amble Pre-Release TODO

Snapshot of engine and DSL work items to address before writing public-facing documentation and opening the repository.

## Core Gameplay Gaps
- Add full “turn off” support (command grammar, handler, trigger condition) so `ItemAbility::TurnOff` is usable in play (`amble_engine/src/item.rs:323`, `amble_engine/src/command.rs:57`, `amble_engine/src/repl_grammar.pest:39`, `amble_engine/src/repl/item.rs:416`).
- Introduce a trigger action to re-hide exits after they are revealed, complementing the existing `RevealExit` flow (`amble_engine/src/trigger/action.rs:195`).
- Improve save loading UX by enumerating existing saves or picking reasonable defaults rather than requiring exact filenames (`amble_engine/src/repl/system.rs:632`).
- Gate development commands behind a feature flag or runtime toggle; `DEV_MODE` should not default to `true` in release builds (`amble_engine/src/lib.rs:22`).

## Engine & DSL Parity
- Add `ModifyNpc` and `ModifyRoom` trigger actions (and corresponding loader support) that parallel `ModifyItem`, enabling data-driven updates to NPCs and rooms (`amble_engine/src/trigger/action.rs:181`).
- Extend the DSL/compiler so every engine-side trigger action—especially new mutate/hide/turn-off features—can be authored without hand-editing TOML (`amble_script/src/lib.rs:187`).
- Backfill integration coverage that drives recent features (item patches, conditional schedules, future NPC/room patches) through the full loader/trigger pipeline (`amble_engine/tests/schedule_conditional_toml.rs`).

## Release Engineering
- Create build/package scripts (or an `xtask`) that produce ready-to-ship bundles per platform, including required `data/` and theme assets (only `amble_engine/tools/regenerate_snippets.rs` exists today).
- Define a saved-game compatibility strategy—migration tooling or documented guidance—since mismatched versions currently just emit a warning (`amble_engine/src/repl/system.rs:638`).
