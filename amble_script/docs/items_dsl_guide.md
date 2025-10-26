# Items DSL Guide

This guide introduces the Items subset of the amble_script DSL and how it maps to the engine’s `items.toml`.

Highlights:
- Support for `name`, `desc`, `portable`, and `location` fields.
- Optional `container state` (`open`, `closed`, `locked`, `transparentClosed`, `transparentLocked`).
- Optional `text` field for readable items and `restricted` flag for non-droppable items.
- `ability` entries compile to `[[items.abilities]]` tables with an optional `target`.
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

Emits:

```
[[items]]
id = "portal_gun"
name = "Portal Gun"
description = "A device."
portable = false
location = { Room = "portal-room" }
container_state = "closed"

[[items.abilities]]
type = "TurnOn"
```

## Locations

The `location` field places the item at start:

```
location inventory player   # player’s inventory
location room portal-room   # in a room
location npc clerk          # held by an NPC
location chest strongbox    # inside a chest/container
location nowhere "note"     # nowhere; note explains when it spawns
```

## Abilities

Ability lines describe interactions or custom behaviors:

```
ability Read
ability Unlock box
```

Each ability becomes an entry in `[[items.abilities]]` with `type` and optional `target`.

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

This compiles to TOML as:

```
[items.interaction_requires]
handle = "insulate"
open = "cut"
```

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

The compiler emits these into the `[[items.consumable]]` tables with the correct structure expected by the engine.

## Library Usage

```
use amble_script::{parse_items, compile_items_to_toml};
let src = std::fs::read_to_string("items.amble")?;
let items = parse_items(&src)?;
let toml = compile_items_to_toml(&items)?;
```

The resulting `toml` string matches `amble_engine/data/items.toml`.
