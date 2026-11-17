# PYLA Roadmap

## Implemented

- Rust workspace and shared IR
- Rust, C++, and Python adapter registry
- Deterministic recursive analysis
- Diagnostics and suggestions
- JSON reports
- Migration plans and isolated output stubs
- Python loop/modulo optimization candidates with PyO3 output
- Native Rust/Python/C++ validation hooks
- Hash-checked patches and transactional rollback
- Git worktree create/status/remove
- Rule success/failure statistics
- Differential output verification

## Next milestones

1. Replace lightweight frontends with Rust Analyzer/Clang/Python AST-backed
   semantic adapters.
2. Add a first truly semantics-preserving transformation with fixtures,
   differential tests, and rollback tests.
3. Add sandboxed validators with timeouts, memory limits, network policy, and
   explicit trust configuration.
4. Add transformation history and rule statistics persistence in JSON.
5. Add dependency/API mapping for Python, Rust, and C++ migrations.
6. Add LSP and library APIs.
7. Add C++ bindings through a stable C ABI.

“Self-improving” means deterministic measurement, regression testing, and
controlled rule promotion. It does not mean silently changing trusted safety
criteria.
