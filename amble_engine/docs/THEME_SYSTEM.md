# Amble Engine Theme System

## Overview

The Amble Engine theme system allows players to customize the color scheme of the game's terminal output. This provides better accessibility, personal preference options, and the ability to match different terminal backgrounds or lighting conditions.

## Using Themes

### In-Game Commands

- `theme` or `theme list` - Display all available themes with the current theme highlighted
- `theme <name>` - Switch to a specific theme (e.g., `theme seaside`)

### Available Themes

#### default
The original Amble color scheme with vibrant colors optimized for dark terminals.
- **Style**: Classic adventure game aesthetic
- **Best for**: Standard dark terminal backgrounds

#### seaside
An ocean-inspired palette featuring blues, corals, and sandy tones.
- **Style**: Calming, nautical atmosphere
- **Best for**: Players who prefer cooler color temperatures

#### forest
Deep greens and earthy browns reminiscent of woodland adventures.
- **Style**: Natural, organic feeling
- **Best for**: Extended play sessions with reduced eye strain

#### monochrome
Classic black and white terminal aesthetic using only grayscale values.
- **Style**: Retro terminal look
- **Best for**: High contrast needs or nostalgic preferences

## Adding Custom Themes

Themes are defined in `amble_engine/data/themes.toml`. To add a new theme:

1. Open `amble_engine/data/themes.toml`
2. Add a new `[[themes]]` section
3. Define all required colors

### Theme Structure

```toml
[[themes]]
name = "my_theme"
description = "Description of your theme"

[themes.colors]
# Prompt and status
prompt = { r = 250, g = 200, b = 100 }      # Command prompt text
prompt_bg = { r = 50, g = 51, b = 50 }      # Command prompt background
status = { r = 20, g = 220, b = 100 }       # Status messages

# General styles
highlight = { r = 255, g = 255, b = 0 }     # Highlighted text
transition = { r = 40, g = 210, b = 160 }   # Movement transitions
subheading = { r = 255, g = 255, b = 255 }  # Section headings

# Items
item = { r = 220, g = 180, b = 40 }         # Item names
item_text = { r = 40, g = 180, b = 40 }     # Item descriptions/text

# NPCs
npc = { r = 50, g = 200, b = 50 }           # NPC names
npc_quote = { r = 100, g = 250, b = 250 }   # NPC dialogue
npc_movement = { r = 40, g = 180, b = 220 } # NPC movement messages

# Rooms
room = { r = 223, g = 77, b = 10 }          # Room names in descriptions
room_titlebar = { r = 223, g = 77, b = 10 } # Room title bar
description = { r = 102, g = 208, b = 250 } # Room descriptions
overlay = { r = 75, g = 180, b = 255 }      # Overlay text

# Triggers and events
triggered = { r = 230, g = 230, b = 30 }    # Triggered event text
trig_icon = { r = 230, g = 80, b = 80 }     # Trigger icons
ambient_icon = { r = 80, g = 80, b = 230 }  # Ambient event icons
ambient_trig = { r = 150, g = 230, b = 30 } # Ambient trigger text

# Exits
exit_visited = { r = 110, g = 220, b = 110 }   # Previously visited exits
exit_locked = { r = 200, g = 50, b = 50 }      # Locked exits
exit_unvisited = { r = 220, g = 180, b = 40 }  # Unvisited exits

# Feedback
error = { r = 230, g = 30, b = 30 }         # Error message text
error_icon = { r = 255, g = 0, b = 0 }      # Error icons
denied = { r = 230, g = 30, b = 30 }        # Access denied messages

# Goals
goal_active = { r = 220, g = 40, b = 220 }     # Active goal text
goal_complete = { r = 110, g = 20, b = 110 }   # Completed goal text

# UI sections
section = { r = 75, g = 80, b = 75 }        # Section dividers
```

## Color Design Guidelines

When creating a new theme, consider:

### Contrast
- Ensure sufficient contrast between text and backgrounds
- Test readability on both dark and light terminal backgrounds
- Maintain distinction between different message types

### Semantic Meaning
- **Green tones**: Success, accessible paths, positive feedback
- **Red tones**: Errors, locked paths, warnings
- **Yellow/Gold**: Important items, highlights, attention
- **Blue/Cyan**: Descriptions, information, dialogue
- **Purple/Magenta**: Goals, special events

### Accessibility
- Avoid relying solely on color to convey information
- Consider colorblind users (red-green colorblindness is most common)
- Test themes with different terminal color settings

### Cohesion
- Choose colors that work harmoniously together
- Maintain a consistent mood throughout the theme
- Consider the game's fantasy adventure setting

## Technical Details

### Architecture
The theme system uses:
- `theme.rs`: Core theme structures and management
- `style.rs`: Application of theme colors to text
- `themes.toml`: Theme definitions

### Theme Loading
Themes are loaded at startup from `amble_engine/data/themes.toml`. If the file is missing or invalid, the system falls back to the hardcoded default theme.

### Performance
Theme colors are cached and only retrieved when text is styled, ensuring minimal performance impact during gameplay.

## Testing Themes

Use the provided test scripts to verify your theme:

```bash
# Show all themes with various game elements
./showcase_themes.sh

# Test specific color categories
./test_all_theme_colors.sh
```

## Troubleshooting

### Theme not appearing
- Ensure the theme name is unique
- Verify all required colors are defined
- Check for TOML syntax errors

### Colors look wrong
- Verify RGB values are between 0-255
- Test on different terminal emulators
- Check terminal's color settings

### Theme won't load
- Look for error messages at game startup
- Ensure `themes.toml` is valid TOML
- Verify file permissions