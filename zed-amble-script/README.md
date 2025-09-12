# Zed Amble Script Extension

This extension provides basic language support for Amble Script in the Zed editor.

## Installation

1. Install the [`tree-sitter-amble-script`](../tree-sitter-amble-script) parser package and run `npm install`.
2. In Zed, clone or copy this `zed-amble-script` directory into your extensions path.
3. Restart Zed; files with the `.amble` extension will use the parser and syntax highlighting.

## Development

- Queries for highlighting, folding, and indentation are in the `queries/` directory.
- The language definition points to the local Tree-sitter grammar.

