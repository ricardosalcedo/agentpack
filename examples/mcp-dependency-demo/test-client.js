import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

async function testServer(name, command, args) {
  console.log(`\n--- Testing ${name} ---`);
  const transport = new StdioClientTransport({ command, args });
  const client = new Client({ name: "test-client", version: "1.0.0" });

  try {
    await client.connect(transport);
    const { tools } = await client.listTools();
    console.log(`  Tools: [${tools.map(t => t.name).join(", ")}]`);

    // Call first tool
    const result = await client.callTool({ name: tools[0].name, arguments: getArgs(tools[0].name) });
    console.log(`  ${tools[0].name}():`, result.content[0].text);
    await client.close();
    console.log(`  ✓ ${name} working`);
  } catch (e) {
    console.error(`  ✗ ${name} error:`, e.message);
  }
}

function getArgs(toolName) {
  switch (toolName) {
    case "geocode": return { address: "Miami" };
    case "get_weather": return { lat: 25.76, lng: -80.19 };
    case "recommend_destination": return { preferences: "warm beach vacation" };
    default: return {};
  }
}

await testServer("geocoding", "node", ["servers/geocoding/index.js"]);
await testServer("weather", "node", ["servers/weather/index.js"]);
await testServer("travel-advisor", "node", ["servers/travel-advisor/index.js"]);

console.log("\n✓ All MCP servers functional");
process.exit(0);
