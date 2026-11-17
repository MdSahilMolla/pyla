# `@pyla/mcp`

MCP server for AI agents using PYLA. The native Rust `pyla` binary remains the
source of truth; this package exposes safe JSON tools over MCP stdio.

```bash
npm install -g @pyla/mcp
export PYLA_BIN=/path/to/pyla
pyla-mcp
```

Configure an MCP client with:

```json
{
  "mcpServers": {
    "pyla": {
      "command": "pyla-mcp",
      "env": {
        "PYLA_BIN": "/path/to/pyla"
      }
    }
  }
}
```

Tools: `pyla_analyze`, `pyla_migrate`, `pyla_validate`, `pyla_optimize`,
`pyla_compare`, and `pyla_worktree`.

Agents should create an isolated worktree before any autonomous modification.
Validation executes project-controlled build commands, so untrusted projects
must run in a sandbox or explicitly trusted environment.
