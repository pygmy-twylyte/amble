# Roadmap: Adding Rooms to amble_script DSL

This plan introduces Room authoring to the amble_script DSL and compiles it to the engine’s `rooms.toml`, mirroring `amble_engine/src/room.rs` and `amble_engine/src/loader/rooms.rs`.

Notes agreed up front:
- Rooms always have `location = "Nowhere"` in the engine. The DSL omits `location`; the compiler emits `location = "Nowhere"` in TOML.
- `visited` defaults to `false`. Keep an option to set `visited true` explicitly (e.g., for the start room).

## Target Syntax (incremental)

Minimal room (core fields only):

```
room high-ridge {
  name "High Isolated Ridge"
  desc """A small, flat ridge..."""
  # visited defaults to false
}
```

With exits and overlays:

```
room two-sheds-landing {
  name "Jackson's Landing"
  desc """A quiet landing tucked along the slope..."""
  visited true

  # Basic exits
  exit up -> guard-post
  exit down -> parish-landing
  exit east -> two-sheds {
    # Exit constraints (all optional)
    locked,
    hidden,
    barred "You'll need to clear the tree from the path first.",
    required_flags(simple cleared-fallen-tree),
    required_items(machete, gasoline)
  }

  # Overlays (all-of conditions)
  overlay if flag set cleaned-plaque-1 {
    text "You're occasionally blinded by a flash of sunlight..."
  }
  overlay if item present margarine {
    text "On the pedestal sits a lone tub of margarine..."
  }
  overlay if npc in state cmot_dibbler happy {
    text "Dibbler hums a sales jingle..."
  }
}
```

Condition shorthands map directly to loader’s `RawOverlayCondition`:
- `flag set/unset/complete <name>`
- `item present/absent <item_sym>`
- `player has/missing item <item_sym>`
- `npc present/absent <npc_sym>`
- `npc in state <npc_sym> <state>` (supports enum strings and custom)
- `item in room <item_sym> <room_sym>`

## Deliverables by Increment

1) Read/Align Target Format
- Review `amble_engine/data/rooms.toml`, `room.rs`, and `loader/rooms.rs`.
- Decide DSL surface that mirrors TOML faithfully but is concise.
- Acceptance: Written spec + example DSL mapping 1–2 rooms to identical TOML.

2) Grammar Scaffolding (rooms-only)
- Extend grammar: `program = { (set_decl | trigger | room)+ }`.
- Room core: `room <ident> { name <string>; desc <string>; visited (true|false)? }`.
- No `location` token in DSL; compiler sets `Nowhere` in output TOML.
- Acceptance: Parse `.amble` with simple rooms into `RoomAst` without exits/overlays.

3) AST + TOML Emission (minimal)
- Add `RoomAst`, `ExitAst`, `OverlayAst` types.
- Implement `compile_rooms_to_toml(&[RoomAst]) -> String` using `toml_edit` and emit `[[rooms]]` blocks with `location = "Nowhere"` and `visited` defaulting to `false` when omitted.
- CLI: support `--out-rooms <path>` (non-breaking; existing triggers continue to use `--out`).
- Acceptance: Golden test: minimal rooms.amble -> rooms.toml (name, base_description, location, visited) matches fixture.

4) Exits (basic)
- Grammar: `exit <direction> -> <room_id>`.
- AST: `ExitAst { to, hidden: bool=false, locked: bool=false, required_flags: [], required_items: [], barred_message: None }`.
- TOML: nested tables `[rooms.exits.<direction>]` with `to = "<id>"`.
- Acceptance: Two directions compile to expected TOML structure.

5) Exit Requirements + Barred Message
- Grammar (optional block): `{ hidden, locked, barred <string>, required_flags(simple f1, seq f2#2), required_items(i1, i2) }`.
- Emit `required_flags` as array of `Flag` inline tables (shape used in engine); `required_items` as string symbols (engine resolves to UUIDs later).
- Acceptance: Match examples like `two-sheds-landing` → `required_flags` + `barred_message` identical.

6) Overlays (core)
- Grammar: `overlay if <cond> ("," <cond>)* { text <string> }` with all-of semantics.
- Conditions supported initially: flag set/unset/complete; item present/absent; item in room; npc present/absent; npc in state; player has/missing item.
- Map directly to loader `RawOverlayCondition` variants with camelCase `type` tags.
- Acceptance: Overlay arrays compile to expected `conditions = [ { type = ... } ]` with `text`.

7) Condition Coverage + Ergonomics
- Add `item in room <item> <room>` to disambiguate `ItemInRoom`.
- Accept `npc in state <npc> { custom = "..." }` via plain string for now; map to loader enum (including custom via serde’s variant structure) per engine expectations.
- Acceptance: Each overlay condition variant has a unit test verifying exact TOML.

8) Lint: Cross-Reference Rooms
- Extend lint to collect from rooms DSL:
  - defined room ids,
  - exit targets (`to`),
  - overlay refs (item_id, npc_id, room_id),
  - exit.required_items and required_flags names.
- Reuse world refs loader; merge rooms defined in current DSL with those already in `rooms.toml` under `--data-dir` so exits can reference new rooms in the same file.
- `--deny-missing` turns warnings into errors.
- Acceptance: Helpful diagnostics (unknown room/item/npc), with suggestions where applicable.

9) CLI Workflow + Docs
- CLI: allow a single `.amble` containing triggers and rooms; emit either or both depending on presence:
  - `--out-triggers <path>` and `--out-rooms <path>`.
  - If only one kind parsed, emit only that; warn if neither.
- Docs: add a Rooms DSL guide with syntax, examples, and TOML mapping; update README usage examples.
- Acceptance: Example usage compiles to rooms.toml identical to selected slices in engine data.

10) Integration Sanity
- Optional helper script to copy compiled `rooms.toml` into `amble_engine/data/` and run the engine for a smoke test.
- Acceptance: Engine loads compiled rooms without loader errors; basic REPL navigation shows expected names/descriptions.

11) Nice-to-haves (later)
- Exit sugar: `exit north to hall locked "Need key" required_flags(simple door-unlocked)` as a single-line form.
- Overlay templates/aliases for repeated texts.
- Reverse-translation tool: rooms.toml → DSL (migration aid).

12) Testing Strategy
- Unit tests per feature increment:
  - Grammar parsing for room core, exits, options, overlays, and each condition.
  - TOML emission checks using `toml_edit` structure rather than only string contains.
- Lint tests for missing targets/ids and `--deny-missing` behavior.
- Keep string assertions ANSI-free as with existing tests.

## Emission Details (TOML)
- Room table: `[[rooms]]`, `id`, `name`, `base_description`, `location = "Nowhere"`, optional `visited`.
- Exits: `[rooms.exits.<direction>]` with fields `to`, optional `hidden`, `locked`, `required_flags`, `required_items`, `barred_message`.
- Overlays: `[[rooms.overlays]]` with `conditions = [ { type = "..." } ]` and `text`.

## Acceptance Summary
- Minimal (Steps 2–3): parse + emit single room with name/desc/visited, location omitted in DSL but present in TOML as `Nowhere`.
- Exits (Steps 4–5): directions and options compile to exact nested tables.
- Overlays (Step 6–7): all supported conditions compile to loader’s shape.
- Lint (Step 8): clear, actionable cross-ref messages.

