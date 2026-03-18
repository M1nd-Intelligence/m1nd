# Perspective Tools

Twelve tools for stateful, navigable exploration of the code graph. Perspectives are m1nd's primary interface for guided codebase exploration -- they synthesize routes (weighted navigation suggestions) from a query, then let you follow, inspect, branch, and compare routes.

## Core Concepts

**Perspective**: A stateful exploration session. Created from a query, it synthesizes a set of *routes* from the current graph state. The agent navigates by following routes, which moves the *focus* and generates new routes.

**Route**: A suggested direction of exploration. Each route points to a target node and carries a score, a family classification, and a path preview.

**Focus**: The current node the perspective is centered on. Following a route moves the focus. `back` restores the previous focus.

**Mode**: Either `"anchored"` (routes stay related to an anchor node) or `"local"` (routes synthesized purely from the current focus). Anchored mode degrades to local if navigation moves more than 8 hops from the anchor.

**Route Set Version**: A version counter for the current route set. Clients must pass this in subsequent calls for staleness detection. If the graph changes between calls, the version changes and stale requests are rejected.

**Lens**: Optional filtering configuration for perspectives (dimension weights, node type filters, etc.).

---

## `m1nd.perspective.start`

Enter a perspective: creates a navigable route surface from a query. Returns the initial set of routes.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `query` | `string` | Yes | -- | Seed query for route synthesis. |
| `anchor_node` | `string` | No | -- | Anchor to a specific node. If provided, activates anchored mode where all routes maintain relevance to this node. |
| `lens` | `object` | No | -- | Starting lens configuration. Controls dimension weights, node type filters, etc. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.start",
    "arguments": {
      "agent_id": "jimi",
      "query": "session management and connection pooling",
      "anchor_node": "file::session_pool.py"
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_001",
  "mode": "anchored",
  "anchor_node": "file::session_pool.py",
  "focus_node": "file::session_pool.py",
  "routes": [
    {
      "route_id": "R_a1b2c3",
      "index": 1,
      "target_node": "file::worker_pool.py",
      "target_label": "worker_pool.py",
      "family": "structural_neighbor",
      "score": 0.89,
      "path_preview": ["session_pool.py", "worker_pool.py"],
      "reason": "High structural coupling with anchor"
    },
    {
      "route_id": "R_d4e5f6",
      "index": 2,
      "target_node": "file::process_manager.py",
      "target_label": "process_manager.py",
      "family": "causal_downstream",
      "score": 0.76,
      "path_preview": ["session_pool.py", "process_manager.py"],
      "reason": "Causal dependency via imports"
    }
  ],
  "total_routes": 12,
  "page": 1,
  "total_pages": 2,
  "route_set_version": 100,
  "cache_generation": 42,
  "suggested": "inspect R_a1b2c3 for structural details"
}
```

### When to Use

- **Guided exploration** -- when you want to explore a topic interactively
- **Codebase navigation** -- start from a known file and discover related code
- **Investigation** -- anchor to a suspicious file and follow routes to find related issues

### Related Tools

- [`m1nd.activate`](activation.md#m1ndactivate) -- one-shot query without navigation state
- [`m1nd.perspective.routes`](#m1ndperspectiveroutes) -- paginate through routes
- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- follow a route to navigate

---

## `m1nd.perspective.routes`

Browse the current route set with pagination.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective to browse. |
| `page` | `integer` | No | `1` | Page number (1-based). |
| `page_size` | `integer` | No | `6` | Routes per page. Clamped to `[1, 10]`. |
| `route_set_version` | `integer` | No | -- | Version from previous response for staleness check. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.routes",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "page": 2,
      "page_size": 6,
      "route_set_version": 100
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_001",
  "mode": "anchored",
  "mode_effective": "anchored",
  "anchor": "file::session_pool.py",
  "focus": "file::session_pool.py",
  "lens_summary": "all dimensions, no type filter",
  "page": 2,
  "total_pages": 2,
  "total_routes": 12,
  "route_set_version": 100,
  "cache_generation": 42,
  "routes": [
    {
      "route_id": "R_g7h8i9",
      "index": 7,
      "target_node": "file::config.py",
      "target_label": "config.py",
      "family": "configuration",
      "score": 0.42,
      "path_preview": ["session_pool.py", "config.py"],
      "reason": "Shared configuration key"
    }
  ],
  "suggested": null,
  "page_size_clamped": false
}
```

### When to Use

- **Pagination** -- browse all available routes when the first page was not enough
- **Full route survey** -- see the complete landscape before deciding which route to follow

### Related Tools

- [`m1nd.perspective.inspect`](#m1ndperspectiveinspect) -- expand a specific route for detail
- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- follow a route to navigate

---

## `m1nd.perspective.inspect`

Expand a route with fuller path, metrics, provenance, affinity candidates, and score breakdown. Does not change the perspective state -- purely informational.

Specify the route by either `route_id` (stable, content-addressed) or `route_index` (1-based position on the current page). Exactly one must be provided.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective containing the route. |
| `route_id` | `string` | No | -- | Stable content-addressed route ID. |
| `route_index` | `integer` | No | -- | 1-based page-local position. |
| `route_set_version` | `integer` | Yes | -- | Route set version for staleness check. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.inspect",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "route_id": "R_a1b2c3",
      "route_set_version": 100
    }
  }
}
```

### Example Response

```json
{
  "route_id": "R_a1b2c3",
  "route_index": 1,
  "family": "structural_neighbor",
  "target_node": "file::worker_pool.py",
  "target_label": "worker_pool.py",
  "target_type": "file",
  "path_preview": ["session_pool.py", "spawner.py", "worker_pool.py"],
  "family_explanation": "Direct structural neighbor via import chain",
  "score": 0.89,
  "score_breakdown": {
    "local_activation": 0.92,
    "path_coherence": 0.88,
    "novelty": 0.75,
    "anchor_relevance": 0.95,
    "continuity": 0.82
  },
  "provenance": {
    "source_path": "backend/worker_pool.py",
    "line_start": 1,
    "line_end": 312,
    "namespace": null,
    "provenance_stale": false
  },
  "peek_available": true,
  "affinity_candidates": [
    { "node_id": "file::fast_pool.py", "label": "fast_pool.py", "strength": 0.72, "reason": "Similar pool pattern" }
  ],
  "response_chars": 450
}
```

### When to Use

- **Before following** -- understand what a route leads to before committing
- **Score analysis** -- see why a route was ranked high or low
- **Provenance check** -- verify the source file exists and is not stale

### Related Tools

- [`m1nd.perspective.peek`](#m1ndperspectivepeek) -- extract actual code content
- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- navigate to the route target
- [`m1nd.perspective.affinity`](#m1ndperspectiveaffinity) -- deeper affinity analysis

---

## `m1nd.perspective.peek`

Extract a small relevant code or documentation slice from a route's target node. Reads the actual source file and returns the relevant excerpt. Security-checked: only reads files within the ingested graph scope.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective containing the route. |
| `route_id` | `string` | No | -- | Stable content-addressed route ID. |
| `route_index` | `integer` | No | -- | 1-based page-local position. |
| `route_set_version` | `integer` | Yes | -- | Route set version for staleness check. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.peek",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "route_index": 1,
      "route_set_version": 100
    }
  }
}
```

### Example Response

```json
{
  "route_id": "R_a1b2c3",
  "route_index": 1,
  "target_node": "file::worker_pool.py",
  "content": {
    "excerpt": "class WorkerPool:\n    \"\"\"Manages a pool of CLI subprocess workers.\"\"\"\n    def __init__(self, max_workers=8):\n        self.pool = {}\n        self.max_workers = max_workers\n        ...",
    "file_path": "backend/worker_pool.py",
    "line_start": 15,
    "line_end": 45,
    "truncated": true
  }
}
```

### When to Use

- **Quick preview** -- see the actual code at a route target without leaving m1nd
- **Decision making** -- peek at code to decide whether to follow a route
- **Investigation** -- read suspicious code without opening a file

### Related Tools

- [`m1nd.perspective.inspect`](#m1ndperspectiveinspect) -- structural information about the route (no code content)
- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- navigate to the route target

---

## `m1nd.perspective.follow`

Follow a route: move focus to the target node and synthesize new routes from the new position. This is the primary navigation action in a perspective. Updates the navigation history for `back`.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective to navigate within. |
| `route_id` | `string` | No | -- | Stable content-addressed route ID. |
| `route_index` | `integer` | No | -- | 1-based page-local position. |
| `route_set_version` | `integer` | Yes | -- | Route set version for staleness check. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.follow",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "route_id": "R_a1b2c3",
      "route_set_version": 100
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_001",
  "previous_focus": "file::session_pool.py",
  "new_focus": "file::worker_pool.py",
  "mode": "anchored",
  "mode_effective": "anchored",
  "routes": [
    {
      "route_id": "R_j1k2l3",
      "index": 1,
      "target_node": "file::process_manager.py",
      "target_label": "process_manager.py",
      "family": "causal_downstream",
      "score": 0.85,
      "path_preview": ["worker_pool.py", "process_manager.py"],
      "reason": "Direct caller of worker pool submit"
    },
    {
      "route_id": "R_m4n5o6",
      "index": 2,
      "target_node": "file::spawner.py",
      "target_label": "spawner.py",
      "family": "structural_neighbor",
      "score": 0.71,
      "path_preview": ["worker_pool.py", "spawner.py"],
      "reason": "Co-manages subprocess lifecycle"
    }
  ],
  "total_routes": 8,
  "page": 1,
  "total_pages": 2,
  "route_set_version": 101,
  "cache_generation": 42,
  "suggested": "inspect R_j1k2l3 -- high causal relevance"
}
```

### When to Use

- **Navigation** -- this is the primary way to move through the graph
- **Guided exploration** -- follow high-scoring routes to discover related code
- **Investigation** -- follow causal chains to trace a bug

### Related Tools

- [`m1nd.perspective.back`](#m1ndperspectiveback) -- undo a follow
- [`m1nd.perspective.branch`](#m1ndperspectivebranch) -- fork before following to explore alternatives

---

## `m1nd.perspective.suggest`

Get the next best move suggestion based on navigation history. Analyzes the routes, the current focus, and the history of followed routes to recommend what to do next.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective to get a suggestion for. |
| `route_set_version` | `integer` | Yes | -- | Route set version for staleness check. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.suggest",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "route_set_version": 101
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_001",
  "suggestion": {
    "action": "follow",
    "route_id": "R_j1k2l3",
    "reason": "High-scoring causal route not yet explored. process_manager.py is a hub node connecting to 47 downstream modules.",
    "confidence": 0.82
  }
}
```

### When to Use

- **Undecided** -- when you are not sure which route to follow
- **Systematic exploration** -- let m1nd guide you through the most informative path
- **Investigation** -- follow the suggestion trail to efficient root cause discovery

### Related Tools

- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- follow the suggested route
- [`m1nd.perspective.inspect`](#m1ndperspectiveinspect) -- inspect the suggested route first

---

## `m1nd.perspective.affinity`

Discover probable connections a route target might have. Returns affinity candidates -- nodes that are not directly connected to the target but show structural affinity (similar neighborhoods, shared patterns).

Epistemic notice: these are **probable** connections, not verified graph edges.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective containing the route. |
| `route_id` | `string` | No | -- | Stable content-addressed route ID. |
| `route_index` | `integer` | No | -- | 1-based page-local position. |
| `route_set_version` | `integer` | Yes | -- | Route set version for staleness check. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.affinity",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "route_id": "R_j1k2l3",
      "route_set_version": 101
    }
  }
}
```

### Example Response

```json
{
  "route_id": "R_j1k2l3",
  "target_node": "file::process_manager.py",
  "notice": "Probable connections, not verified edges.",
  "candidates": [
    { "node_id": "file::healing_manager.py", "label": "healing_manager.py", "strength": 0.68, "reason": "Similar lifecycle management pattern" },
    { "node_id": "file::autonomous_daemon.py", "label": "autonomous_daemon.py", "strength": 0.52, "reason": "Shared subprocess management trait" }
  ]
}
```

### When to Use

- **Discovery** -- find non-obvious connections that the graph does not yet have
- **Architecture analysis** -- identify modules that should be connected but are not
- **Ghost edge validation** -- affinity candidates often become confirmed edges after investigation

### Related Tools

- [`m1nd.missing`](exploration.md#m1ndmissing) -- finds structural holes (broader scope)
- [`m1nd.perspective.inspect`](#m1ndperspectiveinspect) -- affinity candidates are also included in inspect output

---

## `m1nd.perspective.branch`

Fork the current navigation state into a new perspective branch. The branch starts at the same focus with the same route set, but future navigation in the branch is independent. Useful for exploring alternatives without losing your place.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective to branch from. |
| `branch_name` | `string` | No | -- | Optional branch name. Auto-generated if not provided. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.branch",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001",
      "branch_name": "auth-path"
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_001",
  "branch_perspective_id": "persp_jimi_002",
  "branch_name": "auth-path",
  "branched_from_focus": "file::worker_pool.py"
}
```

### When to Use

- **Alternative exploration** -- explore two different paths from the same point
- **Comparative analysis** -- branch, follow different routes, then compare
- **Safe investigation** -- branch before following a risky route

### Related Tools

- [`m1nd.perspective.compare`](#m1ndperspectivecompare) -- compare the branch with the original
- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- navigate within the branch

---

## `m1nd.perspective.back`

Navigate back to the previous focus, restoring the checkpoint state. Like browser back navigation. Pops the latest navigation event from the history stack.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective to navigate back in. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.back",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_001"
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_001",
  "restored_focus": "file::session_pool.py",
  "restored_mode": "anchored",
  "routes": [
    {
      "route_id": "R_a1b2c3",
      "index": 1,
      "target_node": "file::worker_pool.py",
      "target_label": "worker_pool.py",
      "family": "structural_neighbor",
      "score": 0.89,
      "path_preview": ["session_pool.py", "worker_pool.py"],
      "reason": "High structural coupling with anchor"
    }
  ],
  "total_routes": 12,
  "page": 1,
  "total_pages": 2,
  "route_set_version": 102,
  "cache_generation": 42
}
```

### When to Use

- **Undo navigation** -- go back after following a wrong route
- **Breadth-first exploration** -- follow a route, come back, follow another

### Related Tools

- [`m1nd.perspective.follow`](#m1ndperspectivefollow) -- the forward navigation that `back` undoes
- [`m1nd.perspective.branch`](#m1ndperspectivebranch) -- alternative: branch instead of back+follow

---

## `m1nd.perspective.compare`

Compare two perspectives on shared/unique nodes and dimension deltas. Both perspectives must belong to the same agent (V1 restriction).

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id_a` | `string` | Yes | -- | First perspective ID. |
| `perspective_id_b` | `string` | Yes | -- | Second perspective ID. |
| `dimensions` | `string[]` | No | `[]` | Dimensions to compare on. Empty = all dimensions. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.compare",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id_a": "persp_jimi_001",
      "perspective_id_b": "persp_jimi_002"
    }
  }
}
```

### Example Response

```json
{
  "perspective_id_a": "persp_jimi_001",
  "perspective_id_b": "persp_jimi_002",
  "shared_nodes": ["worker_pool.py", "process_manager.py", "session_pool.py"],
  "unique_to_a": ["spawner.py", "config.py"],
  "unique_to_b": ["auth_discovery.py", "middleware.py"],
  "dimension_deltas": [
    { "dimension": "structural", "score_a": 0.82, "score_b": 0.75, "delta": 0.07 },
    { "dimension": "semantic", "score_a": 0.65, "score_b": 0.88, "delta": -0.23 },
    { "dimension": "causal", "score_a": 0.71, "score_b": 0.45, "delta": 0.26 }
  ],
  "response_chars": 380
}
```

### When to Use

- **Branch comparison** -- compare a branch perspective with the original
- **Multi-approach analysis** -- see how two different starting queries reach different parts of the graph
- **Investigation correlation** -- find shared nodes between two independent investigations

### Related Tools

- [`m1nd.perspective.branch`](#m1ndperspectivebranch) -- create branches to compare
- [`m1nd.trail.merge`](memory.md#m1ndtrailmerge) -- merge investigation trails (deeper than compare)

---

## `m1nd.perspective.list`

List all perspectives for an agent. Returns compact summaries with status, focus, route count, and memory usage.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.list",
    "arguments": {
      "agent_id": "jimi"
    }
  }
}
```

### Example Response

```json
{
  "agent_id": "jimi",
  "perspectives": [
    {
      "perspective_id": "persp_jimi_001",
      "mode": "anchored",
      "focus_node": "file::worker_pool.py",
      "route_count": 12,
      "nav_event_count": 3,
      "stale": false,
      "created_at_ms": 1710300000000,
      "last_accessed_ms": 1710300500000
    },
    {
      "perspective_id": "persp_jimi_002",
      "mode": "anchored",
      "focus_node": "file::auth_discovery.py",
      "route_count": 8,
      "nav_event_count": 1,
      "stale": false,
      "created_at_ms": 1710300200000,
      "last_accessed_ms": 1710300400000
    }
  ],
  "total_memory_bytes": 24576
}
```

### When to Use

- **Session overview** -- see all active perspectives
- **Memory management** -- check total memory usage and close stale perspectives
- **Multi-perspective work** -- switch between perspectives

### Related Tools

- [`m1nd.perspective.close`](#m1ndperspectiveclose) -- close perspectives you no longer need

---

## `m1nd.perspective.close`

Close a perspective and release associated locks. Frees memory and stops route caching for this perspective. Cascade-releases any locks that were associated with this perspective.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `perspective_id` | `string` | Yes | -- | Perspective to close. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 12,
  "method": "tools/call",
  "params": {
    "name": "m1nd.perspective.close",
    "arguments": {
      "agent_id": "jimi",
      "perspective_id": "persp_jimi_002"
    }
  }
}
```

### Example Response

```json
{
  "perspective_id": "persp_jimi_002",
  "closed": true,
  "locks_released": ["lock_jimi_003"]
}
```

### When to Use

- **Cleanup** -- close perspectives after investigation is complete
- **Memory pressure** -- close unused perspectives to free memory
- **Session end** -- close all perspectives before ending a session

### Related Tools

- [`m1nd.perspective.list`](#m1ndperspectivelist) -- find perspectives to close

---

## Full Workflow Example

A complete perspective exploration session, from start to close:

```
1. START a perspective from a query
   m1nd.perspective.start(query="authentication and session security")
   -> persp_jimi_001, 12 routes, anchored to auth_discovery.py

2. BROWSE routes (page 1 already returned by start)
   Routes: R01 auth_discovery.py, R02 middleware.py, R03 principal_registry.py, ...

3. INSPECT a high-scoring route before following
   m1nd.perspective.inspect(route_id="R01", route_set_version=100)
   -> score_breakdown, provenance, affinity_candidates

4. FOLLOW the route to navigate
   m1nd.perspective.follow(route_id="R01", route_set_version=100)
   -> new focus: auth_discovery.py, 8 new routes, version=101

5. ASK for a suggestion on what to do next
   m1nd.perspective.suggest(route_set_version=101)
   -> "follow R_x: principal_registry.py, high causal relevance"

6. BRANCH before exploring a risky path
   m1nd.perspective.branch(branch_name="session-path")
   -> new persp_jimi_002, same focus

7. FOLLOW different routes in each branch
   Branch 1: follow toward principal_registry.py
   Branch 2: follow toward session_pool.py

8. COMPARE the two branches
   m1nd.perspective.compare(perspective_id_a="persp_jimi_001", perspective_id_b="persp_jimi_002")
   -> shared: [auth_discovery.py], unique_to_a: [principal_registry.py], unique_to_b: [session_pool.py]

9. BACK to undo the last follow in branch 1
   m1nd.perspective.back()
   -> restored focus: auth_discovery.py

10. CLOSE both perspectives when done
    m1nd.perspective.close(perspective_id="persp_jimi_001")
    m1nd.perspective.close(perspective_id="persp_jimi_002")
```

This workflow demonstrates the full exploration cycle: start with a query, navigate through the graph by following routes, branch to explore alternatives, compare branches, and close when done.
