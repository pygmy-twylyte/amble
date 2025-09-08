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

## Overlays

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

## CLI Usage

```
# Compile rooms to TOML
cargo run -p amble_script -- compile amble_script/examples/rooms_demo.amble --out-rooms /tmp/rooms.toml

# Lint (rooms + triggers) with friendly suggestions and line/column pointers
cargo run -p amble_script -- lint amble_script/examples/rooms_demo.amble --data-dir amble_engine/data --deny-missing
```

Generated files include a header comment indicating the source `.amble` and a “do not edit” notice. Each room entry is prefixed with a comment containing the DSL source line number.

## Tips

- `visited` defaults to `false`. Only emit `visited true` when you need a room to start as visited (e.g., the start room).
- Keep exit requirements bounded; prefer flag gates over open-ended retries for scheduling.
- Use lint early and often; it will suggest likely IDs when you mistype symbols.
