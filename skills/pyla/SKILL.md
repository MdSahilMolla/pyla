---
name: pyla
description: Use PYLA for deterministic codebase analysis, migration, refactoring candidates, validation, differential verification, and isolated worktree execution.
---

# PYLA Agent Skill

PYLA is an AI-free, Rust-first engineering engine. Use it as a deterministic
verification and transformation tool, not as an authority that can prove
arbitrary semantic equivalence.

## Required workflow

1. Analyze the target path with `pyla_analyze`.
2. Create a dedicated worktree with `pyla_worktree(action="create")` before
   autonomous edits.
3. Generate a migration plan or candidate with `pyla_migrate` or
   `pyla_optimize`.
4. Apply changes only in the worktree.
5. Run `pyla_validate` with the target language toolchain.
6. Run `pyla_compare` when source and candidate behavior can be exercised.
7. Keep changes only when validation and relevant differential cases pass.
8. Report JSON results, uncertainty, failures, and the worktree branch so the
   user can inspect, merge, cherry-pick, or discard it.

## Safety rules

- Never modify the user's original checkout.
- Never treat compilation as semantic proof.
- Unknown migration fidelity requires review and differential tests.
- Missing validation is failure, not success.
- Build systems and tests execute project code; sandbox untrusted projects.
- Do not overwrite an existing migration output directory or unrelated files.
- Do not remove a worktree unless the user or task explicitly authorizes it.

## Tool selection

| Goal | Tool |
|---|---|
| Find diagnostics | `pyla_analyze` |
| Plan or emit migration artifacts | `pyla_migrate` |
| Find deterministic optimization candidates | `pyla_optimize` |
| Compile/check a candidate | `pyla_validate` |
| Compare source/candidate outputs | `pyla_compare` |
| Isolate autonomous work | `pyla_worktree` |

Prefer JSON tool results and preserve them in the task report. Never claim a
change is flawless; state exactly which checks passed and which behavior is
still unverified.
