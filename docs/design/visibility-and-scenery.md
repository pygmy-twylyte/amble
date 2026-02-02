# Item Visibility, Scenery, and Hidden Exits

Status: Draft
Last updated: 2026-02-02
Owner: @pygmy-twylyte

## Summary
Add a visibility system that supports:
- Scenery items: discoverable via "look at" and interactions, but not listed in room/container inventories.
- Hidden items: not discoverable until their visibility conditions are met.
- Hidden exits: not listed and not traversable until revealed (current `hidden` flag becomes meaningful).
- Item aliases and room-level default scenery to reduce “word not understood” misses.

This is intentionally smaller than a full lighting system, but still enables lighting-like behavior via conditions.

## Goals
- Allow authors to add descriptive scenery without cluttering the room item list.
- Allow items to be hidden and only discoverable when conditions are met.
- Make hidden exits actually hide from the exit list and block traversal until revealed.
- Keep scenery touchable (usable in puzzles/interaction) while still not listed.
- Improve parser matching with lightweight aliases and default scenery responses.
- Minimize breaking changes to existing content.

## Non-goals
- Full lighting model (light level, emitters, on/off state). This can be layered on later.
- New parser commands or verbs.
- A full NLP/synonym system beyond simple aliases.

## Definitions
- **Listed**: Item appears in auto-generated room/container lists.
- **Scenery**: Item does not appear in lists but can be discovered with `look at` if visible.
- **Hidden**: Item cannot be discovered or interacted with until visible conditions are met.
- **Visible**: A per-item runtime check derived from `visible_when` conditions.
- **Alias**: Alternate text that can match an item in parser search.
- **Default scenery**: Room-local nouns that return a generic or custom “nothing special” response.

## Current Behavior (Baseline)
- Room "look" lists all items in `Room.contents` with no filtering.
- Visibility is container-based only (open/transparent containers). No per-item visibility exists.
- Hidden exits exist as data but are still listed in the exit list.

Relevant code:
- `Room::show` lists items unconditionally: `amble_engine/src/room.rs:174`.
- Visibility scopes: `nearby_visible_items` and `nearby_reachable_items` in `amble_engine/src/world.rs:196`.
- Search scopes in `amble_engine/src/entity_search.rs:46`.
- Hidden exits: `Exit.hidden` in `amble_engine/src/room.rs:22`.

## Proposed Data Model

### ItemVisibility
Add an enum that controls listing and discoverability.

```
enum ItemVisibility {
    Listed,   // default
    Scenery,  // discoverable but not listed
    Hidden,   // not discoverable until visible_when passes
}
```

### Item Visibility Conditions
Add `visible_when: Option<ConditionExpr>` to `ItemDef` / `Item`.
- Evaluated against world state to decide if a Hidden item becomes visible.
- For Listed/Scenery items, `visible_when` can still be used to conditionally hide them.

Condition expression already exists in `amble_data::ConditionExpr`.

### Item Aliases
Add `aliases: Vec<String>` to `ItemDef` / `Item` for alternate search terms.

### Room Default Scenery
Add room-local scenery entries that are not items but can satisfy `look at`:

```
struct RoomSceneryDef {
    name: String,
    desc: Option<String>,
}
```

Room-level optional default response:

```
scenery_default: Option<String>
```

If a matched scenery entry has no `desc`, use `scenery_default` if present, otherwise a built-in fallback:
`"You see nothing remarkable about the {thing}."`

### Exit Visibility
No new exit model for now. Instead:
- Interpret existing `Exit.hidden` as “do not list or allow traversal until revealed.”
- `RevealExit` (already implemented) flips `hidden` to false.

After reveal, traversal is still gated by `locked`, `required_flags`, and `required_items` as it is today.

## Proposed DSL Changes

### Items
Add new fields to item blocks:

```
item ornate_desk {
  name "Ornate Desk"
  desc "An ornately carved mahogany desk with gold filigree."
  location room upstairs_office
  movability fixed "It's far too heavy to move."
  visibility scenery
  aliases "desk", "table", "mahogany desk"
}

item hidden_note {
  name "Hidden Note"
  desc "A folded slip of paper wedged behind the frame."
  location room study
  visibility hidden
  visible when has_flag desk_moved
}
```

`visibility` default is `listed`.
`visible when` defaults to “always” if omitted.

### Room Scenery
Add room-local scenery entries for look/examine only:

```
room foyer {
  name "Foyer"
  desc "A bright entryway with old pipes along the ceiling."

  scenery default "You see nothing remarkable about the {thing}."
  scenery "pipes"
  scenery "vents" desc "The vents hum softly with recycled air."
}
```

`scenery` entries are not items and are only consulted by `look at` / `examine` when no real item matches.

### Room Exits
No DSL changes required; `hidden` already exists on exits and in room patches.
After implementation, `hidden` will hide the exit from the list until revealed.

## Engine Behavior Changes

### Visibility Evaluation
Add helper(s), e.g.:
- `item_is_visible(world, item_id) -> bool`
- `item_is_listed(world, item_id) -> bool`

Rules:
- If `visible_when` is present, evaluate it via `EventCondition::eval(world)`.
- Hidden items are discoverable only if visible_when passes.
- Scenery items are visible (if conditions pass) but not listed.

### Alias Matching
Extend item search to match `aliases` in addition to item names.

### Room Scenery Fallback
If `look at` fails to find a visible item/NPC, check the room’s scenery list:
- If a scenery entry matches, show its `desc` or the room’s `scenery_default`.
- If none match, fall back to the current “word not understood” response.

### Room Item Listing
`Room::show` should list only items that are:
- visible, and
- `ItemVisibility::Listed`

### Container Contents Listing
`Item::show_contents` should apply the same listed/visible filter.
- Scenery contents are not listed automatically, but can be discovered via `look at`.

### Visible vs Reachable Scopes
- `nearby_visible_items`: include Listed + Scenery items that are visible.
- `nearby_reachable_items`: include Listed + Scenery items that are visible (so scenery is touchable).
- `Hidden` items are excluded from both unless visible_when passes.

### Search and Discovery
`find_item_match` should only match items included in the relevant scope:
- This ensures hidden items are not discoverable until revealed.

### Exits
`Room::show_exits` should skip exits where `exit.hidden == true`.
- `RevealExit` already unsets `hidden` and should cause it to appear on subsequent looks.

Movement should treat hidden exits as non-existent until revealed.
After reveal, traversal is still gated by `locked`, `required_flags`, and `required_items`.

## Data Flow / Implementation Outline

1. **Data model**
   - Add `ItemVisibility`, `visible_when`, and `aliases` to `amble_data::ItemDef` and patches.
   - Add `RoomSceneryDef` + `scenery_default` to `RoomDef`.
   - Update defaults and serde.

2. **Runtime model**
   - Add `visibility`, `visible_when`, and `aliases` to `amble_engine::item::Item`.
   - Add `scenery` + `scenery_default` to `amble_engine::room::Room`.
   - Store `visible_when` as `EventCondition` for evaluation efficiency.

3. **Loader**
   - Convert `ConditionExpr` to `EventCondition` in `loader/worlddef.rs`.

4. **DSL**
   - Extend grammar and AST to include `visibility`, `visible when`, `aliases`, and room `scenery`.
   - Update worlddef conversion.

5. **Visibility helpers**
   - Implement shared helper logic and update:
     - `nearby_visible_items`
     - `nearby_reachable_items`
     - `Room::show` item list
     - `Item::show_contents`

6. **Exits**
   - Filter hidden exits in `Room::show_exits`.
   - Block traversal to hidden exits in movement handler.

7. **Docs + Tests**
   - Update item DSL guide with new fields.
   - Add tests for visibility filtering, alias matching, scenery fallback, and hidden exit behavior.

## Backward Compatibility
- Default visibility is `Listed` with no conditions: existing content should behave the same.
- Hidden exits will stop appearing in exit lists (intentional change).
- Hidden exits become non-traversable until revealed (intentional change).

## Open Questions (resolved for this design)
- Scenery should be touchable: **yes**.
- Hidden items discoverable only when revealed: **yes**.
- Hidden exits should be hide-able and non-traversable: **yes**.
- Aliases and room-level default scenery: **included**.

## Optional Follow-ups
- **Lighting system**: explicit room light levels and item emitters; could build on `visible_when`.
