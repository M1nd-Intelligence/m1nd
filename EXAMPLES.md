# m1nd Examples

Real output from running m1nd against a production Python backend (335 files, ~52K lines). Module names below are illustrative of the actual production modules analyzed.

## Ingest

```jsonc
// Request
{"method":"tools/call","params":{"name":"m1nd.ingest","arguments":{
  "agent_id":"dev","source":"filesystem","path":"/project/backend","incremental":false
}}}

// Response (910ms)
{
  "files_processed": 335,
  "nodes_created": 9767,
  "edges_created": 26557,
  "languages": {"python": 335},
  "elapsed_ms": 910
}
```

## Spreading Activation

```jsonc
// Request
{"method":"tools/call","params":{"name":"m1nd.activate","arguments":{
  "agent_id":"dev","query":"connection pool management","top_k":5
}}}

// Response (31ms) — top 5 results
{
  "activated": [
    {"node_id": "file::pool.py", "score": 0.89, "dimension_scores": {"structural": 0.92, "semantic": 0.95, "temporal": 0.78, "causal": 0.71}},
    {"node_id": "file::pool.py::class::ConnectionPool", "score": 0.84},
    {"node_id": "file::worker.py", "score": 0.61},
    {"node_id": "file::pool.py::fn::acquire", "score": 0.58},
    {"node_id": "file::process_manager.py", "score": 0.45}
  ],
  "ghost_edges": [
    {"from": "file::pool.py", "to": "file::recovery.py", "confidence": 0.34}
  ]
}
```

## Blast Radius

```jsonc
// Request: "What breaks if I change handler.py?"
{"method":"tools/call","params":{"name":"m1nd.impact","arguments":{
  "agent_id":"dev","node_id":"file::handler.py","depth":3
}}}

// Response (52ms)
{
  "blast_radius": [
    // 4,271 affected nodes across 3 depths
    {"depth": 1, "nodes": 47},
    {"depth": 2, "nodes": 891},
    {"depth": 3, "nodes": 3333}
  ],
  "total_affected": 4271,
  "pct_of_graph": 43.7,
  "risk": "critical",
  "pagerank": 0.635
}
```

## Hypothesis Testing

```jsonc
// Request: "Does the worker pool have a runtime dependency on the messaging module?"
{"method":"tools/call","params":{"name":"m1nd.hypothesize","arguments":{
  "agent_id":"dev","claim":"worker depends on messaging at runtime"
}}}

// Response (58ms)
{
  "verdict": "likely_true",
  "confidence": 0.72,
  "paths_explored": 25015,
  "evidence": [
    {"path": ["file::worker.py", "file::process_manager.py::fn::cancel", "file::messaging.py"], "hops": 2}
  ],
  "note": "2-hop dependency via cancel function — invisible to grep"
}
```

## Counterfactual Simulation

```jsonc
// Request: "What happens if I delete worker.py?"
{"method":"tools/call","params":{"name":"m1nd.counterfactual","arguments":{
  "agent_id":"dev","node_ids":["file::worker.py"]
}}}

// Response (3ms)
{
  "cascade": [
    {"depth": 1, "affected": 23},
    {"depth": 2, "affected": 456},
    {"depth": 3, "affected": 3710}
  ],
  "total_affected": 4189,
  "orphaned_count": 0,
  "pct_activation_lost": 0.41
}
```

## Structural Hole Detection

```jsonc
// Request: "What's missing around database connection pooling?"
{"method":"tools/call","params":{"name":"m1nd.missing","arguments":{
  "agent_id":"dev","query":"database connection pooling"
}}}

// Response (67ms)
{
  "holes": [
    {"region": "connection lifecycle", "adjacent_nodes": 4, "description": "No dedicated connection pool abstraction"},
    {"region": "pool metrics", "adjacent_nodes": 3, "description": "No pool health monitoring"},
    // ... 7 more structural holes
  ],
  "total_holes": 9
}
```

## Investigation Trail

```jsonc
// Save investigation state
{"method":"tools/call","params":{"name":"m1nd.trail.save","arguments":{
  "agent_id":"dev",
  "label":"auth-leak-investigation",
  "hypotheses":[
    {"statement":"Auth tokens leak through session pool","confidence":0.7,"status":"investigating"},
    {"statement":"Rate limiter missing from auth chain","confidence":0.9,"status":"confirmed"}
  ]
}}}

// Resume next day — exact context restored
{"method":"tools/call","params":{"name":"m1nd.trail.resume","arguments":{
  "agent_id":"dev","trail_id":"trail-abc123"
}}}
// → nodes_reactivated: 47, stale: 2, hypotheses_downgraded: 0
```

## Multi-Repo Federation

```jsonc
// Unify backend + frontend into one graph
{"method":"tools/call","params":{"name":"m1nd.federate","arguments":{
  "agent_id":"dev",
  "repos":[
    {"path":"/project/backend","label":"backend"},
    {"path":"/project/frontend","label":"frontend"}
  ]
}}}

// Response (1.3s)
{
  "unified_nodes": 11217,
  "cross_repo_edges": 18203,
  "repos_federated": 2
}
```

## Lock + Diff (Change Detection)

```jsonc
// Lock a region around handler.py
{"method":"tools/call","params":{"name":"m1nd.lock.create","arguments":{
  "agent_id":"dev","center":"file::handler.py","radius":2
}}}
// → 1,639 nodes, 707 edges locked

// After some code changes + re-ingest...
{"method":"tools/call","params":{"name":"m1nd.lock.diff","arguments":{
  "agent_id":"dev","lock_id":"lock-xyz"
}}}
// Response (0.08μs — yes, microseconds)
{
  "new_nodes": ["file::handler.py::fn::new_method"],
  "removed_nodes": [],
  "weight_changes": 3,
  "structural_changes": true
}
```

## Stacktrace Analysis

```jsonc
// Map an error to structural root causes
{"method":"tools/call","params":{"name":"m1nd.trace","arguments":{
  "agent_id":"dev",
  "error_text":"Traceback: File handler.py line 234 in handle_message\n  File pool.py line 89 in acquire\n  File worker.py line 156 in submit\n  TimeoutError: pool exhausted"
}}}

// Response (3.5ms)
{
  "suspects": [
    {"node": "file::worker.py::fn::submit", "suspiciousness": 0.91, "reason": "terminal frame + high centrality"},
    {"node": "file::pool.py::fn::acquire", "suspiciousness": 0.78, "reason": "resource acquisition"},
    {"node": "file::handler.py::fn::handle_message", "suspiciousness": 0.45, "reason": "entry point"}
  ],
  "related_test_files": ["file::tests/test_worker.py", "file::tests/test_pool.py"]
}
```

## Surgical Context V2 (Multi-file dependency context in one call)

```jsonc
// Get full context for a file including all connected sources
{"method":"tools/call","params":{"name":"m1nd.surgical_context_v2","arguments":{
  "agent_id":"dev",
  "file_path":"backend/chat_handler.py",
  "include_connected_sources": true,
  "max_connected_files": 5,
  "max_lines_per_file": 500
}}}

// Response (1.3ms) — includes target file + all connected files with source
{
  "target": {
    "node_id": "file::backend/chat_handler.py",
    "source": "class ChatHandler:\n    def handle(self, msg):\n        ...",
    "trust_score": 0.82,
    "blast_radius": 14
  },
  "connected_files": [
    {
      "node_id": "file::backend/worker_pool.py",
      "relationship": "callee",
      "source": "class WorkerPool:\n    def acquire(self):\n        ...",
      "lines_included": 143
    },
    {
      "node_id": "file::backend/tests/test_chat_handler.py",
      "relationship": "test",
      "source": "def test_handle_message():\n    handler = ChatHandler()\n    ...",
      "lines_included": 67
    }
  ],
  "total_files": 3,
  "total_lines": 356
}
```

## Search (Literal + Regex full-text search across the graph)

```jsonc
// Request — literal mode: find all nodes referencing a specific secret key
{"method":"tools/call","params":{"name":"m1nd.search","arguments":{
  "agent_id":"dev","query":"ANTHROPIC_API_KEY","mode":"literal","max_results":20
}}}

// Response (4ms)
{
  "matches": [
    {"node_id": "file::backend/core/config.py", "match": "ANTHROPIC_API_KEY", "line": 42, "context": "api_key = os.getenv(\"ANTHROPIC_API_KEY\")"},
    {"node_id": "file::backend/core/config.py", "match": "ANTHROPIC_API_KEY", "line": 118, "context": "if not os.environ.get(\"ANTHROPIC_API_KEY\"):"},
    {"node_id": "file::backend/tests/test_config.py", "match": "ANTHROPIC_API_KEY", "line": 17, "context": "monkeypatch.setenv(\"ANTHROPIC_API_KEY\", \"sk-test\")"}
  ],
  "total_matches": 3,
  "mode": "literal",
  "query": "ANTHROPIC_API_KEY",
  "elapsed_ms": 4
}

// Request — regex mode: find all TODO and FIXME comments across the graph
{"method":"tools/call","params":{"name":"m1nd.search","arguments":{
  "agent_id":"dev","query":"TODO|FIXME","mode":"regex","max_results":50
}}}

// Response (11ms)
{
  "matches": [
    {"node_id": "file::backend/worker.py", "match": "TODO", "line": 89, "context": "# TODO: add backpressure when queue depth exceeds 500"},
    {"node_id": "file::backend/pool.py", "match": "FIXME", "line": 134, "context": "# FIXME: race condition on double-acquire under high load"},
    {"node_id": "file::backend/handler.py", "match": "TODO", "line": 211, "context": "# TODO: remove after session_pool migration"},
    // ... 23 more matches
  ],
  "total_matches": 26,
  "mode": "regex",
  "query": "TODO|FIXME",
  "elapsed_ms": 11
}
```

## Help (Built-in tool reference)

```jsonc
// Request — overview: what tools does m1nd have?
{"method":"tools/call","params":{"name":"m1nd.help","arguments":{
  "agent_id":"dev","topic":"about"
}}}

// Response (0ms)
{
  "overview": "m1nd — neuro-symbolic connectome engine with Hebbian plasticity and spreading activation.",
  "tool_count": 61,
  "categories": [
    {"name": "Foundation", "count": 13, "tools": ["ingest","activate","impact","why","learn","drift","health","seek","scan","timeline","diverge","warmup","federate"]},
    {"name": "Perspective Navigation", "count": 12},
    {"name": "Lock System", "count": 5},
    {"name": "Superpowers", "count": 13},
    {"name": "Superpowers Extended", "count": 9},
    {"name": "Surgical", "count": 4},
    {"name": "Intelligence", "count": 5, "tools": ["search","help","panoramic","savings","report"]}
  ],
  "session_start_recipe": "ingest → activate → warmup → work → learn → savings"
}

// Request — specific tool: how does activate work?
{"method":"tools/call","params":{"name":"m1nd.help","arguments":{
  "agent_id":"dev","topic":"activate"
}}}

// Response (0ms)
{
  "tool": "m1nd.activate",
  "category": "Foundation",
  "description": "Spreading activation query — fires from seed nodes matching query, propagates signal across 4 dimensions (structural, semantic, temporal, causal). Returns ranked nodes with activation scores.",
  "parameters": {
    "agent_id": "string (required)",
    "query": "string — natural language or code concept",
    "top_k": "int (default: 10) — number of results",
    "decay": "float (default: 0.85) — propagation decay per hop"
  },
  "speed": "1.36μs (bench, 1K nodes)",
  "example": "{\"query\": \"connection pool management\", \"agent_id\": \"dev\", \"top_k\": 5}",
  "related_tools": ["seek", "warmup", "learn"]
}
```

## Panoramic (Full-codebase risk panorama)

```jsonc
// Request: get a risk map of the top 50 modules
{"method":"tools/call","params":{"name":"m1nd.panoramic","arguments":{
  "agent_id":"dev","max_modules":50,"min_risk_score":0.3
}}}

// Response (38ms) — 50 modules scanned, 12 above risk threshold 0.3
{
  "scanned_modules": 50,
  "above_threshold": 12,
  "panorama": [
    {
      "node_id": "file::backend/pool.py",
      "risk_score": 0.91,
      "risk_factors": {
        "tremor": 0.88,
        "epidemic_susceptibility": 0.79,
        "trust_score": 0.31,
        "blast_radius_pct": 43.7
      },
      "recommendation": "HIGH: frequent churn + high centrality + low trust. Prioritize review."
    },
    {
      "node_id": "file::backend/worker.py",
      "risk_score": 0.74,
      "risk_factors": {
        "tremor": 0.61,
        "epidemic_susceptibility": 0.83,
        "trust_score": 0.44,
        "blast_radius_pct": 38.2
      },
      "recommendation": "MEDIUM-HIGH: epidemic vector. Add tests before next change."
    },
    {
      "node_id": "file::backend/auth.py",
      "risk_score": 0.52,
      "risk_factors": {
        "tremor": 0.31,
        "epidemic_susceptibility": 0.55,
        "trust_score": 0.71,
        "blast_radius_pct": 19.4
      },
      "recommendation": "MEDIUM: acceptable trust score but moderate blast radius."
    }
    // ... 9 more modules
  ],
  "elapsed_ms": 38
}
```

## Savings (Token economy tracker)

```jsonc
// Request: how many tokens has m1nd saved this session?
{"method":"tools/call","params":{"name":"m1nd.savings","arguments":{
  "agent_id":"dev"
}}}

// Response (0ms)
{
  "session": {
    "queries_served": 47,
    "estimated_tokens_saved": 186400,
    "estimated_cost_saved_usd": 0.558,
    "avg_tokens_per_query_saved": 3966
  },
  "cumulative": {
    "queries_served": 1247,
    "estimated_tokens_saved": 4943800,
    "estimated_cost_saved_usd": 14.83,
    "sessions_tracked": 31
  },
  "baseline_assumption": "avg 4K tokens per direct file-read LLM call at $0.003/1K tokens",
  "note": "m1nd itself consumed 0 LLM tokens. All savings are vs. direct-read baseline."
}

// Request: reset the session counter
{"method":"tools/call","params":{"name":"m1nd.savings","arguments":{
  "agent_id":"dev","reset":true
}}}
// → {"reset": true, "session_cleared": true, "cumulative_preserved": true}
```

## Apply Batch (Atomic multi-file edits with single re-ingest)

```jsonc
// Write multiple files in one atomic operation, re-ingest once
{"method":"tools/call","params":{"name":"m1nd.apply_batch","arguments":{
  "agent_id":"dev",
  "edits": [
    {
      "file_path": "backend/chat_handler.py",
      "new_content": "class ChatHandler:\n    def handle(self, msg: Message) -> Response:\n        ..."
    },
    {
      "file_path": "backend/types.py",
      "new_content": "from dataclasses import dataclass\n\n@dataclass\nclass Message:\n    content: str\n    ..."
    }
  ],
  "atomic": true
}}}

// Response (165ms) — per-file diffs + single re-ingest result
{
  "results": [
    {
      "file_path": "backend/chat_handler.py",
      "success": true,
      "diff": "@@ -1,3 +1,3 @@\n class ChatHandler:\n-    def handle(self, msg):\n+    def handle(self, msg: Message) -> Response:\n         ..."
    },
    {
      "file_path": "backend/types.py",
      "success": true,
      "diff": "@@ -0,0 +1,5 @@\n+from dataclasses import dataclass\n+\n+@dataclass\n+class Message:\n+    content: str\n+    ..."
    }
  ],
  "re_ingest_nodes_updated": 47,
  "total_files": 2,
  "atomic": true
}
```
