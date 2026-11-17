# CLI Reference

Run commands from the workspace root. PYLA uses a typed Clap CLI, so
`--help` and `--version` are available globally:

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
cargo run -p pyla -- --help
cargo run -p pyla -- --version
```

`--json` is intended for automation, dashboards, CI, and security tooling.
Commands use non-zero exit status for operational errors and failed
differential validation.

## npm wrapper

The `npm/pyla` package exposes the native CLI to Node.js. Install/build the
Rust binary first, put it on `PATH` or set `PYLA_BIN`, then use:

```js
const { analyze } = require("@pyla/cli");
const report = analyze(".");
```

The wrapper always requests JSON and throws on process, timeout, exit-status,
or JSON parsing failures.

## MCP server and agent skill

The `npm/pyla-mcp` package exposes PYLA as MCP tools for compatible agents:

```bash
npm install -g ./npm/pyla-mcp
pyla-mcp
```

The agent guidance is in `skills/pyla/SKILL.md`. It requires an isolated
worktree before autonomous changes and treats compilation as distinct from
semantic verification.

To install the native Rust CLI directly:

```bash
cargo install --path crates/pyla-cli
pyla --help
```

Alternatively, install the npm wrapper globally after building the native
binary:

```bash
npm install -g ./npm/pyla
export PYLA_BIN="$PWD/target/debug/pyla"
pyla --help
```
