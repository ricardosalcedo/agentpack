import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const server = new McpServer({ name: "geocoding-mcp", version: "1.0.0" });

const LOCATIONS = {
  "new york": { lat: 40.7128, lng: -74.006 },
  "los angeles": { lat: 34.0522, lng: -118.2437 },
  "miami": { lat: 25.7617, lng: -80.1918 },
  "london": { lat: 51.5074, lng: -0.1278 },
  "tokyo": { lat: 35.6762, lng: 139.6503 },
  "paris": { lat: 48.8566, lng: 2.3522 },
};

server.tool("geocode", "Convert an address/city to lat/lng coordinates",
  { address: z.string().describe("City or address to geocode") },
  async ({ address }) => {
    const key = address.toLowerCase().trim();
    const match = Object.entries(LOCATIONS).find(([k]) => key.includes(k));
    if (match) {
      return { content: [{ type: "text", text: JSON.stringify({ address, ...match[1] }) }] };
    }
    return { content: [{ type: "text", text: JSON.stringify({ address, lat: 0, lng: 0, error: "not found" }) }] };
  }
);

server.tool("reverse_geocode", "Convert lat/lng to a place name",
  { lat: z.number().describe("Latitude"), lng: z.number().describe("Longitude") },
  async ({ lat, lng }) => {
    const match = Object.entries(LOCATIONS).find(
      ([, v]) => Math.abs(v.lat - lat) < 1 && Math.abs(v.lng - lng) < 1
    );
    return { content: [{ type: "text", text: JSON.stringify({ lat, lng, place: match ? match[0] : "unknown" }) }] };
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
