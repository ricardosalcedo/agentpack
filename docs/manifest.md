---
layout: default
title: Manifest Format
---

# Manifest Format (`agentpack.json`)

## Minimal

```json
{
  "name": "io.github.you/my-project",
  "version": "1.0.0",
  "type": "composite",
  "dependencies": {
    "io.github.modelcontextprotocol/filesystem": "^0.2.0"
  }
}
```

## Full

```json
{
  "name": "io.github.you/my-project",
  "version": "1.0.0",
  "description": "My AI system",
  "type": "composite",
  "dependencies": {
    "io.github.modelcontextprotocol/filesystem": "^0.2.0",
    "io.github.modelcontextprotocol/fetch": "^0.2.0"
  },
  "agents": {
    "io.github.you/research-agent": {
      "version": "^1.0.0",
      "capabilities": ["web-research"]
    }
  },
  "requires": [
    {"capability": "geocoding"},
    {"capability": "web-search"}
  ],
  "profiles": {
    "minimal": {
      "description": "Just filesystem",
      "dependencies": ["io.github.modelcontextprotocol/filesystem"],
      "agents": []
    },
    "full": {
      "description": "Everything",
      "dependencies": ["io.github.modelcontextprotocol/filesystem", "io.github.modelcontextprotocol/fetch"],
      "agents": ["io.github.you/research-agent"]
    }
  }
}
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | ✓ | Namespaced identifier (reverse-domain) |
| `version` | ✓ | Semver version |
| `type` | | `composite`, `mcp-server`, or `agent` |
| `description` | | Human-readable description |
| `transport` | | How to run this server (for mcp-server/agent types) |
| `dependencies` | | MCP server dependencies (name → semver constraint) |
| `agents` | | Agent dependencies (name → version or full spec) |
| `requires` | | Semantic capability requirements |
| `profiles` | | Named subsets of dependencies |
| `provides` | | What capabilities this package offers |
| `tools` | | Tool definitions (for catalog/conflict detection) |

## Version Constraints

```
"^1.2.0"           — >=1.2.0, <2.0.0
"~1.2.0"           — >=1.2.0, <1.3.0
">=1.0.0, <2.0.0"  — explicit range
"*"                 — any version (not recommended)
```

## Transport Types

```json
{"type": "stdio", "command": "npx", "args": ["-y", "@mcp/server@1.0.0"]}
{"type": "stdio", "command": "python3", "args": ["agent.py"]}
{"type": "docker", "args": ["my-mcp-image:latest"]}
{"type": "streamable-http", "url": "https://mcp.stripe.com/v2"}
```

## Agent Dependencies

Simple (just version):
```json
"agents": { "io.github.x/agent": "^1.0.0" }
```

Full (with capability requirements):
```json
"agents": {
  "io.github.x/agent": {
    "version": "^1.0.0",
    "capabilities": ["web-research", "summarization"]
  }
}
```
