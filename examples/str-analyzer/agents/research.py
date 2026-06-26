"""Minimal research agent stub — demonstrates an A2A-compatible agent managed by AgentPack."""
import json
import sys


AGENT_CARD = {
    "name": "research-agent",
    "description": "Performs web research for property analysis",
    "skills": [
        {"id": "web-research", "description": "Search and synthesize web sources"},
        {"id": "data-extraction", "description": "Extract structured data from web pages"},
    ],
    "protocol": "a2a",
}


def handle_request(request: dict) -> dict:
    method = request.get("method", "")
    if method == "agent/card":
        return {"result": AGENT_CARD}
    if method == "agent/task":
        query = request.get("params", {}).get("query", "")
        return {"result": {"status": "completed", "output": f"Research results for: {query}"}}
    return {"error": {"code": -32601, "message": f"Unknown method: {method}"}}


def main():
    """JSON-RPC over stdio — one request per line."""
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
