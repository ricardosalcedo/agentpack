"""Minimal pricing agent stub — delegates to research-agent, uses filesystem MCP."""
import json
import sys


AGENT_CARD = {
    "name": "pricing-agent",
    "description": "Estimates STR property pricing using market data",
    "skills": [
        {"id": "property-valuation", "description": "Estimate nightly rate for a property"},
        {"id": "market-comparison", "description": "Compare against similar listings"},
    ],
    "protocol": "a2a",
}


def handle_request(request: dict) -> dict:
    method = request.get("method", "")
    if method == "agent/card":
        return {"result": AGENT_CARD}
    if method == "agent/task":
        address = request.get("params", {}).get("address", "unknown")
        return {"result": {"status": "completed", "output": {
            "address": address,
            "estimated_nightly_rate": 185,
            "confidence": 0.78,
            "comparable_listings": 12,
        }}}
    return {"error": {"code": -32601, "message": f"Unknown method: {method}"}}


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            response = handle_request(request)
            response["id"] = request.get("id")
            response["jsonrpc"] = "2.0"
            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()
        except json.JSONDecodeError:
            pass


if __name__ == "__main__":
    main()
