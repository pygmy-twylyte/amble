#!/bin/bash

# Zed Amble DSL Extension Installer
# This script properly installs the Amble DSL language extension for Zed editor

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
EXTENSION_NAME="amble_dsl"
SOURCE_DIR="zed-amble-script"
ZED_EXTENSIONS_DIR="$HOME/.local/share/zed/extensions/installed"
TARGET_DIR="$ZED_EXTENSIONS_DIR/$EXTENSION_NAME"

echo -e "${BLUE}=== Zed Amble DSL Extension Installer ===${NC}"
echo "Date: $(date)"
echo

# Function to print colored output
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

# Check prerequisites
echo -e "${BLUE}1. Checking Prerequisites${NC}"
echo "----------------------------------------"

# Check if source directory exists
if [ ! -d "$SOURCE_DIR" ]; then
    print_error "Source directory '$SOURCE_DIR' not found!"
    print_info "Make sure you're running this script from the amble project root directory."
    exit 1
fi
print_status "Source directory found: $SOURCE_DIR"

# Check if Zed is installed
if ! command -v zed &> /dev/null; then
    print_warning "Zed command not found in PATH"
    print_info "The extension will still be installed, but make sure Zed is installed."
else
    print_status "Zed editor found: $(which zed)"
fi

# Create Zed extensions directory if it doesn't exist
if [ ! -d "$ZED_EXTENSIONS_DIR" ]; then
    print_info "Creating Zed extensions directory: $ZED_EXTENSIONS_DIR"
    mkdir -p "$ZED_EXTENSIONS_DIR"
fi
print_status "Zed extensions directory ready: $ZED_EXTENSIONS_DIR"

# Check source structure
echo
echo -e "${BLUE}2. Validating Source Structure${NC}"
echo "----------------------------------------"

required_files=(
    "$SOURCE_DIR/extension.toml"
    "$SOURCE_DIR/languages/amble_dsl/config.toml"
    "$SOURCE_DIR/languages/amble_dsl/highlights.scm"
    "$SOURCE_DIR/languages/amble_dsl/folds.scm"
)

for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        print_status "Found: $file"
    else
        print_error "Missing required file: $file"
        exit 1
    fi
done

# Check for compiled grammar
if [ -f "$SOURCE_DIR/grammars/amble_dsl.wasm" ]; then
    print_status "Found compiled grammar: grammars/amble_dsl.wasm"
else
    print_warning "Compiled grammar not found - extension may not work without it"
fi

# Installation method selection
echo
echo -e "${BLUE}3. Installation Method${NC}"
echo "----------------------------------------"

if [ "$1" = "--copy" ] || [ "$1" = "-c" ]; then
    INSTALL_METHOD="copy"
elif [ "$1" = "--symlink" ] || [ "$1" = "-s" ]; then
    INSTALL_METHOD="symlink"
else
    echo "Choose installation method:"
    echo "  [1] Symlink (recommended for development) - changes to source files apply immediately"
    echo "  [2] Copy - creates independent copy of extension"
    echo
    read -p "Enter choice (1 or 2): " choice

    case $choice in
        1) INSTALL_METHOD="symlink" ;;
        2) INSTALL_METHOD="copy" ;;
        *) print_error "Invalid choice. Use 1 or 2."; exit 1 ;;
    esac
fi

print_info "Using installation method: $INSTALL_METHOD"

# Remove existing installation if present
if [ -e "$TARGET_DIR" ]; then
    print_warning "Existing installation found at $TARGET_DIR"
    read -p "Remove existing installation? (y/N): " remove_existing

    if [[ $remove_existing =~ ^[Yy]$ ]]; then
        rm -rf "$TARGET_DIR"
        print_status "Removed existing installation"
    else
        print_error "Installation cancelled - existing installation not removed"
        exit 1
    fi
fi

# Install extension
echo
echo -e "${BLUE}4. Installing Extension${NC}"
echo "----------------------------------------"

if [ "$INSTALL_METHOD" = "symlink" ]; then
    # Create symlink
    SOURCE_ABSOLUTE=$(readlink -f "$SOURCE_DIR")
    ln -s "$SOURCE_ABSOLUTE" "$TARGET_DIR"
    print_status "Created symlink: $TARGET_DIR -> $SOURCE_ABSOLUTE"
    print_info "Changes to source files will automatically apply to the extension"
else
    # Copy files
    cp -r "$SOURCE_DIR" "$TARGET_DIR"
    print_status "Copied extension to: $TARGET_DIR"
    print_info "Extension is now independent of source directory"
fi

# Verify installation
echo
echo -e "${BLUE}5. Verifying Installation${NC}"
echo "----------------------------------------"

if [ -d "$TARGET_DIR" ]; then
    print_status "Extension directory created successfully"
else
    print_error "Extension directory not found after installation"
    exit 1
fi

# Check key files
verification_files=(
    "$TARGET_DIR/extension.toml"
    "$TARGET_DIR/languages/amble_dsl/config.toml"
    "$TARGET_DIR/languages/amble_dsl/highlights.scm"
)

for file in "${verification_files[@]}"; do
    if [ -f "$file" ]; then
        print_status "Verified: $(basename "$file")"
    else
        print_error "Missing after installation: $file"
        exit 1
    fi
done

# Check for queries subdirectory (should not exist)
QUERIES_DIR="$TARGET_DIR/languages/amble_dsl/queries"
if [ -d "$QUERIES_DIR" ]; then
    print_warning "Found 'queries' subdirectory - this may prevent highlighting from working"
    print_info "Zed expects .scm files directly in the language directory, not in a queries/ subdirectory"

    # Offer to fix it
    read -p "Move .scm files from queries/ to parent directory? (Y/n): " fix_queries
    if [[ ! $fix_queries =~ ^[Nn]$ ]]; then
        mv "$QUERIES_DIR"/*.scm "$TARGET_DIR/languages/amble_dsl/" 2>/dev/null || true
        rmdir "$QUERIES_DIR" 2>/dev/null || true
        print_status "Moved .scm files to correct location"
    fi
fi

# Installation complete
echo
echo -e "${GREEN}=== Installation Complete! ===${NC}"
echo "----------------------------------------"

print_status "Amble DSL extension installed successfully"
print_info "Installation location: $TARGET_DIR"
print_info "Installation method: $INSTALL_METHOD"

# Next steps
echo
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Restart Zed completely (close all windows and reopen)"
echo "2. Open a .amble file to test the extension"
echo "3. Check that the language appears as 'Amble DSL' in Zed's status bar"
echo "4. Verify syntax highlighting is working (comments, keywords, strings should have colors)"

echo
echo -e "${YELLOW}Testing Files:${NC}"
echo "• comprehensive_highlight_test.amble - Complete syntax test"
echo "• test_highlighting.amble - Basic test file"
echo "• zed_highlight_test.amble - Generated test file"

echo
echo -e "${YELLOW}Troubleshooting:${NC}"
echo "• Run 'zed --foreground test_file.amble' to see debug output"
echo "• Check that .scm files are directly in languages/amble_dsl/ (not in queries/)"
echo "• Ensure Zed version is compatible with tree-sitter extensions"

# Optional: Offer to open test file
if command -v zed &> /dev/null && [ -f "comprehensive_highlight_test.amble" ]; then
    echo
    read -p "Open test file in Zed now? (Y/n): " open_test
    if [[ ! $open_test =~ ^[Nn]$ ]]; then
        print_info "Opening test file in Zed..."
        zed comprehensive_highlight_test.amble &
        print_status "Zed should now open with the test file - check if syntax highlighting is working!"
    fi
fi

echo
echo -e "${GREEN}Installation script completed successfully!${NC}"
