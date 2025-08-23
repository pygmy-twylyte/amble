# NpcInState Trigger Condition Example

This document demonstrates how to use the `NpcInState` trigger condition in Amble game files, which enables complex NPC behavior and dynamic storytelling based on character moods and states.

## Overview

The `NpcInState` trigger condition fires when a specific NPC is in a particular state or mood. This enables sophisticated character-driven gameplay such as:

- Mood-dependent NPC interactions and dialogue
- Dynamic story progression based on character relationships
- Emotional consequences for player actions
- Complex character development arcs
- State-driven quest availability and outcomes
- Reactive world events based on NPC feelings

## Understanding NPC States

### Standard States
Amble includes several built-in NPC states that cover common emotional conditions:

- `Normal` - Default, neutral state
- `Happy` - Pleased, cheerful, friendly
- `Sad` - Unhappy, melancholy, withdrawn
- `Mad` - Angry, hostile, aggressive  
- `Bored` - Uninterested, listless, seeking stimulation
- `Tired` - Exhausted, weary, low energy

### Custom States
You can create custom states using the `custom:` prefix for more specific character conditions:

- `custom:suspicious` - Character is wary or distrustful
- `custom:excited` - Character is enthusiastic about something
- `custom:terrified` - Character is extremely frightened
- `custom:infatuated` - Character has romantic feelings
- `custom:guilt_ridden` - Character feels remorse
- `custom:determined` - Character is focused on a goal

## Basic Usage

### Simple State-Dependent Event
```toml
[[triggers]]
name = "Happy Guard Lets You Pass"
only_once = false
conditions = [
    { type = "npcInState", npc_id = "castle_guard", state = "Happy" },
    { type = "withNpc", npc_id = "castle_guard" }
]
actions = [
    { type = "showMessage", text = "The guard is humming cheerfully and waves you through with a smile. 'Beautiful day, isn't it?'" },
    { type = "unlockExit", from_room = "castle_gate", direction = "north" }
]
```

### Custom State Example
```toml
[[triggers]]
name = "Suspicious Merchant Refuses Sale"
only_once = false
conditions = [
    { type = "npcInState", npc_id = "traveling_merchant", state = { custom = "suspicious" } },
    { type = "giveToNpc", item_id = "gold_coins", npc_id = "traveling_merchant" }
]
actions = [
    { type = "showMessage", text = "The merchant eyes you warily. 'I don't trust you. Keep your money, I'm not selling anything to the likes of you.'" },
    { type = "npcRefuseItem", npc_id = "traveling_merchant", reason = "The merchant doesn't trust you right now." }
]
```

## Advanced Examples

### Complex Character Relationship System
```toml
# Initial meeting - NPC starts neutral
[[triggers]]
name = "First Meeting with Scholar"
only_once = true
conditions = [
    { type = "talkToNpc", npc_id = "old_scholar" },
    { type = "npcInState", npc_id = "old_scholar", state = "Normal" }
]
actions = [
    { type = "showMessage", text = "The old scholar looks up from his books with mild curiosity. 'Oh, a visitor. How... unexpected.'" },
    { type = "addFlag", flag = { type = "simple", name = "met_scholar" } }
]

# Player brings requested book - scholar becomes happy
[[triggers]]
name = "Scholar Receives Rare Book"
only_once = true
conditions = [
    { type = "giveToNpc", item_id = "ancient_manuscript", npc_id = "old_scholar" }
]
actions = [
    { type = "showMessage", text = "The scholar's eyes light up with joy! 'By the gods, this is the manuscript I've been searching for! You are truly a friend to scholarship!'" },
    { type = "setNpcState", npc_id = "old_scholar", state = "Happy" },
    { type = "awardPoints", amount = 100 }
]

# Happy scholar offers special services
[[triggers]]
name = "Happy Scholar Offers Translation"
only_once = false
conditions = [
    { type = "npcInState", npc_id = "old_scholar", state = "Happy" },
    { type = "giveToNpc", item_id = "mysterious_scroll", npc_id = "old_scholar" }
]
actions = [
    { type = "showMessage", text = "The scholar beams at you. 'Of course I'll translate this for you, my friend! Anything for someone who appreciates knowledge!'" },
    { type = "despawnItem", item_id = "mysterious_scroll" },
    { type = "giveItemToPlayer", npc_id = "old_scholar", item_id = "translated_scroll" }
]

# But if you steal from the scholar, he becomes mad
[[triggers]]
name = "Scholar Discovers Theft"
only_once = true
conditions = [
    { type = "take", item_id = "scholars_notes" },
    { type = "withNpc", npc_id = "old_scholar" }
]
actions = [
    { type = "showMessage", text = "The scholar notices you taking his notes and his face turns red with fury! 'THIEF! How dare you steal from me!'" },
    { type = "setNpcState", npc_id = "old_scholar", state = "Mad" },
    { type = "addFlag", flag = { type = "simple", name = "scholar_angry" } }
]

# Mad scholar won't help anymore
[[triggers]]
name = "Mad Scholar Refuses Help"
only_once = false
conditions = [
    { type = "npcInState", npc_id = "old_scholar", state = "Mad" },
    { type = "talkToNpc", npc_id = "old_scholar" }
]
actions = [
    { type = "showMessage", text = "The scholar glares at you with pure hatred. 'Get out of my sight, you contemptible thief! I'll never help you again!'" }
]
```

### Multi-Character State Interactions
```toml
# Two NPCs affect each other's moods
[[triggers]]
name = "Lovers' Quarrel Aftermath"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "village_maiden", state = "Sad" },
    { type = "npcInState", npc_id = "young_farmer", state = "Mad" },
    { type = "enter", room_id = "village_square" }
]
actions = [
    { type = "showMessage", text = "The tension in the village square is palpable. The maiden sits crying by the well while the farmer glowers from across the square. Their recent quarrel has affected the whole village." },
    { type = "addFlag", flag = { type = "simple", name = "lovers_quarrel_witnessed" } }
]

# Player can intervene to help
[[triggers]]
name = "Mediate Between Lovers"
only_once = true
conditions = [
    { type = "talkToNpc", npc_id = "village_maiden" },
    { type = "hasFlag", flag = "lovers_quarrel_witnessed" },
    { type = "hasItem", item_id = "love_letter" }
]
actions = [
    { type = "showMessage", text = "You show the maiden the love letter you found. Her eyes widen as she reads it. 'He... he wrote this for me? Oh, I've been such a fool!'" },
    { type = "setNpcState", npc_id = "village_maiden", state = "Happy" },
    { type = "setNpcState", npc_id = "young_farmer", state = "Happy" }
]
```

### State-Dependent Quest Availability
```toml
# Quest only available when NPC is in specific state
[[triggers]]
name = "Desperate Noble's Request"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "lord_pemberton", state = { custom = "desperate" } },
    { type = "talkToNpc", npc_id = "lord_pemberton" }
]
actions = [
    { type = "showMessage", text = "Lord Pemberton grabs your arm desperately. 'Please, you must help me! My daughter has been kidnapped by bandits. I'll give you anything - my fortune, my lands - just bring her back!'" },
    { type = "advanceFlag", flag = "rescue_mission" },
    { type = "awardPoints", amount = 50 }
]

# Different interaction when NPC is calm
[[triggers]]
name = "Calm Noble's Dismissal"
only_once = false
conditions = [
    { type = "npcInState", npc_id = "lord_pemberton", state = "Normal" },
    { type = "talkToNpc", npc_id = "lord_pemberton" }
]
actions = [
    { type = "showMessage", text = "Lord Pemberton looks up from his papers with mild annoyance. 'I'm quite busy with estate matters. Unless you have urgent business, please see my steward.'" }
]
```

### Environmental State Changes
```toml
# Weather affects NPC moods
[[triggers]]
name = "Rainy Day Blues"
only_once = true
conditions = [
    { type = "hasFlag", flag = "rainy_weather" },
    { type = "enter", room_id = "village_square" }
]
actions = [
    { type = "showMessage", text = "The persistent rain has dampened everyone's spirits. The villagers move about with long faces and hunched shoulders." },
    { type = "setNpcState", npc_id = "baker", state = "Sad" },
    { type = "setNpcState", npc_id = "blacksmith", state = "Bored" },
    { type = "setNpcState", npc_id = "innkeeper", state = "Tired" }
]

# Sunshine cheers them up
[[triggers]]
name = "Sunny Day Cheer"
only_once = true
conditions = [
    { type = "hasFlag", flag = "sunny_weather" },
    { type = "enter", room_id = "village_square" }
]
actions = [
    { type = "showMessage", text = "The bright sunshine has lifted everyone's mood. Villagers whistle as they work and greet each other warmly." },
    { type = "setNpcState", npc_id = "baker", state = "Happy" },
    { type = "setNpcState", npc_id = "blacksmith", state = "Happy" },
    { type = "setNpcState", npc_id = "innkeeper", state = "Happy" }
]
```

### Progressive Character Development
```toml
# Character grows more confident through story
[[triggers]]
name = "Timid Apprentice Gains Confidence"
only_once = true
conditions = [
    { type = "hasFlag", flag = "apprentice_success" },
    { type = "enter", room_id = "mage_tower" }
]
actions = [
    { type = "showMessage", text = "The apprentice stands straighter now, her eyes bright with newfound confidence. The successful spell casting has transformed her completely." },
    { type = "setNpcState", npc_id = "mage_apprentice", state = { custom = "confident" } }
]

# Confident apprentice offers advanced help
[[triggers]]
name = "Confident Apprentice Offers Advanced Spell"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "mage_apprentice", state = { custom = "confident" } },
    { type = "talkToNpc", npc_id = "mage_apprentice" }
]
actions = [
    { type = "showMessage", text = "The apprentice smiles confidently. 'I've been practicing the master's advanced techniques. Would you like me to enchant your sword? I know I can do it now!'" },
    { type = "addFlag", flag = { type = "simple", name = "enchantment_available" } }
]
```

### Trauma and Recovery System
```toml
# Traumatic event changes NPC state
[[triggers]]
name = "Witness Traumatic Event"
only_once = true
conditions = [
    { type = "hasFlag", flag = "dragon_attack_occurred" },
    { type = "withNpc", npc_id = "farm_boy" }
]
actions = [
    { type = "showMessage", text = "The farm boy's eyes are wide with terror, his hands shaking uncontrollably. The dragon attack has left him deeply traumatized." },
    { type = "setNpcState", npc_id = "farm_boy", state = { custom = "traumatized" } }
]

# Traumatized NPC has different interactions
[[triggers]]
name = "Traumatized Boy Refuses to Help"
only_once = false
conditions = [
    { type = "npcInState", npc_id = "farm_boy", state = { custom = "traumatized" } },
    { type = "talkToNpc", npc_id = "farm_boy" }
]
actions = [
    { type = "showMessage", text = "The boy flinches away from you, his voice barely a whisper. 'I... I can't... the dragon... it was so terrible... please, just leave me alone.'" }
]

# Recovery through player help
[[triggers]]
name = "Comfort Traumatized Boy"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "farm_boy", state = { custom = "traumatized" } },
    { type = "giveToNpc", item_id = "mothers_locket", npc_id = "farm_boy" }
]
actions = [
    { type = "showMessage", text = "The boy clutches his mother's locket to his chest, tears streaming down his face. Slowly, some of the terror fades from his eyes. 'Thank you... thank you for bringing this back to me.'" },
    { type = "setNpcState", npc_id = "farm_boy", state = "Sad" }  # Better than traumatized
]
```

## State Management Patterns

### State Chains and Progressions
```toml
# Create emotional journey for character
# Normal -> Happy -> Excited -> Overconfident -> Humbled

[[triggers]]
name = "Stage 1: Make Guard Happy"
only_once = true
conditions = [
    { type = "giveToNpc", item_id = "ale_mug", npc_id = "gate_guard" }
]
actions = [
    { type = "setNpcState", npc_id = "gate_guard", state = "Happy" }
]

[[triggers]]
name = "Stage 2: Guard Becomes Excited"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "gate_guard", state = "Happy" },
    { type = "giveToNpc", item_id = "promotion_letter", npc_id = "gate_guard" }
]
actions = [
    { type = "setNpcState", npc_id = "gate_guard", state = { custom = "excited" } }
]

[[triggers]]
name = "Stage 3: Guard Becomes Overconfident"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "gate_guard", state = { custom = "excited" } },
    { type = "hasFlag", flag = "praised_guard_skills" }
]
actions = [
    { type = "setNpcState", npc_id = "gate_guard", state = { custom = "overconfident" } }
]

[[triggers]]
name = "Stage 4: Guard Gets Humbled"
only_once = true
conditions = [
    { type = "npcInState", npc_id = "gate_guard", state = { custom = "overconfident" } },
    { type = "hasFlag", flag = "guard_failed_test" }
]
actions = [
    { type = "setNpcState", npc_id = "gate_guard", state = "Normal" },
    { type = "showMessage", text = "The guard's overconfidence has been shaken by his failure. He seems more thoughtful now, perhaps wiser." }
]
```

## Implementation Notes

### State Persistence
- NPC states persist across game sessions when saved/loaded
- States can be changed through trigger actions or other game events
- Each NPC can only be in one state at a time

### Dialogue Integration
- NPCs can have different dialogue sets for each state
- States affect which dialogue lines are available when talking to NPCs
- Custom states require corresponding dialogue entries in NPC definitions

### Performance Considerations
- State checks are efficient and can be used frequently
- Multiple NPCs can be checked simultaneously in complex triggers
- States are internally managed and optimized by the game engine

## Common Patterns

1. **Relationship Tracking**: Use states to represent player relationships with NPCs
2. **Emotional Consequences**: Change NPC states based on player actions
3. **Dynamic Quests**: Make quest availability depend on NPC emotional states
4. **Character Development**: Show character growth through state progressions
5. **Environmental Responses**: Have NPCs react to world events through state changes
6. **Social Systems**: Create interconnected NPC relationships through shared state changes

## Tips for Game Designers

### Planning Character Psychology
- **Consistent Motivation**: Ensure state changes make sense for each character
- **Emotional Range**: Give important NPCs a full range of possible states
- **Recovery Paths**: Provide ways for players to repair damaged relationships
- **Meaningful Consequences**: Make NPC states affect gameplay in significant ways

### Balancing Complexity
- **Start Simple**: Begin with basic Happy/Sad/Mad states before adding custom ones
- **Document Relationships**: Keep track of which NPCs interact and affect each other
- **Test State Chains**: Verify that complex state progressions work as intended
- **Player Agency**: Ensure players have reasonable control over NPC relationships

### Narrative Integration
- **Show, Don't Tell**: Use states to show character development through actions
- **Emotional Pacing**: Match NPC state changes to story beats and player progression
- **World Consistency**: Ensure NPC states make sense within your game's world
- **Player Feedback**: Make NPC state changes visible and understandable to players

### Common Pitfalls to Avoid
- **Arbitrary Changes**: Don't change NPC states without clear in-world reasons
- **Permanent Hostility**: Always provide paths for players to repair relationships
- **Overcomplicated Systems**: Keep state systems manageable and understandable
- **Inconsistent Behavior**: Ensure NPCs behave consistently within their current state

This trigger condition is perfect for creating rich, dynamic characters that respond emotionally to player actions and story events, making the game world feel more alive and reactive to player choices.