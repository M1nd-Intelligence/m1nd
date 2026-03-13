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
