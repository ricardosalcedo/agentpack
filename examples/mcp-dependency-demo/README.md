# MCP Dependency Demo

Three real MCP servers with **actual inter-server dependencies**, managed by AgentPack.

## Architecture

```
travel-advisor (depends on geocoding + weather)
├── geocoding (leaf — no dependencies)
└── weather (depends on geocoding)
```

- **geocoding-mcp** — Converts city names to lat/lng coordinates
- **weather-mcp** — Returns weather data for coordinates *(needs geocoding to resolve city names)*
- **travel-advisor-mcp** — Recommends destinations and plans trips *(needs geocoding + weather)*

## Run it

```bash
# Install server dependencies
cd servers/geocoding && npm install && cd ..
cd servers/weather && npm install && cd ..
cd servers/travel-advisor && npm install && cd ../..

# Resolve the AgentPack dependency graph
agentpack install

# See the dependency tree
agentpack graph

# Start all servers in topological order
agentpack run

# Export for Claude Desktop
agentpack export --target claude-desktop

# Test servers directly
npm install
node test-client.js
```

## What this demonstrates

1. **Dependency declaration** — weather's `agentpack.json` declares it needs geocoding
2. **Transitive deps** — travel-advisor needs weather, which needs geocoding
3. **Topological startup** — `agentpack run` starts geocoding first, then weather, then travel-advisor
4. **Real MCP protocol** — all servers use the official `@modelcontextprotocol/sdk` and respond to `tools/list` and `tools/call`
5. **Export** — generates working Claude Desktop config from the resolved graph

## Servers

### geocoding-mcp
| Tool | Description |
|------|-------------|
| `geocode` | Convert address to lat/lng |
| `reverse_geocode` | Convert lat/lng to place name |

### weather-mcp
| Tool | Description |
|------|-------------|
| `get_weather` | Current weather for coordinates |
| `get_forecast` | 3-day forecast for coordinates |

### travel-advisor-mcp
| Tool | Description |
|------|-------------|
| `recommend_destination` | Suggest a city based on preferences |
| `plan_trip` | Create an itinerary for a destination |
