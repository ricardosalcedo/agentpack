---
layout: default
title: AgentPack — Dependency Manager for MCP Servers & AI Agents
---

# AgentPack

The missing dependency layer for the MCP ecosystem. Resolve, lock, and orchestrate MCP servers and AI agents with one manifest.

## Install

```bash
# One-liner (macOS/Linux)
curl -fsSL https://raw.githubusercontent.com/ricardosalcedo/agentpack/main/install.sh | bash

# Homebrew
brew install ricardosalcedo/tap/agentpack

# From source
cargo install --git https://github.com/ricardosalcedo/agentpack
```

## Quick Start (60 seconds)

```bash
mkdir my-ai-project && cd my-ai-project
agentpack init
agentpack add io.github.modelcontextprotocol/filesystem@^0.2.0
agentpack add io.github.modelcontextprotocol/fetch@^0.2.0
agentpack install
agentpack export --target claude-desktop
```

Done. Your MCP servers are version-locked and conflict-checked.

## Why AgentPack?

| Problem | AgentPack Solution |
|---------|-------------------|
| 502 configs using unpinned `npx` | Lock file with SHA-256 integrity hashes |
| No way to say "server A needs server B" | Dependency graph with transitive resolution |
| Two servers expose `read_file` | Tool conflict detection at install time |
| Copy-paste configs between tools | `agentpack export` to any client |
| Agent needs capabilities from other agents | Capability validation + semantic resolution |
| 12 servers started in random order | Topological startup with health monitoring |

## Documentation

- [Commands Reference](./commands)
- [Manifest Format](./manifest)
- [Profiles](./profiles)
- [Capability Resolution](./capabilities)
- [MCP Gateway](./gateway)
- [Server Catalog](./catalog)
- [CI/CD & Security](./security)
