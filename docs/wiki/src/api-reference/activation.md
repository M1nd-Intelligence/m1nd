# Activation Tools

Three tools for querying the connectome through spreading activation, task-based priming, and resonance analysis.

---

## `m1nd.activate`

Spreading activation query across the connectome. The primary search tool -- propagates signal from seed nodes through the graph across four dimensions (structural, semantic, temporal, causal), with XLR noise cancellation and optional ghost edge / structural hole detection.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | `string` | Yes | -- | Search query for spreading activation. Matched against node labels, tags, and provenance to find seed nodes. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `top_k` | `integer` | No | `20` | Number of top activated nodes to return. |
| `dimensions` | `string[]` | No | `["structural", "semantic", "temporal", "causal"]` | Activation dimensions to include. Each dimension contributes independently to the final activation score. Values: `"structural"`, `"semantic"`, `"temporal"`, `"causal"`. |
| `xlr` | `boolean` | No | `true` | Enable XLR noise cancellation. Filters low-confidence activations to reduce false positives. |
| `include_ghost_edges` | `boolean` | No | `true` | Include ghost edge detection. Ghost edges are probable but unconfirmed connections inferred from activation patterns. |
| `include_structural_holes` | `boolean` | No | `false` | Include structural hole detection. Identifies nodes that should be connected but are not. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "m1nd.activate",
    "arguments": {
      "agent_id": "jimi",
      "query": "session pool management",
      "top_k": 5,
      "include_ghost_edges": true
    }
  }
}
```

### Example Response

```json
{
  "query": "session pool management",
  "seeds": [
    { "node_id": "file::session_pool.py", "label": "session_pool.py", "relevance": 0.95 }
  ],
  "activated": [
    {
      "node_id": "file::session_pool.py",
      "label": "session_pool.py",
      "type": "file",
      "activation": 0.89,
      "dimensions": { "structural": 0.92, "semantic": 0.95, "temporal": 0.78, "causal": 0.71 },
      "pagerank": 0.635,
      "tags": ["session", "pool"],
      "provenance": {
        "source_path": "backend/session_pool.py",
        "line_start": 1,
        "line_end": 245,
        "canonical": true
      }
    },
    {
      "node_id": "file::session_pool.py::class::SessionPool",
      "label": "SessionPool",
      "type": "class",
      "activation": 0.84,
      "dimensions": { "structural": 0.88, "semantic": 0.91, "temporal": 0.72, "causal": 0.65 },
      "pagerank": 0.412,
      "tags": ["pool", "session"],
      "provenance": {
        "source_path": "backend/session_pool.py",
        "line_start": 15,
        "line_end": 180,
        "canonical": true
      }
    }
  ],
  "ghost_edges": [
    {
      "source": "session_pool.py",
      "target": "healing_manager.py",
      "shared_dimensions": ["semantic", "causal"],
      "strength": 0.34
    }
  ],
  "structural_holes": [],
  "plasticity": {
    "edges_strengthened": 12,
    "edges_decayed": 3,
    "ltp_events": 1,
    "priming_nodes": 5
  },
  "elapsed_ms": 31.2
}
```

### When to Use

- **Primary search** -- the default way to ask "what in the codebase relates to X?"
- **Exploration** -- when you know a topic but not the specific files
- **Context building** -- before working on a feature, activate its topic to find all related code
- **Gap detection** -- enable `include_structural_holes` to find missing connections

### Side Effects

Activate has **plasticity side effects**: it strengthens edges between activated nodes and decays inactive edges. This makes the graph learn from usage patterns over time.

### Related Tools

- [`m1nd.warmup`](#m1ndwarmup) -- activate + prime for a specific task
- [`m1nd.seek`](exploration.md#m1ndseek) -- intent-aware search (finds code by purpose, not just keywords)
- [`m1nd.perspective.start`](perspectives.md#m1ndperspectivestart) -- wraps activate into a navigable perspective
- [`m1nd.learn`](memory.md#m1ndlearn) -- explicitly provide feedback on activation results

---

## `m1nd.warmup`

Task-based warmup and priming. Activates the graph around a task description and applies a temporary boost to relevant nodes, preparing the graph for focused work. The boost decays naturally over time.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `task_description` | `string` | Yes | -- | Description of the task to warm up for. Natural language. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `boost_strength` | `number` | No | `0.15` | Priming boost strength applied to relevant nodes. Range: 0.0 to 1.0. Higher values make the primed nodes more dominant in subsequent queries. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "m1nd.warmup",
    "arguments": {
      "agent_id": "jimi",
      "task_description": "Refactor the WhatsApp message routing to support group chats",
      "boost_strength": 0.2
    }
  }
}
```

### Example Response

```json
{
  "task": "Refactor the WhatsApp message routing to support group chats",
  "primed_nodes": 23,
  "top_primed": [
    { "node_id": "file::whatsapp_routes.py", "label": "whatsapp_routes.py", "boost": 0.2 },
    { "node_id": "file::whatsapp_manager.py", "label": "whatsapp_manager.py", "boost": 0.18 },
    { "node_id": "file::whatsapp_models.py", "label": "whatsapp_models.py", "boost": 0.15 },
    { "node_id": "file::chat_handler.py", "label": "chat_handler.py", "boost": 0.12 }
  ],
  "elapsed_ms": 18.5
}
```

### When to Use

- **Session start** -- warm up before a focused work session to bias the graph toward relevant code
- **Context switch** -- when changing tasks, warm up the new topic
- **Before complex queries** -- warmup biases subsequent `activate`, `impact`, and `why` queries toward the warmed-up region

### Side Effects

Applies temporary priming boosts to node activations. These boosts decay naturally and are NOT persisted across server restarts.

### Related Tools

- [`m1nd.activate`](#m1ndactivate) -- raw activation query without the priming boost
- [`m1nd.trail.resume`](memory.md#m1ndtrailresume) -- restores a full investigation context including activation boosts

---

## `m1nd.resonate`

Resonance analysis: standing waves, harmonics, sympathetic pairs, and resonant frequencies in the graph. Identifies nodes that form natural clusters of mutual reinforcement -- the "harmonics" of the connectome.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | `string` | No | -- | Search query to find seed nodes for resonance analysis. Provide either `query` or `node_id` (or neither for global resonance). |
| `node_id` | `string` | No | -- | Specific node identifier to use as seed. Alternative to `query`. |
| `agent_id` | `string` | Yes | -- | Calling agent identifier. |
| `top_k` | `integer` | No | `20` | Number of top resonance results to return. |

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "m1nd.resonate",
    "arguments": {
      "agent_id": "jimi",
      "query": "authentication flow",
      "top_k": 10
    }
  }
}
```

### Example Response

```json
{
  "seed": "authentication flow",
  "harmonics": [
    {
      "node_id": "file::auth_discovery.py",
      "label": "auth_discovery.py",
      "amplitude": 0.92,
      "harmonic_order": 1
    },
    {
      "node_id": "file::middleware.py",
      "label": "middleware.py",
      "amplitude": 0.71,
      "harmonic_order": 2
    },
    {
      "node_id": "file::principal_registry.py",
      "label": "principal_registry.py",
      "amplitude": 0.68,
      "harmonic_order": 2
    }
  ],
  "sympathetic_pairs": [
    { "a": "auth_discovery.py", "b": "principal_registry.py", "coupling": 0.84 }
  ],
  "elapsed_ms": 45.0
}
```

### When to Use

- **Deep structural analysis** -- find natural clusters of mutually reinforcing code
- **Pattern discovery** -- identify which modules form coherent subsystems
- **Architecture review** -- see which modules resonate together (and which do not)
- **Refactoring** -- resonance groups suggest natural module boundaries

### Side Effects

Read-only. No plasticity side effects.

### Related Tools

- [`m1nd.activate`](#m1ndactivate) -- simpler spreading activation without harmonic analysis
- [`m1nd.fingerprint`](analysis.md#m1ndfingerprint) -- finds structurally equivalent nodes
- [`m1nd.missing`](exploration.md#m1ndmissing) -- finds gaps in the resonance structure
