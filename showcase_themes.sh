#!/bin/bash

# Showcase script for Amble theme system
# Demonstrates all available themes with various game elements

echo "╔════════════════════════════════════════╗"
echo "║     Amble Theme System Showcase       ║"
echo "╚════════════════════════════════════════╝"
echo

echo "This script will demonstrate all available themes."
echo "Each theme will show:"
echo "  • Room descriptions and titles"
echo "  • Item listings"
echo "  • Command feedback"
echo "  • Error messages"
echo
echo "Press Enter to continue..."
read

# Run the game with theme showcase commands
cat << 'EOF' | cargo run --bin amble_engine 2>/dev/null | grep -v "^\[INFO" | grep -v "^\[WARN"
echo
echo "════════════════════════════════════════"
echo "           DEFAULT THEME"
echo "════════════════════════════════════════"
theme default
look
look at plaque
take note
inventory
drop note
take nonexistent
goals

echo
echo "════════════════════════════════════════"
echo "           SEASIDE THEME"
echo "════════════════════════════════════════"
theme seaside
look
look at plaque
take note
inventory
drop note
take nonexistent
goals

echo
echo "════════════════════════════════════════"
echo "           FOREST THEME"
echo "════════════════════════════════════════"
theme forest
look
look at plaque
take note
inventory
drop note
take nonexistent
goals

echo
echo "════════════════════════════════════════"
echo "           MONOCHROME THEME"
echo "════════════════════════════════════════"
theme monochrome
look
look at plaque
take note
inventory
drop note
take nonexistent
goals

echo
echo "════════════════════════════════════════"
echo "         THEME LIST DISPLAY"
echo "════════════════════════════════════════"
theme list

quit
y
EOF

echo
echo "╔════════════════════════════════════════╗"
echo "║       Theme Showcase Complete!        ║"
echo "╚════════════════════════════════════════╝"
echo
echo "You can switch themes at any time during gameplay with:"
echo "  • theme <name>     - Switch to a specific theme"
echo "  • theme list       - Show all available themes"
echo "  • theme           - Show all available themes"
echo
echo "Available themes:"
echo "  • default   - The original Amble color scheme"
echo "  • seaside   - Ocean-inspired blues and corals"
echo "  • forest    - Deep greens and earthy browns"
echo "  • monochrome - Classic black and white terminal"
echo
