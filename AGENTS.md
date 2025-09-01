# AI Agent Guidelines for Amble

This document provides context and guidelines for AI agents working on the Amble text adventure game engine.

## Project Overview

Amble is a text-based adventure game engine written in Rust, featuring:
- **REPL-style command interface** for player interaction
- **TOML-based content authoring** for game designers
- **Sophisticated trigger/event system** for game logic
- **Developer debugging tools** (DEV_MODE commands)
- **Theme system** for terminal styling and colors

## Project Structure

```
amble/
├── amble_engine/          # Core game engine (Rust library)
│   ├── src/
│   │   ├── main.rs        # CLI entry point
│   │   ├── repl/          # Command parsing and handling
│   │   │   ├── dev.rs     # Developer-only commands
│   │   │   └── ...
│   │   ├── player.rs      # Player state and flags
│   │   ├── world.rs       # Game world state
│   │   └── trigger/       # Event system
│   ├── data/             # Game content (TOML files)
│   └── tests/            # Integration tests
├── amble_editor/         # Content creation tools
├── examples/             # Sample games and content
└── saved_games/          # Player save files
```

## Core Concepts

### Flags
- **Simple Flags**: Boolean states (present/absent)
- **Sequence Flags**: Multi-step counters with optional limits
- Format: Simple = `"flag_name"`, Sequence = `"flag_name#step"`
- Primary mechanism for tracking game state and progress

### Triggers
- Event-driven system that responds to player actions
- Conditions determine when triggers activate
- Actions define what happens when triggered
- Supports scheduling future events

### DEV_MODE Commands
- Special debugging commands prefixed with `:`
- Only available when `DEV_MODE = true`
- Examples: `:teleport`, `:spawn`, `:adv-seq`, `:set-flag`
- Must provide clear feedback on success/failure

### Conditional Scheduled Events (TOML)
- Schedule future actions that only fire when a condition is true.
- If false at the due turn, control behavior with `onFalse`:
  - `Cancel` (drop the event)
  - `RetryAfter { turns = N }` (reschedule N turns later)
  - `RetryNextTurn` (try again next turn)

New action types in `triggers.toml`:
- `scheduleInIf` – schedule relative to now
- `scheduleOnIf` – schedule on an absolute turn

Example: retry each turn until player is in lobby, then show a message

[[triggers]]
name = "Lobby: Ambient ping if present"
conditions = []
actions = [
  { type = "scheduleInIf",
    turnsAhead = 1,
    condition = { type = "inRoom", room_id = "lobby" },
    onFalse = { type = "retryNextTurn" },
    actions = [
      { type = "showMessage", text = "A distant chime rings." }
    ],
    note = "ambient-lobby-chime"
  }
]

Example: absolute turn, any-of rooms, cancel if false

[[triggers]]
name = "Bonus if still nearby"
conditions = []
actions = [
  { type = "scheduleOnIf",
    onTurn = 20,
    condition = { any = [
      { type = "inRoom", room_id = "hall" },
      { type = "inRoom", room_id = "kitchen" }
    ]},
    onFalse = { type = "cancel" },
    actions = [
      { type = "awardPoints", amount = 5 }
    ],
    note = "bonus-nearby"
  }
]

You can also nest `all` and `any` for more complex conditions:

condition = { all = [
  { type = "hasFlag", flag = "quest-started" },
  { any = [
      { type = "inRoom", room_id = "lab" },
      { type = "inRoom", room_id = "lobby" }
  ]}
]}

### Game Data
- Content stored in TOML files under `amble_engine/data/`
- Uses UUID-based IDs generated from string symbols
- Items, rooms, NPCs, and triggers all defined declaratively

## Scheduler
- **Firing:** Events fire when `current_turn >= on_turn`.
- **Rescheduling:** Applying `onFalse` creates a new event and leaves a tombstone at the old index.
- **Notes:** `note` helps trace events and appears in `:sched` output.
- **Conditions:** `EventCondition` can be a single trigger (`TriggerCondition`) or `all`/`any` combinations; evaluated at fire time.
- **Chance:** `Chance` conditions re-roll on each evaluation.

## Turn Accounting
- **Counts as a turn:** open/close/drop/give/lookAt/lock/move/open/putIn/read/take/takeFrom/talk/turnOn/unlock/useItemOn.
- **Where:** Turn increments once per accepted command in the REPL loop (see `repl.rs`).
- **Order:** After increment → NPC movement → scheduled events → ambient checks.

## DEV Scheduling Tools
- `:sched` lists due turn, index, action count, `on_false`, `cond`, `note`.
- `:schedule cancel <idx>` cancels by index.
- `:schedule delay <idx> <+turns>` requeues later and clears the original entry.

## Testing Tips
- Disable ANSI colors when asserting strings in tests.
- Advance `world.turn_count` and call `repl::check_scheduled_events` to exercise schedules.
- Verify both success (fires) and failure (no-op/reschedule) paths.

## Performance Notes
- Scheduler compacts tombstones automatically when placeholders exceed a threshold.
- Prefer bounded retries for hints (`RetryAfter`) or `Cancel` to avoid runaway queues.

## Loader Extensibility
- Add new TOML actions in `loader/triggers/raw_action.rs` and map to `TriggerAction`.
- Add new conditions in `loader/triggers/raw_condition.rs` and map to `TriggerCondition`.
- Keep engine features reachable via TOML to preserve data-driven content.

## Development Guidelines

### Code Patterns
- **Error Handling**: Use `ViewItem::ActionFailure` for user-facing errors
- **Success Feedback**: Use `ViewItem::ActionSuccess` for confirmations  
- **Logging**: Use `warn!()` for important actions, `info!()` for details
- **Testing**: Add unit tests for all new public functions

### Common Workflows
```bash
# Development
cargo build                    # Compile project
cargo test                    # Run all tests
cargo test --lib              # Unit tests only
cargo test repl::dev::        # Specific module tests

# Debugging
cargo run --bin amble_engine  # Start game with current content
```

### Key Types and Functions
- `AmbleWorld` - Main game state container
- `Player` - Player state, inventory, and flags
- `View` - Output display system for user feedback
- `Flag::simple(name, turn)` - Create boolean flag
- `Flag::sequence(name, limit, turn)` - Create counter flag

## For AI Agents: Important Patterns

### Always Check Existence First
```rust
// ❌ Don't assume items/flags exist
world.player.advance_flag(flag_name);

// ✅ Check first, provide feedback
if world.player.flags.contains(&target) {
    world.player.advance_flag(flag_name);
    // success feedback
} else {
    // error feedback with helpful suggestion
}
```

### Error Messages Should Be Helpful
```rust
// ❌ Generic error
"Flag not found"

// ✅ Actionable error with suggestion
"No sequence flag 'quest_progress' found. Use :init-seq to create it first."
```

### Use Existing Patterns
- Look at `/src/repl/` handlers for command processing examples
- Check `/src/repl/dev.rs` for DEV command patterns
- Study existing tests for testing approaches

### Common Gotchas
- `Flag.value()` returns `"name#step"` format, not just the step number
- ANSI color codes in styled text affect string matching in tests
- DEV commands bypass normal game restrictions - document this clearly
- Always verify existing tests still pass after changes

## Testing Strategy

### Required for New Features
- Unit tests for all public functions
- Error case testing (missing flags, invalid IDs, etc.)
- Success case verification
- No breaking changes to existing functionality

### Test Locations
- Unit tests: inline `#[cfg(test)]` modules
- Integration tests: `/tests/` directory
- Current coverage: 285+ passing tests

## Issue Management

- **GitHub Issues** track bugs and features
- **Labels** indicate scope: `engine`, `content`, `Small Job`, `Medium Job`
- **Branches** use descriptive names like `fix-dev-command-feedback`
- **PRs** should reference issue numbers and include testing notes

## Current Development Focus

- **Developer experience improvements** (better error messages, tooling)
- **Content creation tools** (editor, validation)
- **Engine stability** (comprehensive testing, edge cases)
- **Performance optimization** (where needed)

## Examples of Well-Implemented Features

### Good DEV Command Pattern
See `dev_set_flag_handler()` in `/src/repl/dev.rs`:
- Clear documentation with examples
- Proper error handling and user feedback
- Comprehensive logging
- Unit tests for both success and failure cases

### Good Trigger Implementation  
See trigger actions in `/src/trigger/action.rs`:
- Consistent error handling patterns
- Detailed logging for debugging
- Integration with world state systems
- Comprehensive test coverage

## Red Flags to Avoid

- Silent failures (no user feedback)
- Assumptions about data existence
- Breaking changes without test updates
- Generic or unhelpful error messages
- Missing documentation for public APIs

---

*This document is intended to help AI agents understand the codebase context, patterns, and development practices. Update it as the project evolves.*
