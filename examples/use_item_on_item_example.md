# UseItemOnItem Trigger Condition Example

This document demonstrates how to use the `UseItemOnItem` trigger condition in Amble game files, which enables complex multi-item interactions and tool-based puzzles.

## Overview

The `UseItemOnItem` trigger condition fires when a player uses one item (the tool) on another item (the target) with a specific interaction type. This enables sophisticated gameplay mechanics such as:

- Tool-based puzzles (using keys on locks, matches on candles)
- Crafting and combination systems
- Item transformation and upgrading
- Complex multi-step interactions
- Realistic tool requirements for actions

## Understanding the Components

### Interaction Types
The system supports various interaction types that define how items can be used together:

- `Open` - Opening with tools (keys, lockpicks, crowbars)
- `Burn` - Igniting items (matches on wood, torches on oil)
- `Cut` - Cutting with sharp tools (knives on rope, saws on wood)
- `Break` - Breaking with force tools (hammers on glass, stones on pottery)
- `Clean` - Cleaning with appropriate tools (cloths on dirty items)
- `Repair` - Fixing with repair tools (hammers on broken items)
- `Sharpen` - Sharpening tools (whetstones on blades)
- `Turn` - Rotating mechanisms (wrenches on valves, keys in locks)
- `Cover` - Covering items (sheets over objects)
- `Move` - Moving with assistance (levers on heavy objects)
- `Handle` - Manipulating dangerous items safely (tongs on hot items)
- `Attach` - Connecting items together (rope to anchors)
- `Unlock` - Specifically unlocking mechanisms

### Item Abilities
Items must have corresponding abilities to perform these interactions. The `interaction_requires` field in item definitions maps interaction types to required abilities:

```toml
# Target item that can be burned
[[items]]
id = "wooden_door"
# ... other properties ...
interaction_requires = { Burn = "Ignite" }  # Requires Ignite ability to burn

# Tool item with the required ability
[[items]]
id = "torch"
# ... other properties ...
abilities = ["Ignite"]  # Has the Ignite ability
```

## Basic Usage

### Simple Key and Lock
```toml
[[triggers]]
name = "Unlock Chest with Key"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Unlock", target_id = "treasure_chest", tool_id = "brass_key" }
]
actions = [
    { type = "unlockItem", item_id = "treasure_chest" },
    { type = "showMessage", text = "The brass key turns smoothly in the lock. The chest clicks open!" }
]
```

### Fire-Starting Mechanic
```toml
[[triggers]]
name = "Light Fireplace"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Burn", target_id = "fireplace", tool_id = "lit_torch" }
]
actions = [
    { type = "showMessage", text = "You touch the torch to the kindling. The fireplace roars to life, filling the room with warmth and light." },
    { type = "setItemDescription", item_sym = "fireplace", text = "A cheerful fire crackles in the stone fireplace, casting dancing shadows on the walls." },
    { type = "addFlag", flag = { type = "simple", name = "fireplace_lit" } }
]
```

## Advanced Examples

### Multi-Tool Puzzle
```toml
# First, cut the rope with a knife
[[triggers]]
name = "Cut Rope with Knife"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Cut", target_id = "thick_rope", tool_id = "sharp_knife" }
]
actions = [
    { type = "showMessage", text = "You carefully cut through the thick rope with the sharp knife." },
    { type = "setItemDescription", item_sym = "thick_rope", text = "A severed rope lies on the ground in two pieces." },
    { type = "addFlag", flag = { type = "simple", name = "rope_cut" } }
]

# Then, move the heavy block with a lever
[[triggers]]
name = "Move Block with Lever"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Move", target_id = "stone_block", tool_id = "iron_lever" },
    { type = "hasFlag", flag = "rope_cut" }
]
actions = [
    { type = "showMessage", text = "With the rope no longer holding it in place, you use the iron lever to shift the massive stone block aside, revealing a hidden passage!" },
    { type = "revealExit", direction = "down", exit_from = "ancient_chamber", exit_to = "secret_tunnel" }
]
```

### Item Transformation System
```toml
[[triggers]]
name = "Sharpen Dull Blade"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Sharpen", target_id = "dull_sword", tool_id = "whetstone" }
]
actions = [
    { type = "showMessage", text = "You carefully run the whetstone along the sword's edge. Sparks fly as the metal becomes razor-sharp once again." },
    { type = "despawnItem", item_id = "dull_sword" },
    { type = "spawnItemInInventory", item_id = "sharp_sword" },
    { type = "awardPoints", amount = 25 }
]
```

### Safety Equipment Required
```toml
[[triggers]]
name = "Handle Hot Coals Safely"
only_once = false
conditions = [
    { type = "useItemOnItem", interaction = "Handle", target_id = "glowing_coals", tool_id = "metal_tongs" }
]
actions = [
    { type = "showMessage", text = "Using the metal tongs, you carefully pick up the glowing coals without burning yourself." },
    { type = "spawnItemInInventory", item_id = "hot_coal" }
]

# What happens if you try without proper tools
[[triggers]]
name = "Burn Yourself on Coals"
only_once = false
conditions = [
    { type = "actOnItem", target_sym = "glowing_coals", action = "Handle" },
    { type = "missingItem", item_id = "metal_tongs" }
]
actions = [
    { type = "showMessage", text = "Ouch! The coals are far too hot to handle with your bare hands. You need proper tools for this." }
]
```

### Complex Crafting Chain
```toml
# Step 1: Attach rope to grappling hook
[[triggers]]
name = "Attach Rope to Hook"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Attach", target_id = "grappling_hook", tool_id = "climbing_rope" }
]
actions = [
    { type = "showMessage", text = "You securely tie the climbing rope to the grappling hook." },
    { type = "despawnItem", item_id = "grappling_hook" },
    { type = "despawnItem", item_id = "climbing_rope" },
    { type = "spawnItemInInventory", item_id = "grappling_hook_with_rope" }
]

# Step 2: Use the completed grappling hook assembly
[[triggers]]
name = "Use Grappling Hook Assembly"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Attach", target_id = "castle_wall", tool_id = "grappling_hook_with_rope" },
    { type = "inRoom", room_id = "castle_courtyard" }
]
actions = [
    { type = "showMessage", text = "You throw the grappling hook up to the castle wall. It catches firmly on the battlements!" },
    { type = "revealExit", direction = "up", exit_from = "castle_courtyard", exit_to = "castle_walls" }
]
```

### Conditional Tool Requirements
```toml
[[triggers]]
name = "Open Rusted Lock - Crowbar Method"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Open", target_id = "rusted_chest", tool_id = "iron_crowbar" }
]
actions = [
    { type = "showMessage", text = "You wedge the crowbar under the rusted lock and apply pressure. With a loud crack, the lock breaks apart!" },
    { type = "unlockItem", item_id = "rusted_chest" },
    { type = "setItemDescription", item_sym = "rusted_chest", text = "An old chest with a broken lock, now accessible." }
]

[[triggers]]
name = "Open Rusted Lock - Oil Method"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Clean", target_id = "rusted_chest", tool_id = "penetrating_oil" },
    { type = "missingFlag", flag = "chest_opened" }
]
actions = [
    { type = "showMessage", text = "You apply the penetrating oil to the rusted lock. The rust begins to dissolve, and you can hear the mechanism loosening." },
    { type = "addFlag", flag = { type = "simple", name = "lock_oiled" } }
]

# Follow-up: now the normal key works
[[triggers]]
name = "Open Oiled Lock with Key"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Unlock", target_id = "rusted_chest", tool_id = "old_key" },
    { type = "hasFlag", flag = "lock_oiled" }
]
actions = [
    { type = "showMessage", text = "Thanks to the oil, the old key now turns easily in the lock!" },
    { type = "unlockItem", item_id = "rusted_chest" },
    { type = "addFlag", flag = { type = "simple", name = "chest_opened" } }
]
```

### Dangerous Interactions
```toml
[[triggers]]
name = "Break Glass Safely"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Break", target_id = "stained_glass", tool_id = "hammer" },
    { type = "hasItem", item_id = "safety_goggles" }
]
actions = [
    { type = "showMessage", text = "Wearing your safety goggles, you carefully break the stained glass window with the hammer. Glass shards fall safely away from your eyes." },
    { type = "setItemDescription", item_sym = "stained_glass", text = "A broken window frame with jagged glass remnants." },
    { type = "spawnItemCurrentRoom", item_id = "glass_shards" },
    { type = "revealExit", direction = "out", exit_from = "tower_room", exit_to = "tower_exterior" }
]

[[triggers]]
name = "Break Glass Dangerously"
only_once = true
conditions = [
    { type = "useItemOnItem", interaction = "Break", target_id = "stained_glass", tool_id = "hammer" },
    { type = "missingItem", item_id = "safety_goggles" }
]
actions = [
    { type = "showMessage", text = "You smash the window with the hammer, but glass shards fly everywhere! Some get in your eyes, making it difficult to see clearly." },
    { type = "setItemDescription", item_sym = "stained_glass", text = "A broken window frame with jagged glass remnants." },
    { type = "spawnItemCurrentRoom", item_id = "glass_shards" },
    { type = "addFlag", flag = { type = "simple", name = "vision_impaired" } },
    { type = "revealExit", direction = "out", exit_from = "tower_room", exit_to = "tower_exterior" }
]
```

## Implementation Notes

### Item Setup Requirements
1. **Target items** must specify `interaction_requires` mapping interaction types to required abilities
2. **Tool items** must have the corresponding abilities in their `abilities` set
3. Both items must exist and be accessible to the player

### Interaction Matching
- The trigger only fires if the tool item has the ability required by the target item's `interaction_requires` mapping
- If no requirement is specified for an interaction type, any item can be used as a tool
- The system validates both the interaction type and ability requirement

### Command Syntax
Players can trigger these interactions using various command formats:
- "burn wood with torch"
- "cut rope using knife"
- "open chest with key"
- "break glass with hammer"

## Common Patterns

1. **Tool Validation**: Ensure tools have appropriate abilities before allowing interactions
2. **Item Consumption**: Some interactions should consume the tool (matches) or target (consumables)
3. **Progressive Unlocking**: Use flags to enable more complex tool combinations over time
4. **Safety Mechanics**: Require protective equipment for dangerous operations
5. **Multi-Step Processes**: Chain multiple UseItemOnItem conditions for complex crafting
6. **Alternative Solutions**: Provide multiple tool options for the same problem
7. **Failure Conditions**: Handle cases where players try inappropriate tool combinations

## Tips for Game Designers

### Planning Tool Systems
- Design tool/target relationships early in your game planning
- Consider real-world logic when assigning interaction types and abilities
- Plan for multiple solutions to encourage creative problem-solving
- Document your tool system for consistency across your game

### Balancing Complexity
- Start with simple tool interactions and gradually introduce complexity
- Use clear messaging to help players understand tool requirements
- Provide hints about required tools through item descriptions and environmental storytelling
- Consider the player's inventory management when requiring multiple tools

### Common Pitfalls to Avoid
- Don't make tool requirements too obscure or illogical
- Provide feedback when players attempt invalid tool combinations
- Ensure required tools are reasonably obtainable when needed
- Test interaction chains thoroughly to avoid breaking game progression

### Advanced Techniques
- Use `only_once = false` for tools that can be reused multiple times
- Combine with inventory conditions to ensure players have necessary tools
- Create tool degradation systems by spawning "worn" versions after use
- Use room conditions to make certain tool combinations only work in specific locations
- Layer multiple triggers on the same item pair for complex, evolving interactions

This trigger condition is perfect for creating realistic, puzzle-based gameplay that rewards players for thinking about how different items might work together in logical ways.