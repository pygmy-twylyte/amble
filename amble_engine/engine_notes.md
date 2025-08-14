# Amble Engine Notes
Some notes to help me remember and possible future others understand how this works / why it was done this way. This is put together piecemeal as I run across things that need some explanation.

## AmbleWorld
- contains full game state: player, locations, items, triggers, npcs, spinners

## Items
* can be made single use by creating a despawn trigger conditioned on use.
* despawned / unspawned items have location "Nowhere"

---
## Trigger System

General function: any command / action runs check_triggers() at the end. Triggers are defined with a name, whether they're repeating or one-off, a list of conditions that must be me to fire, and a list of actions to perform when they're met.

Some triggers conditions depend only on world state and can fire independent of player actions. Other conditions are met when particular player actions occur, such as opening a contaniner or leaving a room.

### "verb target with tool" trigger setup
Some triggers uses have evolved over time and names don't reflect this (yet). In particular, in terms of handling <verb><target> with <tool> commands:
* UseItem( item, item ability ) -- should be used for things that depend only on the item and not the target (such as despawning after use, regardless what it's used on)
* UseItemOnItem(interaction, tool, target) -- should only be flavor text that's different depending on the item used (e.g. if flamethrower is used to burn something rather than a lighter) -- no world or item state changes should be made here
* ActOnItem(interaction, target) -- is the only required trigger for an interaction and defines the reaction of the target and world to the action. Examples would include despawning the target, telling the player what happened, awarding points, advancing flags, etc.

check_triggers() returns a Vec<Trigger> of all fired triggers, which allows the command handler to check to see if there was a any triggered reaction (and provide any default handling needed if not)

## Flags
* two types, Simple and Sequence
* Simple flags are boolean "has done this thing at some point" or "has this state"
* Sequence flags can be used to define steps in a puzzle or progression.
* The sequence number can be advanced in triggers.
* A sequence limit is typically set for the final step, but it *can* be infinite.
* Can be used to change NPC state, unlock or reveal exits, advance Goals, create status effects
