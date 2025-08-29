# Room Creation Guide

This comprehensive guide covers creating rooms in the Amble game engine, from basic room definitions to advanced features like conditional exits and dynamic overlays. Whether you're building a simple text adventure or a complex interactive fiction, this guide will help you create immersive and responsive game spaces.

## Overview

Rooms are the fundamental building blocks of your game world. Each room represents a location players can visit, explore, and interact with. Rooms can contain:

- **Basic Properties**: Name, description, and location metadata
- **Exits**: Connections to other rooms with optional requirements
- **Overlays**: Conditional text that changes based on game state
- **Contents**: Items placed within the room
- **NPCs**: Characters present in the room

## Basic Room Structure

### Minimal Room Definition

```toml
[[rooms]]
id = "starting_room"
name = "A Simple Room"
base_description = "You are in a plain, empty room with white walls."
location = "Nowhere"
```

### Complete Basic Room

```toml
[[rooms]]
id = "cottage_interior"
name = "Cozy Cottage"
base_description = """A warm, inviting cottage with wooden beams overhead and a stone fireplace crackling merrily in the corner. Sunlight streams through diamond-paned windows, casting cheerful patterns on the worn wooden floor.

A comfortable reading chair sits near the fireplace, with a small table beside it holding a steaming cup of tea."""
location = "Nowhere"
visited = false
```

## Creating Exits - Connecting Your World

### Basic Exits

Connect rooms with simple directional exits:

```toml
[[rooms]]
id = "forest_clearing"
name = "Forest Clearing"
base_description = "A peaceful clearing surrounded by tall pine trees. Dappled sunlight filters through the canopy above."
location = "Nowhere"

[rooms.exits.north]
to = "forest_path"

[rooms.exits.south]
to = "old_oak_tree"

[rooms.exits.east]
to = "babbling_brook"

[rooms.exits.west]
to = "cottage_interior"
```

### Custom Exit Names

Exits don't have to be cardinal directions - use descriptive names:

```toml
[[rooms]]
id = "cave_entrance"
name = "Cave Entrance"
base_description = "The mouth of a dark cave yawns before you. Cool air flows from within, carrying strange echoes."
location = "Nowhere"

[rooms.exits.inside]
to = "cave_interior"

[rooms.exits.down]
to = "mountain_path"

[rooms.exits.climb]
to = "cliff_ledge"
```

### Advanced Exit Examples

```toml
[[rooms]]
id = "castle_courtyard"
name = "Castle Courtyard"
base_description = "A grand stone courtyard surrounded by towering castle walls. Guards patrol the battlements above."
location = "Nowhere"

# Basic exit
[rooms.exits.south]
to = "castle_gate"

# Multiple destinations from one room
[rooms.exits.tower]
to = "wizard_tower"

[rooms.exits.dungeon]
to = "castle_dungeon"

[rooms.exits.throne]
to = "throne_room"
```

## Conditional Exits

Make exits that only appear or work under specific conditions.

### Flag-Required Exits

```toml
[[rooms]]
id = "library_entrance"
name = "Library Entrance"
base_description = "Massive oak doors block the entrance to the ancient library. A brass nameplate reads 'Restricted Access'."
location = "Nowhere"

# Only available after getting permission
[rooms.exits.inside]
to = "grand_library"
required_flags = [{ type = "simple", name = "library_permission" }]
barred_message = "The doors are locked. You need special permission to enter."

[rooms.exits.south]
to = "academy_hallway"
```

### Item-Required Exits

```toml
[[rooms]]
id = "vault_door"
name = "Bank Vault Door"
base_description = "A massive steel vault door with multiple locks and an electronic keypad. Security cameras track your every movement."
location = "Nowhere"

# Multiple items required
[rooms.exits.inside]
to = "bank_vault"
required_items = ["vault_key", "security_badge", "access_code"]
barred_message = "Access denied. You need a vault key, security badge, and access code to proceed."

[rooms.exits.main]
to = "bank_lobby"
```

### Hidden Exits

Create secret passages that don't show up in normal exit listings:

```toml
[[rooms]]
id = "study_room"
name = "Professor's Study"
base_description = "A cluttered study filled with books, papers, and strange artifacts. The professor's desk is covered with research notes."
location = "Nowhere"

# Normal exit
[rooms.exits.door]
to = "university_hallway"

# Hidden passage behind bookshelf
[rooms.exits.passage]
to = "secret_laboratory"
hidden = true
required_flags = [{ type = "simple", name = "found_secret_passage" }]
barred_message = "You haven't discovered any hidden passages here... yet."
```

## Room Overlays - Dynamic Descriptions

Overlays let you change room descriptions based on game state, creating dynamic, responsive environments.

### Flag-Based Overlays

```toml
[[rooms]]
id = "village_square"
name = "Village Square"
base_description = "The heart of a bustling village, with cobblestone streets radiating out in all directions. A fountain sits in the center."
location = "Nowhere"

# Before the festival
[[rooms.overlays]]
conditions = [{ type = "flagUnset", flag = "festival_begun" }]
text = "The square is quiet today, with only a few villagers going about their daily business."

# During the festival
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "festival_begun" }]
text = "Colorful banners flutter overhead and the square bustles with festival activity! Merchants hawk their wares while children chase ribbons through the crowd."

# After the festival ends
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "festival_ended" }]
text = "Decorations lie scattered about and the fountain is littered with flower petals - evidence of last night's grand celebration."
```

### Item-Based Overlays

```toml
[[rooms]]
id = "treasure_chamber"
name = "Treasure Chamber"
base_description = "A circular chamber with alcoves carved into the walls. Dust motes dance in shafts of light from high windows."
location = "Nowhere"

# When treasure is present
[[rooms.overlays]]
conditions = [{ type = "itemPresent", item_id = "golden_idol" }]
text = "A magnificent golden idol sits on a pedestal in the center of the room, gleaming in the filtered sunlight."

# After treasure is taken
[[rooms.overlays]]
conditions = [{ type = "itemAbsent", item_id = "golden_idol" }]
text = "The empty pedestal in the center looks forlorn without its precious burden."

# Player has the key to the next area
[[rooms.overlays]]
conditions = [{ type = "playerHasItem", item_id = "crystal_key" }]
text = "The crystal key in your possession resonates with a hidden door in the north wall, causing it to glow faintly."

# Player missing required item
[[rooms.overlays]]
conditions = [{ type = "playerMissingItem", item_id = "torch" }]
text = "The shadows in the alcoves are too deep to see without some source of light."
```

### NPC-Based Overlays

```toml
[[rooms]]
id = "tavern_common_room"
name = "The Prancing Pony"
base_description = "A cozy tavern with low wooden beams and a crackling fireplace. The scent of ale and hearty stew fills the air."
location = "Nowhere"

# When the bard is present and happy
[[rooms.overlays]]
conditions = [
    { type = "npcPresent", npc_id = "traveling_bard" },
    { type = "npcInState", npc_id = "traveling_bard", state = "Happy" }
]
text = "A traveling bard sits by the fire, strumming a lute and singing cheerful ballads. The other patrons tap their feet to the rhythm."

# When the bard is present but sad
[[rooms.overlays]]
conditions = [
    { type = "npcPresent", npc_id = "traveling_bard" },
    { type = "npcInState", npc_id = "traveling_bard", state = "Sad" }
]
text = "A melancholy bard sits alone in the corner, plucking sad melodies on his lute while staring into the fire."

# When the bard is absent
[[rooms.overlays]]
conditions = [{ type = "npcAbsent", npc_id = "traveling_bard" }]
text = "The tavern feels quieter than usual without the bard's music to liven the atmosphere."
```

### Complex Multi-Condition Overlays

```toml
[[rooms]]
id = "magic_laboratory"
name = "Wizard's Laboratory"
base_description = "A chaotic laboratory filled with bubbling cauldrons, arcane symbols, and shelves of mysterious ingredients."
location = "Nowhere"

# Multiple conditions must all be true
[[rooms.overlays]]
conditions = [
    { type = "flagSet", flag = "learned_spell" },
    { type = "playerHasItem", item_id = "spell_components" },
    { type = "npcInState", npc_id = "wizard", state = { custom = "teaching" } },
    { type = "itemInRoom", item_id = "magic_circle", room_id = "magic_laboratory" }
]
text = """The wizard gestures to the magic circle inscribed on the floor. "You have the components and the knowledge - now let us see if you can weave the spell correctly!" Magical energy crackles in the air, responding to your presence."""
```

## Advanced Room Features

### Progressive Revelation

Create rooms that change significantly as the story progresses:

```toml
[[rooms]]
id = "mysterious_chamber"
name = "Chamber of Mysteries"
base_description = "A dimly lit stone chamber with ancient carvings on the walls."
location = "Nowhere"

# Initial state - mysterious and foreboding
[[rooms.overlays]]
conditions = [{ type = "flagUnset", flag = "chamber_revealed" }]
text = "Strange symbols cover the walls, their meaning lost to time. The air hums with an otherworldly energy."

# After solving the puzzle - reveals its true nature
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "chamber_revealed" }]
text = """Now that you understand the ancient language, you can see this chamber for what it truly is: a star map! The carvings show the positions of constellations, and the humming comes from a crystalline navigation device embedded in the floor."""

# New exit appears after revelation
[rooms.exits.north]
to = "star_chamber"
required_flags = [{ type = "simple", name = "chamber_revealed" }]
barred_message = "You haven't discovered how to proceed from here yet."

[rooms.exits.south]
to = "underground_passage"
```

### Environmental Storytelling

Use overlays to tell stories through environmental details:

```toml
[[rooms]]
id = "abandoned_camp"
name = "Abandoned Campsite"
base_description = "The remains of what was once a travelers' camp. Scattered belongings and cold ashes tell a story of hasty departure."
location = "Nowhere"

# Default state
[[rooms.overlays]]
conditions = [{ type = "flagUnset", flag = "examined_camp_thoroughly" }]
text = "At first glance, it appears the travelers simply packed up and left, but something feels off about the scene."

# After closer examination
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "examined_camp_thoroughly" }]
text = """Looking more carefully, you notice troubling details: a torn piece of fabric caught on a branch, drag marks in the dirt leading north, and most telling of all, a child's wooden toy half-buried under scattered leaves - surely too precious to abandon willingly."""

# After finding clues, new areas unlock
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "found_tracks" }]
text = "The drag marks in the earth lead clearly north toward the dark heart of the forest."

[rooms.exits.north]
to = "goblin_lair"
required_flags = [{ type = "simple", name = "found_tracks" }]
barred_message = "The northern woods look impenetrable without a clear trail to follow."

[rooms.exits.south]
to = "forest_path"
```

### State-Dependent Exit Availability

Create exits that appear and disappear based on game state:

```toml
[[rooms]]
id = "bridge_ruins"
name = "Collapsed Bridge"
base_description = "The stone bridge that once spanned this gorge has collapsed, leaving only the foundations on either side."
location = "Nowhere"

# Bridge is destroyed - no way across
[[rooms.overlays]]
conditions = [{ type = "flagUnset", flag = "bridge_repaired" }]
text = "The gorge yawns before you, far too wide to jump. You'll need to find another way across - or a way to repair the bridge."

# Bridge has been magically repaired
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "bridge_repaired" }]
text = "Golden light shimmers across the gorge where a magical bridge now spans the gap, solid as stone but gleaming like starlight."

# Exit only exists when bridge is repaired
[rooms.exits.across]
to = "mountain_peaks"
required_flags = [{ type = "simple", name = "bridge_repaired" }]
barred_message = "There's no way to cross the gorge without a bridge."

[rooms.exits.south]
to = "mountain_trail"
```

## Complete Example: Multi-Stage Room

Here's a comprehensive example showing a room that evolves through multiple game states:

```toml
[[rooms]]
id = "royal_garden"
name = "Royal Garden"
base_description = """A once-magnificent formal garden behind the palace, with geometric flower beds, marble statues, and a central fountain."""
location = "Nowhere"

# Garden is withered and sad (initial state)
[[rooms.overlays]]
conditions = [
    { type = "flagUnset", flag = "garden_restored" },
    { type = "flagUnset", flag = "found_garden_key" }
]
text = """The garden has fallen into disrepair. The flower beds are choked with weeds, the fountain stands dry and cracked, and the marble statues are stained with age. A sense of melancholy hangs over the entire space."""

# Found the key but haven't used it yet
[[rooms.overlays]]
conditions = [
    { type = "flagSet", flag = "found_garden_key" },
    { type = "flagUnset", flag = "garden_restored" },
    { type = "playerHasItem", item_id = "garden_key" }
]
text = """Though still in disrepair, you notice something new: a small, ornate keyhole hidden among the roses near the fountain. The garden key in your possession seems to pulse with a warm, green light."""

# Key has been used - garden begins restoration
[[rooms.overlays]]
conditions = [
    { type = "flagSet", flag = "garden_restored" },
    { type = "flagUnset", flag = "garden_fully_bloomed" }
]
text = """The garden is transforming! Green shoots push through the soil, the fountain bubbles with crystal-clear water, and the air shimmers with golden motes of magical light. The restoration has begun, but it will take time to complete."""

# Garden is fully restored (final state)
[[rooms.overlays]]
conditions = [{ type = "flagSet", flag = "garden_fully_bloomed" }]
text = """The garden has been restored to its former glory and beyond! Flowers of impossible beauty bloom in perfect harmony, the fountain dances with liquid silver, and the statues seem to smile with joy. This place radiates peace and healing magic."""

# Different NPCs appear in different states
[[rooms.overlays]]
conditions = [
    { type = "npcPresent", npc_id = "garden_spirit" },
    { type = "npcInState", npc_id = "garden_spirit", state = { custom = "grateful" } }
]
text = """A shimmering garden spirit tends to the restored plants, humming a melody that makes the flowers dance. She smiles warmly whenever she notices you."""

# Different exits available in different states
[rooms.exits.palace]
to = "throne_room"

# Secret grove only accessible when garden is fully restored
[rooms.exits.grove]
to = "hidden_grove"
required_flags = [{ type = "simple", name = "garden_fully_bloomed" }]
barred_message = "The hedge maze remains impenetrable until the garden's magic is fully restored."
hidden = true
```

## Room Organization Patterns

### Hub and Spoke Design

Create central locations that connect to multiple areas:

```toml
# Central hub room
[[rooms]]
id = "town_square"
name = "Market Square"
base_description = "The bustling heart of the town, where all major streets converge around an ancient oak tree."
location = "Nowhere"

[rooms.exits.north]
to = "residential_district"

[rooms.exits.south]
to = "merchant_quarter"

[rooms.exits.east]
to = "temple_district"

[rooms.exits.west]
to = "harbor_district"

[rooms.exits.inn]
to = "travelers_inn"

[rooms.exits.hall]
to = "town_hall"
```

### Linear Progression Areas

Create areas that guide players through a specific sequence:

```toml
# Stage 1 of dungeon
[[rooms]]
id = "dungeon_entrance"
name = "Dungeon Entrance"
base_description = "Stone steps descend into darkness. The air grows cold and still."
location = "Nowhere"

[rooms.exits.down]
to = "first_chamber"

[rooms.exits.up]
to = "castle_basement"

# Stage 2 - can only reach after completing stage 1
[[rooms]]
id = "first_chamber"
name = "Chamber of Trials"
base_description = "A circular chamber with three archways leading deeper into the dungeon."
location = "Nowhere"

[rooms.exits.up]
to = "dungeon_entrance"

[rooms.exits.left]
to = "trial_of_courage"
required_flags = [{ type = "simple", name = "completed_entrance_puzzle" }]

[rooms.exits.right]
to = "trial_of_wisdom"
required_flags = [{ type = "simple", name = "completed_entrance_puzzle" }]

[rooms.exits.center]
to = "trial_of_compassion"
required_flags = [{ type = "simple", name = "completed_entrance_puzzle" }]
```

## Implementation Notes

### Room ID Guidelines
- Use lowercase letters, numbers, and hyphens only ("kebab-case")
- Make IDs descriptive but concise: `throne-room` not `room_where_the_king_sits_all_day`
- Keep consistent naming patterns within areas: `forest_clearing`, `forest_path`, `forest_depths`

### Location Metadata
- The `location` field should always be set to "Nowhere" for rooms (since rooms are the base locations themselves)
- This field is used internally by the engine for location tracking of the player, items, and NPCs -- but not rooms
- Items and NPCs have location fields that reference room IDs, but rooms themselves exist at "Nowhere"

### Room Contents
- These are not defined within rooms.toml!
- Starting locations for items and NPCs are defined within them in items.toml and npcs.toml
- The loader makes a second pass and places items/NPCs into their rooms after all objects have been loaded.

### Performance Considerations
- Overlays are evaluated every time a room is displayed
- All overlay conditions are checked when a room is shown
- The engine handles overlay evaluation efficiently regardless of complexity

## Common Patterns

1. **Discovery and Exploration**: Use hidden exits and item-based overlays to reward thorough exploration
2. **Progressive Revelation**: Show different room aspects as the story unfolds through flag-based overlays
3. **Character Development**: Use NPC-state overlays to reflect relationship changes
4. **Environmental Storytelling**: Layer overlays to tell rich stories through scenery changes
5. **Puzzle Gating**: Use required flags and items to control access and story pacing
6. **Dynamic Atmosphere**: Change room mood and description based on story events

## Tips for Game Designers

### Planning Your World
- **Start with a map**: Sketch out room connections before writing descriptions
- **Think in layers**: Plan base description, then overlay variations for different states
- **Consider pacing**: Use exit requirements to control story progression
- **Plan for changes**: Design rooms that can evolve with your story

### Writing Effective Descriptions
- **Be concise but evocative**: Players will read these descriptions repeatedly
- **Use all senses**: Include sounds, smells, textures, not just visuals
- **Show don't tell**: Use environmental details to convey story and mood
- **Layer information**: Put basic layout in base_description, details in overlays

### Technical Best Practices
- **Test thoroughly**: Verify all exit conditions work as expected
- **Use consistent naming**: Keep room IDs, flag names, and item IDs well-organized
- **Document dependencies**: Note which flags/items each room requires
- **Plan for failure**: Always provide barred_message text for blocked exits

### Common Pitfalls to Avoid
- **Dead ends**: Every room should connect meaningfully to your game world
- **Unclear exits**: Players should understand where exits lead from context
- **Contradictory overlays**: Ensure overlay conditions don't conflict with each other
- **Overwhelming complexity**: Start simple and add complexity gradually

### Accessibility Considerations
- **Clear navigation**: Make exit names intuitive and consistent
- **Alternative paths**: Provide multiple routes when possible for different play styles
- **Descriptive barred messages**: Help players understand what they need to progress
- **Context clues**: Include hints about requirements in room descriptions

Start with simple rooms and establish the basic layout of your first area, and gradually incorporate advanced features as you become comfortable with the system. Remember that the best rooms serve both the story and the gameplay, creating memorable spaces that players enjoy exploring and revisiting -- but not all rooms need all the "bells and whistles!" A strategically placed simple room with a description for storytelling or just part of an NPCs route can break complicated areas up nicely.
