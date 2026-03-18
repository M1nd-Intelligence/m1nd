# m1nd-mcp Build Report

**Date**: 2026-03-15
**Binary**: `target/release/m1nd-mcp` (51.8 MB)
**Build time**: 11.49s (incremental)

## Build Result: SUCCESS

```
cargo build --release
Finished `release` profile [optimized] target(s) in 11.49s
```

No compile errors. Zero warnings.

## Auto-GUI Feature: WORKING

### What it does
When `m1nd-mcp` starts in stdio mode (default, as Claude Code MCP server), it automatically spawns a background HTTP server on port 1337 and opens the browser. This gives a live GUI without needing `--serve` mode.

### Behavior verified

| Mode | GUI auto-launch | Port 1337 | HTTP 200 / | API /api/tools |
|------|----------------|-----------|------------|----------------|
| stdio (default) | YES | YES | YES | YES |
| stdio --no-gui | NO | NO | N/A | N/A |
| --serve | YES (explicit) | YES | YES | YES |

### Startup log (stdio default mode)
```
[m1nd] Domain: code
[m1nd] Loaded graph snapshot: 10744 nodes, 29935 edges
[m1nd] Loaded plasticity state: 29935 synaptic records
[m1nd-mcp] m1nd GUI: http://localhost:1337     ← GUI spawned
[m1nd] Domain: code                            ← second McpServer for GUI state
[m1nd] Loaded graph snapshot: 10744 nodes, 29935 edges
[m1nd] Loaded plasticity state: 29935 synaptic records
[m1nd-mcp] Server ready. 10744 nodes, 29935 edges
```

### Browser auto-open confirmed
Chromium auto-opened and established 2 connections to localhost:1337 (verified via `lsof`).

## Graph State
- **Nodes**: 10,744
- **Edges**: 29,935
- **Plasticity records**: 29,935 synaptic records loaded

## Key Source Files
- `m1nd-mcp/src/main.rs` — auto-GUI logic in `run_stdio_server()` (lines 77-94)
- `m1nd-mcp/src/http_server.rs` — `spawn_background()` (line 159), `open_browser()` (line 409)
- `m1nd-mcp/src/cli.rs` — `--no-gui` flag for headless/CI use
- `m1nd-mcp/Cargo.toml` — `features = ["serve"]` enabled by default

## CLI Modes Summary
```
m1nd-mcp                   # stdio + auto-GUI on :1337 (default)
m1nd-mcp --no-gui          # stdio only (CI, headless)
m1nd-mcp --serve           # HTTP server + UI only
m1nd-mcp --serve --stdio   # both transports, shared state
m1nd-mcp --serve --open    # HTTP + auto-open browser
```
