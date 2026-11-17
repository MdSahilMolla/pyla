# CLI Reference

Run commands from the workspace root:

```text
cargo run -p pyla -- analyze <path>
cargo run -p pyla -- analyze <path> --json
cargo run -p pyla -- migrate <path> --to=<rust|python|cpp>
cargo run -p pyla -- migrate <path> --to=<language> --output=<directory>
cargo run -p pyla -- optimize <python-file> --json
cargo run -p pyla -- validate <path> --language=<rust|python|cpp> --json
cargo run -p pyla -- compare <case-file.json> --json
cargo run -p pyla -- worktree create <repository> <path> <branch>
cargo run -p pyla -- worktree status <repository> <path>
cargo run -p pyla -- worktree remove <repository> <path>
```

`--json` is intended for automation, dashboards, CI, and security tooling.
Commands use non-zero exit status for operational errors and failed
differential validation.
