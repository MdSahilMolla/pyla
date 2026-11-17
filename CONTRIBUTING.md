# Contributing to PYLA

Thanks for your interest in PYLA. The project is an early prototype, so small,
focused contributions are the most useful.

## Getting started

```text
cargo build
cargo test --workspace
cargo clippy --workspace
cargo fmt --all
```

Please make sure tests, clippy, and `cargo fmt --check` pass before opening a
pull request.

## Project layout

| Crate | Purpose |
| --- | --- |
| `pyla-ir` | Shared, language-independent program model |
| `pyla-frontend` | `LanguageAdapter` trait, registry, and the Rust/C++/Python adapters |
| `pyla-analysis` | Walks a project and collects diagnostics |
| `pyla-transform` | Patches, migration plans, stub rendering, rule statistics |
| `pyla-validate` | Runs native tooling (`cargo`, `python3`, `cmake`) |
| `pyla-core` | `Engine` that ties the crates together |
| `pyla-cli` | The `pyla` binary |

## Ground rules

- **Deterministic only.** PYLA must not call AI or network services. The same
  input must always produce the same output.
- **Never modify the source tree.** Analysis and migration read the project;
  generated files go to a separate output directory.
- **No silent success.** A missing tool or skipped step is reported, never
  treated as a pass.
- **Keep dependencies minimal.** The workspace currently has no external
  crates. Discuss new dependencies in an issue first.
- **Add a test** for every bug fix and new rule.

## Good first issues

The current adapters match text line by line instead of parsing. Known gaps:

- Brace and TODO checks also match text inside strings and comments.
- The C++ adapter treats any line containing `(` as a function.
- The Rust adapter picks up `fn ` inside comments.
- Directory walking does not skip `target/`, `.git`, or virtualenvs and follows
  symlinks.
- A single non-UTF-8 source file aborts the whole analysis.
- CLI flags must appear in a fixed order, and unknown values silently fall back
  to Rust.
- Generated stubs can contain duplicate names or target-language keywords.
- Python validation discards `compileall` errors written to stdout.

## Pull requests

1. Open an issue first for anything larger than a bug fix.
2. Keep each pull request to one change.
3. Describe what changed and how you tested it.

By contributing, you agree that your contributions are licensed under the
Apache License 2.0 in `LICENSE` (see section 5 of the license).
