#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  COLLECTION,
  QMD_MCP_HEALTH_URL,
  QMD_MCP_PORT,
  QMD_MCP_URL,
  qmdModelEnvironment,
} from "./config.mjs";
import { resolveQmdCommand } from "./qmd-command.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..", "..");
const qmd = resolveQmdCommand();
const action = process.argv[2] ?? "start";
const mcpHeaders = {
  accept: "application/json, text/event-stream",
  "content-type": "application/json",
};

async function readHealth() {
  try {
    const response = await fetch(QMD_MCP_HEALTH_URL, {
      signal: AbortSignal.timeout(2_000),
    });
    if (!response.ok) {
      return null;
    }
    return await response.json();
  } catch {
    return null;
  }
}

async function postMcp(payload, sessionId) {
  const response = await fetch(QMD_MCP_URL, {
    method: "POST",
    headers: {
      ...mcpHeaders,
      ...(sessionId ? { "mcp-session-id": sessionId } : {}),
    },
    body: JSON.stringify(payload),
    signal: AbortSignal.timeout(10_000),
  });

  if (!response.ok) {
    throw new Error(`QMD MCP request failed with HTTP ${response.status}`);
  }
  return response;
}

async function readIndexStatus() {
  const initializeResponse = await postMcp({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "nextengine-qmd-launcher", version: "1.0" },
    },
  });
  const sessionId = initializeResponse.headers.get("mcp-session-id");
  if (!sessionId) {
    throw new Error("QMD MCP initialize response did not include a session ID");
  }
  await initializeResponse.json();

  await postMcp(
    {
      jsonrpc: "2.0",
      method: "notifications/initialized",
      params: {},
    },
    sessionId,
  );
  const statusResponse = await postMcp(
    {
      jsonrpc: "2.0",
      id: 2,
      method: "tools/call",
      params: { name: "status", arguments: {} },
    },
    sessionId,
  );
  const status = await statusResponse.json();
  return status?.result?.content?.[0]?.text ?? "";
}

async function requireProjectIndex() {
  const status = await readIndexStatus();
  const docsPath = join(projectRoot, "docs");
  if (!status.includes(`- ${COLLECTION}:`) || !status.includes(docsPath)) {
    throw new Error(
      `The daemon at ${QMD_MCP_URL} is not serving ${COLLECTION} from ${docsPath}`,
    );
  }
  return status;
}

async function waitForHealth(timeoutMs = 15_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const health = await readHealth();
    if (health) {
      return health;
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 250));
  }
  return null;
}

function runQmd(args) {
  const result = spawnSync(qmd.command, [...qmd.prefixArgs, ...args], {
    cwd: projectRoot,
    env: qmdModelEnvironment(),
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

if (action === "status") {
  const health = await readHealth();
  if (!health) {
    console.error(`QMD MCP daemon is not reachable at ${QMD_MCP_HEALTH_URL}`);
    process.exit(1);
  }
  const indexStatus = await requireProjectIndex();
  console.log(
    JSON.stringify({ url: QMD_MCP_URL, ...health, indexStatus }, null, 2),
  );
} else if (action === "stop") {
  runQmd(["mcp", "stop"]);
} else if (action === "start") {
  const existingHealth = await readHealth();
  if (existingHealth) {
    await requireProjectIndex();
    console.log(`QMD MCP daemon is already running at ${QMD_MCP_URL}`);
  } else {
    runQmd([
      "mcp",
      "--http",
      "--port",
      String(QMD_MCP_PORT),
      "--daemon",
    ]);
    const health = await waitForHealth();
    if (!health) {
      throw new Error(
        `QMD MCP daemon did not become healthy at ${QMD_MCP_HEALTH_URL}`,
      );
    }
    await requireProjectIndex();
    console.log(`QMD MCP daemon is ready at ${QMD_MCP_URL}`);
  }
} else {
  console.error("Usage: node tools/qmd/daemon.mjs [start|status|stop]");
  process.exit(2);
}
