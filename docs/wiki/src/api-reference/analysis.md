# Analysis Tools

Seven tools for impact analysis, prediction, counterfactual simulation, fingerprinting, hypothesis testing, structural diffing, and drift detection.

---

## `m1nd.impact`

Impact radius / blast analysis for a node. Propagates signal outward from a source node to determine which other nodes would be affected by a change. Supports forward (downstream), reverse (upstream), and bidirectional analysis.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `node_id` | `string` | Yes | -- | Target node identifier. Can be an external_id (`file::backend/config.py`) or a node label. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `direction` | `string` | No | `"forward"` | Propagation direction. Values: `"forward"` (what does this affect?), `"reverse"` (what affects this?), `"both"`. |
| `include_causal_chains` | `boolean` | No | `true` | Include causal chain detection. Shows the specific paths through which impact propagates. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "m1nd.impact",
    "arguments": {
      "agent_id": "jimi",
      "node_id": "file::chat_handler.py",
      "direction": "forward",
      "include_causal_chains": true
    }
  }
}
```

### Example Response

```json
{
  "source": "file::chat_handler.py",
  "source_label": "chat_handler.py",
  "direction": "forward",
  "blast_radius": [
    { "node_id": "file::chat_routes.py", "label": "chat_routes.py", "type": "file", "signal_strength": 0.91, "hop_distance": 1 },
    { "node_id": "file::ws_relay.py", "label": "ws_relay.py", "type": "file", "signal_strength": 0.78, "hop_distance": 1 },
    { "node_id": "file::stream_parser.py", "label": "stream_parser.py", "type": "file", "signal_strength": 0.65, "hop_distance": 2 }
  ],
  "total_energy": 4271.0,
  "max_hops_reached": 3,
  "causal_chains": [
    {
      "path": ["chat_handler.py", "chat_routes.py", "main.py"],
      "relations": ["imported_by", "registered_in"],
      "cumulative_strength": 0.82
    }
  ]
}
```

### When to Use

- **Before modifying code** -- understand the blast radius before touching a file
- **Risk assessment** -- high total_energy = high-risk change
- **Scope validation** -- verify that a planned change does not leak beyond expected boundaries
- **Reverse analysis** -- find all upstream dependencies that could cause a bug in a given module

### Related Tools

- [`m1nd.predict`](#m1ndpredict) -- predicts which files will co-change (more actionable than blast radius)
- [`m1nd.counterfactual`](#m1ndcounterfactual) -- simulates deletion rather than change
- [`m1nd.validate_plan`](lifecycle.md#m1ndvalidate_plan) -- validates an entire modification plan

---

## `m1nd.predict`

Co-change prediction for a modified node. Given a node that was just changed, predicts which other nodes are most likely to need changes too, ranked by confidence. Uses historical co-change patterns, structural proximity, and velocity scoring.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `changed_node` | `string` | Yes | -- | Node identifier that was changed. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `top_k` | `integer` | No | `10` | Number of top predictions to return. |
| `include_velocity` | `boolean` | No | `true` | Include velocity scoring. Velocity considers how recently and frequently nodes co-changed. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "m1nd.predict",
    "arguments": {
      "agent_id": "jimi",
      "changed_node": "file::session_pool.py",
      "top_k": 5
    }
  }
}
```

### Example Response

```json
{
  "changed_node": "file::session_pool.py",
  "predictions": [
    { "node_id": "file::worker_pool.py", "label": "worker_pool.py", "confidence": 0.89, "velocity": 0.72, "reason": "high co-change frequency + structural coupling" },
    { "node_id": "file::process_manager.py", "label": "process_manager.py", "confidence": 0.76, "velocity": 0.65, "reason": "imports session_pool" },
    { "node_id": "file::tests/test_session_pool.py", "label": "test_session_pool.py", "confidence": 0.71, "velocity": 0.80, "reason": "test file" },
    { "node_id": "file::spawner.py", "label": "spawner.py", "confidence": 0.54, "velocity": 0.41, "reason": "2-hop dependency" },
    { "node_id": "file::config.py", "label": "config.py", "confidence": 0.32, "velocity": 0.28, "reason": "shared configuration" }
  ],
  "elapsed_ms": 8.3
}
```

### When to Use

- **After modifying a module** -- find what else needs updating
- **Before committing** -- verify you have not missed a co-change partner
- **Code review** -- check if a PR is missing changes to coupled modules

### Related Tools

- [`m1nd.impact`](#m1ndimpact) -- blast radius (broader, less actionable)
- [`m1nd.timeline`](exploration.md#m1ndtimeline) -- detailed co-change history
- [`m1nd.validate_plan`](lifecycle.md#m1ndvalidate_plan) -- validates an entire plan against co-change predictions

---

## `m1nd.counterfactual`

What-if node removal simulation. Simulates removing one or more nodes from the graph and reports the cascade: orphaned nodes, lost activation energy, and the resulting blast radius. Non-destructive -- the graph is not modified.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `node_ids` | `string[]` | Yes | -- | Node identifiers to simulate removal of. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `include_cascade` | `boolean` | No | `true` | Include cascade analysis. Shows multi-hop propagation of the removal. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "m1nd.counterfactual",
    "arguments": {
      "agent_id": "jimi",
      "node_ids": ["file::spawner.py"],
      "include_cascade": true
    }
  }
}
```

### Example Response

```json
{
  "removed_nodes": ["file::spawner.py"],
  "cascade": [
    { "depth": 1, "affected": 23 },
    { "depth": 2, "affected": 456 },
    { "depth": 3, "affected": 3710 }
  ],
  "total_affected": 4189,
  "orphaned_count": 0,
  "pct_activation_lost": 0.41,
  "elapsed_ms": 3.1
}
```

### When to Use

- **Before deleting/rewriting modules** -- understand the full cascade before removing code
- **Dependency audit** -- find modules whose removal would be catastrophic
- **Architecture planning** -- evaluate the cost of removing a subsystem

### Related Tools

- [`m1nd.impact`](#m1ndimpact) -- change impact (less extreme than removal)
- [`m1nd.hypothesize`](#m1ndhypothesize) -- test a structural claim about dependencies

---

## `m1nd.fingerprint`

Activation fingerprint and equivalence detection. Computes a structural fingerprint for a target node (or the entire graph) and finds functionally equivalent or duplicate nodes. Uses probe queries to generate activation patterns and compares them via cosine similarity.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `target_node` | `string` | No | -- | Target node to find equivalents for. If omitted, performs global fingerprinting. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `similarity_threshold` | `number` | No | `0.85` | Cosine similarity threshold for equivalence. Range: 0.0 to 1.0. Lower values find more (but weaker) matches. |
| `probe_queries` | `string[]` | No | -- | Optional probe queries for fingerprinting. Auto-generated from the node's neighborhood if omitted. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "m1nd.fingerprint",
    "arguments": {
      "agent_id": "jimi",
      "target_node": "file::session_pool.py",
      "similarity_threshold": 0.7
    }
  }
}
```

### Example Response

```json
{
  "target": "file::session_pool.py",
  "fingerprint": [0.92, 0.45, 0.78, 0.33, 0.67],
  "equivalents": [
    { "node_id": "file::worker_pool.py", "label": "worker_pool.py", "similarity": 0.88, "reason": "similar pool lifecycle pattern" },
    { "node_id": "file::fast_pool.py", "label": "fast_pool.py", "similarity": 0.74, "reason": "shared structural role" }
  ],
  "elapsed_ms": 55.2
}
```

### When to Use

- **Duplicate detection** -- find code doing the same thing in different places
- **Consolidation audit** -- identify candidates for unification
- **Post-build review** -- verify new code does not duplicate existing functionality

### Related Tools

- [`m1nd.resonate`](activation.md#m1ndresonate) -- finds harmonically coupled nodes (complementary, not duplicate)
- [`m1nd.activate`](activation.md#m1ndactivate) -- simpler search without equivalence scoring

---

## `m1nd.hypothesize`

Graph-based hypothesis testing. Takes a natural language claim about the codebase, parses it into a structural query pattern, and returns evidence for and against the claim with a Bayesian confidence score.

Supported claim patterns (auto-detected from natural language): `NEVER_CALLS`, `ALWAYS_BEFORE`, `DEPENDS_ON`, `NO_DEPENDENCY`, `COUPLING`, `ISOLATED`, `GATEWAY`, `CIRCULAR`.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `claim` | `string` | Yes | -- | Natural language claim about the codebase. Examples: `"chat_handler never validates session tokens"`, `"all external calls go through smart_router"`, `"critic is independent of whatsapp"`. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `max_hops` | `integer` | No | `5` | Maximum BFS hops for evidence search. |
| `include_ghost_edges` | `boolean` | No | `true` | Include ghost edges as weak evidence. Ghost edges count as lower-weight supporting evidence. |
| `include_partial_flow` | `boolean` | No | `true` | Include partial flow when full path not found. Shows how far the search reached. |
| `path_budget` | `integer` | No | `1000` | Budget cap for all-paths enumeration. Limits computation on dense graphs. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "m1nd.hypothesize",
    "arguments": {
      "agent_id": "jimi",
      "claim": "worker_pool depends on whatsapp_manager at runtime"
    }
  }
}
```

### Example Response

```json
{
  "claim": "worker_pool depends on whatsapp_manager at runtime",
  "claim_type": "depends_on",
  "subject_nodes": ["file::worker_pool.py"],
  "object_nodes": ["file::whatsapp_manager.py"],
  "verdict": "likely_true",
  "confidence": 0.72,
  "supporting_evidence": [
    {
      "type": "path_found",
      "description": "2-hop path via process_manager.cancel",
      "likelihood_factor": 3.5,
      "nodes": ["file::worker_pool.py", "file::process_manager.py", "file::whatsapp_manager.py"],
      "relations": ["calls", "imports"],
      "path_weight": 0.68
    }
  ],
  "contradicting_evidence": [],
  "paths_explored": 25015,
  "elapsed_ms": 58.0
}
```

### Verdict Values

| Verdict | Confidence Range | Meaning |
|---------|-----------------|---------|
| `"likely_true"` | > 0.8 | Strong structural evidence supports the claim |
| `"likely_false"` | < 0.2 | Strong structural evidence contradicts the claim |
| `"inconclusive"` | 0.2 -- 0.8 | Evidence exists both for and against |

### When to Use

- **Architecture validation** -- test claims about module boundaries and dependencies
- **Bug investigation** -- test whether a suspected dependency exists
- **Code review** -- verify architectural invariants are maintained
- **Security audit** -- test isolation claims (e.g. "auth module is isolated from user input")

### Related Tools

- [`m1nd.why`](memory.md#m1ndwhy) -- finds the path between two specific nodes
- [`m1nd.impact`](#m1ndimpact) -- measures downstream impact rather than testing a claim
- [`m1nd.scan`](exploration.md#m1ndscan) -- structural analysis with predefined patterns

---

## `m1nd.differential`

Focused structural diff between two graph snapshots. Compares edges, weights, nodes, and coupling between snapshot A and snapshot B, optionally narrowed by a focus question.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `snapshot_a` | `string` | Yes | -- | Path to snapshot A, or `"current"` for the in-memory graph. |
| `snapshot_b` | `string` | Yes | -- | Path to snapshot B, or `"current"` for the in-memory graph. |
| `question` | `string` | No | -- | Focus filter question. Narrows the diff output. Examples: `"what new coupling was introduced?"`, `"what modules became isolated?"`. |
| `focus_nodes` | `string[]` | No | `[]` | Limit diff to neighborhood of specific nodes. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "m1nd.differential",
    "arguments": {
      "agent_id": "jimi",
      "snapshot_a": "/path/to/before.json",
      "snapshot_b": "current",
      "question": "what new coupling was introduced?"
    }
  }
}
```

### Example Response

```json
{
  "snapshot_a": "/path/to/before.json",
  "snapshot_b": "current",
  "new_edges": [
    { "source": "whatsapp_routes.py", "target": "chat_handler.py", "relation": "calls", "weight": 0.45 }
  ],
  "removed_edges": [],
  "weight_changes": [
    { "source": "session_pool.py", "target": "worker_pool.py", "relation": "imports", "old_weight": 0.5, "new_weight": 0.78, "delta": 0.28 }
  ],
  "new_nodes": ["file::whatsapp_chat_bridge.py"],
  "removed_nodes": [],
  "coupling_deltas": [
    { "community_a": "whatsapp", "community_b": "chat", "old_coupling": 0.2, "new_coupling": 0.65, "delta": 0.45 }
  ],
  "summary": "1 new edge, 1 new node, 1 weight change, 1 coupling increase",
  "elapsed_ms": 120.5
}
```

### When to Use

- **Pre/post refactor comparison** -- snapshot before, refactor, then diff against current
- **PR review** -- compare graph before and after a branch's changes
- **Architecture drift monitoring** -- periodic snapshot comparison

### Related Tools

- [`m1nd.diverge`](#m1nddiverge) -- higher-level drift analysis with anomaly detection
- [`m1nd.lock.diff`](lifecycle.md#m1ndlockdiff) -- diff within a locked region (no snapshot file needed)

---

## `m1nd.diverge`

Structural drift detection between a baseline and the current graph state. Higher-level than `differential` -- includes anomaly detection (test deficits, velocity spikes, new coupling), coupling matrix changes, and a Jaccard-based structural drift score.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `baseline` | `string` | Yes | -- | Baseline reference. Values: ISO date (`"2026-03-01"`), git ref (SHA or tag), or `"last_session"` to use the saved GraphFingerprint. |
| `scope` | `string` | No | -- | File path glob to limit scope. Example: `"backend/stormender*"`. `None` = all nodes. |
| `include_coupling_changes` | `boolean` | No | `true` | Include coupling matrix delta between communities. |
| `include_anomalies` | `boolean` | No | `true` | Detect anomalies: test deficits, velocity spikes, new coupling, isolation. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "m1nd.diverge",
    "arguments": {
      "agent_id": "jimi",
      "baseline": "2026-03-01",
      "scope": "backend/stormender*"
    }
  }
}
```

### Example Response

```json
{
  "baseline": "2026-03-01",
  "baseline_commit": "a1b2c3d",
  "scope": "backend/stormender*",
  "structural_drift": 0.23,
  "new_nodes": ["file::stormender_v2_runtime_guard.py"],
  "removed_nodes": [],
  "modified_nodes": [
    { "file": "stormender_v2.py", "delta": "+450/-30", "growth_ratio": 1.85 }
  ],
  "coupling_changes": [
    { "pair": ["stormender_v2.py", "stormender_v2_routes.py"], "was": 0.4, "now": 0.72, "direction": "strengthened" }
  ],
  "anomalies": [
    { "type": "test_deficit", "file": "stormender_v2_runtime_guard.py", "detail": "New file with 0 test files", "severity": "warning" },
    { "type": "velocity_spike", "file": "stormender_v2.py", "detail": "450 lines added in 12 days", "severity": "info" }
  ],
  "summary": "23% structural drift. 1 new file (untested). Stormender coupling strengthened.",
  "elapsed_ms": 85.0
}
```

### Anomaly Types

| Type | Description |
|------|-------------|
| `test_deficit` | New or modified file with no corresponding test file |
| `velocity_spike` | Unusually high churn rate |
| `new_coupling` | Previously independent modules are now coupled |
| `isolation` | Module that was connected became isolated |

### When to Use

- **Session start** -- `m1nd.drift` shows weight-level changes; `m1nd.diverge` shows structural-level changes
- **Sprint retrospective** -- how much did the architecture change this sprint?
- **Quality gate** -- flag files with test deficits before merging

### Related Tools

- [`m1nd.drift`](memory.md#m1nddrift) -- weight-level drift (lighter, faster)
- [`m1nd.differential`](#m1nddifferential) -- lower-level snapshot diff
- [`m1nd.timeline`](exploration.md#m1ndtimeline) -- single-node temporal history
