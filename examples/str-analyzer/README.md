# STR Analyzer — AgentPack Example

A multi-agent system for short-term rental property analysis, demonstrating AgentPack's dependency management.

## System Architecture

```
STR Analyzer (composite)
├── MCP: filesystem v2.0.1     ← read/write local files
├── MCP: fetch v2.0.0          ← fetch web content
├── Agent: research-agent v1.0.0
│     └── needs: fetch MCP
│     └── provides: [web-research, data-extraction]
└── Agent: pricing-agent v1.0.0
      └── needs: filesystem MCP
      └── needs: research-agent
      └── provides: [property-valuation, market-comparison]
```

## Commands

```bash
# Resolve all dependencies
agentpack install

# Show the full graph
agentpack graph

# Validate agent capabilities match requirements
agentpack validate

# Check for security issues
agentpack audit

# Export for Claude Desktop
agentpack export --target claude-desktop

# Start all services
agentpack run
```

## What this demonstrates

1. **Mixed dependencies** — MCP servers + agents in one manifest
2. **Transitive deps** — pricing-agent depends on research-agent, which depends on fetch MCP
3. **Capability validation** — research-agent must provide `web-research` and `data-extraction`
4. **Topological startup** — MCPs start first, then agents in dependency order
5. **Config export** — generates Claude Desktop config from the resolved graph
