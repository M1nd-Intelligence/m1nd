# Frequently Asked Questions

## Why m1nd?

### Why not just use grep/ripgrep?

Grep finds text. m1nd finds structure, relationships, and consequences. Grep tells you "this string appears on line 47 of auth.py." m1nd tells you "auth.py is structurally connected to 12 modules, and changing it cascades to 4,271 nodes across 3 layers." Grep answers in 5ms but cannot reason about dependencies, simulate deletions, or learn which results were useful. m1nd answers in 31ms and gets smarter every time you use it. Use both: grep for exact text, m1nd for structural reasoning.

### How does m1nd compare to AI-powered code search (Cursor, GitHub Copilot search)?

AI-powered code search sends your query (and often your code) to an LLM, costing tokens per search. Cursor's codebase indexing and Copilot's @workspace search embed your code into vectors and retrieve top-K chunks -- each query burns $0.05-$0.50 in API costs. m1nd runs locally in Rust with zero LLM calls, zero tokens, and zero network round-trips. A spreading activation query returns ranked results in 31ms at $0.00. m1nd also maintains a persistent graph that learns from feedback via Hebbian plasticity -- something no embedding-based search can do.

### What's the real cost of LLM-based code search?

Every LLM-based code search burns tokens: the query, the retrieved context, and the model's processing. At 100 searches per day, that is $5-$50/day per developer in API costs alone. In early 2026, Cursor users reported monthly overages exceeding $22,000 for teams with heavy codebase search usage. m1nd processes the same queries locally for $0.00 -- no API keys, no cloud egress, no token metering, no surprise bills. The 8MB binary pays for itself on day one.

### Does m1nd replace my IDE's search?

No. m1nd complements your IDE's search, not replaces it. Use your IDE's go-to-definition and find-references for precise, language-server-powered navigation. Use grep/ripgrep for fast text matching. Use m1nd for the questions those tools cannot answer: blast radius analysis, counterfactual simulation ("what breaks if I delete this module?"), structural hole detection ("what is missing?"), and hypothesis testing. m1nd adds a reasoning layer on top of your existing toolchain through 43 MCP tools that any compatible editor can call.

### Why local-first? Why not cloud?

Three reasons: cost, privacy, and latency. Cloud code search means your proprietary code leaves your machine on every query, each query costs tokens, and network round-trips add 100-500ms of latency. m1nd runs as a single 8MB binary on your machine. Your code never leaves localhost. Queries resolve in 31-77ms. There are no API keys to manage, no cloud accounts to provision, no egress fees, and no vendor lock-in. The graph persists to a local JSON file you own and can inspect, copy, or delete at any time.

### What makes m1nd "agent-native"?

m1nd was built from the ground up as an MCP server, not retrofitted. It exposes 43 tools that AI agents call natively -- spreading activation, impact analysis, counterfactual simulation, hypothesis testing, and a full Perspective navigation system with 12 stateful tools for browsing the code graph. Multiple agents share the same graph instance simultaneously, each with isolated perspectives. The graph learns from agent feedback via Hebbian plasticity, getting smarter with every interaction. No other code intelligence tool provides persistent, learning, multi-agent structural reasoning over MCP.

---

## General

### What is m1nd?

m1nd is a neuro-symbolic connectome engine for code intelligence. It parses your codebase into a weighted graph and provides 43 MCP tools for querying, learning, and navigating it. Built in Rust, it runs locally as an MCP server and works with any MCP-compatible AI agent (Claude Code, Cursor, Windsurf, Zed, etc.).

The key differentiator: the graph *learns*. When you tell m1nd which results were useful and which were not, it adjusts edge weights via Hebbian plasticity. Over time, the graph adapts to how your team thinks about your codebase.

### How is m1nd different from grep / ripgrep?

Grep finds *text*. m1nd finds *structure and relationships*.

Grep tells you which files contain the word "authentication". m1nd tells you which modules are structurally connected to authentication, which ones are likely co-changed when auth changes, which hidden dependencies exist between auth and seemingly unrelated modules, and what would break if you removed the auth module.

Grep is fast and essential. m1nd answers questions that grep cannot even formulate.

### How is m1nd different from RAG?

RAG (Retrieval-Augmented Generation) embeds code chunks into vectors and retrieves the top-K most similar chunks for each query. Each retrieval is independent -- RAG has no memory of previous queries and no understanding of relationships between results.

m1nd maintains a persistent graph where relationships are first-class citizens. The graph learns from feedback, remembers investigations across sessions, and can answer structural questions ("what breaks if I delete X?", "does A depend on B at runtime?") that RAG cannot.

RAG is useful for semantic similarity search. m1nd is useful for structural reasoning. They are complementary, not competing.

### How is m1nd different from static analysis tools (tree-sitter, ast-grep, Sourcegraph)?

Static analysis tools parse code into ASTs and compute call graphs, type hierarchies, and cross-references. These are accurate but frozen -- they represent the code at a single point in time and cannot answer "what if?" questions.

m1nd uses similar structural information as a starting point but adds three things static analysis cannot: (1) Hebbian learning that adapts the graph based on usage, (2) temporal intelligence from co-change history, and (3) simulation engines for hypotheses and counterfactuals.

Sourcegraph is a search engine. m1nd is a reasoning engine.

### Do I need an LLM to use m1nd?

No. m1nd makes zero LLM calls. It is pure Rust computation: graph algorithms, spreading activation, Hebbian plasticity. No API keys, no network calls, no token costs.

m1nd is designed to *work alongside* LLMs (as an MCP tool that agents call), but the graph engine itself is completely self-contained.

### What languages does m1nd support?

m1nd has dedicated extractors for:

- Python (.py)
- Rust (.rs)
- TypeScript (.ts, .tsx)
- JavaScript (.js, .jsx)
- Go (.go)
- Java (.java)

All other file types use a generic fallback extractor that identifies functions, classes, and imports through heuristic pattern matching. The generic extractor produces a less detailed graph but still captures useful structure.

Tree-sitter integration is planned, which would add support for 64+ languages.

### Is m1nd open source?

Yes. MIT license. Source at [github.com/cosmophonix/m1nd](https://github.com/cosmophonix/m1nd).

---

## Installation

### What platforms does m1nd support?

Any platform where Rust compiles. Tested on:

- macOS (ARM64 and x86_64)
- Linux (x86_64, ARM64)
- Windows (x86_64)

### What is the minimum Rust version?

Rust 1.75 or later. The project uses the 2021 edition.

### How large is the binary?

Approximately 8MB for a release build. No runtime dependencies, no shared libraries. The binary is fully self-contained.

### Can I install from crates.io?

Yes: `cargo install m1nd-mcp`

---

## Usage

### How many files can m1nd handle?

Tested up to 10,000+ files. At ~2MB of memory for 10,000 nodes, a 100K-file codebase would need roughly 20MB for the graph. This is well within modern machine capacity.

At 400K+ files, the in-memory graph starts to become a consideration (~80MB), but it still works. m1nd was optimized for codebases in the 100 to 50,000 file range.

### Does it work with non-code files?

Yes. m1nd has a `json` adapter for structured data and a `memory` adapter for text corpora. You can also use the generic fallback extractor for any text file. The graph is not limited to source code -- it is a general-purpose knowledge graph that happens to have excellent code extractors.

### How much memory does m1nd use?

The graph itself is compact: ~2MB for a 10,000-node graph with 26,000 edges (Compressed Sparse Row format). The MCP server process uses additional memory for the JSON-RPC layer, perspective state, lock baselines, and trail storage. A typical production instance serving a 335-file codebase uses under 50MB total.

### Can I ingest multiple codebases into one graph?

Yes, through the `federate` tool. It ingests multiple repositories into a unified graph with automatic cross-repository edge detection:

```jsonc
{"method":"tools/call","params":{"name":"m1nd.federate","arguments":{
  "agent_id":"dev",
  "repos":[
    {"name":"backend","path":"/project/backend"},
    {"name":"frontend","path":"/project/frontend"}
  ]
}}}
```

### Can I do incremental ingests?

Yes. Pass `"incremental": true` to the `ingest` tool. Incremental ingest only re-processes files that changed since the last ingest, preserving learned edge weights for unchanged regions.

---

## Architecture

### Where is the graph stored?

In memory during runtime. Optionally persisted to JSON files on disk via the `M1ND_GRAPH_SOURCE` and `M1ND_PLASTICITY_STATE` environment variables.

Without persistence configured, the graph is lost when the process exits. Always configure persistence for production use.

### How often does it persist?

By default, every 50 queries. Also on shutdown. The `auto_persist_interval` configuration parameter controls this.

### Can I export the graph?

The persisted graph state is a JSON file (`graph_snapshot.json`). You can read, copy, and process it with standard JSON tools. The format includes all nodes, edges, PageRank scores, and metadata.

The plasticity state is a separate JSON file (`plasticity_state.json`) containing per-edge synaptic records (learned weights).

### What is the graph format?

Compressed Sparse Row (CSR) with forward and reverse adjacency lists. Each node has an external ID (e.g., `file::auth.py`), a type (file, class, function, module), metadata, and a PageRank score. Each edge has a type (imports, calls, inherits, co_change), a base weight, and an optional plasticity weight.

### What is XLR noise cancellation?

Borrowed from professional audio engineering. Like a balanced XLR cable, m1nd transmits the activation signal on two inverted channels and subtracts common-mode noise at the receiver. This reduces false positives in activation queries by cancelling out signals that propagate through generic hub nodes (like `config.py` or `utils.py`) rather than through meaningful structural paths.

XLR is enabled by default. Pass `"xlr": false` to `activate` to disable it for a single query.

---

## MCP Protocol

### What MCP clients work with m1nd?

Any client that speaks MCP over stdio. Tested and verified:

- Claude Code
- Cursor
- Windsurf
- Zed
- Cline
- Roo Code
- Continue
- OpenCode
- GitHub Copilot (MCP mode)
- Amazon Q Developer

### Can I use m1nd without MCP?

The server speaks JSON-RPC over stdio, which is the MCP transport. You can send raw JSON-RPC from any program that can write to stdin and read from stdout. No MCP client library is required -- the protocol is just JSON over pipes.

### What MCP protocol version does m1nd implement?

Protocol version `2024-11-05`. The server reports this in the `initialize` response.

### Does m1nd support MCP notifications?

Yes. The server silently ignores incoming notifications per the MCP specification.

---

## Performance

### How fast is spreading activation?

31-77ms for a 9,767-node graph, returning 15-20 ranked results. The variance depends on query specificity and graph density around the activated region.

### How fast is ingest?

910ms for 335 Python files producing 9,767 nodes and 26,557 edges. This includes parsing, reference resolution, edge creation, and PageRank computation.

### How fast is learning?

Sub-millisecond. A `learn` call with feedback on 2-3 nodes adjusts hundreds of edges in under 1ms.

### What about 100K files?

Ingest would take roughly 30-45 seconds (linear scaling from the 335-file benchmark). Activation would be 100-200ms. The graph would occupy ~20MB in memory. These are estimates -- actual performance depends on code density and language.

### What is the lock diff speed?

0.08 microseconds (80 nanoseconds). Lock diff is a constant-time operation -- it compares fingerprints, not individual nodes. It is essentially free.

### Full benchmark table

All numbers from real execution against a production Python backend (335 files, ~52K lines):

| Operation | Time | Scale |
|-----------|------|-------|
| Full ingest | 910ms | 335 files, 9,767 nodes, 26,557 edges |
| Spreading activation | 31-77ms | 15 results from 9,767 nodes |
| Blast radius (depth=3) | 5-52ms | Up to 4,271 affected nodes |
| Stacktrace analysis | 3.5ms | 5 frames, 4 suspects ranked |
| Plan validation | 10ms | 7 files, 43,152 blast radius |
| Counterfactual cascade | 3ms | Full BFS on 26,557 edges |
| Hypothesis testing | 58ms | 25,015 paths explored |
| Pattern scan (all 8) | 38ms | 335 files, 50 findings per pattern |
| Multi-repo federation | 1.3s | 11,217 nodes, 18,203 cross-repo edges |
| Lock diff | 0.08us | 1,639-node subgraph comparison |
| Trail merge | 1.2ms | 5 hypotheses, 3 conflicts detected |

---

## Plasticity and Learning

### How does Hebbian learning work?

"Neurons that fire together wire together." When you call `learn` with `feedback: "correct"` and a list of node IDs, m1nd identifies all edges on paths between those nodes and increases their weights (Long-Term Potentiation, LTP). When you call with `feedback: "wrong"`, it decreases weights on paths leading to the marked nodes (Long-Term Depression, LTD).

The `strength` parameter (default 0.2) controls how aggressively weights shift. The `learning_rate` server configuration (default 0.08) provides a global scaling factor.

### Can I reset learned weights?

Yes. Delete the `plasticity_state.json` file and restart the server. Alternatively, re-ingest the codebase, which rebuilds the graph from scratch (but does not clear the plasticity state file -- you need to delete it separately or pass `"incremental": false` and delete the plasticity file).

### Does the graph overfit?

Hebbian learning includes homeostatic normalization to prevent runaway weight amplification. Edge weights are bounded and periodically normalized so that heavily-used paths do not completely dominate. The `decay_rate` parameter (default 0.005) provides a slow decay toward baseline weights over time.

In practice, overfitting requires thousands of feedback signals consistently reinforcing the same narrow paths. Normal usage produces a well-distributed weight landscape.

### What is "partial" feedback?

The `learn` tool accepts three feedback types:

- `correct` -- strengthen paths (LTP)
- `wrong` -- weaken paths (LTD)
- `partial` -- mixed signal. Applies a mild strengthening (half the LTP strength) to acknowledged nodes while slightly weakening peripheral paths.

---

## Perspectives

### What are perspectives?

A perspective is a stateful navigation session through the graph. You start one with a query or anchor node, and m1nd synthesizes a "route surface" -- a ranked set of paths you can follow. As you navigate (follow, back, branch), the perspective maintains breadcrumb history and updates the route surface.

Think of it as a browser for the code graph. You have a current "page" (focus node), links (routes), history (back button), and bookmarks (branches).

### How many perspectives can be open simultaneously?

There is no hard limit. Each perspective occupies a small amount of memory (proportional to the number of visited nodes). In practice, 10-20 concurrent perspectives per agent is typical. Close perspectives you are done with to free memory.

### Do perspectives persist across sessions?

Perspectives are in-memory only and are lost when the server restarts. For persistent investigation state, use the trail system (`trail.save` / `trail.resume`).

### Can I branch a perspective?

Yes. `perspective.branch` forks the current navigation state into a new independent perspective. Both the original and the branch have the same history up to the branch point, and diverge afterward. This is useful for exploring "what if I go this way instead?" without losing your current position.

---

## Comparison

### m1nd vs tree-sitter

tree-sitter is a parser. It produces ASTs from source code. m1nd uses structural information similar to what tree-sitter extracts, but builds a *weighted, learning graph* on top of it. tree-sitter tells you what the code *is*. m1nd tells you what the code *means* in context, what changed, what might break, and what is missing.

They are complementary. tree-sitter integration is planned for m1nd to expand language support.

### m1nd vs ast-grep

ast-grep is a structural search tool -- it finds code patterns using AST matching. m1nd does not do pattern matching on syntax trees. Instead, it does graph-level reasoning: spreading activation, hypothesis testing, counterfactual simulation, learning. ast-grep answers "where does this pattern appear?" m1nd answers "what is connected to this, what breaks if it changes, and what am I missing?"

### m1nd vs RAG (Retrieval-Augmented Generation)

RAG embeds code into vectors and retrieves similar chunks. Each retrieval is independent and stateless. m1nd maintains a persistent graph with relationships, learning, and investigation state. RAG cannot answer structural questions, simulate deletions, or learn from feedback.

RAG costs LLM tokens per query. m1nd costs zero tokens per query.

### m1nd vs Sourcegraph / CodeGraph / SCIP

Sourcegraph provides cross-repository code search and navigation based on SCIP (Source Code Intelligence Protocol) indexing. It produces accurate, language-server-quality code intelligence.

m1nd is not a code search engine. It is a reasoning engine. Sourcegraph tells you "function X is defined here and called here." m1nd tells you "if you change function X, here are the 4,189 nodes in the cascade, and the graph predicts these 3 files probably need changes too."

Sourcegraph is a hosted SaaS product. m1nd is a local binary with zero cost per query.

### m1nd vs GitHub Copilot context

Copilot uses a mix of embeddings and heuristics to select context files for the LLM. It does not maintain a persistent graph, does not learn from feedback, and does not support structural queries.

m1nd can be used *alongside* Copilot -- through MCP, Copilot can call m1nd tools to get smarter context before generating code.

---

## Contributing

### How do I contribute?

See [CONTRIBUTING.md](https://github.com/cosmophonix/m1nd/blob/main/CONTRIBUTING.md). Fork the repo, create a branch, make your changes with tests, and open a PR.

### What is the test suite?

```bash
cargo test --all
```

Tests cover the core graph engine, plasticity, spreading activation, hypothesis engine, all MCP tool handlers, and the JSON-RPC protocol layer.

### What are the code style requirements?

- `cargo fmt` before committing
- `cargo clippy --all -- -D warnings` must pass
- No `unsafe` without an explanatory comment
- All new code needs tests

### What areas need the most help?

1. **Language extractors**: Adding tree-sitter integration or new language-specific extractors
2. **Graph algorithms**: Community detection, better decay functions, embedding-based semantic scoring
3. **Benchmarks**: Running m1nd on diverse codebases and reporting real-world performance numbers
4. **Documentation**: Tutorials, examples, and translations

### Where do I report bugs?

GitHub Issues at [github.com/cosmophonix/m1nd/issues](https://github.com/cosmophonix/m1nd/issues). Use the `bug` label.
