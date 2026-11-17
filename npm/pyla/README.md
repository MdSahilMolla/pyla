# `@pyla/cli`

Node.js wrapper for the native PYLA CLI. The Rust binary remains the source of
truth; this package provides JavaScript automation and a `pyla` executable.

Install the native binary separately, then set `PYLA_BIN` if it is not on
`PATH`:

```bash
export PYLA_BIN=/path/to/pyla
npm install @pyla/cli
```

Then run:

```bash
pyla --help
pyla analyze . --json
```

```js
const { analyze, migrate, validate } = require("@pyla/cli");

const report = analyze(".");
const plans = migrate("./service", "python", { output: "/tmp/service-python" });
const result = validate(".", "rust");
```

All API methods request JSON from the native CLI and throw `PylaError` for
non-zero exit status, missing binaries, timeouts, or malformed output.

```bash
pyla-js analyze . --json
```

The package does not execute generated code or bypass PYLA's worktree,
validation, and rollback safety boundaries.
