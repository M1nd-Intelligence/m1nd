# API Reference Overview

m1nd exposes **43 MCP tools** over JSON-RPC 2.0 via stdio. Every tool requires `agent_id` as a parameter. Tools are organized into seven groups.

## Tool Index

### Core Activation

| Tool | Description |
|------|-------------|
| [`m1nd.activate`](activation.md#m1ndactivate) | Spreading activation query across the connectome |
| [`m1nd.warmup`](activation.md#m1ndwarmup) | Task-based warmup and priming |
| [`m1nd.resonate`](activation.md#m1ndresonate) | Resonance analysis: harmonics, sympathetic pairs, and resonant frequencies |

### Analysis

| Tool | Description |
|------|-------------|
| [`m1nd.impact`](analysis.md#m1ndimpact) | Impact radius / blast analysis for a node |
| [`m1nd.predict`](analysis.md#m1ndpredict) | Co-change prediction for a modified node |
| [`m1nd.counterfactual`](analysis.md#m1ndcounterfactual) | What-if node removal simulation |
| [`m1nd.fingerprint`](analysis.md#m1ndfingerprint) | Activation fingerprint and equivalence detection |
| [`m1nd.hypothesize`](analysis.md#m1ndhypothesize) | Graph-based hypothesis testing against structural claims |
| [`m1nd.differential`](analysis.md#m1nddifferential) | Focused structural diff between two graph snapshots |
| [`m1nd.diverge`](analysis.md#m1nddiverge) | Structural drift between a baseline and current graph state |

### Memory & Learning

| Tool | Description |
|------|-------------|
| [`m1nd.learn`](memory.md#m1ndlearn) | Explicit feedback-based edge adjustment |
| [`m1nd.drift`](memory.md#m1nddrift) | Weight and structural drift analysis |
| [`m1nd.why`](memory.md#m1ndwhy) | Path explanation between two nodes |
| [`m1nd.trail.save`](memory.md#m1ndtrailsave) | Persist current investigation state |
| [`m1nd.trail.resume`](memory.md#m1ndtrailresume) | Restore a saved investigation |
| [`m1nd.trail.list`](memory.md#m1ndtraillist) | List saved investigation trails |
| [`m1nd.trail.merge`](memory.md#m1ndtrailmerge) | Combine two or more investigation trails |

### Exploration

| Tool | Description |
|------|-------------|
| [`m1nd.seek`](exploration.md#m1ndseek) | Intent-aware semantic code search |
| [`m1nd.scan`](exploration.md#m1ndscan) | Pattern-aware structural code analysis |
| [`m1nd.missing`](exploration.md#m1ndmissing) | Detect structural holes and missing connections |
| [`m1nd.trace`](exploration.md#m1ndtrace) | Map runtime errors to structural root causes |
| [`m1nd.timeline`](exploration.md#m1ndtimeline) | Git-based temporal history for a node |
| [`m1nd.federate`](exploration.md#m1ndfederate) | Multi-repository federated graph ingestion |

### Perspectives

| Tool | Description |
|------|-------------|
| [`m1nd.perspective.start`](perspectives.md#m1ndperspectivestart) | Enter a perspective: navigable route surface from a query |
| [`m1nd.perspective.routes`](perspectives.md#m1ndperspectiveroutes) | Browse the current route set with pagination |
| [`m1nd.perspective.inspect`](perspectives.md#m1ndperspectiveinspect) | Expand a route with metrics, provenance, and affinity |
| [`m1nd.perspective.peek`](perspectives.md#m1ndperspectivepeek) | Extract a code/doc slice from a route target |
| [`m1nd.perspective.follow`](perspectives.md#m1ndperspectivefollow) | Follow a route: move focus to target, synthesize new routes |
| [`m1nd.perspective.suggest`](perspectives.md#m1ndperspectivesuggest) | Get the next best move suggestion |
| [`m1nd.perspective.affinity`](perspectives.md#m1ndperspectiveaffinity) | Discover probable connections a route target might have |
| [`m1nd.perspective.branch`](perspectives.md#m1ndperspectivebranch) | Fork navigation state into a new branch |
| [`m1nd.perspective.back`](perspectives.md#m1ndperspectiveback) | Navigate back to previous focus |
| [`m1nd.perspective.compare`](perspectives.md#m1ndperspectivecompare) | Compare two perspectives on shared/unique nodes |
| [`m1nd.perspective.list`](perspectives.md#m1ndperspectivelist) | List all perspectives for an agent |
| [`m1nd.perspective.close`](perspectives.md#m1ndperspectiveclose) | Close a perspective and release associated locks |

### Lifecycle & Locks

| Tool | Description |
|------|-------------|
| [`m1nd.ingest`](lifecycle.md#m1ndingest) | Ingest or re-ingest a codebase, descriptor, or memory corpus |
| [`m1nd.health`](lifecycle.md#m1ndhealth) | Server health and statistics |
| [`m1nd.validate_plan`](lifecycle.md#m1ndvalidate_plan) | Validate a modification plan against the code graph |
| [`m1nd.lock.create`](lifecycle.md#m1ndlockcreate) | Pin a subgraph region and capture a baseline |
| [`m1nd.lock.watch`](lifecycle.md#m1ndlockwatch) | Set a watcher strategy on a lock |
| [`m1nd.lock.diff`](lifecycle.md#m1ndlockdiff) | Compute what changed in a locked region since baseline |
| [`m1nd.lock.rebase`](lifecycle.md#m1ndlockrebase) | Re-capture lock baseline from current graph |
| [`m1nd.lock.release`](lifecycle.md#m1ndlockrelease) | Release a lock and free its resources |

---

## Common Parameters

Every tool requires the `agent_id` parameter:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `agent_id` | `string` | Yes | Identifier for the calling agent. Used for session tracking, perspective ownership, lock ownership, and multi-agent coordination. |

Agent IDs are free-form strings. Convention: use the agent's name or role identifier (e.g. `"jimi"`, `"hacker-auth"`, `"forge-build"`).

## JSON-RPC Request Format

m1nd uses the [MCP protocol](https://modelcontextprotocol.io/) over JSON-RPC 2.0 via stdio. All tool calls use the `tools/call` method.

### Request structure

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "agent_id": "jimi",
      "query": "session management"
    }
  }
}
```

### Successful response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{ ... pretty-printed JSON result ... }"
      }
    ]
  }
}
```

The `text` field contains the tool-specific output as a pretty-printed JSON string. Parse it to get the structured result.

### Error response (tool execution error)

Tool execution errors are returned as MCP `isError` content, **not** as JSON-RPC errors:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Error: Node not found: file::nonexistent.py"
      }
    ],
    "isError": true
  }
}
```

### Error response (protocol error)

JSON-RPC protocol errors use standard error codes:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found: bad_method"
  }
}
```

## Error Codes

| Code | Meaning |
|------|---------|
| `-32700` | Parse error -- invalid JSON |
| `-32601` | Method not found -- unknown JSON-RPC method |
| `-32603` | Internal error -- server-level failure |

Tool-specific errors (node not found, perspective not found, lock ownership violation, etc.) are returned via `isError: true` in the content, not as JSON-RPC errors.

## Transport

m1nd supports two stdio transport modes, auto-detected per message:

- **Framed**: `Content-Length: N\r\n\r\n{json}` -- standard MCP/LSP framing. Used by Claude Code, Cursor, and most MCP clients.
- **Line**: `{json}\n` -- one JSON object per line. Used by simple scripts and testing.

The server auto-detects which mode each incoming message uses and responds in the same mode.

## Tool Name Normalization

Tool names accept both dots and underscores as separators. The server normalizes `_` to `.` before dispatch:

- `m1nd.activate` and `m1nd_activate` both work
- `m1nd.perspective.start` and `m1nd_perspective_start` both work
- `m1nd.lock.create` and `m1nd_lock_create` both work

## Protocol Handshake

Before calling tools, MCP clients perform a handshake:

```json
{"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}
```

Response:

```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": { "name": "m1nd-mcp", "version": "0.1.0" },
    "capabilities": { "tools": {} }
  }
}
```

Then list available tools:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}
```

This returns the full schema for all 43 tools with `inputSchema` for each.
