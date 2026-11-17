# Safety Rules

These rules are architectural requirements, not optional conventions:

1. Never modify the user's original working tree during autonomous work.
2. Never overwrite unrelated, untracked, or concurrently changed files.
3. Verify source hashes before applying a candidate.
4. Apply changes transactionally and preserve rollback data until validation passes.
5. Treat build systems, package managers, scripts, and tests as executable project code.
6. Never treat missing validation as successful validation.
7. Never call generated code semantically correct based only on compilation.
8. Keep uncertain migrations review-required.
9. Do not use shell interpolation for project-controlled names or commands.
10. Emit explicit errors; do not silently downgrade failures to success.
11. Keep rule self-improvement limited to measured statistics and ordering until
    a human-approved policy permits more.
12. Record transformations, validation results, and failures in structured JSON.

Security review must cover path handling, external process execution, worktree
operations, generated code, and patch rollback.
