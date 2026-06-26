#!/usr/bin/env node
/**
 * AgentPack MCP Gateway Server
 *
 * Connects to your AI tool as a single MCP server, then routes tool calls
 * to the appropriate backend servers managed by AgentPack.
 *
 * Usage:
 *   agentpack-gateway              # reads agentpack.lock from cwd
 *   agentpack-gateway --lock ./agentpack.lock
 */

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import { z } from "zod";
import { readFileSync } from "fs";
import { resolve } from "path";

// --- Config ---
const lockPath = process.argv.includes("--lock")
  ? process.argv[process.argv.indexOf("--lock") + 1]
  : resolve(process.cwd(), "agentpack.lock");

const lock = JSON.parse(readFileSync(lockPath, "utf-8"));

// --- State ---
const backends = new Map(); // name -> { client, tools }
const toolRouter = new Map(); // "namespace.toolName" -> { backend, originalName }

// --- Gateway Server ---
const gateway = new McpServer({
  name: "agentpack-gateway",
  version: "0.1.0",
});

// --- Boot: connect to all backend servers ---
async function connectBackends() {
  for (const [name, entry] of Object.entries(lock.resolved)) {
    if (!entry.transport || entry.transport.type !== "stdio") continue;
    if (!entry.transport.command) continue;

    const prefix = name.split("/").pop();

    try {
      const transport = new StdioClientTransport({
        command: entry.transport.command,
        args: entry.transport.args || [],
      });
      const client = new Client({ name: `agentpack->${name}`, version: "0.1.0" });
      await client.connect(transport);

      const { tools } = await client.listTools();
      backends.set(name, { client, tools, prefix });

      // Register each tool with namespace prefix
      for (const tool of tools) {
        const namespacedName = `${prefix}.${tool.name}`;
        toolRouter.set(namespacedName, { backend: name, originalName: tool.name });

        // Register the tool on the gateway
        gateway.tool(
          namespacedName,
          `[${prefix}] ${tool.description || tool.name}`,
          // Accept any JSON object as input (pass-through)
          { input: z.record(z.any()).optional().describe("Tool arguments") },
          async ({ input }) => {
            const result = await client.callTool({
              name: tool.name,
              arguments: input || {},
            });
            return result;
          }
        );
      }

      process.stderr.write(`[gateway] ✓ ${name} (${tools.length} tools)\n`);
    } catch (e) {
      process.stderr.write(`[gateway] ✗ ${name}: ${e.message}\n`);
    }
  }
}

// --- Also register a meta tool for discovery ---
gateway.tool(
  "agentpack.list_servers",
  "List all servers managed by AgentPack and their tools",
  {},
  async () => {
    const servers = [];
    for (const [name, { tools, prefix }] of backends) {
      servers.push({
        name,
        prefix,
        tools: tools.map((t) => `${prefix}.${t.name}`),
      });
    }
    return { content: [{ type: "text", text: JSON.stringify(servers, null, 2) }] };
  }
);

gateway.tool(
  "agentpack.server_health",
  "Check health of all managed servers",
  {},
  async () => {
    const health = [];
    for (const [name, { client }] of backends) {
      try {
        await client.ping();
        health.push({ name, status: "healthy" });
      } catch {
        health.push({ name, status: "unhealthy" });
      }
    }
    return { content: [{ type: "text", text: JSON.stringify(health, null, 2) }] };
  }
);

// --- Start ---
await connectBackends();
const transport = new StdioServerTransport();
await gateway.connect(transport);
