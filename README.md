# PYLA

**Deterministic code engineering for analysis, migration, refactoring, and
validation.**

PYLA is a Rust-first, AI-free platform for working with real codebases. It
builds a shared intermediate representation across languages, produces
reviewable engineering candidates, validates changes with native toolchains,
and operates in isolated Git worktrees so the user's checkout stays safe.

> **Status: early prototype (`0.1.0`).** The current frontends use lightweight
> structural parsing. Migration output is intentionally conservative: generated
> stubs can compile without proving semantic equivalence. Review and
> differential verification are required for uncertain changes.

## What it can do

- Analyze Rust, C/C++, and Python source files.
- Report deterministic diagnostics and suggestions.
- Generate migration plans and Rust, Python, or C++ stubs.
- Detect deterministic Python optimization candidates and emit PyO3 Rust code.
- Validate Rust with Cargo, Python with `compileall`, and C/C++ with CMake.
- Compare source and candidate outputs through JSON differential cases.
- Apply hash-checked patches transactionally and roll back failed validation.
- Create isolated Git worktrees for autonomous changes.
- Emit stable JSON for CI, dashboards, scripts, MCP, and agent tooling.
- Expose a Node.js API and MCP server without duplicating the Rust engine.

## Quick start

Requirements: Rust stable and the relevant native toolchain for the project
being validated.

```bash
git clone <repository-url>
cd pyla
cargo test --workspace
cargo run -p pyla -- --help
```

Analyze a project:

```bash
cargo run -p pyla -- analyze .
cargo run -p pyla -- --json analyze .
```

Validate a project:

```bash
cargo run -p pyla -- validate . --language rust
```

## Migration and optimization

Plan a migration without modifying the source:

```bash
cargo run -p pyla -- migrate ./project --to python
```

Generate artifacts in a separate directory:

```bash
cargo run -p pyla -- migrate ./project \
  --to rust \
  --output /tmp/pyla-migration \
  --json
```

Find deterministic Python optimization candidates:

```bash
cargo run -p pyla -- optimize ./project/function.py --json
```

Generated artifacts are not automatically accepted as behavior-preserving.
Use compiler validation and differential cases:

```json
[
  ["1", "false", "false"],
  ["7", "true", "true"],
  ["8", "false", "false"]
]
```

```bash
cargo run -p pyla -- compare cases.json --json
```

## Safe autonomous work

Create a dedicated branch and worktree before allowing autonomous changes:

```bash
cargo run -p pyla -- worktree create \
  /path/to/repository \
  /tmp/pyla-worktree \
  pyla/analysis

cargo run -p pyla -- worktree status /path/to/repository /tmp/pyla-worktree
cargo run -p pyla -- worktree remove /path/to/repository /tmp/pyla-worktree
```

The original checkout is never the execution target. Candidates must pass
validation before they are retained; missing validation is failure, not
success. Build systems and tests execute project-controlled code, so untrusted
projects should run in a sandbox or explicitly trusted environment.

## Architecture

```text
source
  -> language adapter
  -> common IR
  -> diagnostics and rules
  -> migration/refactoring candidate
  -> isolated worktree
  -> transactional patch
  -> compiler/tests/differential checks
  -> commit or rollback
```

| Crate | Responsibility |
|---|---|
| `pyla-ir` | Shared program, function, type, and source-location model |
| `pyla-frontend` | Language adapters, registry, parsing, diagnostics |
| `pyla-analysis` | Deterministic recursive codebase analysis |
| `pyla-transform` | Plans, candidates, patches, transactions, worktrees |
| `pyla-validate` | Native tool detection and validation |
| `pyla-core` | Public orchestration API |
| `pyla` | Typed command-line interface |

See [`context/`](context/README.md) for the developer architecture, safety
rules, adapter contract, CLI reference, and roadmap.

## Node.js and MCP integrations

The native Rust binary is the source of truth. Optional wrappers expose it to
JavaScript and compatible agents:

```bash
# Native CLI
cargo install --path crates/pyla-cli
pyla --help

# Node.js wrapper
npm install -g ./npm/pyla
export PYLA_BIN="$(command -v pyla)"
pyla analyze . --json

# MCP server
npm install -g ./npm/pyla-mcp
pyla-mcp
```

The agent workflow is documented in [`skills/pyla/SKILL.md`](skills/pyla/SKILL.md).

## Development

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The notebook in [`demo/demo.ipynb`](demo/demo.ipynb) is a separate
experimental migration demonstration. It is not part of the AI-free Rust
engine and may require external notebook dependencies.

## License

PYLA is released under the [Apache License 2.0](LICENSE).
