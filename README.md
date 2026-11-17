# PYLA

PYLA is a deterministic, AI-free code engineering platform. Version 0.1
provides a Rust workspace with a language-independent IR, pluggable language
adapters, source scanning, and a CLI.

Licensed under Apache-2.0.

> **Status: early prototype.** The adapters currently use line-based pattern
> matching, not full parsers, so function discovery and diagnostics can produce
> false positives. See [CONTRIBUTING.md](CONTRIBUTING.md) for known gaps.

```text
cargo run -- analyze ./path/to/project
cargo run -- migrate ./path/to/project --to=Python
cargo run -- migrate ./path/to/project --to=Python --output=./migration-output
cargo run -- validate ./path/to/project --language=rust
cargo run -- analyze ./path/to/project --json
cargo run -- optimize ./path/to/function.py --json
cargo run -- compare ./comparison-cases.json --json
cargo run -- worktree create /repo /tmp/pyla-worktree pyla/analysis
```

The adapters currently identify Rust, C/C++, and Python source files, build a
shared IR, and emit deterministic structural diagnostics with suggestions.
Migration produces a plan and reviewed candidates; it does not overwrite the
source tree. Analysis is intentionally separate from transformation. Future
automatic refactoring will only be added behind compiler/test validation,
concurrency checks, snapshots, and rollback boundaries.

With `--output`, migration emits explicitly generated target-language stubs in
a separate directory. It refuses to write inside the source tree or overwrite
existing generated files.

`pyla-transform` also provides rule statistics. A rule can record whether a
validated change succeeded or failed and expose its success rate. This gives
PYLA a deterministic feedback loop for ordering trusted rules without changing
their validation requirements or using an AI service.

Refactoring execution is transactional: a candidate is applied in the isolated
worktree, the configured toolchain validates it, and PYLA commits the result
only on success. Missing or failing validation automatically rolls the exact
file contents back. This prevents a failed migration or refactor from becoming
an accepted change; no system can honestly guarantee semantic perfection, so
human review and project tests remain part of the acceptance boundary.

Validation uses native tooling when available: `cargo check` for Rust,
`python3 -m compileall` for Python, and `cmake --build .` for C/C++ build
directories. Missing tools are reported explicitly and are never treated as
successful validation.

`optimize` applies the notebook’s deterministic AST idea to Python source. It
detects loop/modulo optimization candidates and emits PyO3 Rust module source
for review. It does not use Gemini, modify the input file, or execute generated
code; generation must be followed by isolated worktree validation.

Migration fidelity is explicit. Unknown translations remain review-required,
even if they compile. Differential verification consumes JSON triples of
`[input, source_output, candidate_output]` and fails when any output differs.

Add `--json` to `analyze`, `migrate`, or `validate` for stable structured
output suitable for automation, dashboards, and security tooling. JSON is the
canonical machine-readable format; the default CLI output remains optimized
for humans.

For autonomous work, create an isolated Git worktree first. PYLA can then
analyze, generate, validate, and modify that checkout without touching the
user's original working tree. The user can inspect, merge, cherry-pick, or
discard the branch afterward:

```text
pyla worktree create <repository> <worktree-path> <branch>
pyla worktree status <repository> <worktree-path>
pyla worktree remove <repository> <worktree-path>
```

## Demo notebook

`demo/demo.ipynb` is a separate Google Colab experiment, not part of the Rust
workspace. Some of its cells call the Gemini API (with a deterministic
fallback), so it does not follow PYLA's AI-free rule. Running it needs your own
`GOOGLE_API_KEY` set as a Colab secret.

## License

PYLA is released under the [Apache License 2.0](LICENSE). See [NOTICE](NOTICE) for attribution.
