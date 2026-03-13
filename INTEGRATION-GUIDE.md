# m1nd Integration Guide

**Version**: 0.1.0 | **Protocol**: MCP 2024-11-05 | **Date**: 2026-03-12

Manual for connecting ANY agent to m1nd as an MCP server. Covers connection, tool usage patterns, domain configuration, and best practices.

---

## 1. Connection

m1nd runs as a JSON-RPC stdio MCP server. One process, one graph, multiple agents.

### Binary

```
target/release/m1nd-mcp
```

### Configuration

Three options, in order of precedence:

**1. Config file (CLI argument)**
```bash
./m1nd-mcp /path/to/config.json
```

```json
{
  "graph_source": "./graph_snapshot.json",
  "plasticity_state": "./plasticity_state.json",
  "auto_persist_interval": 50,
  "learning_rate": 0.08,
  "decay_rate": 0.005,
  "xlr_enabled": true,
  "max_concurrent_reads": 32,
  "write_queue_size": 64,
  "domain": "code"
}
```

**2. Environment variables**
```bash
M1ND_GRAPH_SOURCE=./graph_snapshot.json
M1ND_PLASTICITY_STATE=./plasticity_state.json
M1ND_XLR_ENABLED=true
```

**3. Defaults** (graph_snapshot.json in working directory, XLR on, code domain)

### Domain Selection

Set `"domain"` in config or omit for default:

| Domain | Use Case | Git Co-change | Relation Types |
|--------|----------|---------------|----------------|
| `"code"` (default) | Source code analysis | Yes | contains, imports, calls, references, implements |
| `"music"` | DAW / audio production | No | routes_to, sends_to, controls, modulates, contains, monitors |
| `"generic"` | Any domain | No | contains, references, depends_on, produces, consumes |

### MCP Handshake

Every connecting agent must complete the MCP handshake before calling tools.

**Step 1: Initialize**
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "serverInfo": { "name": "m1nd-mcp", "version": "0.1.0" },
    "capabilities": { "tools": {} }
  }
}
```

**Step 2: Acknowledge**
```json
{"jsonrpc":"2.0","id":2,"method":"notifications/initialized","params":{}}
```

**Step 3 (optional): List tools**
```json
{"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}
```

Returns all 13 tool schemas with full inputSchema definitions.

### Calling Tools

All tool calls use the `tools/call` method:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "query": "spreading activation",
      "agent_id": "jimi-001"
    }
  }
}
```

Every tool requires `agent_id`. This is how m1nd tracks which agent is doing what.

### Response Format

**Success:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [{
      "type": "text",
      "text": "{ ... pretty-printed JSON result ... }"
    }]
  }
}
```

**Tool error (bad input, no results, etc.):**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [{
      "type": "text",
      "text": "Error: Node not found: xyz"
    }],
    "isError": true
  }
}
```

**Protocol error (parse failure, unknown method):**
```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": { "code": -32700, "message": "Parse error: ..." }
}
```

Error codes: `-32700` (parse error), `-32601` (method not found), `-32603` (internal error).

---

## 2. Quick Start

Minimal sequence to get useful results from a code repository:

```
1. initialize
2. notifications/initialized
3. m1nd.ingest   → load the codebase into the graph
4. m1nd.activate → query it
5. m1nd.learn    → tell m1nd which results were useful
```

### Example: Analyze a Rust Project

```json
// Step 3: Ingest
{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{
  "name":"m1nd.ingest",
  "arguments":{"path":"/home/user/my-project","agent_id":"my-agent"}
}}

// Step 4: Query
{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{
  "name":"m1nd.activate",
  "arguments":{"query":"error handling","agent_id":"my-agent"}
}}

// Step 5: Feedback
{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{
  "name":"m1nd.learn",
  "arguments":{
    "query":"error handling",
    "agent_id":"my-agent",
    "feedback":"correct",
    "node_ids":["src/error.rs","handle_error"]
  }
}}
```

After Step 3, the graph is persisted to disk. On next startup, m1nd loads it automatically -- no re-ingest needed unless the source changes.

---

## 3. Tool Reference

### Discovery Tools -- Answering Questions

| Tool | Use When | Key Arguments |
|------|----------|---------------|
| `activate` | "What's related to X?" | `query`, `top_k` (default 20), `dimensions`, `xlr` |
| `why` | "How are A and B connected?" | `source`, `target`, `max_hops` (default 6) |
| `missing` | "What am I overlooking?" | `query`, `min_sibling_activation` (default 0.3) |
| `fingerprint` | "Is this a duplicate?" | `target_node`, `similarity_threshold` (default 0.85) |

### Change Analysis Tools -- Planning Changes

| Tool | Use When | Key Arguments |
|------|----------|---------------|
| `impact` | "What does changing X affect?" | `node_id`, `direction` (forward/reverse/both) |
| `predict` | "What else will change?" | `changed_node`, `top_k` (default 10), `include_velocity` |
| `counterfactual` | "What if we remove X?" | `node_ids` (array), `include_cascade` |

### Learning Tools -- Improving Over Time

| Tool | Use When | Key Arguments |
|------|----------|---------------|
| `learn` | "This result was right/wrong" | `query`, `feedback` (correct/wrong/partial), `node_ids`, `strength` (default 0.2) |
| `drift` | "What changed since last time?" | `since` (default "last_session"), `include_weight_drift` |
| `warmup` | "Prepare for task X" | `task_description`, `boost_strength` (default 0.15) |

### System Tools

| Tool | Use When | Key Arguments |
|------|----------|---------------|
| `ingest` | Loading new data | `path`, `adapter` (code/json), `incremental` |
| `resonate` | Deep structural analysis | `query` or `node_id` (one required), `top_k` (default 20) |
| `health` | Diagnostics | (none beyond `agent_id`) |

---

## 4. Tool Parameter Reference

Every tool requires `agent_id` (string). All other parameters listed below.

### m1nd.activate

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | -- | Search query for spreading activation |
| `top_k` | integer | no | 20 | Number of top results |
| `dimensions` | string[] | no | all four | Which dimensions: "structural", "semantic", "temporal", "causal" |
| `xlr` | boolean | no | true | XLR noise cancellation |
| `include_ghost_edges` | boolean | no | true | Detect edges that should exist but don't |
| `include_structural_holes` | boolean | no | false | Detect structural holes in results |

### m1nd.impact

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `node_id` | string | yes | -- | Node to analyze |
| `direction` | string | no | "forward" | "forward", "reverse", or "both" |
| `include_causal_chains` | boolean | no | true | Include causal chain detection |

### m1nd.missing

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | -- | Search query to find structural holes around |
| `min_sibling_activation` | number | no | 0.3 | Minimum sibling activation threshold |

### m1nd.why

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | string | yes | -- | Source node ID |
| `target` | string | yes | -- | Target node ID |
| `max_hops` | integer | no | 6 | Maximum hops in path search |

### m1nd.warmup

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `task_description` | string | yes | -- | Description of the task to prime for |
| `boost_strength` | number | no | 0.15 | Priming boost strength |

### m1nd.counterfactual

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `node_ids` | string[] | yes | -- | Nodes to simulate removal of |
| `include_cascade` | boolean | no | true | Include cascade analysis |

### m1nd.predict

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `changed_node` | string | yes | -- | Node that was changed |
| `top_k` | integer | no | 10 | Number of top predictions |
| `include_velocity` | boolean | no | true | Include velocity scoring |

### m1nd.fingerprint

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `target_node` | string | no | -- | Node to find equivalents for |
| `similarity_threshold` | number | no | 0.85 | Cosine similarity threshold |
| `probe_queries` | string[] | no | -- | Probe queries for fingerprinting |

### m1nd.drift

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `since` | string | no | "last_session" | Baseline reference point |
| `include_weight_drift` | boolean | no | true | Include edge weight drift |

### m1nd.learn

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | yes | -- | Original query this feedback relates to |
| `feedback` | string | yes | -- | "correct", "wrong", or "partial" |
| `node_ids` | string[] | yes | -- | Nodes to apply feedback to |
| `strength` | number | no | 0.2 | Feedback strength |

### m1nd.ingest

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `path` | string | yes | -- | Filesystem path to source |
| `adapter` | string | no | "code" | "code" or "json" |
| `incremental` | boolean | no | false | Incremental mode (code adapter only) |

### m1nd.resonate

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | no* | -- | Search query for seed nodes |
| `node_id` | string | no* | -- | Specific node as seed |
| `top_k` | integer | no | 20 | Number of top results |

*One of `query` or `node_id` must be provided.

### m1nd.health

No additional parameters. Only `agent_id` is required.

Returns: node/edge counts, queries processed, uptime, memory usage, persistence status, active sessions.

---

## 5. Usage Patterns

### Pattern: Research Session

Agent explores a topic, finds connections, identifies gaps, provides feedback.

```
ingest(path) ->
activate(topic) ->
why(concept_a, concept_b) ->
missing(topic) ->
learn(feedback)
```

When to use: An agent is tasked with understanding a codebase or domain area before making decisions.

### Pattern: Code Change Planning

Agent plans changes, checks blast radius, predicts co-changes, simulates removals.

```
ingest(path) ->
impact(file_to_change) ->
predict(file_to_change) ->
counterfactual(modules_to_rewrite) ->
warmup(task_description)
```

When to use: Before any non-trivial code change. The output of `impact` and `predict` should inform the scope of the change.

### Pattern: Orchestrator Build Loop

The core orchestrator loop that runs after each agent completes a module during a layered parallel build.

```
Per completed module:
  1. ingest(incremental=true)            -- update the graph
  2. learn("correct", touched_modules)   -- plasticity records co-changes
  3. predict(changed_module)             -- alert next agent of likely co-changes
  4. warmup(next_task)                   -- prime context for next agent
  5. impact(module)                      -- check if change escaped scope
  6. missing()                           -- new structural holes? spawn fix agent
```

When to use: During the layered build phase. This loop makes the orchestrator progressively smarter as the build proceeds -- each completed module feeds plasticity data back into the graph.

### Pattern: Memory Enhancement

Agent uses m1nd as persistent, evolving memory across sessions.

```
Session start:
  drift(since="last_session")     -- what changed?

During work:
  activate(current_topic)         -- relevant context from memory
  warmup(current_task)            -- bias toward current focus

After decisions:
  learn("correct", key_nodes)     -- strengthen useful connections

Session end:
  (auto-persisted)                -- state survives restart
```

When to use: Any long-lived agent (the orchestrator agent, build coordinators) that benefits from remembering what worked across sessions.

### Pattern: Reconnaissance (Build Phase 1)

Before writing a spec for changes to an existing codebase.

```
ingest(existing_codebase) ->
impact(planned_change_targets)    -- blast radius informs spec scope
missing(feature_area)             -- structural holes the feature must bridge
why(module_a, module_b)           -- invisible dependency chains to preserve
```

### Pattern: Hardening Intelligence (Build Phase 2)

Feed intelligence to hardening swarm agents.

```
counterfactual(modules_being_rewritten) -- load-bearing code identification
predict(historically_volatile_file)     -- "when auth.rs changes, what else changes?"
fingerprint()                           -- duplicate module detection for spec consolidation
```

### Pattern: Integration Audit (Build Phase 5)

Validate the complete system after all build layers are done.

```
activate(real_queries)             -- does the system find what it should?
counterfactual(critical_modules)   -- single points of failure?
missing()                          -- structural holes in the complete graph?
fingerprint()                      -- suspiciously identical modules = copy-paste bugs
```

### Pattern: Creative / Music Production

Using the JSON adapter with music domain.

```
ingest(adapter="json", path="project.json") ->
impact("master_bus")                          -- what's affected by master changes?
counterfactual(["reverb_plugin"])              -- what if we remove this?
missing(query="signal flow")                   -- routing gaps?
resonate(query="frequency response")           -- harmonic analysis
```

Requires `"domain": "music"` in server config for appropriate temporal decay rates and relation types.

---

## 6. Domain Configuration

### Code Domain (default)

Used for source code analysis. Supports Rust, Python, TypeScript, Go, Java, and a generic fallback for other languages.

- **Adapter**: `"code"`
- **Node types**: File, Directory, Function, Class, Struct, Enum, Module, Type, Reference
- **Relations**: contains, imports, calls, references, implements
- **Temporal**: Git-based co-change detection, per-type decay rates (Files: 7 days, Functions: 14 days, Classes/Structs: 21 days, Modules: 30 days)

### JSON Descriptor (any domain)

For non-code domains or hand-crafted graphs. Point m1nd at a JSON file that describes nodes and edges explicitly.

- **Adapter**: `"json"`
- **Node types**: Anything from the supported set
- **Relations**: Any string
- **Temporal**: Timestamp-based if provided

**JSON Descriptor Format:**

```json
{
  "nodes": [
    {
      "id": "master_bus",
      "label": "Master Bus",
      "type": "System",
      "tags": ["audio", "routing"]
    },
    {
      "id": "eq_plugin",
      "label": "Parametric EQ",
      "type": "Process",
      "tags": ["plugin", "eq"]
    }
  ],
  "edges": [
    {
      "source": "eq_plugin",
      "target": "master_bus",
      "relation": "routes_to",
      "weight": 1.0
    }
  ]
}
```

**Supported node types:** File, Directory, Function, Class, Struct, Enum, Type, Module, Reference, Concept, Material, Process, Product, Supplier, Regulatory, System, Cost, Custom.

### Music Domain

Server config: `"domain": "music"`

Temporal decay rates tuned for audio production workflows:
- System nodes (rooms, buses): 30-day half-life
- Process nodes (plugins, effects): 14-day half-life
- Material nodes (audio signals): 7-day half-life
- Concept nodes (presets, templates): 21-day half-life

Relations: routes_to, sends_to, controls, modulates, contains, monitors.

### Generic Domain

Server config: `"domain": "generic"`

No assumptions about the data. Flat 14-day half-life for all node types. Relations: contains, references, depends_on, produces, consumes.

---

## 7. How Activation Works

Understanding activation is essential for using m1nd effectively. This is not keyword search.

### Four Dimensions

Every query propagates signal across four independent dimensions, then merges:

| Dimension | What It Captures | Source |
|-----------|------------------|--------|
| **Structural** | Graph topology (edges, hops, PageRank) | Static code structure |
| **Semantic** | Label similarity (n-grams, co-occurrence) | Node labels and tags |
| **Temporal** | Co-change patterns, velocity, decay | Git history, timestamps |
| **Causal** | Cause-effect chains | Call graphs, dependency direction |

You can filter dimensions with the `dimensions` parameter on `activate`:

```json
{"dimensions": ["structural", "semantic"]}
```

This is useful when you want structural analysis without temporal noise, or purely semantic similarity.

### XLR Noise Cancellation

XLR runs a differential signal/noise analysis on activation results. It suppresses nodes that activate strongly but non-specifically (high PageRank hub nodes that connect to everything). Enabled by default. Disable with `"xlr": false` if you want raw, unfiltered activation.

### Plasticity

m1nd learns from usage. When you call `learn` with "correct" feedback, edges between the referenced nodes strengthen (LTP -- long-term potentiation). When you call it with "wrong", they weaken (LTD -- long-term depression). "partial" applies a mild strengthening.

This means the same `activate` query will return better results over time if agents consistently provide feedback. The plasticity state persists across restarts.

### Ghost Edges

`activate` can detect "ghost edges" -- connections that should exist based on the activation pattern but don't exist in the graph. These are essentially implicit relationships. Useful for discovering non-obvious connections.

---

## 8. Best Practices

### Always learn

Every time an agent uses `activate` results to make a decision, it should call `learn()` with feedback. This is how m1nd improves. Hebbian plasticity requires signal -- a graph that never receives feedback has static weights.

```json
// Agent used activation results and they were helpful
{"name":"m1nd.learn","arguments":{
  "query":"error handling",
  "agent_id":"jimi-001",
  "feedback":"correct",
  "node_ids":["src/error.rs","handle_error","M1ndError"]
}}
```

### Use warmup before complex tasks

`warmup` primes the graph -- subsequent `activate` queries are contextually biased toward the task. Call it once at the start of a focused work session.

```json
{"name":"m1nd.warmup","arguments":{
  "task_description":"implement OAuth2 authentication flow",
  "agent_id":"build-agent-3"
}}
```

### Check drift at session start

`drift` shows what changed since the last session. This gives the agent immediate context on what evolved while it was offline.

### Use missing regularly

Structural holes are the most valuable signal m1nd provides. They reveal what you don't know you're missing -- modules that should be connected but aren't, test files that should exist but don't, abstractions that are implied but never created.

### Let persistence work

m1nd auto-persists every 50 queries (configurable) and on shutdown. It loads the saved graph on restart. Don't re-ingest unless the source data actually changed.

### Use predict before committing changes

After modifying a module, `predict` tells you what else is likely to need changes based on co-change history. This catches the "forgot to update the tests" and "broke the downstream consumer" problems.

### Use counterfactual before removing code

Before deleting or rewriting a module, `counterfactual` tells you what activation loss would result. Keystones (single points of failure) will show extreme cascade effects -- these need careful handling.

### Use consistent agent_id values

m1nd tracks sessions per agent_id. Use stable, meaningful identifiers: `"jimi-001"`, `"stormender-orchestrator"`, `"build-agent-L2-auth"`. This enables drift tracking and session analytics.

---

## 9. Architecture Notes for Agent Developers

### Statefulness

m1nd is stateful. The property graph lives in memory, backed by periodic persistence to JSON files on disk. This is not a stateless API -- the order of operations matters. Ingest before activate. Learn after activate. Warmup before a focused session.

### Multi-Agent Access

Multiple agents share one m1nd instance. All writes (`learn`, `ingest`) are visible to all agents immediately. Read operations (`activate`, `impact`, etc.) are concurrent-safe via `RwLock`. The graph is the shared source of truth.

### Persistence

Two files are persisted:
- `graph_snapshot.json` -- the full property graph (nodes, edges, weights, metadata)
- `plasticity_state.json` -- synaptic state (which edges strengthened/weakened and by how much)

Graph is saved first (source of truth). Plasticity is saved second. If plasticity save fails after graph succeeds, the server logs a warning but continues.

On startup, the server loads the graph snapshot, finalizes it (computes PageRank, builds CSR), then imports plasticity state. If either file is missing or corrupt, it starts fresh.

### Engine Rebuild After Ingest

After a full `ingest`, all engines (semantic indexes, temporal engine, plasticity engine) are rebuilt from the new graph. This is automatic -- no agent action needed. But it means a full ingest is more expensive than incremental.

### Auto-Persist Trigger

State is persisted every N queries (default 50, configurable via `auto_persist_interval`). Also persisted automatically after every full ingest and on graceful shutdown (SIGINT).

### Performance

The self-ingest benchmark (m1nd ingesting its own 32-file, ~15,500 LOC codebase):
- 693 nodes, 2007 edges
- All 13 tools respond in under 100ms
- No NaN, no panics, no JSON parse errors across 15+ sequential RPC messages

---

## 10. Shutdown

m1nd handles SIGINT gracefully:
1. Persists graph and plasticity state to disk
2. Flushes all writes
3. Exits cleanly

Agents should close their stdin pipe to the m1nd process when done. The server exits on EOF.

---

## Appendix A: Supported Languages (Code Adapter)

| Language | Extracted Symbols |
|----------|-------------------|
| Rust | structs, enums, fns, impls, traits, use-paths |
| Python | classes, defs, decorators, imports, type hints |
| TypeScript / JavaScript | classes, functions, imports, exports |
| Go | structs, funcs, interfaces |
| Java | classes, methods, interfaces |
| Generic fallback | Regex-based extraction for unknown languages |

Extractors are regex-based (not tree-sitter AST). They work well enough for structural graph construction. The extractor interface is language-polymorphic -- tree-sitter can be swapped in without changing the tool API.

## Appendix B: Flywheel Effect

m1nd gets smarter as it's used. Understanding this is key to getting maximum value.

| Phase | What m1nd Knows |
|-------|-----------------|
| After `ingest` | Raw graph structure: who imports whom, what contains what |
| After first `activate` + `learn` cycle | Which connections matter for specific queries |
| After a layered build (Phase 4) | Which modules actually change together (real co-change data from the build itself) |
| After calibration (Phase 6) | Human-refined weights from "that doesn't look right" feedback |
| After multiple sessions | Accumulated plasticity from all agents across all sessions |

Each build feeds m1nd. m1nd guides the next build better. The next build feeds m1nd more. This is the flywheel.
