import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const server = new McpServer({ name: "travel-advisor-mcp", version: "1.0.0" });

server.tool("recommend_destination", "Recommend a travel destination based on preferences",
  { preferences: z.string().describe("Travel preferences (e.g. 'warm beach', 'city culture')") },
  async ({ preferences }) => {
    const prefs = preferences.toLowerCase();
    let rec;
    if (prefs.includes("beach") || prefs.includes("warm")) {
      rec = { city: "Miami", reason: "Warm tropical weather, beautiful beaches" };
    } else if (prefs.includes("culture") || prefs.includes("city")) {
      rec = { city: "Paris", reason: "World-class museums, architecture, cuisine" };
    } else {
      rec = { city: "Tokyo", reason: "Unique blend of tradition and modernity" };
    }
    rec.note = "Use geocoding-mcp to get coordinates, then weather-mcp for forecast";
    return { content: [{ type: "text", text: JSON.stringify(rec) }] };
  }
);

server.tool("plan_trip", "Create a trip plan (requires geocoding + weather data upstream)",
  { destination: z.string().describe("Destination city"), days: z.number().describe("Number of days") },
  async ({ destination, days }) => {
    const plan = {
      destination, days,
      requires: ["geocoding-mcp for coordinates", "weather-mcp for forecast"],
      itinerary: Array.from({ length: days }, (_, i) => ({
        day: i + 1,
        activity: i === 0 ? "Arrival and exploration" : i === days - 1 ? "Departure" : "Sightseeing",
      })),
    };
    return { content: [{ type: "text", text: JSON.stringify(plan) }] };
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
