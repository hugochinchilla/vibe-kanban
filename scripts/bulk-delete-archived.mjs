#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    "delete-branches": { type: "boolean", default: false },
    "delete-remote": { type: "boolean", default: false },
    host: { type: "string", default: "127.0.0.1" },
    port: { type: "string" },
    help: { type: "boolean", short: "h", default: false },
  },
});

if (values.help) {
  console.log(`Usage: pnpm bulk-delete-archived [options]

Deletes every archived workspace via the running vibe-kanban server.
Reads the port from the standard port file at ${join(tmpdir(), "vibe-kanban", "vibe-kanban.port")}.

Options:
  --delete-branches   Also delete the git branch for each workspace
  --delete-remote     Also remove the matching workspace on the remote (cloud) server
  --host <host>       Server host (default: 127.0.0.1)
  --port <port>       Override port (default: read from port file)
  -h, --help          Show this help
`);
  process.exit(0);
}

async function resolvePort() {
  if (values.port) return Number(values.port);
  const portFile = join(tmpdir(), "vibe-kanban", "vibe-kanban.port");
  const raw = await readFile(portFile, "utf8");
  const parsed = JSON.parse(raw);
  if (typeof parsed.main_port !== "number") {
    throw new Error(`Port file ${portFile} is missing main_port`);
  }
  return parsed.main_port;
}

const port = await resolvePort().catch((err) => {
  console.error(
    `Could not resolve server port: ${err.message}\nIs the vibe-kanban server running? Pass --port to override.`,
  );
  process.exit(1);
});

const params = new URLSearchParams();
if (values["delete-branches"]) params.set("delete_branches", "true");
if (values["delete-remote"]) params.set("delete_remote", "true");
const qs = params.toString();
const url = `http://${values.host}:${port}/api/workspaces/bulk-delete-archived${qs ? `?${qs}` : ""}`;

const response = await fetch(url, { method: "POST" });
const body = await response.json().catch(() => null);

if (!response.ok || !body?.success) {
  console.error(`Request failed: HTTP ${response.status}`);
  if (body?.message) console.error(body.message);
  process.exit(1);
}

const { deleted, skipped } = body.data;
console.log(`Deleted ${deleted} archived workspace(s).`);
if (skipped.length > 0) {
  console.log(`Skipped ${skipped.length}:`);
  for (const item of skipped) {
    console.log(`  - ${item.workspace_id}: ${item.reason}`);
  }
}
