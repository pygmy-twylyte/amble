# Amble DSL Snippets Cheat Sheet

This document lists all available code snippets for the Amble DSL Zed extension. Simply type the trigger word and press `Tab` to expand the snippet.

## 🏗️ **Basic Structure Snippets**

### `room` + Tab
```
room identifier {
  name "Room Name"
  desc "Room description"
  visited false
  
}
```
Creates a basic room definition with all common properties.

### `roomfull` + Tab
```
room identifier {
  name "Room Name"
  desc "Room description"
  visited false
  exit direction -> destination
  
}
```
Creates a room definition with an exit included.

### `roomtemplate` + Tab
```
# Area Name

room identifier {
  name "Room Name"
  desc """Multi-line description
  Line 2
  Line 3"""
  visited false
  exit direction -> destination
}

```
Complete room template with section comment and multi-line description.

### `item` + Tab
```
item identifier {
  name "Item Name"
  desc "Item description"
  portable true
  
}
```
Creates a basic item definition.

### `itemtemplate` + Tab
```
item identifier {
  name "Item Name"
  description """Multi-line description
  Line 2
  Line 3"""
  portable true
}

```
Complete item template with multi-line description.

### `npc` + Tab
```
npc identifier {
  name "NPC Name"
  desc "NPC description"
  location room room_id
  
}
```
Creates a basic NPC definition.

## 🎯 **Goal and Condition Snippets**

### `goal` + Tab
```
goal identifier {
  name "Goal Title"
  desc "Goal description"
  group required
  done when condition
}

```
Creates a basic goal definition.

### `goalcomplex` + Tab
```
goal identifier {
  name "Goal Title"
  desc "Goal description"
  group required
  done when has item item_id
  done when reached room room_id
  done when goal complete goal_id
}

```
Creates a goal with multiple completion conditions.

### Individual Condition Snippets

#### `doneitem` + Tab
```
done when has item identifier
```

#### `doneroom` + Tab
```
done when reached room identifier
```

#### `doneflag` + Tab
```
done when has flag identifier
```

#### `donegoal` + Tab
```
done when goal complete identifier
```

## 🔧 **Advanced Structure Snippets**

### `trigger` + Tab
```
trigger identifier {
  condition "condition_name"
  action "action_description"
  
}
```
Creates a trigger definition.

### `spinner` + Tab
```
spinner identifier {
  wedge "option1" width 30
  wedge "option2" width 40
  wedge "option3"
  
}
```
Creates a spinner with multiple wedges.

### `set` + Tab
```
let set identifier = (item1, item2, item3)
```
Creates a set declaration.

## 🛠️ **Utility Snippets**

### `exit` + Tab
```
exit direction -> destination
```
Adds an exit line (useful within room definitions).

### `wedge` + Tab
```
wedge "option" width 25
```
Adds a wedge line (useful within spinner definitions).

### `section` + Tab
```
# ================================================
# Section Title
# ================================================

```
Creates a section divider comment.

### `desclarge` + Tab
```
desc """Multi-line description
Line 2
Line 3"""
```
Creates a multi-line description block.

## 📝 **Usage Tips**

### Tab Stops
Snippets use tab stops (`$1`, `$2`, etc.) to let you quickly fill in values:
1. Type the trigger word and press `Tab`
2. The snippet expands with the first field selected
3. Type your value and press `Tab` to move to the next field
4. Continue until all fields are filled
5. Press `Tab` one final time to move to the end position (`$0`)

### Common Patterns

**Quick Room Creation:**
1. `section` + Tab → "Room Definitions"
2. `roomtemplate` + Tab → Fill in room details
3. Repeat for additional rooms

**Goal Setup:**
1. `goalcomplex` + Tab → Multi-condition goal
2. Add more conditions with `doneitem`, `doneroom`, etc.

**Item Collection:**
1. `section` + Tab → "Items"
2. Multiple `item` + Tab for quick item creation
3. Use `itemtemplate` for detailed items

### Snippet Categories Summary

| Category | Snippets | Purpose |
|----------|----------|---------|
| **Rooms** | `room`, `roomfull`, `roomtemplate` | Room definitions |
| **Items** | `item`, `itemtemplate` | Item definitions |
| **NPCs** | `npc` | Character definitions |
| **Goals** | `goal`, `goalcomplex`, `done*` | Objective system |
| **Advanced** | `trigger`, `spinner`, `set` | Complex mechanics |
| **Utility** | `exit`, `wedge`, `section`, `desclarge` | Building blocks |

## 🎨 **Best Practices**

1. **Start with templates**: Use `roomtemplate` and `itemtemplate` for detailed definitions
2. **Use sections**: Organize your code with `section` snippets
3. **Multi-line descriptions**: Use `desclarge` for rich, detailed text
4. **Build incrementally**: Start with basic snippets, then enhance with specific additions

---

*These snippets are designed to make Amble DSL development faster and more consistent. The tab stop system allows for rapid content creation while maintaining proper structure.*
