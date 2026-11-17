# Execution Model

## Safe change lifecycle

1. Analyze the source tree without modifying it.
2. Create a dedicated Git worktree and branch.
3. Generate a candidate patch or migration artifact in that worktree.
4. Confirm the expected source hash before applying a patch.
5. Apply the candidate transactionally.
6. Run the target compiler, tests, and static checks.
7. Commit the candidate only when required validation passes.
8. Restore the exact previous bytes on validation failure.
9. Report the result as JSON and leave merge/discard decisions to the user.

The original checkout must remain untouched. Users may inspect, merge,
cherry-pick, or discard the PYLA branch.

## Current validation

- Rust: `cargo check --quiet`
- Python: `python3 -m compileall -q .`
- C/C++: `cmake --build .`

Tool execution is project-code execution. It must eventually run in a sandbox
or an explicitly trusted, least-privileged worktree with resource limits.

## Semantic fidelity

Compilation is not proof of behavior preservation. Migration plans expose:

- `Unknown`: review and differential testing required
- `PatternVerified`: only for a narrowly recognized deterministic pattern

The differential verifier compares JSON triples of input, source output, and
candidate output and fails on any mismatch.
