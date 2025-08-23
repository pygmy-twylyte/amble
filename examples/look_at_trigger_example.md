# LookAt Trigger Condition Example

This document demonstrates how to use the new `LookAt` trigger condition in Amble game files.

## Overview

The `LookAt` trigger condition fires when a player examines a specific item using the "look at" command. This enables various gameplay mechanics such as:

- Advancing goals when players discover items
- Providing additional narrative context
- Triggering NPC reactions to player observations
- Applying status effects based on what the player observes

## Basic Usage

### TOML Configuration

```toml
[[triggers]]
name = "Mysterious Book Observation"
only_once = true
conditions = [
    { type = "lookAt", item_id = "mysterious_book" }
]
actions = [
    { type = "showMessage", text = "As you examine the book closely, you notice strange symbols glowing faintly on its cover." },
    { type = "advanceFlag", flag = "discovered_magic_book" }
]
```

## Advanced Examples

### Goal Advancement
```toml
[[triggers]]
name = "Key Discovery Goal"
only_once = true
conditions = [
    { type = "lookAt", item_id = "hidden_key" }
]
actions = [
    { type = "advanceFlag", flag = "find_the_key" },
    { type = "showMessage", text = "You've found the key! This might open that locked door you saw earlier." }
]
```

### NPC Reaction
```toml
[[triggers]]
name = "Guard Notices Weapon Examination"
only_once = false
conditions = [
    { type = "lookAt", item_id = "forbidden_sword" },
    { type = "withNpc", npc_id = "palace_guard" }
]
actions = [
    { type = "showMessage", text = "The guard notices you examining the forbidden weapon and becomes suspicious." },
    { type = "setNpcState", npc_id = "palace_guard", state = "custom:suspicious" }
]
```

### Conditional Item Spawn
```toml
[[triggers]]
name = "Reveal Secret Compartment"
only_once = true
conditions = [
    { type = "lookAt", item_id = "ornate_desk" },
    { type = "hasFlag", flag = "knows_about_secret" }
]
actions = [
    { type = "showMessage", text = "Now that you know what to look for, you spot a hidden compartment in the desk!" },
    { type = "spawnItemInContainer", item_id = "secret_document", container_id = "ornate_desk" }
]
```

### Multi-Stage Discovery
```toml
[[triggers]]
name = "First Clue Examination"
only_once = true
conditions = [
    { type = "lookAt", item_id = "ancient_statue" }
]
actions = [
    { type = "setFlag", flag = "examined_statue" },
    { type = "showMessage", text = "The statue has unusual markings. You feel like you're missing something important." }
]

[[triggers]]
name = "Second Look With Knowledge"
only_once = true
conditions = [
    { type = "lookAt", item_id = "ancient_statue" },
    { type = "hasFlag", flag = "learned_ancient_language" },
    { type = "hasFlag", flag = "examined_statue" }
]
actions = [
    { type = "showMessage", text = "Now that you understand the ancient language, the statue's markings reveal a hidden message about the temple's secret entrance!" },
    { type = "advanceFlag", flag = "temple_mystery" }
]
```

### Discovery with Point Rewards
```toml
[[triggers]]
name = "Artifact Discovery"
only_once = true
conditions = [
    { type = "lookAt", item_id = "crystal_orb" }
]
actions = [
    { type = "showMessage", text = "The crystal orb pulses with an inner light as you examine it closely. You sense great power within." },
    { type = "awardPoints", amount = 50 },
    { type = "setFlag", flag = "discovered_power_source" }
]
```

### Spawn Multiple Related Items
```toml
[[triggers]]
name = "Bookshelf Secrets"
only_once = true
conditions = [
    { type = "lookAt", item_id = "dusty_bookshelf" },
    { type = "hasItem", item_id = "magnifying_glass" }
]
actions = [
    { type = "showMessage", text = "Using the magnifying glass, you notice that several books have been moved recently. Behind them, you find hidden compartments!" },
    { type = "spawnItemInContainer", item_id = "ancient_map", container_id = "dusty_bookshelf" },
    { type = "spawnItemInContainer", item_id = "silver_key", container_id = "dusty_bookshelf" },
    { type = "spawnItemInContainer", item_id = "diary_pages", container_id = "dusty_bookshelf" }
]
```

### Environmental Storytelling
```toml
[[triggers]]
name = "Battlefield Examination"
only_once = false
conditions = [
    { type = "lookAt", item_id = "rusted_armor" }
]
actions = [
    { type = "showMessage", text = """Looking closer at the armor, you see it bears the crest of the Royal Guard.

Whoever wore this fought bravely here, long ago. The dents and scratches tell a story of a desperate last stand.""" }
]
```

## Implementation Notes

- The `LookAt` condition only triggers for items, not NPCs
- It fires every time the player looks at the specified item (unless the trigger is marked as `only_once = true`)
- Use it in combination with other conditions to create sophisticated interactions
- The condition uses the item's symbol/ID as specified in your game data files

## Common Patterns

1. **Discovery Rewards**: Award points or advance goals when players examine important items
2. **Contextual Information**: Provide different descriptions based on player knowledge/state
3. **Environmental Storytelling**: Reveal lore through careful observation
4. **Puzzle Mechanics**: Require players to examine items in specific orders or states
5. **Dynamic Content**: Spawn new items or unlock areas based on player curiosity
6. **Progressive Revelation**: Show different information on repeated examinations based on game state

## Tips for Game Designers

- Use `only_once = false` for items that might reveal different information over time
- Combine `LookAt` with inventory checks (`hasItem`) to create tool-dependent discoveries
- Use flag conditions to gate advanced information behind story progression
- Consider spawning related items to reward thorough exploration
- Layer multiple triggers on the same item for complex, evolving narratives

This trigger condition encourages thorough exploration and rewards players for paying attention to their environment, making it perfect for mystery games, puzzle adventures, and rich narrative experiences.
