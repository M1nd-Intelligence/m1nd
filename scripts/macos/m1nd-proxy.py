#!/usr/bin/env python3
"""
m1nd stdio→HTTP MCP proxy
Keeps the m1nd HTTP server alive and forwards stdio JSON-RPC to it.
Used by Claude/Antigravity host as the MCP command.
"""
import sys
import json
import subprocess
import time
import urllib.request
import urllib.error
import os
import signal
import threading

M1ND_PORT = int(os.environ.get("M1ND_PORT", "1337"))
M1ND_BIN = os.environ.get(
    "M1ND_BIN",
    "/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/target/release/m1nd-mcp"
)
M1ND_GRAPH = os.environ.get(
    "M1ND_GRAPH_SOURCE",
    "/Users/cosmophonix/.m1nd/graph.json"
)
M1ND_PLAST = os.environ.get(
    "M1ND_PLASTICITY_STATE",
    "/Users/cosmophonix/.m1nd/plasticity.json"
)

_server_proc = None
_server_lock = threading.Lock()


def ensure_server_running():
    """Start or verify the m1nd HTTP server is alive."""
    global _server_proc
    with _server_lock:
        # Check if already responding
        try:
            req = urllib.request.Request(
                f"http://127.0.0.1:{M1ND_PORT}/api/health",
                headers={"Accept": "application/json"}
            )
            urllib.request.urlopen(req, timeout=2)
            return True  # already running
        except Exception:
            pass

        # Not running — spawn it
        env = os.environ.copy()
        env["M1ND_GRAPH_SOURCE"] = M1ND_GRAPH
        env["M1ND_PLASTICITY_STATE"] = M1ND_PLAST

        _server_proc = subprocess.Popen(
            [M1ND_BIN, "--serve", "--no-gui", "--port", str(M1ND_PORT)],
            env=env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            preexec_fn=os.setsid
        )

        # Wait for ready (up to 30s)
        for _ in range(60):
            time.sleep(0.5)
            try:
                req = urllib.request.Request(
                    f"http://127.0.0.1:{M1ND_PORT}/api/health",
                    headers={"Accept": "application/json"}
                )
                urllib.request.urlopen(req, timeout=2)
                return True
            except Exception:
                pass

        return False


def http_tool_call(tool_name: str, arguments: dict) -> dict:
    """Call a tool on the m1nd HTTP server via /api/tools/{tool_name}."""
    payload = json.dumps(arguments).encode()

    req = urllib.request.Request(
        f"http://127.0.0.1:{M1ND_PORT}/api/tools/{tool_name}",
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            raw = resp.read()
            if not raw.strip():
                return {"text": "ok"}
            return json.loads(raw)
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        return {"error": f"HTTP {e.code}: {body}"}
    except Exception as e:
        return {"error": str(e)}


def make_response(req_id, content_text: str) -> dict:
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "content": [{"type": "text", "text": content_text}]
        }
    }


def make_error(req_id, message: str) -> dict:
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "error": {"code": -32603, "message": message}
    }


def handle_initialize(req_id):
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "m1nd-proxy", "version": "0.1.0"},
            "instructions": "m1nd HTTP proxy — forwards to :1337"
        }
    }


def main():
    # Ensure server before accepting any tool calls
    ensure_server_running()

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        method = msg.get("method", "")
        req_id = msg.get("id")

        if method == "initialize":
            resp = handle_initialize(req_id)
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
            continue

        if method == "notifications/initialized":
            # No response needed
            continue

        if method == "tools/list":
            # Fetch tool list from HTTP server
            try:
                req = urllib.request.Request(
                    f"http://127.0.0.1:{M1ND_PORT}/api/tools",
                    headers={"Accept": "application/json"}
                )
                with urllib.request.urlopen(req, timeout=10) as r:
                    tools = json.loads(r.read())
            except Exception:
                tools = {"tools": []}
            resp = {"jsonrpc": "2.0", "id": req_id, "result": tools}
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
            continue

        if method == "tools/call":
            params = msg.get("params", {})
            tool_name = params.get("name", "")
            arguments = params.get("arguments", {})

            # Ensure server is alive
            ensure_server_running()

            result = http_tool_call(tool_name, arguments)

            if "error" in result:
                resp = make_error(req_id, result["error"])
            else:
                # m1nd HTTP returns {"content": [...]} or plain dict
                if "content" in result:
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": result}
                elif "text" in result:
                    resp = make_response(req_id, result["text"])
                else:
                    # wrap scalar/object response as text
                    resp = make_response(req_id, json.dumps(result, indent=2))

            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
            continue

        # Unknown method
        if req_id is not None:
            resp = make_error(req_id, f"Unknown method: {method}")
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()


if __name__ == "__main__":
    main()
