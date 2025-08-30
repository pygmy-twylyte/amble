#!/bin/bash

# Comprehensive test script for Amble theme system
# This script tests all color categories in both themes

echo "=================================="
echo "Amble Theme System - Full Test"
echo "=================================="
echo

# Test commands that showcase all different color categories
cat << 'EOF' | cargo run --bin amble_engine 2>/dev/null | grep -v "^\[INFO" | grep -v "^\[WARN"
echo "==== DEFAULT THEME ===="
theme list
look
look at plaque
inventory
goals
go up
go down
take note
inventory
drop note
help
echo "==== SWITCHING TO SEASIDE ===="
theme seaside
theme list
look
look at plaque
inventory
goals
go up
go down
take note
inventory
drop note
help
echo "==== BACK TO DEFAULT ===="
theme default
look
quit
y
EOF

echo
echo "=================================="
echo "Theme test completed!"
echo "=================================="
echo
echo "The test showcased the following styled elements:"
echo "- Room titles and descriptions (room colors)"
echo "- Items in rooms and inventory (item colors)"
echo "- NPCs when present (npc colors)"
echo "- Exits with different visit states (exit colors)"
echo "- Goals active/complete (goal colors)"
echo "- Status messages and prompts (status/prompt colors)"
echo "- Error messages (error colors)"
echo "- Help text (various text styles)"
echo "- Section dividers (section colors)"
echo
