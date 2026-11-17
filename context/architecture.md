# PYLA Architecture

PYLA is a Rust-first, deterministic code-engineering platform. Its purpose is
to analyze, suggest, migrate, refactor, validate, and safely apply changes to
software projects. C++ is a primary target language and integration language;
the core remains language-independent.

## Pipeline

```text
source files
  -> language adapter
  -> common IR
  -> diagnostics and rules
  -> migration/refactoring candidates
  -> isolated worktree
  -> transactional patch
  -> compiler/tests/static validation
  -> commit or rollback
```

Analysis is separate from execution. A diagnostic or candidate never implies
that a change is authorized.

## Workspace crates

| Crate | Responsibility |
|---|---|
| `pyla-ir` | Common program, function, type, node, and source-location model |
| `pyla-frontend` | `LanguageAdapter`, registry, Rust/C++/Python frontends, diagnostics |
| `pyla-analysis` | Deterministic recursive scanning and analysis reports |
| `pyla-transform` | Candidates, migration plans, patches, transactions, worktrees, rule stats |
| `pyla-validate` | Native tool detection and project validation |
| `pyla-core` | Public orchestration engine |
| `pyla` | CLI |

## Data format

JSON is the canonical machine-readable format. Rust types intended for
external output derive `Serialize` and `Deserialize`. Human-readable CLI
output remains available when `--json` is omitted.
