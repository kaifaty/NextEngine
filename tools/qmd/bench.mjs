#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { qmdModelEnvironment } from "./config.mjs";
import { resolveQmdCommand } from "./qmd-command.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..", "..");
const qmd = resolveQmdCommand();
const userArgs = process.argv.slice(2);
const benchArgs =
  userArgs.length > 0
    ? userArgs
    : ["tools/qmd/bench.json", "--json"];

const result = spawnSync(
  qmd.command,
  [...qmd.prefixArgs, "bench", ...benchArgs],
  {
    cwd: projectRoot,
    env: qmdModelEnvironment(),
    stdio: "inherit",
  },
);

if (result.error) {
  throw result.error;
}
process.exitCode = result.status ?? 1;
