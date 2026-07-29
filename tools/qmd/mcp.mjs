#!/usr/bin/env node

import { spawn } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { qmdModelEnvironment } from "./config.mjs";
import { resolveQmdCommand } from "./qmd-command.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..", "..");
const qmd = resolveQmdCommand();

const child = spawn(qmd.command, [...qmd.prefixArgs, "mcp"], {
  cwd: projectRoot,
  env: qmdModelEnvironment(),
  stdio: "inherit",
});

child.on("error", (error) => {
  console.error(`Failed to start QMD MCP server: ${error.message}`);
  process.exitCode = 1;
});

child.on("exit", (code, signal) => {
  if (signal) {
    console.error(`QMD MCP server exited after signal ${signal}`);
  }
  process.exitCode = code ?? 1;
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => {
    if (!child.killed) {
      child.kill(signal);
    }
  });
}
