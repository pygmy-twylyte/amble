# Zed Editor Setup for Amble

The `zed-amble-ext` project provides (I hope) a first-class authoring experience for Amble’s DSL inside the [Zed editor](https://zed.dev). It bundles syntax highlighting, code completion, symbol navigation, inline diagnostics, and formatting powered by a tree-sitter parser and a language server (written in Rust).

## Install the extension

1. Install Zed (macOS or Linux builds are available from [zed.dev/download](https://zed.dev/download)).
2. Clone the extension repository somewhere convenient:
   ```bash
   git clone https://github.com/pygmy-twylyte/zed-amble-ext.git
   ```
3. Install the extension into Zed (run from inside the cloned directory):
   ```bash
   cd zed-amble-ext
   zed --install-extension .
   ```
   Zed copies the extension into its extensions directory and will load it on the next launch.

   Alternatively, you can choose "Install Dev Extension" from the Zed Extensions dialog, and choose the zed-amble-ext directory (wherever you cloned it.)

4. Restart Zed if it was already running, then open the Amble workspace. `.amble` files should now pick up highlighting, completion, lint diagnostics, symbol search, and jump-to-definition support.

> **Tip:** repeat steps 2–4 (or `git pull` followed by `zed --install-extension .`) whenever the extension repo receives updates.

## Optional tree-sitter grammar

The extension depends on `tree-sitter-amble` for parsing. Zed fetches prebuilt grammars via the extension, but you can build locally for experimentation:

```bash
git clone https://github.com/pygmy-twylyte/tree-sitter-amble.git
cd tree-sitter-amble
npm install
npm run build
```

Point the extension’s grammar path at your local build if you want to test grammar changes before publishing them.

## Using the tooling

- Open any `.amble` file to get syntax-aware editing. Diagnostics will surface parser errors inline as you type.
- Use **Go to Definition** (`cmd`/`ctrl`+click or `F12`) to jump between trigger, item, room, NPC, spinner, and goal symbols.
- Hit **cmd/ctrl+P** and search for `Amble:` commands to explore formatting or linting actions exposed by the language server.
- When you lint or compile via the CLI (`amble_script lint …`, `amble_script compile …`), the extension mirrors diagnostics in the Problems panel.
- The Outline panel should populate automatically for easy navigation between definitions within a file.
- Zed tasks are also defined for common refresh / build / packaging processes.

Pair the extension with the CLI quickstart in the main [README](./README.md#quickstart) for a smooth authoring loop.
