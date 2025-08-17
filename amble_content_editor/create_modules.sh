#!/bin/bash

# Create README
cat > amble_content_editor/README.md << 'EOF'
# Amble Content Editor

A comprehensive GTK4-based content editor for the Amble game engine, designed to make creating and editing game content intuitive and error-free.

## Features

### Core Functionality
- **TOML Validation**: Real-time validation of TOML syntax and structure
- **Undo/Redo System**: Complete undo/redo support with unlimited history
- **Auto-save & Backups**: Automatic backups with configurable intervals
- **Multi-file Editing**: Tab-based interface for editing multiple files simultaneously

### Smart Editing
- **Entity Dropdowns**: Smart dropdown selectors for rooms, items, NPCs, and other game entities
- **Trigger Editor**: Visual trigger editor with dropdowns for conditions and actions
- **Reference Validation**: Automatic validation of entity references (e.g., room exits, item locations)
- **Comment Support**: Preserve and edit comments in TOML files, especially for triggers

### User Interface
- **GTK4 + Libadwaita**: Modern, native Linux desktop experience
- **Dark/Light Theme**: Follows system theme preferences
- **Search & Replace**: Global search across all game files
- **Property Inspector**: Visual property editor for game entities
- **Status Indicators**: Real-time validation status and file modification indicators

## Installation

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev libadwaita-1-dev

# Fedora
sudo dnf install gtk4-devel libadwaita-devel

# Arch
sudo pacman -S gtk4 libadwaita
```

### Building
```bash
cd amble_content_editor
cargo build --release
```

### Running
```bash
cargo run --release
# Or after building:
./target/release/amble-editor
```

## Usage

### Opening a Project
1. Click "Open Project" or press `Ctrl+O`
2. Navigate to your `amble_engine/data` directory
3. The editor will load all TOML files automatically

### Editing Entities

#### Rooms
- Visual room editor with exit management
- Dropdown selectors for connected rooms
- Overlay condition editor
- Flag requirement editor for exits

#### Items
- Item property editor with all fields
- Container state management
- Ability editor with type selection
- Location selector with room/NPC/container options

#### NPCs
- NPC state editor
- Dialogue management per state
- Movement pattern configuration
- Location assignment

#### Triggers
- Visual trigger builder
- Condition selector with appropriate fields
- Action selector with parameter helpers
- Comment field for documentation
- "Only once" toggle

### Validation

The editor provides three levels of validation:

1. **Syntax Validation**: TOML syntax checking
2. **Reference Validation**: Ensures all entity references exist
3. **Logic Validation**: Checks for logical inconsistencies

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New Project |
| `Ctrl+O` | Open Project |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+F` | Find |
| `Ctrl+H` | Find & Replace |
| `Ctrl+Shift+V` | Validate All |
| `Ctrl+Q` | Quit |

## Architecture

The editor is built with a modular architecture:

- `app.rs`: Main application logic and state management
- `ui/`: User interface components
  - `editors/`: Specific editors for each entity type
  - `widgets/`: Reusable UI widgets
- `data/`: Data models and TOML handling
- `validation/`: Validation logic
- `utils/`: Utility functions and helpers

## Contributing

Contributions are welcome! Please ensure:
1. Code follows Rust best practices
2. New features include tests
3. UI changes follow GNOME HIG guidelines
4. Documentation is updated

## License

Same as the Amble engine - MIT OR Apache-2.0
EOF

chmod +x amble_content_editor/create_modules.sh
