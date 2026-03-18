# m1nd Deployment & Production Setup

While `m1nd` can be run dynamically via stdio MCP calls (where the IDE starts and stops the process), this leads to high latency on large codebases because the graph (which can exceed 100MB) must be loaded into RAM on every call.

For a true "always-on" AI nervous system, `m1nd` is designed to run as a persistent HTTP server. IDEs and Agents then communicate with it via a lightweight stdio-to-HTTP proxy.

## Architecture

1. **Persistent Server (`m1nd-mcp --serve`)**: Runs constantly in the background, keeping the graph loaded in RAM for sub-millisecond query responses.
2. **LaunchAgent / Daemon**: Ensures the server starts on boot and restarts if it crashes.
3. **Smart Ingest / File Watcher**: A background service that watches project directories (`WatchPaths`) and incrementally updates the graph when files change, bypassing noise (like `node_modules` or `Pods`).
4. **Stdio Proxy**: A tiny Python script that your IDE (Claude Code, Cursor, Antigravity) calls as its "MCP server". It forwards the JSON-RPC calls via HTTP to the persistent server.

## 1. Setup the Persistent Server

Ensure you have your environment variables set for persistent storage.

Create a LaunchAgent (for macOS) at `~/Library/LaunchAgents/world.m1nd.mcp-server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>world.m1nd.mcp-server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/m1nd-mcp</string>
        <string>--serve</string>
        <string>--port</string>
        <string>1337</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>M1ND_GRAPH_SOURCE</key>
        <string>/Users/youruser/.m1nd/graph.json</string>
        <key>M1ND_PLASTICITY_STATE</key>
        <string>/Users/youruser/.m1nd/plasticity.json</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load it:
```bash
launchctl load ~/Library/LaunchAgents/world.m1nd.mcp-server.plist
```

## 2. Configure the Stdio Proxy

Instead of pointing your IDE to the `m1nd-mcp` binary, point it to the provided `m1nd-proxy.py` (found in `scripts/macos/m1nd-proxy.py`).

**Example `mcp.json` (Claude Code):**

```json
{
  "mcpServers": {
    "m1nd": {
      "command": "python3",
      "args": ["/Users/youruser/.m1nd/m1nd-proxy.py"],
      "env": {
        "M1ND_PORT": "1337"
      }
    }
  }
}
```

## 3. Smart Namespace Ingest (Noise Reduction)

If your workspace contains massive dependencies (e.g., iOS Pods, `node_modules`), a raw ingest will pollute the graph and degrade semantic search quality.

Use the `smart-ingest.py` and `file-watcher.py` scripts (in `scripts/macos/`) to:
1. Only ingest specific relevant namespaces (`mode="merge"`).
2. Automatically trigger incremental syncs when files are modified.

By segregating your graph intelligently and keeping it persistently in RAM, `m1nd` operates at maximum physics speed with zero startup overhead.
