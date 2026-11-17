"use strict";

const { spawnSync } = require("node:child_process");

class PylaError extends Error {
  constructor(message, result) {
    super(message);
    this.name = "PylaError";
    this.status = result?.status ?? null;
    this.stderr = result?.stderr ?? "";
    this.stdout = result?.stdout ?? "";
  }
}

function resolveBinary() {
  return process.env.PYLA_BIN || "pyla";
}

function run(args, options = {}) {
  const result = spawnSync(resolveBinary(), [...args, "--json"], {
    cwd: options.cwd,
    encoding: "utf8",
    timeout: options.timeout ?? 120_000,
    maxBuffer: options.maxBuffer ?? 10 * 1024 * 1024,
    env: { ...process.env, ...(options.env || {}) },
  });
  if (result.error) {
    throw new PylaError(`Unable to run PYLA: ${result.error.message}`, result);
  }
  if (result.status !== 0) {
    throw new PylaError(result.stderr.trim() || `PYLA exited with status ${result.status}`, result);
  }
  try {
    return JSON.parse(result.stdout);
  } catch (error) {
    throw new PylaError(`PYLA returned invalid JSON: ${error.message}`, result);
  }
}

function analyze(path, options) {
  return run(["analyze", path], options);
}

function migrate(path, to, options = {}) {
  const args = ["migrate", path, "--to", to];
  if (options.output) args.push("--output", options.output);
  return run(args, options);
}

function validate(path, language, options) {
  return run(["validate", path, "--language", language], options);
}

function optimize(path, options) {
  return run(["optimize", path], options);
}

function compare(caseFile, options) {
  return run(["compare", caseFile], options);
}

function worktree(action, repository, path, branch, options) {
  const args = ["worktree", action, repository, path];
  if (action === "create") args.push(branch);
  return run(args, options);
}

module.exports = {
  PylaError,
  analyze,
  migrate,
  validate,
  optimize,
  compare,
  worktree,
};
