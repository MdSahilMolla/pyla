#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const binary = process.env.PYLA_BIN || "pyla";
const timeout = Number(process.env.PYLA_MCP_TIMEOUT_MS || 120000);

function invoke(args) {
  const result = spawnSync(binary, [...args, "--json"], {
    encoding: "utf8",
    timeout,
    maxBuffer: 10 * 1024 * 1024,
    env: process.env,
  });
  if (result.error) throw new Error(`PYLA execution failed: ${result.error.message}`);
  if (result.status !== 0) {
    throw new Error(result.stderr?.trim() || `PYLA exited with status ${result.status}`);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`PYLA returned invalid JSON: ${error.message}`);
  }
}

function textResult(value) {
  return { content: [{ type: "text", text: JSON.stringify(value, null, 2) }] };
}

const server = new McpServer({
  name: "pyla",
  version: "0.1.0",
});

server.registerTool(
  "pyla_analyze",
  {
    description: "Analyze a codebase with PYLA. Read-only and deterministic.",
    inputSchema: { path: z.string().describe("Project or file path") },
  },
  async ({ path }) => textResult(invoke(["analyze", path])),
);

server.registerTool(
  "pyla_migrate",
  {
    description: "Create a migration plan or isolated output artifacts. Never overwrites the source tree.",
    inputSchema: {
      path: z.string(),
      to: z.enum(["rust", "python", "cpp"]),
      output: z.string().optional().describe("Separate output directory; omit for planning only"),
    },
  },
  async ({ path, to, output }) => {
    const args = ["migrate", path, "--to", to];
    if (output) args.push("--output", output);
    return textResult(invoke(args));
  },
);

server.registerTool(
  "pyla_validate",
  {
    description: "Validate a project with its native toolchain. Run in a trusted or sandboxed worktree.",
    inputSchema: {
      path: z.string(),
      language: z.enum(["rust", "python", "cpp"]),
    },
  },
  async ({ path, language }) => textResult(invoke(["validate", path, "--language", language])),
);

server.registerTool(
  "pyla_optimize",
  {
    description: "Find deterministic optimization candidates without modifying or executing generated code.",
    inputSchema: { path: z.string() },
  },
  async ({ path }) => textResult(invoke(["optimize", path])),
);

server.registerTool(
  "pyla_compare",
  {
    description: "Run differential verification from a JSON case file; fails on any output mismatch.",
    inputSchema: { caseFile: z.string() },
  },
  async ({ caseFile }) => textResult(invoke(["compare", caseFile])),
);

server.registerTool(
  "pyla_worktree",
  {
    description: "Create, inspect, or remove an isolated Git worktree for autonomous changes.",
    inputSchema: {
      action: z.enum(["create", "status", "remove"]),
      repository: z.string(),
      path: z.string(),
      branch: z.string().optional(),
    },
  },
  async ({ action, repository, path, branch }) => {
    const args = ["worktree", action, repository, path];
    if (action === "create") {
      if (!branch) throw new Error("branch is required when creating a worktree");
      args.push(branch);
    }
    return textResult(invoke(args));
  },
);

await server.connect(new StdioServerTransport());
