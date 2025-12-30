# Rooms DSL Guide

This guide introduces the Rooms subset of the amble_script DSL and how it maps to the engine’s `rooms.toml`.

Highlights:
- DSL omits `location`; compiler emits `location = "Nowhere"`.
- `visited` defaults to `false`; specify `visited true` to mark as visited.
- Exits support `hidden`, `locked`, `barred`, `required_items`, and `required_flags` (flag names only).
- Overlays support all-of conditions and a `text` body.

## Minimal Room

```
room high-ridge {
  name "High Isolated Ridge"
  desc """A small, flat ridge..."""
}
```

Emits:

```
[[rooms]]
# room high-ridge (source line N)
id = "high-ridge"
name = "High Isolated Ridge"
base_description = "A small, flat ridge..."
location = "Nowhere"
```

## Exits

```
room two-sheds-landing {
  name "Jackson's Landing"
  desc "..."

  exit up   -> guard-post { locked, barred "Need to clear the tree.", required_flags(cleared-fallen-tree) }
  exit down -> parish-landing
  exit east -> two-sheds { required_items(machete, gasoline) }
}
```

Emits nested tables:

```
[rooms.exits.up]
to = "guard-post"
locked = true
barred_message = "Need to clear the tree."
required_flags = [{ type = "simple", name = "cleared-fallen-tree" }]

[rooms.exits.down]
to = "parish-landing"

[rooms.exits.east]
to = "two-sheds"
required_items = ["machete", "gasoline"]
```

Notes:
- `required_flags(...)` accepts flag names (e.g., `cleared-fallen-tree`). Sequence steps are not required; the engine matches flags by name.

Quoted directions:

You can use quoted exit directions to allow spaces or special characters in the direction name. These emit quoted TOML keys automatically.

```
room shoreline {
  name "Shoreline"
  desc "..."
  exit "along the shore" -> dunes
}

# Emits
[rooms.exits."along the shore"]
to = "dunes"
```

## Overlays

Overlay condition lists can be written directly after `if` or wrapped in parentheses for clarity. The following examples omit parentheses, but `overlay if (flag set got-towel) { ... }` would also be valid.

```
room front-entrance {
  name "Front Entrance"
  desc "..."

  overlay if flag set got-towel {
    text "The doors unlatch and open slightly."
  }

  overlay if npc present cmot_dibbler, npc in state cmot_dibbler happy {
    text "Dibbler beams and offers a celebratory sausage-inna-bun."
  }

  overlay if npc in state emh custom "want-emitter" {
    text "The EMH fidgets restlessly, craving a mobile emitter."
  }

  overlay if item in room margarine st-alfonzo-parish {
    text "On the pedestal sits a tub of margarine."
  }
}
```

For paired binary conditions like flags or presence checks, you can group the two outcomes into a single overlay block:

```
room locker-room {
  name "Locker Room"
  desc "..."

  overlay if flag has-key {
    set "The locker hangs open."
    unset "The locker door is tightly shut."
  }
}
```

Emits overlay entries:

```
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "got-towel" }]
text = "The doors unlatch and open slightly."

[[rooms.overlays]]
conditions = [
  { type = "npcPresent", npc_id = "cmot_dibbler" },
  { type = "npcInState", npc_id = "cmot_dibbler", state = "happy" }
]
text = "Dibbler beams and offers a celebratory sausage-inna-bun."

[[rooms.overlays]]
conditions = [{ type = "npcInState", npc_id = "emh", state = { custom = "want-emitter" } }]
text = "The EMH fidgets restlessly, craving a mobile emitter."

[[rooms.overlays]]
conditions = [{ type = "itemInRoom", item_id = "margarine", room_id = "st-alfonzo-parish" }]
text = "On the pedestal sits a tub of margarine."
```

### NPC State Block (Sugar)

Combine multiple NPC state overlays in one block using the `npc <id> here` form:

```
overlay if npc emh here {
  normal "EMH behaving normally."
  happy "EMH is singing a tune."
  custom(want-emitter) "EMH won't stop griping about his missing emitter."
}
```

This expands to three overlays equivalent to writing separate entries with conditions:

```
[[rooms.overlays]]
conditions = [{ type = "npcPresent", npc_id = "emh" }, { type = "npcInState", npc_id = "emh", state = "normal" }]
text = "EMH behaving normally."

[[rooms.overlays]]
conditions = [{ type = "npcPresent", npc_id = "emh" }, { type = "npcInState", npc_id = "emh", state = "happy" }]
text = "EMH is singing a tune."

[[rooms.overlays]]
conditions = [{ type = "npcPresent", npc_id = "emh" }, { type = "npcInState", npc_id = "emh", state = { custom = "want-emitter" } }]
text = "EMH won't stop griping about his missing emitter."
```

Notes:
- The `custom(name)` form accepts an identifier; it maps to the engine’s `{ custom = "name" }` state.
- Each line inside the block becomes its own overlay with `npcPresent` + `npcInState` conditions.
- You can still use the explicit form with `overlay if (npc present X, npc in state X Y) { ... }`.

## CLI Usage

```
# Compile rooms (and any goals in the same file) to TOML
cargo run -p amble_script -- compile \
  amble_script/data/Amble/areas/bldg_perimeter/rooms/front_entrance.amble \
  --out-rooms /tmp/rooms.toml --out-goals /tmp/goals.toml

# Lint a file with engine data for symbol validation
cargo run -p amble_script -- lint \
  amble_script/data/Amble/areas/bldg_perimeter/rooms/front_entrance.amble \
  --data-dir amble_engine/data --deny-missing
```

Generated files include a header comment indicating the source `.amble` and a “do not edit” notice. Each room entry is prefixed with a comment containing the DSL source line number.

## Tips

- `visited` defaults to `false`. Only set `visited = true` if you need a room to start as visited (e.g., the start room). The engine marks rooms as the player explores.
- Use lint early and often; it will suggest likely IDs when you mistype symbols.
