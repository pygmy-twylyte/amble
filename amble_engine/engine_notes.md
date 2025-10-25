# Amble Engine Notes
Some notes to help me remember and possible future others understand how this works / why it was done this way. This is put together piecemeal as I run across things that might need some explanation.

---

## AmbleWorld
- contains full game state: player, locations, items, triggers, npcs, spinners, flags, scheduled events, goals, player's location history, more
- saved games are .ron (Rust Object Notation) files containing a complete AmbleWorld snapshot, so loaded games have all flags and scheduled events in progess intact

---

## Items
* can be made single use by creating a despawn trigger conditioned on use.
* despawned / unspawned items have location "Nowhere"

---

## Trigger System

General function: any command / action runs check_triggers() with a set of matching TriggerConditions. Triggers are defined with a name, whether they're repeating or one-off, a list of conditions that must be met to fire, and a list of actions (which may also contain additional condition checks) to perform when they're met.

Some triggers conditions depend only on world state and can fire independent of player actions. Other conditions are met when particular player actions occur, such as opening a contaniner or leaving a room.

### "verb target with tool" trigger setup
Some triggers uses have evolved over time and names don't reflect this (yet). In particular, in terms of handling <verb><target> with <tool> commands:
* UseItem( item, item ability ) -- (***DSL: `when use item <item_id> ability <ability>`***) -- for abilities that don't require any interaction with anything else ('turn on' for example)
* UseItemOnItem(interaction, tool, target) -- (***DSL: `when use item <tool_id> on item <target_id> interaction <item_interaction_type>`***) -- typically used for flavor text that's different depending on the item used (e.g. if flamethrower is used to burn something rather than a lighter) -- no world or item state changes should be made here
* ActOnItem(interaction, target) -- allows triggers to work on interactions regardless of specific tool. Example: a trigger that reacts to lighting a fuse, regardless of whether you use a lighter or a candle or a laser -- or any item that has the required "ignite" ability.

check_triggers() returns a Vec<Trigger> of all fired triggers, which allows the command handler to check to see if there was any particular triggered reaction (and then provide any default handling needed if not)

---

## Flags
* two types, Simple and Sequence
* Simple flags are boolean "has done this thing at some point" or "has this state"
* Sequence flags can be used to define steps in a puzzle or progressive severity of a condition, for example.
* The sequence number can be advanced in triggers
* A sequence limit is typically set for the final step, but it *can* be infinite (well, at least up to `USIZE_MAX`!).
* Can be used to change NPC state, unlock or reveal exits, advance Goals, create status effects, change room appearance, many more applications

---

## Spinners
* imported from my 'gametools' crate
* provide randomized, customizable rephrasing of common message types to keep them more interesting
* provide intermittent, location-based "ambient" messages for environment
* wedges are Strings and can be weighted by being given different 'widths'
* intermittent messages are created using a `chance` trigger condition
* spinner wedges can be added at runtime by triggers, allowing game events to color messages that follow
