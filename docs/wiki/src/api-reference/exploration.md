# Exploration Tools

Six tools for intent-aware search, pattern-based scanning, structural hole detection, stacktrace analysis, temporal history, and multi-repository federation.

---

## `m1nd.seek`

Intent-aware semantic code search. Finds code by **purpose**, not text pattern. Combines keyword matching, graph activation (PageRank), and trigram similarity for ranking.

Unlike `activate` (which propagates signal through edges), `seek` scores every node independently against the query, making it better for finding specific code when you know what it *does* but not where it *lives*.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | `string` | Yes | -- | Natural language description of what the agent is looking for. Example: `"code that validates user credentials"`. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `top_k` | `integer` | No | `20` | Maximum results to return. |
| `scope` | `string` | No | -- | File path prefix to limit search scope. Example: `"backend/"`. `None` = entire graph. |
| `node_types` | `string[]` | No | `[]` | Filter by node type: `"function"`, `"class"`, `"struct"`, `"module"`, `"file"`. Empty = all types. |
| `min_score` | `number` | No | `0.1` | Minimum combined score threshold. Range: 0.0 to 1.0. |
| `graph_rerank` | `boolean` | No | `true` | Whether to run graph re-ranking (PageRank weighting) on candidates. Disable for pure text matching. |

### Scoring Formula (V1)

```
combined = keyword_match * 0.6 + graph_activation * 0.3 + trigram * 0.1
```

V2 upgrade path will replace keyword matching with real embeddings (fastembed-rs + jina-embeddings-v2-base-code).

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "m1nd.seek",
    "arguments": {
      "agent_id": "jimi",
      "query": "code that validates user credentials",
      "top_k": 5,
      "node_types": ["function", "class"]
    }
  }
}
```

### Example Response

```json
{
  "query": "code that validates user credentials",
  "results": [
    {
      "node_id": "file::auth_discovery.py::fn::validate_credentials",
      "label": "validate_credentials",
      "type": "function",
      "score": 0.87,
      "score_breakdown": {
        "embedding_similarity": 0.92,
        "graph_activation": 0.78,
        "temporal_recency": 0.65
      },
      "intent_summary": "Validates user credentials against provider store",
      "file_path": "backend/auth_discovery.py",
      "line_start": 45,
      "line_end": 82,
      "connections": [
        { "node_id": "file::principal_registry.py", "label": "principal_registry.py", "relation": "calls" }
      ]
    }
  ],
  "total_candidates_scanned": 9767,
  "embeddings_used": false,
  "elapsed_ms": 25.0
}
```

### When to Use

- **"Find the code that does X"** -- when you know the purpose, not the filename
- **Codebase onboarding** -- exploring unfamiliar code by intent
- **Pre-modification search** -- find all code related to a feature before changing it

### Related Tools

- [`m1nd.activate`](activation.md#m1ndactivate) -- graph-propagation search (better for exploring neighborhoods)
- [`m1nd.scan`](#m1ndscan) -- pattern-based structural analysis (finds anti-patterns, not features)

---

## `m1nd.scan`

Pattern-aware structural code analysis with graph-validated findings. Detects structural issues using predefined patterns, then validates each finding against the graph to filter false positives. Works across file boundaries.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `pattern` | `string` | Yes | -- | Pattern ID or custom pattern string. Built-in patterns: `"error_handling"`, `"resource_cleanup"`, `"api_surface"`, `"state_mutation"`, `"concurrency"`, `"auth_boundary"`, `"test_coverage"`, `"dependency_injection"`. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `scope` | `string` | No | -- | File path prefix to limit scan scope. |
| `severity_min` | `number` | No | `0.3` | Minimum severity threshold. Range: 0.0 to 1.0. |
| `graph_validate` | `boolean` | No | `true` | Validate findings against graph edges (cross-file analysis). Disable for raw pattern matching only. |
| `limit` | `integer` | No | `50` | Maximum findings to return. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "m1nd.scan",
    "arguments": {
      "agent_id": "jimi",
      "pattern": "error_handling",
      "scope": "backend/",
      "severity_min": 0.5
    }
  }
}
```

### Example Response

```json
{
  "pattern": "error_handling",
  "findings": [
    {
      "pattern": "error_handling",
      "status": "confirmed",
      "severity": 0.78,
      "node_id": "file::spawner.py::fn::spawn_agent",
      "label": "spawn_agent",
      "file_path": "backend/spawner.py",
      "line": 89,
      "message": "Bare except clause catches all exceptions including KeyboardInterrupt",
      "graph_context": [
        { "node_id": "file::process_manager.py", "label": "process_manager.py", "relation": "calls" }
      ]
    }
  ],
  "files_scanned": 77,
  "total_matches_raw": 23,
  "total_matches_validated": 8,
  "elapsed_ms": 150.0
}
```

### Finding Status Values

| Status | Meaning |
|--------|---------|
| `"confirmed"` | Graph validation confirms the issue is real |
| `"mitigated"` | The issue exists but is handled by a related module |
| `"false_positive"` | Graph context shows the pattern match is not actually an issue |

### When to Use

- **Code quality audit** -- scan for anti-patterns across the codebase
- **Security review** -- use `"auth_boundary"` to find auth bypass paths
- **Pre-deploy check** -- scan for `"error_handling"` and `"resource_cleanup"` issues
- **Test gaps** -- use `"test_coverage"` to find untested code

### Related Tools

- [`m1nd.hypothesize`](analysis.md#m1ndhypothesize) -- test a specific structural claim
- [`m1nd.trace`](#m1ndtrace) -- analyze a specific error (not patterns)

---

## `m1nd.missing`

Detect structural holes and missing connections. Given a topic query, finds areas where the graph suggests something *should* exist but does not. Identifies absent abstractions, missing connections, and incomplete patterns.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | `string` | Yes | -- | Search query to find structural holes around. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `min_sibling_activation` | `number` | No | `0.3` | Minimum sibling activation threshold. Siblings with activation below this are not considered for hole detection. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "m1nd.missing",
    "arguments": {
      "agent_id": "jimi",
      "query": "database connection pooling",
      "min_sibling_activation": 0.3
    }
  }
}
```

### Example Response

```json
{
  "query": "database connection pooling",
  "holes": [
    {
      "node_id": "structural_hole_1",
      "label": "connection lifecycle",
      "type": "structural_hole",
      "reason": "No dedicated connection pool abstraction -- 4 adjacent modules manage connections independently"
    },
    {
      "node_id": "structural_hole_2",
      "label": "pool metrics",
      "type": "structural_hole",
      "reason": "No pool health monitoring -- 3 adjacent modules expose metrics but pool does not"
    }
  ],
  "total_holes": 9,
  "elapsed_ms": 67.0
}
```

### When to Use

- **Gap analysis** -- "what am I missing?" before implementing a feature
- **Pre-spec** -- identify areas that need design before building
- **Architecture review** -- find missing abstractions or connections
- **Feature completeness** -- after building, check for structural holes around the feature

### Related Tools

- [`m1nd.activate`](activation.md#m1ndactivate) -- activate with `include_structural_holes: true` for inline hole detection
- [`m1nd.hypothesize`](analysis.md#m1ndhypothesize) -- test a specific claim about missing structure

---

## `m1nd.trace`

Map runtime errors to structural root causes via stacktrace analysis. Parses the stacktrace, maps frames to graph nodes, and scores each node's suspiciousness using trace depth, modification recency, and centrality. Also finds co-change suspects (files modified around the same time as the top suspect).

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `error_text` | `string` | Yes | -- | Full error output (stacktrace + error message). |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `language` | `string` | No | -- | Language hint: `"python"`, `"rust"`, `"typescript"`, `"javascript"`, `"go"`. Auto-detected if omitted. |
| `window_hours` | `number` | No | `24.0` | Temporal window (hours) for co-change suspect scan. |
| `top_k` | `integer` | No | `10` | Max suspects to return. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "m1nd.trace",
    "arguments": {
      "agent_id": "jimi",
      "error_text": "Traceback (most recent call last):\n  File \"backend/chat_handler.py\", line 234, in handle_message\n  File \"backend/session_pool.py\", line 89, in acquire\n  File \"backend/worker_pool.py\", line 156, in submit\nTimeoutError: pool exhausted",
      "language": "python",
      "top_k": 5
    }
  }
}
```

### Example Response

```json
{
  "language_detected": "python",
  "error_type": "TimeoutError",
  "error_message": "pool exhausted",
  "frames_parsed": 3,
  "frames_mapped": 3,
  "suspects": [
    {
      "node_id": "file::worker_pool.py::fn::submit",
      "label": "submit",
      "type": "function",
      "suspiciousness": 0.91,
      "signals": { "trace_depth_score": 1.0, "recency_score": 0.85, "centrality_score": 0.88 },
      "file_path": "backend/worker_pool.py",
      "line_start": 150,
      "line_end": 175,
      "related_callers": ["session_pool.py::acquire"]
    },
    {
      "node_id": "file::session_pool.py::fn::acquire",
      "label": "acquire",
      "type": "function",
      "suspiciousness": 0.78,
      "signals": { "trace_depth_score": 0.67, "recency_score": 0.72, "centrality_score": 0.65 },
      "file_path": "backend/session_pool.py",
      "line_start": 80,
      "line_end": 110,
      "related_callers": ["chat_handler.py::handle_message"]
    }
  ],
  "co_change_suspects": [
    { "node_id": "file::config.py", "label": "config.py", "modified_at": 1710295000.0, "reason": "Modified within 2h of top suspect" }
  ],
  "causal_chain": ["worker_pool.py::submit", "session_pool.py::acquire", "chat_handler.py::handle_message"],
  "fix_scope": {
    "files_to_inspect": ["backend/worker_pool.py", "backend/session_pool.py", "backend/config.py"],
    "estimated_blast_radius": 23,
    "risk_level": "medium"
  },
  "unmapped_frames": [],
  "elapsed_ms": 3.5
}
```

### Suspiciousness Scoring

| Signal | Weight | Description |
|--------|--------|-------------|
| `trace_depth_score` | High | 1.0 = deepest frame (most specific); decays linearly |
| `recency_score` | Medium | Exponential decay from last modification time |
| `centrality_score` | Medium | Normalized PageRank centrality |

### When to Use

- **Bug investigation** -- paste an error and get ranked suspects
- **Root cause analysis** -- the causal chain shows the error propagation path
- **Fix scoping** -- `fix_scope` tells you which files to inspect and the risk level

### Related Tools

- [`m1nd.hypothesize`](analysis.md#m1ndhypothesize) -- test a hypothesis about the root cause
- [`m1nd.impact`](analysis.md#m1ndimpact) -- assess the blast radius of a fix
- [`m1nd.validate_plan`](lifecycle.md#m1ndvalidate_plan) -- validate your fix plan before implementing

---

## `m1nd.timeline`

Git-based temporal history for a node. Returns the change history, co-change partners, velocity, stability score, and churn data for a specific file or module.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `node` | `string` | Yes | -- | Node external_id. Example: `"file::backend/chat_handler.py"`. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `depth` | `string` | No | `"30d"` | Time depth. Values: `"7d"`, `"30d"`, `"90d"`, `"all"`. |
| `include_co_changes` | `boolean` | No | `true` | Include co-changed files with coupling scores. |
| `include_churn` | `boolean` | No | `true` | Include lines added/deleted churn data. |
| `top_k` | `integer` | No | `10` | Max co-change partners to return. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "m1nd.timeline",
    "arguments": {
      "agent_id": "jimi",
      "node": "file::backend/chat_handler.py",
      "depth": "30d",
      "top_k": 5
    }
  }
}
```

### Example Response

```json
{
  "node": "file::backend/chat_handler.py",
  "depth": "30d",
  "changes": [
    {
      "date": "2026-03-10",
      "commit": "a1b2c3d",
      "author": "cosmophonix",
      "delta": "+45/-12",
      "co_changed": ["stream_parser.py", "chat_routes.py"]
    },
    {
      "date": "2026-03-05",
      "commit": "e4f5g6h",
      "author": "cosmophonix",
      "delta": "+120/-30",
      "co_changed": ["session_pool.py", "worker_pool.py"]
    }
  ],
  "co_changed_with": [
    { "file": "stream_parser.py", "times": 8, "coupling_degree": 0.72 },
    { "file": "chat_routes.py", "times": 6, "coupling_degree": 0.55 },
    { "file": "session_pool.py", "times": 4, "coupling_degree": 0.38 }
  ],
  "velocity": "accelerating",
  "stability_score": 0.35,
  "pattern": "churning",
  "total_churn": { "lines_added": 580, "lines_deleted": 120 },
  "commit_count_in_window": 12,
  "elapsed_ms": 45.0
}
```

### Velocity Values

| Value | Meaning |
|-------|---------|
| `"accelerating"` | Change frequency is increasing |
| `"decelerating"` | Change frequency is decreasing |
| `"stable"` | Consistent change rate |

### Pattern Values

| Value | Meaning |
|-------|---------|
| `"expanding"` | Growing (net positive churn) |
| `"shrinking"` | Reducing (net negative churn) |
| `"churning"` | High add+delete with little net growth |
| `"dormant"` | Few or no changes in the window |
| `"stable"` | Small, consistent changes |

### When to Use

- **Hotspot detection** -- find files that change too frequently (stability_score < 0.3)
- **Co-change discovery** -- find files that always change together (coupling_degree > 0.6)
- **Refactoring signals** -- churning files may need redesign

### Related Tools

- [`m1nd.diverge`](analysis.md#m1nddiverge) -- structural drift across the whole codebase
- [`m1nd.predict`](analysis.md#m1ndpredict) -- predict co-changes for a given modification

---

## `m1nd.federate`

Ingest multiple repositories into a unified federated graph with automatic cross-repo edge detection. After federation, all existing query tools (`activate`, `impact`, `why`, etc.) traverse cross-repo edges automatically.

Node IDs in the federated graph use `{repo_name}::file::path` format.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `repos` | `object[]` | Yes | -- | List of repositories to federate. Each object has: `name` (string, required -- namespace prefix), `path` (string, required -- absolute path), `adapter` (string, default `"code"`). |
| `detect_cross_repo_edges` | `boolean` | No | `true` | Auto-detect cross-repo edges: config, API, import, type, deployment, MCP contract. |
| `incremental` | `boolean` | No | `false` | Only re-ingest repos that changed since last federation. |

### Cross-Repo Edge Types

| Edge Type | Description |
|-----------|-------------|
| `shared_config` | Two repos reference the same configuration key |
| `api_contract` | One repo's API client matches another's API server |
| `package_dep` | Direct package dependency |
| `shared_type` | Same type/interface definition used across repos |
| `deployment_dep` | Deployment configuration dependency |
| `mcp_contract` | MCP tool consumer/provider relationship |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "m1nd.federate",
    "arguments": {
      "agent_id": "jimi",
      "repos": [
        { "name": "backend", "path": "/project/backend" },
        { "name": "frontend", "path": "/project/frontend" },
        { "name": "mcp-server", "path": "/project/mcp-server", "adapter": "code" }
      ],
      "detect_cross_repo_edges": true,
      "incremental": false
    }
  }
}
```

### Example Response

```json
{
  "repos_ingested": [
    { "name": "backend", "path": "/project/backend", "node_count": 9767, "edge_count": 26557, "from_cache": false, "ingest_ms": 910.0 },
    { "name": "frontend", "path": "/project/frontend", "node_count": 1200, "edge_count": 3500, "from_cache": false, "ingest_ms": 320.0 },
    { "name": "mcp-server", "path": "/project/mcp-server", "node_count": 250, "edge_count": 680, "from_cache": false, "ingest_ms": 85.0 }
  ],
  "total_nodes": 11217,
  "total_edges": 30737,
  "cross_repo_edges": [
    {
      "source_repo": "frontend",
      "target_repo": "backend",
      "source_node": "frontend::file::src/lib/apiConfig.ts",
      "target_node": "backend::file::main.py",
      "edge_type": "api_contract",
      "relation": "calls_api",
      "weight": 0.85,
      "causal_strength": 0.72
    },
    {
      "source_repo": "mcp-server",
      "target_repo": "backend",
      "source_node": "mcp-server::file::src/mcp-server.js",
      "target_node": "backend::file::main.py",
      "edge_type": "mcp_contract",
      "relation": "calls_api",
      "weight": 0.78,
      "causal_strength": 0.65
    }
  ],
  "cross_repo_edge_count": 18203,
  "incremental": false,
  "skipped_repos": [],
  "elapsed_ms": 1315.0
}
```

### When to Use

- **Multi-repo projects** -- analyze dependencies across frontend, backend, and infrastructure repos
- **Monorepo decomposition** -- understand how packages depend on each other
- **API impact analysis** -- find which repos are affected by an API change

### Related Tools

- [`m1nd.ingest`](lifecycle.md#m1ndingest) -- single-repo ingestion
- [`m1nd.impact`](analysis.md#m1ndimpact) -- blast radius analysis (works across federated repos)
- [`m1nd.why`](memory.md#m1ndwhy) -- path explanation (traverses cross-repo edges)
