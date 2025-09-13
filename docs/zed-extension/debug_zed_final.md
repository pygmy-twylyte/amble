# Zed Amble DSL Extension Debugging Guide

## Current Status

### ✅ What's Working
- **Extension Installation**: Correctly installed in `~/.config/zed/extensions/amble_dsl/`
- **Language Detection**: `.amble` files are recognized as "Amble DSL"
- **Grammar Loading**: Tree-sitter grammar is loaded (folding works)
- **File Association**: Path suffixes correctly map `.amble` files
- **Grammar Compilation**: WASM file exists and appears valid
- **Query Syntax**: Highlight queries have valid tree-sitter syntax

### ❌ What's NOT Working  
- **Syntax Highlighting**: No colors applied to any syntax elements
- **All text appears plain**: Comments, keywords, strings all same color
- **Affects all .amble files**: Problem is consistent across files

## Technical Analysis

### Extension Structure
```
~/.config/zed/extensions/amble_dsl/
├── extension.toml              # Extension configuration
├── grammars/
│   ├── amble_dsl.wasm         # Compiled grammar (19KB) ✅
│   └── amble_dsl/             # Grammar source copy
│       └── queries/
│           └── highlights.scm  # Highlight queries
└── languages/
    └── amble_dsl/
        ├── config.toml        # Language configuration ✅
        └── queries/
            ├── highlights.scm  # Primary highlight queries ✅
            ├── folds.scm      # Folding queries (working) ✅
            ├── indents.scm    # Indent queries ✅
            └── tags.scm       # Tag queries ✅
```

### Grammar Verification
- **Name**: `amble_dsl` (matches config)
- **Parsing**: Successfully parses test files
- **Nodes**: Produces expected AST nodes (`comment`, `string`, `number`, etc.)
- **Rev**: `e942fe3ee1e05ff2312d546c8884906827ee2720`

### Query File Status
- **Location**: Present in both `languages/amble_dsl/queries/` and `grammars/amble_dsl/queries/`
- **Syntax**: Valid tree-sitter query syntax
- **Scopes**: Using standard TextMate scope names
- **Content**: Currently minimal test (comment, string, number only)

## Debugging Steps

### 1. Verify Query Loading Location
Zed might be looking for queries in a different location than expected.

**Test locations:**
- `~/.config/zed/extensions/amble_dsl/languages/amble_dsl/queries/highlights.scm`
- `~/.config/zed/extensions/amble_dsl/grammars/amble_dsl/queries/highlights.scm`

**Action**: Ensure both files are identical and contain valid queries.

### 2. Check Zed Logs
Zed may be logging errors when loading extensions or queries.

**Log locations to check:**
- `~/.config/zed/logs/`
- Console output when running `zed --foreground`

**Look for:**
- Extension loading errors
- Tree-sitter compilation errors
- Query parsing failures
- Grammar loading issues

### 3. Test with Absolute Minimal Query
Create the simplest possible highlight query to isolate the issue.

**Ultra-minimal test:**
```scheme
; Test: Single comment highlight only
(comment) @comment
```

**If this doesn't work**: The issue is fundamental (grammar loading, query location, or Zed compatibility).

### 4. Grammar Name Consistency Check
Verify all references to the grammar use consistent naming.

**Files to check:**
- `extension.toml`: `[grammars.amble_dsl]`
- `languages/amble_dsl/config.toml`: `grammar = "amble_dsl"`
- `tree-sitter-amble-script/grammar.js`: `name: 'amble_dsl'`

### 5. Compare with Working Extension
Find a working Zed language extension for comparison.

**Check for differences in:**
- Directory structure
- Configuration format
- Query file naming
- Scope naming conventions

## Possible Root Causes

### 1. Query File Priority
**Issue**: Zed may prioritize queries from `grammars/` over `languages/`
**Solution**: Ensure both locations have identical, working queries

### 2. Scope Name Incompatibility  
**Issue**: Zed may not support certain TextMate scope names
**Solution**: Test with the most basic scopes: `@comment`, `@string`, `@keyword`

### 3. Grammar/Query Mismatch
**Issue**: Queries reference nodes that the grammar doesn't produce
**Solution**: Verify node names match grammar output exactly

### 4. Extension Loading Order
**Issue**: Extension may be loaded before grammar is ready
**Solution**: Complete Zed restart after any changes

### 5. Tree-sitter Version Compatibility
**Issue**: Query syntax may be for newer/older tree-sitter version
**Solution**: Check Zed's tree-sitter version requirements

### 6. WASM Compilation Issues
**Issue**: Grammar WASM may be corrupted or incompatible
**Solution**: Regenerate WASM file or check compilation errors

## Systematic Testing Protocol

### Phase 1: Minimal Validation
1. Create single-line highlight query: `(comment) @comment`
2. Restart Zed completely
3. Open test file with comments
4. Check if comments are highlighted

**If fails**: Issue is fundamental (location, loading, compatibility)
**If works**: Gradually add more query rules

### Phase 2: Incremental Building
1. Add string highlighting: `(string) @string`
2. Add number highlighting: `(number) @number`
3. Add keyword highlighting: `"room" @keyword`
4. Test after each addition

### Phase 3: Advanced Features
1. Add complex node queries: `(room_def (identifier) @type)`
2. Add nested queries and conditional patterns
3. Optimize for performance and completeness

## Recovery Actions

### If Nothing Works
1. **Remove extension**: `rm -rf ~/.config/zed/extensions/amble_dsl`
2. **Clean restart**: Restart Zed
3. **Manual reinstall**: Copy fresh extension files
4. **Test with different extension**: Verify Zed extensions work generally

### If Partial Success
1. **Document what works**: Note which queries/scopes succeed
2. **Build incrementally**: Add complexity gradually
3. **Compare patterns**: Use working parts as templates

## Reference Information

### Standard TextMate Scopes
```
@comment           # Comments
@string            # String literals  
@number            # Numeric literals
@keyword           # Language keywords
@variable          # Variable names
@constant          # Constants
@operator          # Operators
@punctuation       # Punctuation marks
@type              # Type names
@function          # Function names
```

### Tree-sitter Query Syntax
```scheme
; Node pattern
(node_name) @scope

; String literal
"literal_text" @scope

; Nested pattern
(parent (child) @scope)

; Conditional pattern
(node_name . "keyword" . (identifier) @variable)
```

### Zed Extension Configuration
- `extension.toml`: Main extension metadata
- `languages/*/config.toml`: Language-specific settings
- `queries/*.scm`: Tree-sitter query files
- `grammars/*.wasm`: Compiled grammar binaries

## Next Steps

1. **Check Zed logs** for any error messages
2. **Test ultra-minimal query** (comment only)
3. **Compare with working extension** structure
4. **Post to Zed community** if issue persists
5. **Consider alternative approaches** (manual theme integration)

## Files to Monitor

- `~/.config/zed/extensions/amble_dsl/languages/amble_dsl/queries/highlights.scm`
- `~/.config/zed/logs/Zed.log` (or similar)
- Test files: `test_highlighting.amble`, `zed_highlight_test.amble`

## Success Criteria

- Comments appear in different color (gray/green)
- Strings appear highlighted (typically green/red)  
- Keywords appear highlighted (typically blue/purple)
- File recognized as "Amble DSL" in status bar
- Consistent highlighting across all `.amble` files

---

**Last Updated**: Current troubleshooting session
**Status**: Investigating why valid queries don't produce highlighting
**Priority**: High - core functionality missing