# Items DSL Guide

This guide introduces the Items subset of the amble_script DSL and how it maps to the engine’s `items.toml`.

Highlights:
- Support for `name`, `desc`, `portable`, and `location` fields.
- Optional `container state` (`open`, `closed`, `locked`, `transparentClosed`, `transparentLocked`).
- Optional `text` field for readable items and `restricted` flag for non-droppable items.
- `ability` entries compile to `[[items.abilities]]` tables with an optional `target`.

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
restricted true  # item cannot be dropped
```

`text` is emitted as the item’s readable text. `restricted` defaults to `false` and is only emitted when set to `true`.

## Library Usage

```
use amble_script::{parse_items, compile_items_to_toml};
let src = std::fs::read_to_string("items.amble")?;
let items = parse_items(&src)?;
let toml = compile_items_to_toml(&items)?;
```

The resulting `toml` string matches `amble_engine/data/items.toml`.
