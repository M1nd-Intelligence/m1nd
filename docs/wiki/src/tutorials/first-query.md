# First Query: The Full Cycle

This tutorial walks you through the core m1nd workflow: ingest, activate, learn, and observe the graph getting smarter. Then we explore structural holes and counterfactual simulation.

**Prerequisites**: You have completed the [Quick Start](quickstart.md) and have m1nd running with a codebase ingested.

All examples use the JSON-RPC wire format. If you are working through an MCP client (Claude Code, Cursor, etc.), the client sends these calls for you when you invoke the tools by name.

## Step 1: Ingest Your Codebase

If you have not already ingested, do it now:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.ingest",
    "arguments": {
      "path": "/your/project",
      "agent_id": "dev"
    }
  }
}
```

Response:

```json
{
  "files_processed": 335,
  "nodes_created": 9767,
  "edges_created": 26557,
  "languages": {"python": 335},
  "elapsed_ms": 910
}
```

The graph now contains structural nodes (files, classes, functions) and edges (imports, calls, inheritance, co-change patterns). PageRank has been computed, giving each node a centrality score.

## Step 2: First Activation

Ask the graph about session pool management:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "query": "session pool management",
      "agent_id": "dev",
      "top_k": 5
    }
  }
}
```

Response:

```json
{
  "activated": [
    {
      "node_id": "file::pool.py",
      "score": 0.89,
      "dimension_scores": {
        "structural": 0.92,
        "semantic": 0.95,
        "temporal": 0.78,
        "causal": 0.71
      }
    },
    {"node_id": "file::pool.py::class::ConnectionPool", "score": 0.84},
    {"node_id": "file::worker.py", "score": 0.61},
    {"node_id": "file::pool.py::fn::acquire", "score": 0.58},
    {"node_id": "file::process_manager.py", "score": 0.45}
  ],
  "ghost_edges": [
    {
      "from": "file::pool.py",
      "to": "file::recovery.py",
      "confidence": 0.34
    }
  ]
}
```

**Reading the results**:

- **`score`**: Combined 4-dimensional activation score (0.0 to 1.0)
- **`dimension_scores`**: Breakdown by structural (graph distance, PageRank), semantic (token overlap), temporal (co-change history), and causal (suspiciousness)
- **`ghost_edges`**: Connections the graph inferred but that are not explicit in code. Here, `pool.py` and `recovery.py` are structurally unconnected but co-activate together -- a hidden dependency worth investigating.

Note the scores. We will come back to this query after teaching the graph.

## Step 3: Teach the Graph (Hebbian Learning)

The top two results (`pool.py` and the `ConnectionPool` class) were exactly what we needed. Tell the graph:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.learn",
    "arguments": {
      "query": "session pool management",
      "agent_id": "dev",
      "feedback": "correct",
      "node_ids": [
        "file::pool.py",
        "file::pool.py::class::ConnectionPool"
      ],
      "strength": 0.2
    }
  }
}
```

Response:

```json
{
  "edges_strengthened": 740,
  "edges_weakened": 0,
  "plasticity_records": 740,
  "learning_type": "hebbian_ltp"
}
```

**What happened**: Hebbian Long-Term Potentiation (LTP) strengthened 740 edges along paths connecting the confirmed-useful nodes. "Neurons that fire together wire together." The next time anyone queries this region of the graph, those paths carry more signal.

Now suppose `worker.py` (score 0.61) was not actually relevant. Mark it wrong:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.learn",
    "arguments": {
      "query": "session pool management",
      "agent_id": "dev",
      "feedback": "wrong",
      "node_ids": ["file::worker.py"],
      "strength": 0.2
    }
  }
}
```

Response:

```json
{
  "edges_strengthened": 0,
  "edges_weakened": 312,
  "plasticity_records": 1052,
  "learning_type": "hebbian_ltd"
}
```

Long-Term Depression (LTD) weakened 312 edges leading to `worker.py` from this query region. The graph now knows: for session pool queries, `worker.py` is noise.

## Step 4: Activate Again -- See the Improvement

Run the exact same query:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "query": "session pool management",
      "agent_id": "dev",
      "top_k": 5
    }
  }
}
```

Expected changes:

```json
{
  "activated": [
    {"node_id": "file::pool.py", "score": 0.93},
    {"node_id": "file::pool.py::class::ConnectionPool", "score": 0.88},
    {"node_id": "file::pool.py::fn::acquire", "score": 0.65},
    {"node_id": "file::process_manager.py", "score": 0.47},
    {"node_id": "file::recovery.py", "score": 0.39}
  ]
}
```

Compare with Step 2:

| Node | Before | After | Change |
|------|--------|-------|--------|
| `pool.py` | 0.89 | 0.93 | +0.04 (strengthened) |
| `ConnectionPool` class | 0.84 | 0.88 | +0.04 (strengthened) |
| `worker.py` | 0.61 | dropped | Pushed below top-5 (weakened) |
| `recovery.py` | ghost only | 0.39 | Promoted from ghost to main results |

The graph learned. `worker.py` fell out of the top results. `recovery.py`, previously only a ghost edge, got promoted because the strengthened paths through `pool.py` now carry more signal to its neighborhood.

**This is the core value proposition of m1nd**: every interaction makes the graph smarter. No other code intelligence tool does this.

## Step 5: Structural Hole Detection

Ask the graph what is *missing* around a topic:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.missing",
    "arguments": {
      "query": "database connection pooling",
      "agent_id": "dev"
    }
  }
}
```

Response:

```json
{
  "holes": [
    {
      "region": "connection lifecycle",
      "adjacent_nodes": 4,
      "description": "No dedicated connection pool abstraction"
    },
    {
      "region": "pool metrics",
      "adjacent_nodes": 3,
      "description": "No pool health monitoring"
    },
    {
      "region": "graceful drain",
      "adjacent_nodes": 2,
      "description": "No connection drain on shutdown"
    }
  ],
  "total_holes": 9
}
```

**What happened**: m1nd activated the "database connection pooling" region of the graph and looked for *gaps* -- areas where the graph's structure predicts a node should exist but none does. These are structural holes: places where other codebases of similar shape would have components but yours does not.

This is not a linter or rule-based checker. It is topology-based gap detection. The graph's shape implies these components should exist, based on the relationships between the nodes that do exist.

## Step 6: Counterfactual Simulation

Before deleting or rewriting a module, simulate the consequences:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.counterfactual",
    "arguments": {
      "node_ids": ["file::worker.py"],
      "agent_id": "dev"
    }
  }
}
```

Response:

```json
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

**Reading the results**:

- **Depth 1**: 23 nodes directly depend on `worker.py`
- **Depth 2**: 456 more nodes depend on those 23
- **Depth 3**: 3,710 more -- a cascade explosion
- **Total**: 4,189 nodes affected out of ~9,767 (42.9% of the graph)
- **Activation lost**: 41% of the graph's total activation capacity would be disrupted

Compare this with removing `config.py`:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.counterfactual",
    "arguments": {
      "node_ids": ["file::config.py"],
      "agent_id": "dev"
    }
  }
}
```

Response:

```json
{
  "cascade": [
    {"depth": 1, "affected": 89},
    {"depth": 2, "affected": 1234},
    {"depth": 3, "affected": 1208}
  ],
  "total_affected": 2531,
  "orphaned_count": 3,
  "pct_activation_lost": 0.28
}
```

Despite `config.py` having more direct dependents (89 vs 23), its total cascade is smaller (2,531 vs 4,189). `worker.py` sits at a structural chokepoint where downstream nodes have more transitive dependencies. This insight is impossible to get from `grep` or import analysis alone -- it requires full graph traversal.

## Step 7: Hypothesis Testing (Bonus)

Test a structural claim against the graph:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.hypothesize",
    "arguments": {
      "claim": "worker depends on messaging at runtime",
      "agent_id": "dev"
    }
  }
}
```

Response:

```json
{
  "verdict": "likely_true",
  "confidence": 0.72,
  "paths_explored": 25015,
  "evidence": [
    {
      "path": [
        "file::worker.py",
        "file::process_manager.py::fn::cancel",
        "file::messaging.py"
      ],
      "hops": 2
    }
  ],
  "note": "2-hop dependency via cancel function -- invisible to grep"
}
```

The hypothesis engine explored 25,015 paths in 58ms and found a 2-hop dependency that no text search could reveal: the worker pool reaches the WhatsApp manager through a cancel function in the process manager. This is the kind of hidden coupling that causes production incidents.

## Summary: The Learning Loop

The core m1nd workflow is a feedback loop:

```
ingest  -->  activate  -->  use results  -->  learn  -->  activate again
  |                                              |
  |              (graph gets smarter)            |
  +----------------------------------------------+
```

Every `learn` call shifts edge weights. Every subsequent `activate` benefits from accumulated learning. Over sessions, the graph adapts to how *your team* thinks about *your codebase*.

Additional tools layer on top of this foundation:

| Tool | When to Use |
|------|-------------|
| `missing` | Before designing new features -- find what your codebase lacks |
| `counterfactual` | Before deleting or rewriting -- simulate the blast radius |
| `hypothesize` | When debugging -- test assumptions about hidden dependencies |
| `impact` | Before modifying a file -- understand the blast radius |
| `predict` | After modifying a file -- which other files probably need changes too |
| `trace` | When an error occurs -- map stacktraces to structural root causes |

Next: [Multi-Agent Tutorial](multi-agent.md) -- how multiple agents share one graph.
