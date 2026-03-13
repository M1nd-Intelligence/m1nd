# Introduction

m1nd is a neuro-symbolic connectome engine built in Rust. It ingests a codebase into a weighted, directed graph --- then lets you fire queries into that graph and watch where the energy goes. Signal propagates across structural, semantic, temporal, and causal dimensions. The graph learns from every interaction via Hebbian plasticity: paths you use get stronger, paths you ignore decay. The result is a code intelligence layer that adapts to how you think about your codebase. It ships as an [MCP](https://modelcontextprotocol.io/) server with 43 tools, runs on stdio, and works with any MCP-compatible client.

## The Problem

AI coding agents are powerful reasoners but terrible navigators. An LLM can analyze a function you paste into its context window, but it cannot *find* the right function in a codebase of 10,000 files without burning tokens on speculative grep, glob, and tree walks. The existing tools fail in different ways:

- **Full-text search** finds what you *said*, not what you *meant*. Searching for "authentication" won't surface the session middleware that enforces it.
- **RAG** retrieves chunks by embedding similarity, but each retrieval is stateless. It has no memory of what it retrieved last time, and no way to express relationships between results.
- **Static analysis** produces call graphs and ASTs, but they are frozen snapshots. They cannot answer "what if I remove this module?" or "what changed since my last session?"
- **Knowledge graphs** require manual curation and only return what was explicitly encoded.

m1nd solves this with a fundamentally different approach: spreading activation on a learned graph. Instead of matching tokens or embedding vectors, it propagates energy from seed nodes through weighted edges and observes the activation pattern that emerges. The graph topology determines where signal flows. Hebbian learning determines how strongly.

```
335 files -> 9,767 nodes -> 26,557 edges in 0.91s
activate in 31ms | impact in 5ms | trace in 3.5ms | learn in <1ms
```

No LLM calls. No API keys. No network. The binary is ~8MB of pure Rust.

## Key Capabilities

### Spreading Activation

The core query primitive. Fire a signal into the graph from one or more seed nodes. The signal propagates through the CSR adjacency structure, decaying at each hop, inhibited by negative edges, and scored across four dimensions:

| Dimension | Source | What It Captures |
|-----------|--------|------------------|
| Structural | CSR adjacency + PageRank | Graph distance, edge types, centrality |
| Semantic | Trigram matching on identifiers | Naming patterns, token overlap |
| Temporal | Git history + learn feedback | Co-change frequency, recency decay |
| Causal | Stacktrace mapping + call chains | Error proximity, suspiciousness |

The engine selects between a wavefront (BFS-parallel) and a heap (priority-queue) propagation strategy at runtime based on seed density and average degree. Results are merged with adaptive dimension weighting and a resonance bonus for nodes that score across 3 or 4 dimensions.

```jsonc
// Ask: "What's related to authentication?"
{"method": "tools/call", "params": {
  "name": "m1nd.activate",
  "arguments": {"query": "authentication", "agent_id": "dev"}
}}
// -> auth.py fires -> propagates to session, middleware, JWT, user model
//    4-dimensional relevance ranking in 31ms
```

### Hebbian Plasticity

The graph learns. When you tell m1nd that results were useful, edge weights strengthen along the activated paths (`delta_w = learning_rate * activation_src * activation_tgt`). When results go unused, inactive edges decay. After sustained strengthening, edges receive a permanent Long-Term Potentiation (LTP) bonus. After sustained weakening, Long-Term Depression (LTD) applies a permanent penalty. Homeostatic normalization prevents runaway weights by scaling incoming edges when their sum exceeds a ceiling.

```jsonc
// Tell the graph what was useful
{"method": "tools/call", "params": {
  "name": "m1nd.learn",
  "arguments": {
    "feedback": "correct",
    "node_ids": ["file::auth.py", "file::middleware.py"],
    "agent_id": "dev"
  }
}}
// -> 740 edges strengthened. Next query for "authentication" is smarter.
```

Plasticity state persists to disk. Across sessions, the graph evolves to match how your team navigates the codebase.

### Structural Hole Detection

`m1nd.missing` finds gaps in the graph --- nodes or edges that *should* exist based on the surrounding topology but don't. If every other module in a cluster has error handling and one doesn't, that's a structural hole. If two subsystems communicate through a single bridge node, that's a fragility point. This turns the graph into a specification-free audit tool.

### Counterfactual Simulation

"What breaks if I delete `spawner.py`?" The counterfactual engine uses a zero-allocation bitset mask (no graph clone) to virtually remove nodes and their incident edges, then re-runs spreading activation to measure the impact. The output includes orphaned nodes, weakened nodes, reachability loss, and cascade depth. Synergy analysis reveals whether removing two modules together is worse than the sum of removing them individually.

```
counterfactual("spawner.py") -> 4,189 affected nodes, cascade at depth 3
counterfactual("config.py")  -> 2,531 affected nodes (despite being universally imported)
```

### Perspectives

Stateful graph navigation. Open a perspective anchored to a node, list available routes, follow edges, peek at source code, and branch explorations. Perspectives carry confidence calibration and epistemic safety checks. Two agents can open independent perspectives on the same graph and later compare them to find shared nodes and divergent conclusions.

### XLR Noise Cancellation

Borrowed from professional audio engineering. Like a balanced XLR cable, m1nd transmits signal on two inverted channels --- hot (from your query seeds) and cold (from automatically selected anti-seeds that are structurally similar but semantically distant). The cold signal cancels common-mode noise: the generic infrastructure that every query touches. What survives is the differential signal specific to your actual question. Sigmoid gating, density-adaptive strength, and seed immunity prevent over-cancellation.

## Who This Is For

- **AI agent developers** who need their agents to navigate code without wasting context tokens on trial-and-error search.
- **IDE and tool builders** looking for an MCP-compatible code intelligence backend that goes beyond static analysis.
- **Anyone using MCP clients** (Claude Code, Cursor, Windsurf, Zed, Cline, Roo Code, Continue, OpenCode, Amazon Q, GitHub Copilot) who wants a graph that learns from their workflow.
- **Multi-agent orchestrators** who need a shared, persistent code graph that multiple agents can query, learn from, and lock regions of concurrently.

m1nd is *not* a replacement for full-text search, and it does not use neural embeddings (v1 uses trigram matching for the semantic dimension). If you need "find code that *means* X but never uses the word," m1nd will not do it yet. See [the FAQ](faq.md) for more on current limitations.

## How to Read This Wiki

This wiki is organized into four sections. Read them in order for a full understanding, or jump to the section you need.

**[Architecture](architecture/overview.md)** --- How m1nd is built. The three crates (`m1nd-core`, `m1nd-ingest`, `m1nd-mcp`), the CSR graph representation, the activation engine hierarchy, and the MCP server protocol layer. Start here if you want to contribute or understand the internals.

**[Concepts](concepts/spreading-activation.md)** --- The ideas behind the implementation. Spreading activation, Hebbian plasticity, XLR noise cancellation, and structural hole detection explained with enough depth to reason about behavior and tune parameters.

**[API Reference](api-reference/overview.md)** --- All 43 MCP tools documented with parameters, return types, and usage examples. Organized by function: activation and queries, analysis and prediction, memory and learning, exploration and discovery, perspective navigation, and lifecycle administration.

**[Tutorials](tutorials/quickstart.md)** --- Practical walkthroughs. Quick start, your first query, and multi-agent workflows with trail save/resume/merge.

The **[Benchmarks](benchmarks.md)** page has real performance numbers from a production codebase (335 files, ~52K lines, 9,767 nodes, 26,557 edges). The **[Changelog](changelog.md)** tracks what changed between versions.

---

```
cargo build --release
./target/release/m1nd-mcp
```

The server starts, listens on stdio, and waits for JSON-RPC. The graph is empty until you call `m1nd.ingest`. From there, every query teaches it something.
