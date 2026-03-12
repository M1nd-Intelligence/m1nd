<p align="center">
  <img src=".github/logo.jpg" alt="m1nd" width="480" />
</p>

<p align="center">
  <strong>Cognitive Graph Engine</strong> -- spreading activation, noise cancellation, and Hebbian learning over knowledge graphs.<br/>
  Built for LLM agents via MCP.
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#the-13-tools">13 Tools</a> &middot;
  <a href="#use-cases">Use Cases</a> &middot;
  <a href="INTEGRATION-GUIDE.md">Integration Guide</a> &middot;
  <a href="EXAMPLES.md">Live Examples</a>
</p>

---

m1nd doesn't search your data -- it *activates* it. Query a concept, and the graph lights up: connected nodes fire with decaying signal across four dimensions (structural, semantic, temporal, causal), noise gets cancelled via XLR differential processing, and the system learns from feedback via Hebbian plasticity.

Built in Rust. 13 MCP tools. 159 tests. Domain-agnostic.

```
You ask:  "What's related to authentication?"
m1nd:     Activates auth module → propagates to session, middleware, JWT, user model
          → detects structural hole (no rate limiter connected)
          → predicts co-change with password reset module
          → all in one query, ranked by multi-dimensional relevance
```

---

## Why m1nd Exists

LLM agents are powerful reasoners but poor navigators. They can analyze code, write solutions, and debug problems -- but they struggle with:

- **"What does this change affect?"** -- blast radius is invisible without structural analysis
- **"What am I missing?"** -- structural holes in knowledge are undetectable by keyword search
- **"What else will change?"** -- co-change patterns require historical context
- **"How are these connected?"** -- dependency chains span files, modules, and abstraction layers

m1nd gives agents a nervous system. Instead of grepping through flat data, agents query a living graph that activates, learns, and improves over time.

---

## How It Works

### The Core Loop

```
INGEST    →  Build property graph from source data (code, JSON, any domain)
ACTIVATE  →  Spreading activation across 4 dimensions with XLR noise cancellation
LEARN     →  Hebbian plasticity: correct results strengthen connections, wrong results weaken them
PERSIST   →  Graph + plasticity state saved to disk, loaded on restart
```

### Four Activation Dimensions

| Dimension | What It Captures |
|-----------|-----------------|
| **Structural** | Graph topology: edges, PageRank, community structure |
| **Semantic** | Label similarity: char n-grams, co-occurrence (PPMI), synonym expansion |
| **Temporal** | Time dynamics: recency decay, change velocity, co-change history |
| **Causal** | Dependency flow: directed causation along import/call/contain edges |

### Key Algorithms

- **Spreading Activation** -- wavefront propagation with configurable decay, hop limits, and energy budgets
- **XLR Noise Cancellation** -- differential signal processing: maintains signal and noise CSR graphs, gates activation through adaptive sigmoid
- **Hebbian Plasticity** -- LTP (long-term potentiation) and LTD (long-term depression) on edge weights based on agent feedback
- **Standing Wave Resonance** -- harmonic analysis reveals natural frequencies and sympathetic node pairs in the graph
- **Counterfactual Simulation** -- "what if we remove node X?" via removal masks with cascade analysis and synergy detection

---

## The 13 Tools

m1nd exposes 13 tools over the Model Context Protocol (MCP), callable by any LLM agent via JSON-RPC stdio.

### Discovery

| Tool | Purpose |
|------|---------|
| `m1nd.activate` | Spreading activation query -- "what's related to X?" |
| `m1nd.why` | Path explanation -- "how are A and B connected?" |
| `m1nd.missing` | Structural hole detection -- "what's missing from this picture?" |
| `m1nd.fingerprint` | Equivalence detection -- "are these two things duplicates?" |
| `m1nd.resonate` | Harmonic analysis -- standing waves, resonant frequencies, sympathetic pairs |

### Change Analysis

| Tool | Purpose |
|------|---------|
| `m1nd.impact` | Blast radius -- "what does changing X affect?" |
| `m1nd.predict` | Co-change prediction -- "what else will need to change?" |
| `m1nd.counterfactual` | Removal simulation -- "what breaks if we remove X?" |

### Learning

| Tool | Purpose |
|------|---------|
| `m1nd.learn` | Hebbian feedback -- "this result was correct / wrong / partial" |
| `m1nd.drift` | Weight drift analysis -- "what changed since last session?" |
| `m1nd.warmup` | Context priming -- "prepare for task X" |

### System

| Tool | Purpose |
|------|---------|
| `m1nd.ingest` | Load data into the graph (code extractor or JSON descriptor) |
| `m1nd.health` | Diagnostics -- node/edge counts, sessions, persistence status |

---

## Quick Start

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/m1nd-mcp
```

m1nd starts as a JSON-RPC stdio server. Send MCP messages to stdin, receive responses on stdout.

### First Session

```jsonc
// 1. Handshake
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}

// 2. Ingest a codebase
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"m1nd.ingest","arguments":{"path":"/your/project","agent_id":"my-agent"}}}

// 3. Query
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"m1nd.activate","arguments":{"query":"authentication flow","agent_id":"my-agent"}}}

// 4. Learn from results
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"m1nd.learn","arguments":{"query":"authentication flow","agent_id":"my-agent","feedback":"correct","node_ids":["file::src/auth.rs","file::src/session.rs"]}}}
```

State persists automatically. On restart, m1nd loads the saved graph and plasticity state.

---

## Domain Support

m1nd is domain-agnostic. Code intelligence is one adapter -- the engine works on any property graph.

### Code (default)

```jsonc
{"name":"m1nd.ingest","arguments":{"path":"/repo","agent_id":"a","adapter":"code"}}
```

Extracts structure from 6 languages: Rust, Python, TypeScript/JavaScript, Go, Java, and a generic fallback. Detects functions, classes, structs, enums, traits, imports, calls, type references, decorators, and more. Populates co-change data from git history.

### JSON Descriptor (any domain)

```jsonc
{"name":"m1nd.ingest","arguments":{"path":"graph.json","agent_id":"a","adapter":"json"}}
```

Ingest any graph from a JSON file:

```json
{
  "nodes": [
    { "id": "concept::activation", "label": "Spreading Activation", "type": "Concept", "tags": ["core"] },
    { "id": "concept::plasticity", "label": "Hebbian Plasticity", "type": "Process", "tags": ["learning"] }
  ],
  "edges": [
    { "source": "concept::activation", "target": "concept::plasticity", "relation": "enables", "weight": 0.8 }
  ]
}
```

Supported node types: `File`, `Directory`, `Function`, `Class`, `Struct`, `Enum`, `Type`, `Module`, `Reference`, `Concept`, `Material`, `Process`, `Product`, `Supplier`, `Regulatory`, `System`, `Cost`, `Custom`.

### Domain Configuration

m1nd ships with decay rate presets for different domains:

| Domain | Node decay example | Git co-change |
|--------|--------------------|---------------|
| `code` | Functions: 14d, Modules: 30d | Yes |
| `music` | Plugins: 14d, Buses: 30d | No |
| `generic` | All: 14d default | No |

---

## Use Cases

### LLM Agent Memory

Replace flat-file memory with a semantic graph that activates, learns, and detects gaps.

```
Session start:  m1nd.drift()     → "what changed since yesterday?"
During work:    m1nd.activate()  → relevant context by spreading activation
After decision: m1nd.learn()     → strengthen connections that worked
End of session: auto-persist     → next session starts where this one left off
```

### Build Orchestration

An AI build orchestrator uses m1nd as its brain to decide which agent to spawn, what context to provide, and what impact changes will have.

```
Agent completes module X →
  m1nd.ingest (re-ingest) →
  m1nd.learn ("correct", touched modules) →
  m1nd.predict (what else needs changes?) →
  m1nd.warmup (prime next agent's context) →
  m1nd.impact (did the change escape scope?) →
  m1nd.missing (new structural holes?)
```

### Code Intelligence

Standard code analysis with spreading activation instead of keyword search.

- **Blast radius**: `m1nd.impact("auth.rs")` -- what's affected?
- **Co-change**: `m1nd.predict("auth.rs")` -- what else will change?
- **Keystones**: `m1nd.counterfactual(["auth.rs"])` -- what breaks without this?
- **Gaps**: `m1nd.missing("authentication")` -- what's missing?
- **Explanation**: `m1nd.why("auth.rs", "session.rs")` -- how are these connected?

### Research and Knowledge Management

Ingest notes, papers, and concepts as a JSON graph. Use activation to find connections, missing to find gaps, and learn to refine the knowledge structure over time.

### Any Domain

If your problem has entities and relationships, m1nd can model it. The spreading activation engine, Hebbian plasticity, resonance analysis, and counterfactual simulation work on any graph topology.

---

## Architecture

```
m1nd-core/          Engine: graph, activation, plasticity, resonance, temporal, counterfactual
m1nd-ingest/        Ingestion: code extractors (6 languages), JSON adapter, reference resolver
m1nd-mcp/           Server: JSON-RPC stdio, MCP protocol, 13 tool handlers, session management
```

### m1nd-core (~10,400 LOC)

The domain-agnostic engine. CSR graph with atomic weights, 4-dimensional spreading activation, XLR noise cancellation, Hebbian plasticity with LTP/LTD, standing wave resonance, Louvain community detection, counterfactual simulation, temporal decay with per-type half-lives.

### m1nd-ingest (~2,800 LOC)

Data ingestion layer. Code extractors for Rust, Python, TypeScript, Go, Java with comment/string stripping, brace expansion, decorator detection, enum variant extraction, and method call tracking. JSON adapter for arbitrary domains. Cross-file reference resolution with proximity disambiguation. Rayon-parallelized extraction.

### m1nd-mcp (~2,500 LOC)

MCP server. JSON-RPC stdio transport, full MCP 2024-11-05 compliance with inputSchema for all tools, agent session tracking, auto-persistence, configurable domain presets.

---

## Metrics

| Metric | Value |
|--------|-------|
| Language | Rust |
| Lines of code | ~15,500 |
| Source files | 32 |
| Tests | 159 |
| MCP tools | 13 |
| Supported languages | 6 + generic fallback |
| Binary size (ARM64 release) | 3.8 MB |
| Self-ingest | 693 nodes, 2007 edges |
| Persistence | Round-trip verified |

---

## Documentation

- **[EXAMPLES.md](EXAMPLES.md)** -- Real CLI output from m1nd running against its own codebase
- **[INTEGRATION-GUIDE.md](INTEGRATION-GUIDE.md)** -- Complete tool reference, usage patterns, domain configuration, best practices
- **[FINAL-REPORT.md](FINAL-REPORT.md)** -- Architecture deep dive, build methodology, validation results

---

## License

MIT

---

<p align="center">
  <strong>MAX ELIAS KLEINSCHMIDT -- COSMOPHONIX INTELLIGENCE</strong>
</p>
