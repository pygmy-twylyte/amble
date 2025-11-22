## Branch Plan · Engine Response Sequencing

### Current Status
- `ViewEntry` struct introduced and `View.items` now stores entries instead of raw `ViewItem`s.
- `View::push` (and any immediate callers) compile because the new wrapper mirrors the prior API, but no new metadata/policy has been added yet.
- No DSL / loader / scheduler changes in place; trigger actions still carry no priority hints.

### Goals
1. Allow authors (or the engine) to assign priorities to individual world-response emissions so narrative beats render in a sensible order.
2. Preserve existing section grouping, spinner behavior, and NPC sorting guarantees unless an override is explicitly provided.
3. Provide regression coverage (unit + integration) so sequencing stays predictable.

### Task Checklist

#### 1. Finish `ViewEntry` Plumbing (Engine)
- [ ] Audit every call site of `view.push` to ensure it now constructs a `ViewEntry` with the correct default `Section`. (Files: `amble_engine/src/view.rs`, `amble_engine/src/trigger/action.rs`, `amble_engine/src/repl/**/*.rs`, etc.)
- [ ] Add helper constructors (`ViewEntry::new(item: ViewItem)` or `View::push_item(item)`) to remove duplicated section/tag extraction.
- [ ] Introduce optional `priority: i32` and `order: u64` fields on `ViewEntry`.
  - `priority` defaults to 0 unless set by action metadata.
  - `order` auto-increments per `View` so stable ordering is preserved for equal priorities.

#### 2. Render Logic Updates (Engine)
- [ ] Update `View::flush` / section helpers to work over `ViewEntry`s directly (no `clone()` of `ViewItem`).
- [ ] For `WorldResponse` (and eventually any other section we care about), gather entries, sort by `(priority, order)`, then dispatch to the existing printers. Preserve alphabetical NPC grouping by applying their current secondary sort within a priority bucket.
- [ ] Add unit tests in `amble_engine/src/view.rs` verifying that custom priorities reorder responses without affecting section boundaries.

#### 3. Runtime API for Priorities
- [ ] Decide on API surface for runtime callers (e.g., `view.push_with_priority(item, priority)` or `ViewEntry::with_priority`).
- [ ] Modify trigger action handlers that emit `WorldResponse` items (`ShowMessage`, `NpcSays*`, `AwardPoints`, status changes, spinner messages, etc.) so they can accept an optional priority parameter (default 0 for backward compatibility).
- [ ] Extend `Trigger` to hold `Vec<ScriptedAction>` (struct containing `TriggerAction` + metadata).
- [ ] Update scheduler structs (`ScheduledEvent`, `schedule_in*`, etc.) so they carry the metadata through delayed execution.

#### 4. Authoring Pipeline (amble_script + loader)
- [ ] Extend DSL grammar to allow optional `priority <int>` qualifiers.
  - Syntax idea: `do priority -10 show "..."` OR `do show priority -10 "..."`.
  - Parser changes in `amble_script/src/parser.rs`, AST changes in `amble_script/src/lib.rs`.
- [ ] Thread the priority through `action_to_value`, `RawTriggerAction`, and `TriggerAction` conversion.
- [ ] Document the new syntax in `docs/` (DSL handbook) and add examples (e.g., hint radio).

#### 5. Regression Coverage & Fixtures
- [ ] Add parser tests for the new syntax (round-trip).
- [ ] Add loader tests ensuring TOML with/without `priority` deserializes correctly.
- [ ] Add engine integration test replicating issue #162 (ensure the “radio clicks on” line now precedes dialogue when given higher priority).
- [ ] Consider a golden-output test for `View::world_reaction`.

#### 6. Cleanup / Communication
- [ ] Update CHANGELOG / release notes summarizing the new capability.
- [ ] Leave notes in issue #162 referencing this plan when merging.
- [ ] Verify there are no unused helpers introduced during the scaffolding (clippy + fmt pass).
