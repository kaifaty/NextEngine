#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  existsSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  COLLECTION,
  EMBEDDING_MODEL,
  GENERATE_MODEL,
  qmdModelEnvironment,
  REQUIRED_QMD_VERSION,
  RERANK_MODEL,
} from "./config.mjs";
import { resolveQmdCommand } from "./qmd-command.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..", "..");
const qmd = resolveQmdCommand();

function runQmd(args, { capture = false } = {}) {
  return execFileSync(qmd.command, [...qmd.prefixArgs, ...args], {
    cwd: projectRoot,
    encoding: capture ? "utf8" : undefined,
    env: qmdModelEnvironment(),
    stdio: capture ? ["ignore", "pipe", "pipe"] : "inherit",
  });
}

function replaceModel(config, key, value) {
  const expression = new RegExp(`^(\\s*${key}:\\s*).+$`, "m");
  if (!expression.test(config)) {
    throw new Error(`QMD config is missing models.${key}`);
  }
  return config.replace(expression, `$1"${value}"`);
}

function ensureContext(path, description) {
  runQmd(["context", "add", path, description]);
}

let versionOutput;
try {
  versionOutput = runQmd(["--version"], { capture: true }).trim();
} catch {
  throw new Error(
    `QMD is not installed. Run: npm install --global @tobilu/qmd@${REQUIRED_QMD_VERSION}`,
  );
}

if (!versionOutput.includes(REQUIRED_QMD_VERSION)) {
  console.warn(
    `Expected QMD ${REQUIRED_QMD_VERSION}, found ${versionOutput}. ` +
      "The setup will continue, but update the pinned version after validating compatibility.",
  );
}

runQmd(["init"]);

const indexConfigPath = join(projectRoot, ".qmd", "index.yml");
if (!existsSync(indexConfigPath)) {
  throw new Error(`QMD did not create ${indexConfigPath}`);
}

const originalConfig = readFileSync(indexConfigPath, "utf8");
let updatedConfig = replaceModel(originalConfig, "embed", EMBEDDING_MODEL);
updatedConfig = replaceModel(updatedConfig, "rerank", RERANK_MODEL);
updatedConfig = replaceModel(updatedConfig, "generate", GENERATE_MODEL);
if (updatedConfig !== originalConfig) {
  writeFileSync(indexConfigPath, updatedConfig, "utf8");
}

const collections = runQmd(["collection", "list"], { capture: true });
if (!new RegExp(`(^|\\s)${COLLECTION}(\\s|$)`, "m").test(collections)) {
  runQmd([
    "collection",
    "add",
    join(projectRoot, "docs"),
    "--name",
    COLLECTION,
    "--mask",
    "**/*.md",
  ]);
}

ensureContext(
  `qmd://${COLLECTION}/`,
  "NextEngine project documentation. Use search to discover candidate sources; full source documents and repository precedence rules remain authoritative.",
);
ensureContext(
  `qmd://${COLLECTION}/architecture`,
  "Normative NextEngine architecture: product contract, system architecture, subsystem specifications, glossary, traceability and architecture decisions.",
);
ensureContext(
  `qmd://${COLLECTION}/architecture/adr`,
  "Architecture Decision Records. Check status and supersedes relationships before treating a decision as authoritative.",
);
ensureContext(
  `qmd://${COLLECTION}/roadmap.md`,
  "Living product roadmap and sequencing context. It is planning context, not normative architecture.",
);
ensureContext(
  `qmd://${COLLECTION}/plans`,
  "Implementation plans. They explain intended execution but do not override Accepted specifications or ADRs.",
);
ensureContext(
  `qmd://${COLLECTION}/reviews`,
  "Historical architecture and implementation reviews. Findings may be superseded; verify against current authoritative documents.",
);
ensureContext(
  `qmd://${COLLECTION}/development`,
  "Developer-facing source layout, training capability and local workflow documentation.",
);

runQmd(["update"]);
runQmd(["embed", "-c", COLLECTION]);
runQmd(["doctor"]);
runQmd(["status"]);
