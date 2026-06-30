---
layout: default
title: Commands Reference
---

# Commands Reference

## `agentpack init`

Create a new `agentpack.json` manifest in the current directory.

```bash
agentpack init
```

## `agentpack add`

Add a dependency to your manifest.

```bash
# MCP server dependency
agentpack add io.github.modelcontextprotocol/filesystem@^0.2.0

# Agent dependency
agentpack add --agent io.github.me/research-agent@^1.0.0
```

## `agentpack import`

Auto-introspect any running MCP server and generate its manifest.

```bash
agentpack import --name io.github.x/server --command npx -- -y @some/mcp-pkg
```

Connects to the server, calls `tools/list`, writes `packages/<name>/agentpack.json`.

## `agentpack install`

Resolve the dependency graph, detect conflicts, write the lock file.

```bash
agentpack install                    # all deps
agentpack install --profile minimal  # only profile subset
```

## `agentpack search`

Find MCP servers in the catalog by name, capability, or tool.

```bash
agentpack search database    # finds postgres, sqlite, upstash
agentpack search browser     # finds puppeteer, playwright
agentpack search payments    # finds stripe
```

## `agentpack graph`

Visualize the full dependency tree.

```bash
agentpack graph
```

```
MCP Servers:
  ⚙ io.github.x/geocoding @ 1.0.0
    └─ [stdio] node servers/geocoding/index.js
  ⚙ io.github.x/weather @ 1.0.0
    └─ needs mcp: geocoding @ ^1.0.0

Agents:
  🤖 io.github.me/research-agent @ 1.0.0
    └─ provides: [web-research, data-extraction]
```

## `agentpack validate`

Check that agent capability requirements match what dependencies provide.

```bash
agentpack validate
```

## `agentpack audit`

Security and configuration checks.

```bash
agentpack audit
```

Checks: unpinned versions, integrity mismatches, tool conflicts, missing credentials.

## `agentpack export`

Generate native config for your AI tool.

```bash
agentpack export --target claude-desktop   # claude_desktop_config.json
agentpack export --target vscode           # .vscode/mcp.json
agentpack export --target kiro             # .kiro/mcp.json
agentpack export --target cursor           # .cursor/mcp.json
agentpack export --target gateway          # agentpack-gateway.json
```

Supports `--profile` to export a subset.

## `agentpack run`

Start all servers/agents in topological order.

```bash
agentpack run
```

Supports stdio and Docker transports. Monitors health. Ctrl+C for graceful shutdown.

## `agentpack update`

Check for newer versions of local packages.

```bash
agentpack update
```

## `agentpack doctor`

Diagnose your environment.

```bash
agentpack doctor
```

Checks: node, npx, python3, docker, manifest, lock file, integrity.

## `agentpack fetch`

Pull a remote manifest and cache locally.

```bash
agentpack fetch io.github.x/server --source github:user/repo
agentpack fetch io.github.x/server --source https://example.com/agentpack.json
```
