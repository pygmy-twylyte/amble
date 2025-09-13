# Zed Language Extension Setup Guide

This document explains how to properly set up a language extension for Zed editor, based on lessons learned from developing the Amble DSL extension.

## 🚨 Critical Discovery: Extension Installation Location

**IMPORTANT**: Zed extensions are installed to a different location than initially expected!

### Incorrect Location (doesn't work)
```
~/.config/zed/extensions/your_extension/
```

### Correct Location (works!)
```
~/.local/share/zed/extensions/installed/your_extension/
```

## 📁 Correct Directory Structure

For a language extension to work properly in Zed, the structure must be:

```
~/.local/share/zed/extensions/installed/your_extension/
├── extension.toml                    # Extension configuration
├── languages/
│   └── your_language/
│       ├── config.toml              # Language configuration
│       ├── highlights.scm           # ⚠️ DIRECTLY HERE, not in queries/
│       ├── folds.scm               # ⚠️ DIRECTLY HERE, not in queries/
│       ├── indents.scm             # ⚠️ DIRECTLY HERE, not in queries/
│       └── tags.scm                # ⚠️ DIRECTLY HERE, not in queries/
└── grammars/
    ├── your_language.wasm          # Compiled grammar (optional)
    └── your_language/              # Grammar source (optional)
        └── [grammar files...]
```

### 🔴 Common Mistake: Queries Subdirectory

**This structure DOES NOT WORK:**
```
languages/
└── your_language/
    ├── config.toml
    └── queries/          # ❌ Wrong! Zed doesn't look here
        ├── highlights.scm
        ├── folds.scm
        └── [other .scm files]
```

**This structure WORKS:**
```
languages/
└── your_language/
    ├── config.toml
    ├── highlights.scm    # ✅ Correct! Directly in language directory
    ├── folds.scm        # ✅ Correct!
    └── [other .scm files]
```

## 🔧 Installation Methods

### Method 1: Direct Copy
```bash
# Copy your extension to the correct location
cp -r your-extension-directory ~/.local/share/zed/extensions/installed/your_extension_name
```

### Method 2: Symlink (for development)
```bash
# Create a symlink for easy development
ln -s /path/to/your/extension/source ~/.local/share/zed/extensions/installed/your_extension_name
```

The symlink method is great for development because changes to your source files automatically apply to the installed extension.

## 📋 Extension Configuration Files

### extension.toml
```toml
id = "your_extension_id"
name = "Your Language Name"
version = "0.0.1"
schema_version = 1
authors = ["Your Name <your.email@example.com>"]
description = "Description of your language extension"

# Register language definitions
languages = ["languages/your_language"]

# Optional: Grammar configuration (if using remote grammar)
[grammars.your_language]
repository = "https://github.com/user/tree-sitter-your-language"
rev = "commit_sha_here"
```

### languages/your_language/config.toml
```toml
name = "Your Language Name"
grammar = "your_language"
path_suffixes = ["your_ext"]           # File extensions (without dot)
line_comments = ["# "]                 # Comment syntax
scopes = ["source.your_language"]      # Language scope
```

### languages/your_language/highlights.scm
```scheme
; Syntax highlighting queries using tree-sitter query syntax
; Use standard TextMate scope names for maximum theme compatibility

(comment) @comment.line
(string) @string.quoted
(number) @constant.numeric
(boolean) @constant.language.boolean

; Keywords
"keyword1" @keyword
"keyword2" @keyword

; Operators
"=" @keyword.operator
"->" @keyword.operator

; Punctuation
"{" @punctuation.definition.block.begin
"}" @punctuation.definition.block.end

; Identifiers
(identifier) @variable.other
```

## 🛠️ Development Workflow

### 1. Create Extension Structure
```bash
# Create your extension directory
mkdir -p your-extension/languages/your_language

# Create required files
touch your-extension/extension.toml
touch your-extension/languages/your_language/config.toml
touch your-extension/languages/your_language/highlights.scm
```

### 2. Install for Development
```bash
# Symlink for easy development
ln -s $(pwd)/your-extension ~/.local/share/zed/extensions/installed/your_extension_id
```

### 3. Test and Iterate
1. Edit your source files
2. Restart Zed completely (important!)
3. Open a test file with your language extension
4. Verify language detection and syntax highlighting

## 🚨 Troubleshooting

### Extension Not Loading
- Check that files are in `~/.local/share/zed/extensions/installed/` (not `~/.config/zed/`)
- Ensure `extension.toml` has correct `id` and `languages` configuration
- Restart Zed completely after installation

### Language Not Detected
- Verify `path_suffixes` in `config.toml` matches your file extensions
- Check that grammar name in `config.toml` matches grammar name in tree-sitter grammar
- Ensure `languages` array in `extension.toml` points to correct directory

### Syntax Highlighting Not Working
- **Most common**: `.scm` files are in `queries/` subdirectory instead of directly in language directory
- Move all `.scm` files from `languages/your_lang/queries/` to `languages/your_lang/`
- Check that highlight queries use valid tree-sitter syntax
- Verify scope names are compatible with your theme
- Restart Zed after query changes

### Debugging Commands
```bash
# Check if extension is installed
ls -la ~/.local/share/zed/extensions/installed/

# Run Zed with debug output
zed --foreground your_test_file.ext

# Check symlink status (if using symlink installation)
ls -la ~/.local/share/zed/extensions/installed/your_extension
```

## 📚 Example: Amble DSL Extension

The Amble DSL extension provides a complete working example:

```
amble/zed-amble-script/
├── extension.toml
├── languages/
│   └── amble_dsl/
│       ├── config.toml
│       ├── highlights.scm      # Comprehensive syntax highlighting
│       ├── folds.scm          # Code folding rules
│       ├── indents.scm        # Indentation rules
│       └── tags.scm           # Symbol tagging
└── grammars/
    ├── amble_dsl.wasm         # Compiled grammar
    └── [other grammar files...]
```

Installed as:
```bash
ln -s ~/Code/rust/amble/zed-amble-script ~/.local/share/zed/extensions/installed/amble_dsl
```

## ✅ Success Indicators

When your extension is working correctly:

1. **Extension loads**: No errors in `zed --foreground` output
2. **Language detection**: Files show correct language in Zed status bar
3. **Syntax highlighting**: Comments, keywords, strings appear in different colors
4. **Code folding**: Braces and blocks can be folded/unfolded
5. **Consistent behavior**: Works across all files with your extension

## 🎯 Key Takeaways

1. **Location matters**: Use `~/.local/share/zed/extensions/installed/`
2. **No queries folder**: Put `.scm` files directly in `languages/your_lang/`
3. **Restart required**: Always restart Zed completely after changes
4. **Symlinks work great**: Perfect for extension development
5. **Debug with logs**: Use `zed --foreground` to see error messages

This setup ensures your Zed language extension will load and function properly!