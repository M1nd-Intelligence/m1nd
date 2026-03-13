# Multi-Agent Usage

m1nd is designed for multi-agent systems. One m1nd instance serves many agents simultaneously. This tutorial covers agent identity, concurrent access, perspective isolation, trail sharing, and a real-world example of how a production system uses m1nd with a fleet of agents.

## How It Works

m1nd runs as a single process with a single shared graph. Multiple MCP clients connect to the same instance (or multiple instances reading from the same persisted state). Every tool call includes an `agent_id` parameter that identifies the caller.

```
  Agent A (Claude Code)  ----+
                              |
  Agent B (Cursor)       ----+----> m1nd-mcp (single graph)
                              |
  Agent C (custom agent) ----+
```

The graph is shared. Learning by one agent benefits all agents. Perspectives are isolated per agent. Trails can be shared across agents.

## Agent ID Conventions

Every m1nd tool requires an `agent_id` parameter. This is a free-form string, but consistent naming matters:

```
agent_id: "orchestrator"    -- orchestrator
agent_id: "auditor-1"       -- security hardening agent
agent_id: "builder-api"     -- API building agent
agent_id: "analyzer-core"   -- performance analysis agent
```

**Recommended convention**: `{archetype}-{task}` for short-lived task agents, simple names for persistent agents.

Rules:
- Agent IDs are case-sensitive
- Use lowercase with hyphens
- m1nd tracks all agent IDs it has seen (visible in `health` output)
- Agent ID determines perspective ownership and trail ownership

## Shared Graph, Individual Learning

When Agent A calls `learn` with feedback, the edge weight changes are visible to all agents immediately:

```jsonc
// Agent A: "session_pool.py was useful for my auth investigation"
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.learn",
    "arguments": {
      "query": "authentication flow",
      "agent_id": "agent-a",
      "feedback": "correct",
      "node_ids": ["file::session_pool.py"]
    }
  }
}
// -> 740 edges strengthened
```

```jsonc
// Agent B: immediately benefits from stronger session_pool.py edges
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "query": "session management",
      "agent_id": "agent-b",
      "top_k": 5
    }
  }
}
// -> session_pool.py scores higher than it would have before Agent A's feedback
```

This is by design. The graph represents collective intelligence about the codebase. Every agent's learning contributes to the whole.

## Perspective Isolation

Perspectives are m1nd's navigation system -- a stateful exploration session anchored to a node, with a route surface, breadcrumb history, and focus tracking.

Perspectives are isolated per agent. Agent A's perspectives are not visible to Agent B, and navigation in one perspective does not affect another.

### Agent A: Start a Perspective

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.start",
    "arguments": {
      "agent_id": "agent-a",
      "query": "authentication middleware",
      "anchor_node": "file::middleware.py"
    }
  }
}
```

Response:

```json
{
  "perspective_id": "persp-a1b2c3",
  "focus": "file::middleware.py",
  "routes": [
    {"route_id": "r-001", "target": "file::auth.py", "label": "imports", "score": 0.92},
    {"route_id": "r-002", "target": "file::session.py", "label": "calls", "score": 0.78}
  ],
  "route_set_version": 1
}
```

### Agent B: Has Its Own Perspectives

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.start",
    "arguments": {
      "agent_id": "agent-b",
      "query": "worker pool scaling"
    }
  }
}
```

Agent B gets a completely independent perspective. Its routes, focus, and history are separate from Agent A's.

### Listing Perspectives

Each agent only sees its own:

```jsonc
// Agent A sees only its perspectives
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.list",
    "arguments": {"agent_id": "agent-a"}
  }
}
// -> [{"perspective_id": "persp-a1b2c3", "focus": "file::middleware.py", ...}]

// Agent B sees only its perspectives
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.list",
    "arguments": {"agent_id": "agent-b"}
  }
}
// -> [{"perspective_id": "persp-d4e5f6", "focus": "file::worker_pool.py", ...}]
```

### Comparing Perspectives

You can compare two perspectives owned by the same agent using `perspective.compare`. This is useful for discovering where two investigation branches overlap:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.compare",
    "arguments": {
      "agent_id": "agent-a",
      "perspective_id_a": "persp-a1b2c3",
      "perspective_id_b": "persp-a4d5e6"
    }
  }
}
```

Response:

```json
{
  "shared_nodes": ["file::process_manager.py", "file::config.py"],
  "unique_to_a": ["file::auth.py", "file::middleware.py"],
  "unique_to_b": ["file::worker_pool.py", "file::spawner.py"],
  "dimension_deltas": {
    "structural": 0.12,
    "semantic": 0.34,
    "temporal": 0.08
  }
}
```

> **Note**: In V1, both perspectives must belong to the same agent. Cross-agent perspective comparison is planned for V2. To compare findings across agents, use the trail system (trail.save + trail.merge) instead.

## Lock System for Concurrent Access

When multiple agents might modify the same region of the codebase simultaneously, the lock system prevents conflicts.

### Agent A: Lock the Auth Region

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.lock.create",
    "arguments": {
      "agent_id": "agent-a",
      "scope": "subgraph",
      "root_nodes": ["file::auth.py"],
      "radius": 2
    }
  }
}
```

Response:

```json
{
  "lock_id": "lock-xyz789",
  "nodes_locked": 156,
  "edges_locked": 423,
  "scope": "subgraph",
  "radius": 2
}
```

### Set a Watch Strategy

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.lock.watch",
    "arguments": {
      "agent_id": "agent-a",
      "lock_id": "lock-xyz789",
      "strategy": "on_ingest"
    }
  }
}
```

Now, whenever any agent triggers an ingest that touches the locked region, the lock records the changes.

### Check for Changes

After Agent B modifies some code and re-ingests:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.lock.diff",
    "arguments": {
      "agent_id": "agent-a",
      "lock_id": "lock-xyz789"
    }
  }
}
```

Response (in 0.08 microseconds):

```json
{
  "new_nodes": ["file::auth.py::fn::validate_token_v2"],
  "removed_nodes": [],
  "weight_changes": 3,
  "structural_changes": true
}
```

Agent A now knows exactly what changed in its locked region, without scanning the entire graph.

### Rebase or Release

```jsonc
// Accept changes and update baseline
{"method":"tools/call","params":{"name":"m1nd.lock.rebase","arguments":{
  "agent_id":"agent-a","lock_id":"lock-xyz789"
}}}

// Or release when done
{"method":"tools/call","params":{"name":"m1nd.lock.release","arguments":{
  "agent_id":"agent-a","lock_id":"lock-xyz789"
}}}
```

## Trail Sharing

Trails are investigation snapshots: visited nodes, hypotheses, conclusions, open questions, and activation boosts. They persist across sessions and can be shared between agents.

### Agent A: Save an Investigation

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.save",
    "arguments": {
      "agent_id": "agent-a",
      "label": "auth-token-leak-investigation",
      "hypotheses": [
        {
          "statement": "Auth tokens leak through session pool",
          "confidence": 0.7,
          "status": "investigating"
        },
        {
          "statement": "Rate limiter missing from auth chain",
          "confidence": 0.9,
          "status": "confirmed"
        }
      ],
      "open_questions": [
        "Does the healing manager observe token lifecycle?",
        "Is there a token rotation policy?"
      ],
      "tags": ["security", "auth", "session"]
    }
  }
}
```

Response:

```json
{
  "trail_id": "trail-abc123",
  "nodes_captured": 47,
  "hypotheses_saved": 2,
  "activation_boosts_saved": 12
}
```

### Agent B: Resume Agent A's Trail

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.resume",
    "arguments": {
      "agent_id": "agent-b",
      "trail_id": "trail-abc123"
    }
  }
}
```

Response:

```json
{
  "trail_id": "trail-abc123",
  "nodes_reactivated": 47,
  "stale_nodes": 2,
  "hypotheses_restored": 2,
  "hypotheses_downgraded": 0
}
```

Agent B now has Agent A's exact cognitive context: the same nodes are activated, the same hypotheses are loaded, and any stale nodes (changed since the trail was saved) are flagged.

### Merging Trails from Multiple Agents

When two agents investigate independently and you want to combine findings:

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.merge",
    "arguments": {
      "agent_id": "orchestrator",
      "trail_ids": ["trail-abc123", "trail-def456"],
      "label": "combined-auth-investigation"
    }
  }
}
```

Response:

```json
{
  "merged_trail_id": "trail-merged-789",
  "total_nodes": 83,
  "shared_nodes": 12,
  "conflicts": [
    {
      "node": "file::auth.py",
      "trail_a_hypothesis": "token leak source",
      "trail_b_hypothesis": "not involved"
    }
  ],
  "conflict_count": 3
}
```

The merge automatically detects where independent investigations converged (12 shared nodes) and where they conflict (3 disagreements). This is essential for synthesizing multi-agent research.

### Browsing Trails

```jsonc
{
  "method": "tools/call",
  "params": {
    "name": "m1nd.trail.list",
    "arguments": {
      "agent_id": "orchestrator",
      "filter_tags": ["security"]
    }
  }
}
```

## Real-World Example: Multi-Agent Production System

The following example is based on a production system that uses m1nd as its shared code intelligence layer. Here is how it works in practice:

### Architecture

```
orchestrator
  |
  +-- auditor-1 (security agent)       --+
  +-- builder-api (API builder)         --+-- All share one m1nd instance
  +-- analyzer-core (performance)       --+
  +-- watcher-files (file watcher)      --+
  +-- reviewer-quality (code reviewer)  --+
  +-- ... additional agents            --+
```

One m1nd instance serves all agents. The graph covers a Python backend, a React frontend, and server infrastructure.

### Orchestrator Boot Sequence

When the orchestrator starts a session:

```jsonc
// Step 1: Check m1nd health
{"method":"tools/call","params":{"name":"m1nd.health","arguments":{"agent_id":"orchestrator"}}}

// Step 2: Check what changed since last session
{"method":"tools/call","params":{"name":"m1nd.drift","arguments":{"agent_id":"orchestrator","since":"last_session"}}}

// Step 3: Re-ingest if the graph is stale
{"method":"tools/call","params":{"name":"m1nd.ingest","arguments":{
  "path":"/project/backend","agent_id":"orchestrator","incremental":true
}}}
```

### Task Delegation with Graph Context

When the orchestrator delegates a security hardening task:

```jsonc
// Before spawning the security agent, get blast radius context
{"method":"tools/call","params":{"name":"m1nd.impact","arguments":{
  "node_id":"file::auth.py","agent_id":"orchestrator"
}}}

// Warm up the graph for the security task
{"method":"tools/call","params":{"name":"m1nd.warmup","arguments":{
  "task_description":"harden authentication token validation","agent_id":"orchestrator"
}}}

// The security agent then uses the primed graph
{"method":"tools/call","params":{"name":"m1nd.activate","arguments":{
  "query":"token validation vulnerabilities","agent_id":"auditor-1"
}}}
```

### Collective Learning

After each agent completes its task, it provides feedback:

```jsonc
// Security agent found useful results
{"method":"tools/call","params":{"name":"m1nd.learn","arguments":{
  "query":"token validation vulnerabilities",
  "agent_id":"auditor-1",
  "feedback":"correct",
  "node_ids":["file::auth.py","file::middleware.py","file::session_pool.py"]
}}}

// Performance agent found different useful results
{"method":"tools/call","params":{"name":"m1nd.learn","arguments":{
  "query":"connection pool bottleneck",
  "agent_id":"analyzer-core",
  "feedback":"correct",
  "node_ids":["file::worker_pool.py","file::process_manager.py"]
}}}
```

Over a session with many agents, the graph accumulates thousands of learning signals. Each agent benefits from every other agent's discoveries.

### Investigation Handoff

When Agent A finds something that Agent B needs to investigate:

```jsonc
// Agent A saves its investigation
{"method":"tools/call","params":{"name":"m1nd.trail.save","arguments":{
  "agent_id":"auditor-1",
  "label":"session-hijack-vector",
  "tags":["security","critical"]
}}}

// Orchestrator merges with Agent B's independent findings
{"method":"tools/call","params":{"name":"m1nd.trail.merge","arguments":{
  "agent_id":"orchestrator",
  "trail_ids":["trail-auditor-001","trail-analyzer-002"]
}}}
```

## Best Practices

1. **Use consistent agent IDs.** The same agent should always use the same ID across sessions. This enables drift detection and trail continuity.

2. **Learn after every useful activation.** The more feedback the graph gets, the smarter it becomes. Make `learn` calls automatic in your agent loop.

3. **Use locks for overlapping work.** If two agents might modify the same code region, lock it first. Lock diffs are essentially free (0.08 microseconds).

4. **Save trails at investigation checkpoints.** Trails are cheap to save and invaluable for handoff, resume, and post-mortem analysis.

5. **Merge trails for synthesis.** When multiple agents investigate the same area independently, merge their trails to find convergence and conflicts.

6. **Warm up before focused tasks.** `warmup` primes the graph for a specific task, boosting relevant regions before the agent starts querying.

7. **Use `drift` at session start.** After any period of inactivity, check what changed. This recovers context efficiently.
