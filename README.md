# AgentPack

> The dependency manager for MCP servers and AI agents.

AgentPack resolves, installs, and orchestrates dependencies between MCP servers and AI agents — like npm, but for the agentic AI stack.

## The Problem

You're building an AI agent that needs 5 MCP servers and 2 sub-agents. Today you:
- Hand-wire each server in a JSON config file
- Manually track which versions work together
- Get no warning when two servers expose conflicting tool names
- Have no way to declare "my agent needs *these* other agents"
- Copy-paste configs between Claude Desktop, VS Code, and your custom client

AgentPack fixes all of this with one manifest, one lock file, and one command.

## Install

```bash
# macOS (Homebrew)
brew install ricardosalcedo/tap/agentpack

# From source
cargo install --git https://github.com/ricardosalcedo/agentpack

# Or download a binary from releases
curl -fsSL https://github.com/ricardosalcedo/agentpack/releases/latest/download/agentpack-darwin-arm64 -o /usr/local/bin/agentpack
chmod +x /usr/local/bin/agentpack
```

## Quick Start

```bash
# Initialize a project
agentpack init

# Add MCP server dependencies
agentpack add io.github.modelcontextprotocol/filesystem@^2.0.0
agentpack add io.github.modelcontextprotocol/fetch@^2.0.0

# Add agent dependencies
agentpack add --agent io.github.you/research-agent@^1.0.0

# Resolve everything, lock versions, check for conflicts
agentpack install

# See the full dependency graph
agentpack graph
```

Output:
```
Dependency graph:

  MCP Servers:
    ⚙ io.github.modelcontextprotocol/filesystem @ 2.0.1
      └─ [stdio] npx -y @modelcontextprotocol/server-filesystem@2.0.1
    ⚙ io.github.modelcontextprotocol/fetch @ 2.0.0
      └─ [stdio] npx -y @modelcontextprotocol/server-fetch@2.0.0

  Agents:
    🤖 io.github.you/research-agent @ 1.0.0
      └─ [stdio] python3 agents/research.py
      └─ needs mcp: io.github.modelcontextprotocol/fetch @ ^2.0.0
      provides: [web-research, data-extraction]
```

## Commands

| Command | What it does |
|---------|-------------|
| `agentpack init` | Create `agentpack.json` manifest |
| `agentpack add <pkg>@<ver>` | Add an MCP server dependency |
| `agentpack add --agent <pkg>` | Add an agent dependency |
| `agentpack fetch <pkg> --source <url>` | Pull manifest from remote (github:, https://) |
| `agentpack install` | Resolve graph → detect conflicts → write lock file |
| `agentpack graph` | Visualize the dependency tree |
| `agentpack validate` | Check agent capability contracts |
| `agentpack audit` | Security scan (unpinned versions, integrity, conflicts) |
| `agentpack export --target <t>` | Generate config for `claude-desktop` or `vscode` |
| `agentpack run` | Start all servers/agents in topological order |

## Manifest Format (`agentpack.json`)

```json
{
  "name": "io.github.you/my-system",
  "version": "1.0.0",
  "type": "composite",
  "dependencies": {
    "io.github.modelcontextprotocol/filesystem": "^2.0.0",
    "io.github.stripe/payments": "~3.0.0"
  },
  "agents": {
    "io.github.you/research-agent": {
      "version": "^1.0.0",
      "capabilities": ["web-research"]
    }
  }
}
```

Agents declare what they provide:

```json
{
  "name": "io.github.you/research-agent",
  "type": "agent",
  "provides": {
    "capabilities": ["web-research", "data-extraction"],
    "protocol": "a2a"
  },
  "dependencies": {
    "io.github.modelcontextprotocol/fetch": "^2.0.0"
  }
}
```

## Key Features

- **Unified graph** — MCP servers + agents in one manifest, resolved together
- **Semver resolution** — `^1.2.0`, `~2.0.0`, `>=3.0.0, <4.0.0`
- **Lock file** — deterministic installs with SHA-256 integrity hashes
- **Tool conflict detection** — warns when two servers expose the same tool name
- **Capability validation** — ensures agents provide what consumers require
- **Tamper detection** — `agentpack audit` catches modified packages
- **Config export** — generates Claude Desktop and VS Code configs from the lock file
- **Topological startup** — `agentpack run` starts deps before dependents
- **Credential management** — scoped env var injection per server/agent

## How It's Different

| | MCP Registry | GitHub MCP Registry | Smithery | **AgentPack** |
|---|---|---|---|---|
| Discover servers | ✅ | ✅ | ✅ | ✅ |
| Inter-server dependencies | ❌ | ❌ | ❌ | ✅ |
| Agent dependencies | ❌ | ❌ | ❌ | ✅ |
| Version resolution + lock | ❌ | ❌ | ❌ | ✅ |
| Tool conflict detection | ❌ | ❌ | ❌ | ✅ |
| Capability validation | ❌ | ❌ | ❌ | ✅ |
| Runtime orchestration | ❌ | ❌ | Partial | ✅ |
| Export to existing clients | ❌ | ❌ | ❌ | ✅ |

## Project Status

**v0.1 — Prototype.** Core resolution, conflict detection, and export work. The registry is local-first (manifests in `packages/` or fetched from URLs). Hosted registry coming in v0.2.

## Contributing

PRs welcome. The codebase is ~600 lines of Rust with no async runtime. To develop:

```bash
git clone https://github.com/ricardosalcedo/agentpack
cd agentpack
cargo test -- --test-threads=1
cargo run -- --help
```

## License

MIT
