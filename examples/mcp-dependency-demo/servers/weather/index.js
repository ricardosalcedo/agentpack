import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod";

const server = new McpServer({ name: "weather-mcp", version: "1.0.0" });

function getWeather(lat, lng) {
  const temp = Math.round(20 + (lat / 10) + Math.sin(lng) * 5);
  const conditions = lat > 40 ? "cloudy" : lat > 25 ? "sunny" : "tropical";
  return { temp_celsius: temp, conditions, humidity: Math.round(50 + Math.cos(lng) * 20) };
}

server.tool("get_weather", "Get current weather for a lat/lng location",
  { lat: z.number().describe("Latitude"), lng: z.number().describe("Longitude") },
  async ({ lat, lng }) => {
    const weather = getWeather(lat, lng);
    return { content: [{ type: "text", text: JSON.stringify({ lat, lng, ...weather }) }] };
  }
);

server.tool("get_forecast", "Get 3-day forecast for a lat/lng location",
  { lat: z.number().describe("Latitude"), lng: z.number().describe("Longitude") },
  async ({ lat, lng }) => {
    const days = [0, 1, 2].map((d) => ({ day: d, ...getWeather(lat + d * 0.1, lng) }));
    return { content: [{ type: "text", text: JSON.stringify({ lat, lng, forecast: days }) }] };
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);
