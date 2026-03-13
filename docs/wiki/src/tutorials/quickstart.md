# Quick Start

Five minutes from zero to your first query.

## Prerequisites

- **Rust toolchain**: 1.75+ (install via [rustup](https://rustup.rs/))
- **A codebase**: Any project with Python, Rust, TypeScript/JavaScript, Go, or Java files. Other languages use a generic fallback extractor.

## Installation

### From source (recommended)

```bash
git clone https://github.com/cosmophonix/m1nd.git
cd m1nd
cargo build --release
```

The binary is at `./target/release/m1nd-mcp` (~8MB). Copy it wherever you want:

```bash
cp target/release/m1nd-mcp /usr/local/bin/
```

### From crates.io

```bash
cargo install m1nd-mcp
```

### Verify the build

```bash
m1nd-mcp --help
```

The binary should start and wait for JSON-RPC input on stdin. Press Ctrl+C to exit.

## Configuration

m1nd is an MCP server that communicates over stdio using JSON-RPC. You configure it in your AI client's MCP settings.

### Claude Code

Add to your Claude Code MCP configuration (`.mcp.json` in your project root, or `~/.claude/mcp.json` globally):

```json
{
  "mcpServers": {
    "m1nd": {
      "command": "/path/to/m1nd-mcp",
      "env": {
        "M1ND_GRAPH_SOURCE": "/tmp/m1nd-graph.json",
        "M1ND_PLASTICITY_STATE": "/tmp/m1nd-plasticity.json"
      }
    }
  }
}
```

### Cursor

In Cursor settings, navigate to **MCP Servers** and add:

```json
{
  "m1nd": {
    "command": "/path/to/m1nd-mcp",
    "env": {
      "M1ND_GRAPH_SOURCE": "/tmp/m1nd-graph.json",
      "M1ND_PLASTICITY_STATE": "/tmp/m1nd-plasticity.json"
    }
  }
}
```

### Windsurf / Cline / Roo Code / Continue

Any MCP-compatible client works. The pattern is the same: point the client at the `m1nd-mcp` binary and optionally set the environment variables for persistence.

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `M1ND_GRAPH_SOURCE` | Path to persist graph state between sessions | In-memory only (lost on exit) |
| `M1ND_PLASTICITY_STATE` | Path to persist learned edge weights | In-memory only (lost on exit) |

**Recommendation**: Always set both variables. Without persistence, every restart discards learned weights and you lose the graph's accumulated intelligence.

### Advanced Configuration

The server accepts these configuration parameters (set via environment):

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `learning_rate` | 0.08 | How aggressively the graph learns from feedback |
| `decay_rate` | 0.005 | Rate of edge weight decay over time |
| `xlr_enabled` | true | Enable XLR noise cancellation |
| `auto_persist_interval` | 50 | Persist state every N queries |
| `max_concurrent_reads` | 32 | Maximum concurrent read operations |
| `domain` | "code" | Domain preset: `code`, `music`, `memory`, or `generic` |

## First Run: Ingest a Project

Once your MCP client is configured and m1nd is running, ingest your codebase. This is the foundation for everything else.

If you are using m1nd through an MCP client like Claude Code, the client sends the JSON-RPC calls for you when you invoke the tools. The raw JSON-RPC is shown here for clarity.

### Step 1: Ingest

```jsonc
// Ingest your project
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.ingest",
    "arguments": {
      "path": "/path/to/your/project",
      "agent_id": "dev"
    }
  }
}
```

Expected response (times will vary by project size):

```json
{
  "files_processed": 335,
  "nodes_created": 9767,
  "edges_created": 26557,
  "languages": {"python": 335},
  "elapsed_ms": 910
}
```

**What happened**: m1nd parsed every file, extracted structural elements (modules, classes, functions, imports), resolved references between them, built a compressed graph, and computed PageRank centrality for every node.

### Step 2: Verify with Health Check

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.health",
    "arguments": {
      "agent_id": "dev"
    }
  }
}
```

Expected response:

```json
{
  "status": "ok",
  "nodes": 9767,
  "edges": 26557,
  "finalized": true,
  "plasticity_records": 0,
  "agents_seen": ["dev"],
  "queries_served": 1,
  "uptime_seconds": 12
}
```

If you see `"nodes": 0`, re-check the path you passed to `ingest`.

### Step 3: Your First Query

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "query": "authentication",
      "agent_id": "dev",
      "top_k": 5
    }
  }
}
```

Expected response:

```json
{
  "activated": [
    {
      "node_id": "file::auth.py",
      "score": 0.89,
      "dimension_scores": {
        "structural": 0.92,
        "semantic": 0.95,
        "temporal": 0.78,
        "causal": 0.71
      }
    },
    {"node_id": "file::middleware.py", "score": 0.72},
    {"node_id": "file::session.py", "score": 0.61}
  ],
  "ghost_edges": [
    {"from": "file::auth.py", "to": "file::rate_limiter.py", "confidence": 0.34}
  ]
}
```

**What happened**: m1nd fired a signal into the graph from nodes matching "authentication" and let it propagate across structural, semantic, temporal, and causal dimensions. Noise was cancelled via XLR differential processing. The results are ranked by multi-dimensional activation score, not just text matching.

## What Next

You now have a working m1nd instance. From here:

- **[First Query Tutorial](first-query.md)**: Step-by-step walkthrough of the full activate-learn-activate cycle, counterfactual simulation, and structural hole detection.
- **[Multi-Agent Tutorial](multi-agent.md)**: How to use m1nd with multiple agents sharing one graph.
- **[FAQ](../faq.md)**: Common questions and answers.

## Troubleshooting

**"No graph snapshot found, starting fresh"** -- This is normal on first run. The graph is empty until you call `ingest`.

**Ingest returns 0 files** -- Check that the path is correct and contains supported source files. m1nd currently has extractors for Python (.py), Rust (.rs), TypeScript/JavaScript (.ts/.js/.tsx/.jsx), Go (.go), and Java (.java). Other files use a generic fallback extractor.

**MCP client does not see m1nd tools** -- Verify the binary path is correct and the binary is executable (`chmod +x m1nd-mcp`). Check your client's MCP server logs for connection errors.

**Permission denied** -- Ensure the `M1ND_GRAPH_SOURCE` and `M1ND_PLASTICITY_STATE` paths are writable.
