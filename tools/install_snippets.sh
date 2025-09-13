#!/bin/bash

# Amble DSL Snippets Installer for Zed Editor
# This script installs Amble DSL code snippets to the correct location for Zed

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SNIPPETS_DIR="$HOME/.config/zed/snippets"
SNIPPETS_FILE="$SNIPPETS_DIR/amble.json"
SOURCE_SNIPPETS="docs/snippets/amble.json"

echo -e "${BLUE}=== Amble DSL Snippets Installer ===${NC}"
echo "Date: $(date)"
echo

print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}!${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}i${NC} $1"
}

# Check if we're in the right directory
if [ ! -d "zed-amble-script" ]; then
    print_error "This script must be run from the amble project root directory"
    print_info "Please cd to the directory containing zed-amble-script/"
    exit 1
fi

# Create the snippets source file from the current extension setup
echo -e "${BLUE}1. Creating Snippets File${NC}"
echo "----------------------------------------"

# Create docs/snippets directory
mkdir -p docs/snippets

# Generate the snippets JSON content
cat > "$SOURCE_SNIPPETS" << 'EOF'
{
  "room": {
    "prefix": "room",
    "body": [
      "room ${1:identifier} {",
      "  name \"${2:Room Name}\"",
      "  desc \"${3:Room description}\"",
      "  visited ${4:false}",
      "  ${0}",
      "}"
    ],
    "description": "Create a new room definition"
  },

  "roomfull": {
    "prefix": "roomfull",
    "body": [
      "room ${1:identifier} {",
      "  name \"${2:Room Name}\"",
      "  desc \"${3:Room description}\"",
      "  visited ${4:false}",
      "  exit ${5:direction} -> ${6:destination}",
      "  ${0}",
      "}"
    ],
    "description": "Create a room with an exit"
  },

  "item": {
    "prefix": "item",
    "body": [
      "item ${1:identifier} {",
      "  name \"${2:Item Name}\"",
      "  desc \"${3:Item description}\"",
      "  portable ${4:true}",
      "  ${0}",
      "}"
    ],
    "description": "Create a new item definition"
  },

  "npc": {
    "prefix": "npc",
    "body": [
      "npc ${1:identifier} {",
      "  name \"${2:NPC Name}\"",
      "  desc \"${3:NPC description}\"",
      "  location room ${4:room_id}",
      "  ${0}",
      "}"
    ],
    "description": "Create a new NPC definition"
  },

  "goal": {
    "prefix": "goal",
    "body": [
      "goal ${1:identifier} \"${2:Goal Title}\"",
      "desc \"${3:Goal description}\"",
      "group ${4:required}",
      "done when ${5:condition}",
      "${0}"
    ],
    "description": "Create a new goal definition"
  },

  "goalcomplex": {
    "prefix": "goalcomplex",
    "body": [
      "goal ${1:identifier} \"${2:Goal Title}\"",
      "desc \"${3:Goal description}\"",
      "group ${4:required}",
      "done when has item ${5:item_id}",
      "done when reached room ${6:room_id}",
      "done when goal complete ${7:goal_id}",
      "${0}"
    ],
    "description": "Create a goal with multiple conditions"
  },

  "trigger": {
    "prefix": "trigger",
    "body": [
      "trigger ${1:identifier} {",
      "  condition \"${2:condition_name}\"",
      "  action \"${3:action_description}\"",
      "  ${0}",
      "}"
    ],
    "description": "Create a new trigger definition"
  },

  "spinner": {
    "prefix": "spinner",
    "body": [
      "spinner ${1:identifier} {",
      "  wedge \"${2:option1}\" width ${3:30}",
      "  wedge \"${4:option2}\" width ${5:40}",
      "  wedge \"${6:option3}\"",
      "  ${0}",
      "}"
    ],
    "description": "Create a new spinner definition"
  },

  "set": {
    "prefix": "set",
    "body": [
      "let set ${1:identifier} = (${2:item1}, ${3:item2}, ${4:item3})${0}"
    ],
    "description": "Create a new set declaration"
  },

  "exit": {
    "prefix": "exit",
    "body": [
      "exit ${1:direction} -> ${2:destination}${0}"
    ],
    "description": "Add an exit to a room"
  },

  "doneitem": {
    "prefix": "doneitem",
    "body": [
      "done when has item ${1:item_id}${0}"
    ],
    "description": "Goal completion condition: has item"
  },

  "doneroom": {
    "prefix": "doneroom",
    "body": [
      "done when reached room ${1:room_id}${0}"
    ],
    "description": "Goal completion condition: reached room"
  },

  "doneflag": {
    "prefix": "doneflag",
    "body": [
      "done when has flag ${1:flag_name}${0}"
    ],
    "description": "Goal completion condition: has flag"
  },

  "donegoal": {
    "prefix": "donegoal",
    "body": [
      "done when goal complete ${1:goal_id}${0}"
    ],
    "description": "Goal completion condition: goal complete"
  },

  "wedge": {
    "prefix": "wedge",
    "body": [
      "wedge \"${1:option}\" width ${2:25}${0}"
    ],
    "description": "Add a wedge to a spinner"
  },

  "section": {
    "prefix": "section",
    "body": [
      "# ================================================",
      "# ${1:Section Title}",
      "# ================================================",
      "${0}"
    ],
    "description": "Create a comment section divider"
  },

  "desclarge": {
    "prefix": "desclarge",
    "body": [
      "desc \"\"\"${1:First line}",
      "${2:Second line}",
      "${3:Third line}\"\"\"${0}"
    ],
    "description": "Create a multi-line description"
  },

  "roomtemplate": {
    "prefix": "roomtemplate",
    "body": [
      "# ${1:Area Name}",
      "",
      "room ${2:identifier} {",
      "  name \"${3:Room Name}\"",
      "  desc \"\"\"${4:Multi-line description}",
      "  ${5:Line 2}",
      "  ${6:Line 3}\"\"\"",
      "  visited false",
      "  exit ${7:direction} -> ${8:destination}",
      "}",
      "${0}"
    ],
    "description": "Complete room template with multi-line description"
  },

  "itemtemplate": {
    "prefix": "itemtemplate",
    "body": [
      "item ${1:identifier} {",
      "  name \"${2:Item Name}\"",
      "  description \"\"\"${3:Multi-line description}",
      "  ${4:Line 2}",
      "  ${5:Line 3}\"\"\"",
      "  portable ${6:true}",
      "}",
      "${0}"
    ],
    "description": "Complete item template with multi-line description"
  }
}
EOF

print_status "Created snippets file: $SOURCE_SNIPPETS"

# Check if Zed is installed
echo
echo -e "${BLUE}2. Checking Zed Installation${NC}"
echo "----------------------------------------"

if command -v zed &> /dev/null; then
    print_status "Zed editor found: $(which zed)"
else
    print_warning "Zed command not found - snippets will be installed but ensure Zed is installed"
fi

# Create Zed snippets directory
echo
echo -e "${BLUE}3. Setting Up Snippets Directory${NC}"
echo "----------------------------------------"

if [ ! -d "$SNIPPETS_DIR" ]; then
    print_info "Creating Zed snippets directory: $SNIPPETS_DIR"
    mkdir -p "$SNIPPETS_DIR"
fi
print_status "Snippets directory ready: $SNIPPETS_DIR"

# Handle existing snippets file
echo
echo -e "${BLUE}4. Installing Snippets${NC}"
echo "----------------------------------------"

if [ -f "$SNIPPETS_FILE" ]; then
    print_warning "Existing Amble snippets found at $SNIPPETS_FILE"

    # Create backup
    BACKUP_FILE="${SNIPPETS_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$SNIPPETS_FILE" "$BACKUP_FILE"
    print_info "Created backup: $BACKUP_FILE"

    read -p "Replace existing snippets? (Y/n): " replace_snippets
    if [[ $replace_snippets =~ ^[Nn]$ ]]; then
        print_info "Installation cancelled - existing snippets preserved"
        exit 0
    fi
fi

# Copy snippets to Zed directory
cp "$SOURCE_SNIPPETS" "$SNIPPETS_FILE"
print_status "Installed snippets to: $SNIPPETS_FILE"

# Verify installation
echo
echo -e "${BLUE}5. Verifying Installation${NC}"
echo "----------------------------------------"

if [ -f "$SNIPPETS_FILE" ]; then
    snippet_count=$(jq 'keys | length' "$SNIPPETS_FILE" 2>/dev/null || echo "unknown")
    print_status "Snippets file exists with $snippet_count snippets"
else
    print_error "Snippets file not found after installation"
    exit 1
fi

# Installation complete
echo
echo -e "${GREEN}=== Installation Complete! ===${NC}"
echo "----------------------------------------"

print_status "Amble DSL snippets installed successfully"
print_info "Location: $SNIPPETS_FILE"
print_info "Snippets count: $snippet_count"

# Usage instructions
echo
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Restart Zed completely (close all windows and reopen)"
echo "2. Open a .amble file in Zed"
echo "3. Try typing a snippet trigger and press Tab:"
echo "   • room + Tab → Room definition template"
echo "   • goal + Tab → Goal definition template"
echo "   • section + Tab → Section comment divider"

echo
echo -e "${YELLOW}Available Snippets:${NC}"
echo "📋 Structure: room, roomfull, item, npc, goal, goalcomplex"
echo "🔧 Advanced: trigger, spinner, set"
echo "🎯 Conditions: doneitem, doneroom, doneflag, donegoal"
echo "⚙️  Utility: exit, wedge, section, desclarge"
echo "📄 Templates: roomtemplate, itemtemplate"

echo
echo -e "${YELLOW}Troubleshooting:${NC}"
echo "• If snippets don't appear, restart Zed completely"
echo "• Make sure you're in a .amble file (shows 'Amble DSL' in status bar)"
echo "• Type the full trigger word before pressing Tab"
echo "• Check that completion is enabled in Zed settings"

echo
print_status "Happy coding with Amble DSL! 🎮"
