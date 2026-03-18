# Planned Issues — m1nd

Issues staged for filing on GitHub. These represent the v0.3.0 and v0.4.0 roadmap.

---

## 1. feat: Embedding-based semantic similarity (fastembed)

**Label**: enhancement, v0.3.0

Replace keyword matching in `activate`/`seek` with vector embeddings for true semantic search.
`fastembed` 5.12.1 is verified on crates.io and supports ONNX inference locally with no API calls.
Currently deferred from v0.2.0 due to ONNX build complexity on cross-compiled targets.

The semantic dimension in the 4D activation score currently uses trigram TF-IDF. This works for
identifier overlap but fails on "find code that *means* authentication but never uses the word."
Embedding-based similarity would close this gap while preserving zero-API-call semantics.

---

## 2. feat: ast-grep pattern integration for scan tool

**Label**: enhancement, v0.3.0

`ast-grep-core` 0.41.1 is available on crates.io and provides structural pattern matching via YAML
rule files. The current `scan` tool in `m1nd-mcp` uses heuristic regex patterns against node
identifiers, which produces false positives on minified code and misses structural patterns.

Integrating `ast-grep-core` would allow rules like "find all functions that call X without
checking the return value" — structural queries the regex approach cannot express.

---

## 3. feat: Desktop tray application

**Label**: enhancement, v0.4.0

A system tray app (Tauri or similar) that shows:
- Graph health (node/edge counts, last ingest time)
- Active agents + lock holders (from `lock.*` tools)
- Quick-fire common queries without opening a terminal
- Plasticity drift indicator (when the graph has diverged from disk state)

Useful for teams where multiple agents share the same m1nd instance. The tray app would connect
to the existing JSON-RPC stdio server via a local socket bridge.

---

## 4. perf: flow_simulate optimization for dense graphs

**Label**: performance

BFS particle propagation in `flow_simulate` (used internally by `activate` for the causal
dimension) can be slow on graphs with >50K edges. In the current implementation, each particle
is tracked individually through the BFS queue, leading to O(particles × edges) work.

Observed: on a 50K-edge test graph, `activate` latency jumps from ~31ms to ~340ms when
`infection_rate` is set above 0.4. The epidemic model saturates too quickly on dense subgraphs
even at default settings (see related bug #9).

Candidates: parallel particle simulation via Rayon, bitset-based batch propagation similar to
the `RemovalMask` approach used in counterfactuals.

---

## 5. docs: Add video tutorial for first-time setup

**Label**: documentation

The README has solid text instructions and the wiki has detailed tool reference, but a video
walkthrough would lower the friction for first-time users who want to see ingest → activate →
learn in a real codebase before committing to building from source.

Suggested content:
1. Clone + `cargo build --release` (real terminal, real timing)
2. Add to Claude Code config
3. First `ingest` call — show what the output means
4. `activate` → `why` → `missing` sequence
5. `learn` feedback — explain Hebbian plasticity visually

---

## 6. feat: WebSocket transport for real-time graph updates

**Label**: enhancement

Currently the `m1nd-mcp` server is JSON-RPC over stdio only. A WebSocket transport would enable:
- Bidirectional communication for a future GUI (m1nd-ui)
- Real-time push of plasticity weight changes as agents learn
- Browser-based graph visualization without a Tauri wrapper

The stdio transport would remain the primary MCP path; WebSocket would be additive for
GUI/dashboard use cases.

---

## 7. test: Increase test coverage for perspective navigation

**Label**: testing

`perspective.suggest` and `perspective.affinity` have minimal test coverage in the current
test suite. `test_perspective_usecases.py` covers the happy path for `start`/`follow`/`back`
but misses:
- `suggest` with no open perspective session (should return graceful error)
- `affinity` on disconnected graph components
- `branch` + `compare` round-trip with conflicting node weights
- Concurrent perspective sessions from two agents on the same graph

These cases have produced silent wrong results in past manual testing.

---

## 8. feat: Git blame integration for temporal dimension

**Label**: enhancement

The temporal dimension currently uses co-change frequency and Hebbian feedback to weight recency.
Git blame data would add actual authorship timestamps per line, allowing the temporal score to
reflect real modification recency rather than just session-level feedback.

Implementation path: `m1nd-ingest` already runs `git log --follow` for co-change history.
Extending it to run `git blame --porcelain` per file and map blame timestamps to node weights
would give the temporal dimension a ground-truth signal independent of agent feedback.

---

## 9. bug: epidemic model saturates too quickly on dense subgraphs

**Label**: bug

The BFS epidemic model used in `activate`'s causal dimension can reach 97% infection coverage
in 3 iterations on dense subgraphs, even with `infection_rate=0.1` (the default).

Steps to reproduce:
1. Ingest a codebase with a highly-connected hub module (e.g., a shared `utils.py` with 200+
   import edges)
2. Call `activate(query="utils", agent_id=...)` with default parameters
3. Observe: causal scores are uniformly high across nearly all nodes — signal saturates

The v0.2.0 release partially mitigated this with early-stopping when coverage exceeds 80%, but
edge cases remain when the hub is the query target itself (self-loop in the epidemic start node).

Expected: causal scores should discriminate. Actual: they flatten to ~0.95 across the graph.

---

## 10. feat: Export graph to Neo4j/Cypher format

**Label**: enhancement

Currently `m1nd` supports Mermaid export (used for the architecture diagram in README). Adding
Cypher export for Neo4j would let teams:
- Visualize the full graph in Neo4j Browser or Bloom
- Run Cypher queries alongside m1nd's activation queries
- Integrate with existing graph databases in data engineering stacks

The export would serialize the CSR graph to `.cypher` CREATE statements. Node properties would
include the four dimension base scores; relationship properties would include the current
plasticity weight.
