# Memory & Learning Tools

Seven tools for feedback-based learning, drift analysis, path explanation, and investigation trail management.

---

## `m1nd.learn`

Explicit feedback-based edge adjustment. After using `activate` or other query tools, call `learn` to tell m1nd whether the results were correct, wrong, or partial. This applies Hebbian learning to strengthen or weaken edges between the query seeds and the reported nodes.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | `string` | Yes | -- | The original query this feedback relates to. Must match the query used in the activation. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `feedback` | `string` | Yes | -- | Feedback type. Values: `"correct"` (strengthen edges), `"wrong"` (weaken edges), `"partial"` (strengthen confirmed nodes only). |
| `node_ids` | `string[]` | Yes | -- | Node identifiers to apply feedback to. For `"correct"`, these are the relevant results. For `"wrong"`, these are the irrelevant ones. |
| `strength` | `number` | No | `0.2` | Feedback strength for edge adjustment. Range: 0.0 to 1.0. Higher = stronger plasticity effect. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "m1nd.learn",
    "arguments": {
      "agent_id": "jimi",
      "query": "session pool management",
      "feedback": "correct",
      "node_ids": ["file::pool.py", "file::worker.py"],
      "strength": 0.3
    }
  }
}
```

### Example Response

```json
{
  "query": "session pool management",
  "feedback": "correct",
  "edges_adjusted": 8,
  "nodes_affected": 2,
  "plasticity_delta": 0.3,
  "elapsed_ms": 2.1
}
```

### When to Use

- **After every activate where results were used** -- always provide feedback to improve future results
- **After investigation** -- report which activated nodes were actually relevant
- **Continuous improvement** -- the graph learns from your feedback over time

### Side Effects

Modifies edge weights in the graph. Changes are persisted on the next auto-persist cycle (every 50 queries) and on shutdown.

### Related Tools

- [`m1nd.activate`](activation.md#m1ndactivate) -- the query tool whose results you are providing feedback on
- [`m1nd.drift`](#m1nddrift) -- see cumulative weight changes from learning

---

## `m1nd.drift`

Weight and structural drift analysis. Compares the current graph state against a baseline (typically `"last_session"`) to show what changed -- new edges, removed edges, and weight drift. Useful for context recovery at session start.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `since` | `string` | No | `"last_session"` | Baseline reference point. Values: `"last_session"` (saved state from previous session), or a timestamp. |
| `include_weight_drift` | `boolean` | No | `true` | Include edge weight drift analysis. Shows which edges strengthened or weakened. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "m1nd.drift",
    "arguments": {
      "agent_id": "jimi",
      "since": "last_session",
      "include_weight_drift": true
    }
  }
}
```

### Example Response

```json
{
  "since": "last_session",
  "node_count_delta": 15,
  "edge_count_delta": 42,
  "new_nodes": ["file::forem_publisher.py", "file::forem_routes.py"],
  "removed_nodes": [],
  "top_weight_drifts": [
    { "edge": "pool.py -> worker.py", "old": 0.45, "new": 0.72, "delta": 0.27 },
    { "edge": "handler.py -> parser.py", "old": 0.60, "new": 0.48, "delta": -0.12 }
  ],
  "elapsed_ms": 12.0
}
```

### When to Use

- **Session start** -- first tool to call after `m1nd.health` to recover context
- **After ingest** -- see what the new ingest changed
- **After extended learning** -- track cumulative drift from feedback

### Related Tools

- [`m1nd.diverge`](analysis.md#m1nddiverge) -- higher-level structural drift with anomaly detection
- [`m1nd.health`](lifecycle.md#m1ndhealth) -- basic server health (call before drift)

---

## `m1nd.why`

Path explanation between two nodes. Finds and explains the relationship paths connecting a source node to a target node. Returns all paths up to `max_hops`, ranked by cumulative edge strength.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `source` | `string` | Yes | -- | Source node identifier. |
| `target` | `string` | Yes | -- | Target node identifier. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `max_hops` | `integer` | No | `6` | Maximum hops in path search. Higher values find more indirect paths but take longer. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "m1nd.why",
    "arguments": {
      "agent_id": "jimi",
      "source": "file::worker.py",
      "target": "file::messaging.py",
      "max_hops": 4
    }
  }
}
```

### Example Response

```json
{
  "source": "file::worker.py",
  "target": "file::messaging.py",
  "paths": [
    {
      "nodes": ["worker.py", "process_manager.py", "messaging.py"],
      "relations": ["calls::cancel", "imports"],
      "cumulative_strength": 0.68,
      "hops": 2
    },
    {
      "nodes": ["worker.py", "worker.py", "handler.py", "messaging.py"],
      "relations": ["imported_by", "calls", "imports"],
      "cumulative_strength": 0.31,
      "hops": 3
    }
  ],
  "total_paths_found": 2,
  "elapsed_ms": 15.0
}
```

### When to Use

- **Understanding dependencies** -- "why are these two modules connected?"
- **Tracing influence** -- find the relationship chain between distant modules
- **Bug investigation** -- understand how a change in A could affect B

### Related Tools

- [`m1nd.hypothesize`](analysis.md#m1ndhypothesize) -- tests a claim about the relationship (more powerful)
- [`m1nd.impact`](analysis.md#m1ndimpact) -- finds all affected nodes (broader scope)

---

## `m1nd.trail.save`

Persist the current investigation state -- nodes visited, hypotheses formed, conclusions reached, and open questions. Captures activation boosts for later restoration.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `label` | `string` | Yes | -- | Human-readable label for this investigation. |
| `hypotheses` | `object[]` | No | `[]` | Hypotheses formed during investigation. Each object has: `statement` (string, required), `confidence` (number, default 0.5), `supporting_nodes` (string[]), `contradicting_nodes` (string[]). |
| `conclusions` | `object[]` | No | `[]` | Conclusions reached. Each object has: `statement` (string, required), `confidence` (number, default 0.5), `from_hypotheses` (string[]), `supporting_nodes` (string[]). |
| `open_questions` | `string[]` | No | `[]` | Open questions remaining for future investigation. |
| `tags` | `string[]` | No | `[]` | Tags for organization and search. |
| `summary` | `string` | No | -- | Optional summary. Auto-generated if omitted. |
| `visited_nodes` | `object[]` | No | `[]` | Explicitly list visited nodes with annotations. Each object has: `node_external_id` (string, required), `annotation` (string, optional), `relevance` (number, default 0.5). If omitted, captured from active perspective state. |
| `activation_boosts` | `object` | No | `{}` | Map of `node_external_id` to boost weight `[0.0, 1.0]`. Re-injected on resume. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.save",
    "arguments": {
      "agent_id": "jimi",
      "label": "auth-leak-investigation",
      "hypotheses": [
        {
          "statement": "Auth tokens leak through session pool",
          "confidence": 0.7,
          "supporting_nodes": ["file::pool.py", "file::auth_discovery.py"]
        },
        {
          "statement": "Rate limiter missing from auth chain",
          "confidence": 0.9,
          "supporting_nodes": ["file::middleware.py"]
        }
      ],
      "open_questions": ["Does the rate limiter apply to WebSocket connections?"],
      "tags": ["security", "auth", "session"],
      "activation_boosts": {
        "file::pool.py": 0.8,
        "file::auth_discovery.py": 0.6
      }
    }
  }
}
```

### Example Response

```json
{
  "trail_id": "trail_jimi_001_a1b2c3",
  "label": "auth-leak-investigation",
  "agent_id": "jimi",
  "nodes_saved": 47,
  "hypotheses_saved": 2,
  "conclusions_saved": 0,
  "open_questions_saved": 1,
  "graph_generation_at_creation": 42,
  "created_at_ms": 1710300000000
}
```

### When to Use

- **End of investigation session** -- save your work before ending a session
- **Before context compaction** -- checkpoint your investigation state
- **Cross-session continuity** -- resume exactly where you left off

### Related Tools

- [`m1nd.trail.resume`](#m1ndtrailresume) -- restore a saved trail
- [`m1nd.trail.list`](#m1ndtraillist) -- find saved trails
- [`m1nd.trail.merge`](#m1ndtrailmerge) -- combine trails from parallel investigations

---

## `m1nd.trail.resume`

Restore a saved investigation. Re-injects activation boosts into the graph, validates that saved nodes still exist, detects staleness, and optionally downgrades hypotheses whose supporting nodes are missing.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `trail_id` | `string` | Yes | -- | Trail ID to resume (from `trail.save` or `trail.list`). |
| `force` | `boolean` | No | `false` | Resume even if trail is stale (>50% missing nodes). Default behavior: refuse to resume stale trails. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.resume",
    "arguments": {
      "agent_id": "jimi",
      "trail_id": "trail_jimi_001_a1b2c3"
    }
  }
}
```

### Example Response

```json
{
  "trail_id": "trail_jimi_001_a1b2c3",
  "label": "auth-leak-investigation",
  "stale": false,
  "generations_behind": 3,
  "missing_nodes": [],
  "nodes_reactivated": 47,
  "hypotheses_downgraded": [],
  "trail": {
    "trail_id": "trail_jimi_001_a1b2c3",
    "agent_id": "jimi",
    "label": "auth-leak-investigation",
    "status": "active",
    "created_at_ms": 1710300000000,
    "last_modified_ms": 1710300000000,
    "node_count": 47,
    "hypothesis_count": 2,
    "conclusion_count": 0,
    "open_question_count": 1,
    "tags": ["security", "auth", "session"],
    "summary": "Investigating auth token leaks through session pool"
  },
  "elapsed_ms": 22.5
}
```

### When to Use

- **Session start** -- restore a previous investigation
- **Cross-agent handoff** -- agent B resumes agent A's trail
- **After re-ingest** -- check if investigation nodes survived the graph update

### Related Tools

- [`m1nd.trail.save`](#m1ndtrailsave) -- save a trail to resume later
- [`m1nd.warmup`](activation.md#m1ndwarmup) -- simpler priming without full trail restoration

---

## `m1nd.trail.list`

List saved investigation trails with optional filters. Returns compact summaries suitable for selecting a trail to resume.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `filter_agent_id` | `string` | No | -- | Filter to a specific agent's trails. `None` = all agents. |
| `filter_status` | `string` | No | -- | Filter by status: `"active"`, `"saved"`, `"archived"`, `"stale"`, `"merged"`. |
| `filter_tags` | `string[]` | No | `[]` | Filter by tags (any match). |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.list",
    "arguments": {
      "agent_id": "jimi",
      "filter_status": "saved",
      "filter_tags": ["security"]
    }
  }
}
```

### Example Response

```json
{
  "trails": [
    {
      "trail_id": "trail_jimi_001_a1b2c3",
      "agent_id": "jimi",
      "label": "auth-leak-investigation",
      "status": "saved",
      "created_at_ms": 1710300000000,
      "last_modified_ms": 1710300000000,
      "node_count": 47,
      "hypothesis_count": 2,
      "conclusion_count": 0,
      "open_question_count": 1,
      "tags": ["security", "auth", "session"]
    }
  ],
  "total_count": 1
}
```

### When to Use

- **Session start** -- see what investigations are available to resume
- **Multi-agent coordination** -- see trails from other agents
- **Cleanup** -- find stale or merged trails

### Related Tools

- [`m1nd.trail.resume`](#m1ndtrailresume) -- resume a trail from this list
- [`m1nd.trail.merge`](#m1ndtrailmerge) -- combine related trails

---

## `m1nd.trail.merge`

Combine two or more investigation trails. Merges visited nodes, hypotheses, and conclusions. Uses confidence+recency scoring for conflict resolution. Discovers cross-connections between independently explored areas.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `trail_ids` | `string[]` | Yes | -- | Two or more trail IDs to merge. |
| `label` | `string` | No | -- | Label for the merged trail. Auto-generated if omitted. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.merge",
    "arguments": {
      "agent_id": "jimi",
      "trail_ids": ["trail_jimi_001_a1b2c3", "trail_analyst_002_d4e5f6"],
      "label": "combined-auth-investigation"
    }
  }
}
```

### Example Response

```json
{
  "merged_trail_id": "trail_jimi_003_g7h8i9",
  "label": "combined-auth-investigation",
  "source_trails": ["trail_jimi_001_a1b2c3", "trail_analyst_002_d4e5f6"],
  "nodes_merged": 83,
  "hypotheses_merged": 5,
  "conflicts": [
    {
      "hypothesis_a": "Session pool leaks tokens",
      "hypothesis_b": "Session pool tokens are properly scoped",
      "resolution": "resolved",
      "winner": "Session pool leaks tokens",
      "score_delta": 0.35
    }
  ],
  "connections_discovered": [
    {
      "type": "bridge_edge",
      "detail": "auth_discovery.py connects the auth trail to the session trail",
      "from_node": "file::auth_discovery.py",
      "to_node": "file::pool.py",
      "weight": 0.72
    }
  ],
  "elapsed_ms": 45.0
}
```

### Conflict Resolution

When merging hypotheses that contradict each other:
- **confidence+recency** scoring determines the winner
- If the score delta is too small, the conflict is marked `"unresolved"` for human review
- Source trails are set to `"merged"` status after a successful merge

### When to Use

- **Multi-agent investigation** -- combine findings from parallel agents
- **Investigation continuation** -- merge an old investigation with new findings
- **Consolidation** -- clean up related but separate investigation threads

### Related Tools

- [`m1nd.trail.save`](#m1ndtrailsave) -- save individual trails
- [`m1nd.trail.list`](#m1ndtraillist) -- find trails to merge
