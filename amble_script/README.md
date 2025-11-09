# amble_script

`amble_script` is the domain-specific language, parser, and compiler that powers Amble worlds. It turns human-friendly `.amble` sources into the TOML data consumed by `amble_engine`, and ships linting plus reverse-engineering helpers for round-tripping content.

## Highlights

- Pest grammar designed to read like natural narrative prose while staying precise.
- Compiler emits structured TOML for rooms, items, triggers, spinners, NPCs, and goals.
- Linter catches unresolved references, type mismatches, and missing assets early.
- CLI supports compiling single files, entire directories, or lint-only passes.

## CLI Usage

```bash
# Compile a single DSL file to TOML
cargo run -p amble_script -- compile path/to/file.amble --out path/to/out.toml

# Compile a directory of DSL files into category TOMLs expected by the engine
cargo run -p amble_script -- compile-dir amble_script/data/Amble --out-dir amble_engine/data

# Lint files (optionally deny missing references)
cargo run -p amble_script -- lint path/to/file.or.dir --deny-missing
```

Generated TOML includes headers noting the source path and hash so it is easy to trace provenance.

## Documentation

Read the Creator Handbook and reference materials in `docs/`:

- `amble_script/docs/dsl_creator_handbook.md`
- `docs/README.md` (project overview)

## License

MIT License – see the repository root `LICENSE`.
