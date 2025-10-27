# Amble `xtask` Guide

`cargo xtask` provides repeatable automation for building, packaging, and maintaining the workspace. Use it instead of memorising long cargo command lines or ad‑hoc shell scripts.

Invoke any command with `cargo xtask <subcommand> …`; pass `--help` at any level for usage details.

---

## Quick Reference

```bash
# Build the engine in release mode (default profile) with DEV_MODE disabled
cargo xtask build-engine

# Build the engine with developer commands enabled for the current target triple
cargo xtask build-engine --dev-mode enabled

# Package engine + data into target/dist/*.zip
cargo xtask package engine

# Package engine, amble_script CLI, compiled data, and DSL sources
cargo xtask package full --format zip

# Recompile DSL content and lint it against the generated TOML
cargo xtask content refresh --deny-missing
```

---

## `build-engine`

Compiles the `amble_engine` crate with optional feature flags and target/profile overrides.

Options:

| Flag | Default | Description |
| --- | --- | --- |
| `--dev-mode {enabled,disabled}` | `disabled` | Adds the `dev-mode` feature so the binary ships with developer commands (`:teleport`, `:spawn`, etc.). |
| `--profile {debug,release}` | `release` | Chooses the cargo profile; `debug` builds faster, `release` is optimised. |
| `--target <triple>` | host target | Cross-compile for another platform (e.g. `--target x86_64-pc-windows-msvc`). Requires the target toolchain to be installed via `rustup`. |

The command is a thin wrapper around `cargo build -p amble_engine` with the appropriate flags; build artefacts land in the standard `target/<profile>/` directories.

---

## `package`

Bundles ready-to-share builds into an archive or staged directory under `target/dist/`.

Subcommands:

### `package engine`
Creates a distribution containing:
- `bin/amble_engine` (or `.exe` on Windows) built with the requested profile/target.
- `amble_engine/data/` TOML files (current compiled world data).

### `package full`
Includes everything from `package engine` plus:
- `bin/amble_script` CLI (same profile/target).
- `amble_script/data/Amble/` DSL sources so recipients can recompile content.

Common options (shared by both engine/full):

| Flag | Default | Description |
| --- | --- | --- |
| `--target <triple>` | host triple | Cross-compile; see `build-engine`. |
| `--profile {debug,release}` | `release` | Build profile for both binaries. |
| `--dev-mode {enabled,disabled}` | `disabled` | Embed DEV commands in packaged binaries. |
| `--dist-dir <dir>` | `target/dist/` | Output directory for staged packages and archives. |
| `--format {zip,directory}` | `zip` | `zip` creates `<name>.zip`; `directory` leaves an unpacked tree. |
| `--name <string>` | auto-generated | Overrides the generated package name (otherwise `amble-engine-<ver>-<triple>` or `amble-suite-<engine_ver>-<script_ver>-<triple>`). |

The command ensures binaries exist before packaging, so it will trigger fresh builds if necessary.

---

## `content refresh`

Regenerates TOML data from `.amble` sources and lints the results. This is the fastest way to ensure content changes compile cleanly before committing.

Options:

| Flag | Default | Description |
| --- | --- | --- |
| `--source <dir>` | `amble_script/data/Amble` | Root of `.amble` files to compile. |
| `--out-dir <dir>` | `amble_engine/data` | Destination for compiled TOML. The command creates the directory if missing. |
| `--deny-missing` | off | When set, `amble_script lint` fails if referenced symbols are missing (excellent for CI). |

The task runs two steps in sequence:
1. `cargo run -p amble_script --bin amble_script -- compile-dir …`
2. `cargo run -p amble_script --bin amble_script -- lint …`

Both commands execute in the workspace root, so relative paths match those in the repository.

---

## Implementation Notes

- `Workspace::detect()` uses `cargo metadata` to locate the workspace root, target directory, package versions, and host triple.
- Packaging uses `walkdir` + `zip` to mirror directory structures and mark binaries executable inside archives.
- All subprocesses emit descriptive labels; failures bubble up with contextual error messages to simplify debugging.

When automating additional workflows (e.g., documentation builds or release signing), prefer extending the existing CLI rather than scattering scripts—`xtask` keeps project automation discoverable and version-controlled.
