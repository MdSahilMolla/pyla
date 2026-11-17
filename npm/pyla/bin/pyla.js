#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const args = process.argv.slice(2);
const binary = process.env.PYLA_BIN;
if (!binary) {
  console.error("PYLA_BIN is not set. Install the native PYLA binary or set PYLA_BIN to its path.");
  process.exit(1);
}
const result = spawnSync(binary, args, { stdio: "inherit" });
if (result.error) {
  console.error(`Unable to run PYLA: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 1);
