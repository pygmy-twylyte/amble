#!/bin/bash

# Test script for Amble theme system
# This script showcases different colored elements in both themes

echo "Testing Amble Theme System"
echo "=========================="
echo

# Test commands that showcase different color categories
cat << 'EOF' | cargo run --bin amble_engine 2>/dev/null | grep -v "^\[INFO" | grep -v "^\[WARN"
theme list
look
inventory
goals
theme seaside
look
inventory
goals
help
theme default
quit
y
EOF

echo
echo "Theme test completed!"
