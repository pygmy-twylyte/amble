# Items DSL Guide

This guide introduces the Items subset of the amble_script DSL and how it maps into the engine’s `WorldDef` (`world.ron`).

Highlights:
- Support for `name`, `desc`, `portable`, and `location` fields.
- Optional `container state` (`open`, `closed`, `locked`, `transparentClosed`, `transparentLocked`).
- Visibility controls: `visibility`, `visible when`, and `aliases` for matching.
- Optional `text` field for readable items and `restricted` flag for non-droppable items.
- `ability` entries compile into `ItemDef.abilities` entries with an optional `target`.
- Interaction requirements: `requires <ability> to <interaction>` compiles to `interaction_requires`.
- `consumable { … }` blocks model limited-use items that despawn or transform after depletion.

## Minimal Item

```
item portal_gun {
  name "Portal Gun"
  desc "A device."
  portable false
  container state closed
  location room portal-room
  ability TurnOn
}
```

This produces an `ItemDef` with the corresponding `id`, `name`, `desc`, `movability`, `location`, `container_state`, and `abilities` fields.

## Locations

The `location` field places the item at start:

```
location inventory player   # player’s inventory
location room portal-room   # in a room
location npc clerk          # held by an NPC
location chest strongbox    # inside a chest/container
location nowhere "note"     # nowhere; note explains when it spawns
```

## Visibility, Scenery, and Aliases

Items default to `visibility listed`, which means they appear in room/container lists and are discoverable with `look at`.

```
visibility scenery   # discoverable but not listed
visibility hidden    # only discoverable once visible when condition passes

visible when has flag desk_moved

aliases "desk", "table", "mahogany desk"
```

- `visibility listed|scenery|hidden` controls whether the item is listed.
- `visible when <condition>` uses the same condition syntax as triggers (`has flag`, `has item`, `any(...)`, etc.).
- `aliases` adds alternate terms that can match the item in player input.

## Abilities

Ability lines describe interactions or custom behaviors:

```
ability Read
ability Unlock box
```

Each ability becomes an entry in `ItemDef.abilities` with `type` and optional `target`.

## Optional Text & Restricted Items

```
text "Authorized personnel only."
ability Read

restricted true  # item cannot be taken by player, but it is portable and can be given to them or unrestricted later
portable true # if false, item cannot be moved under any circumstances
```

`text` is emitted as the item’s readable text. `restricted` defaults to `false` and is only emitted when set to `true`.
Note: the "read" and "examine" player commands are synonyms as far as the engine is concerned, so this extra text field can be used for extra detail descriptions or clues in addition to items that actually have legible text.

## Interaction Requirements

Use `requires <ability> to <interaction>` to gate an interaction behind an item ability. Examples:

```
# Requires that the acting item has the 'insulate' ability to handle this item
requires insulate to handle

# Requires that the acting item has the 'cut' ability to open this item
requires cut to open
```

This compiles into `ItemDef.interaction_requires` with the matching interaction → ability mapping.

## Consumables

Attach a `consumable { … }` block to define limited-use tools, medicine, or gadgets:

```
consumable {
  uses_left 3
  consume_on ability TurnOn
  when_consumed replace inventory drained-battery
}
```

Available options:

- `uses_left <n>` sets how many charges remain (must be ≥ 0).
- `consume_on ability <Ability> [<target>]` declares which abilities consume a charge. Provide multiple lines for multiple abilities.
- `when_consumed …` chooses what happens at zero charges:
  - `when_consumed despawn` removes the item.
  - `when_consumed replace inventory <item>` swaps it for another item in the player’s inventory.
  - `when_consumed replace current room <item>` drops a replacement into the room where it was used.

The compiler emits these into `ItemDef.consumable` with the correct structure expected by the engine.

## Library Usage

```
use amble_script::{GameAst, PlayerAst, parse_items, worlddef_from_asts};
use ron::ser::PrettyConfig;
let src = std::fs::read_to_string("items.amble")?;
let items = parse_items(&src)?;
let game = GameAst {
    title: "Demo".into(),
    intro: "Intro".into(),
    player: PlayerAst {
        name: "The Candidate".into(),
        description: "An adventurer.".into(),
        max_hp: 20,
        start_room: "foyer".into(),
    },
    scoring: None,
};
let worlddef = worlddef_from_asts(Some(&game), &[], &[], &items, &[], &[], &[])?;
let ron = ron::ser::to_string_pretty(&worlddef, PrettyConfig::default())?;
```

The resulting `ron` string can be written to `world.ron` for engine loading.
