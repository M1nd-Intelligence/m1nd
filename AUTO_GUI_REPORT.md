# m1nd Auto-GUI Launch Report

## What Changed

When m1nd-mcp starts in stdio MCP mode (the default, no `--serve` flag), it now automatically spawns an HTTP server on port 1337 in the background and opens the browser to `http://localhost:1337`.

### New CLI Flag

- `--no-gui` -- Disables the auto-launched HTTP GUI. Use for CI, headless servers, or when the GUI is not needed.

### Behavior

| Mode | Command | Result |
|------|---------|--------|
| Default (stdio + GUI) | `m1nd-mcp` | JSON-RPC stdio + HTTP GUI on :1337 + browser opens |
| Headless stdio | `m1nd-mcp --no-gui` | JSON-RPC stdio only (old behavior) |
| Explicit HTTP | `m1nd-mcp --serve` | HTTP server only (unchanged) |
| Both transports | `m1nd-mcp --serve --stdio` | HTTP + stdio shared state (unchanged) |

### Implementation Details

- The background HTTP server is spawned via `tokio::spawn` -- non-blocking to the stdio loop.
- Browser auto-open uses `open` on macOS, `xdg-open` on Linux, `cmd /C start` on Windows (reuses existing `open_browser()` from http_server.rs).
- The GUI server creates its own `McpServer` instance from the same config. This avoids the `&mut self` exclusivity conflict with the stdio `McpServer::serve()` loop.
- Status line printed to stderr: `[m1nd-mcp] m1nd GUI: http://localhost:1337` -- does not interfere with stdio JSON-RPC on stdout.
- If the HTTP port is already in use (e.g., another m1nd instance), the GUI silently fails and stdio continues normally.
- Feature-gated behind `serve` (default-enabled). Without the feature, `--no-gui` is silently ignored and no GUI spawns.

### Files Modified

| File | Change |
|------|--------|
| `m1nd-mcp/src/cli.rs` | Added `--no-gui` flag |
| `m1nd-mcp/src/http_server.rs` | Added `spawn_background()` public function |
| `m1nd-mcp/src/main.rs` | Updated header comments, `run_stdio_server()` signature, spawn logic |

### Verification

- `cargo check` -- clean (default features, serve enabled)
- `cargo check --no-default-features` -- clean (serve disabled)
