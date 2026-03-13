# Hebbian Plasticity

m1nd's graph learns. Every query changes the edge weights. Paths that lead to useful results get stronger. Paths that lead to noise get weaker. Over time, the graph evolves to match how *your* team thinks about *your* codebase. No other code intelligence tool does this.

## The neuroscience principle

In 1949, Donald Hebb proposed a theory of synaptic learning: *"When an axon of cell A is near enough to excite cell B and repeatedly or persistently takes part in firing it, some growth process or metabolic change takes place in one or both cells such that A's efficiency, as one of the cells firing B, is increased."*

The popular summary: **neurons that fire together wire together.**

The converse is equally important: neurons that fire independently weaken their connections. This bidirectional learning -- strengthening co-active pathways, weakening inactive ones -- is the foundation of how biological neural networks adapt to experience.

m1nd applies this principle to code graphs. Nodes are modules. Edges are relationships. "Firing" means being activated by a spreading activation query. When an agent confirms that a result was useful, the edges connecting those activated nodes strengthen. When the agent marks results as wrong, those paths weaken. The graph remembers what worked.

## The five-step learning cycle

Every query that passes through m1nd's `PlasticityEngine` triggers a five-step update cycle. This runs automatically -- no explicit training phase required.

### Step 1: Hebbian strengthening

For every edge where both the source and target were activated in the query results, the weight increases:

```
delta_w = learning_rate * activation_source * activation_target
new_weight = min(current_weight + delta_w, weight_cap)
```

The default learning rate is **0.08** (from `LearningRate::DEFAULT`). The weight cap is **3.0** -- no edge can grow stronger than 3x its original weight. This prevents runaway positive feedback.

From the source code:

```rust
// Hebbian: delta_w = lr * act_src * act_tgt
let delta = lr * src_act.get() * tgt_act;
let new_weight = (current + delta).min(cap);
```

The product `activation_source * activation_target` means that strongly co-activated pairs get the largest boost. A pair where both nodes scored 0.8 gets `0.08 * 0.8 * 0.8 = 0.051` added to their edge weight. A pair where one scored 0.2 gets only `0.08 * 0.8 * 0.2 = 0.013`. This is faithful to Hebb's rule: the strength of the update is proportional to the correlation of the firing.

### Step 2: Synaptic decay

Edges whose source nodes were **not** activated in this query decay slightly:

```
new_weight = max(current_weight * (1 - decay_rate), weight_floor)
```

The default decay rate is **0.005** per query (0.5%). The weight floor is **0.05** -- edges never decay below 5% of their original weight. This ensures that even unused paths retain some connectivity, preventing the graph from fragmenting.

```rust
let decay_factor = 1.0 - self.config.decay_rate.get(); // 0.995
let new_weight = (current * decay_factor).max(floor);
```

The asymmetry is intentional. Strengthening applies a fixed delta (additive). Decay applies a multiplicative factor. This means strong edges resist decay (a weight-3.0 edge loses 0.015 per query) while weak edges decay faster in relative terms (a weight-0.1 edge loses 0.0005 per query). Frequently activated paths grow monotonically. Rarely activated paths slowly fade but never disappear.

### Step 3: Long-Term Potentiation / Long-Term Depression

After an edge has been strengthened **5 consecutive times** (the LTP threshold), it receives a one-time **+0.15** bonus:

```rust
if !graph.edge_plasticity.ltp_applied[j]
    && graph.edge_plasticity.strengthen_count[j] >= self.config.ltp_threshold
{
    let new_weight = (current + self.config.ltp_bonus.get()).min(cap);
    graph.edge_plasticity.ltp_applied[j] = true;
}
```

Conversely, after an edge has been weakened **5 consecutive times** (the LTD threshold), it receives a one-time **-0.15** penalty:

```rust
if !graph.edge_plasticity.ltd_applied[j]
    && graph.edge_plasticity.weaken_count[j] >= self.config.ltd_threshold
{
    let new_weight = (current - self.config.ltd_penalty.get()).max(floor);
    graph.edge_plasticity.ltd_applied[j] = true;
}
```

These thresholds model biological LTP/LTD -- the transition from short-term to long-term memory. Five consecutive activations is a signal of sustained relevance, not a fluke. The one-time bonus/penalty is permanent: it does not reset, and it does not apply again for the same edge. This prevents unbounded weight inflation from repeated queries.

### Step 4: Homeostatic normalization

After strengthening and LTP/LTD, the total incoming weight for each node is checked against a ceiling of **5.0**:

```rust
if total_incoming > ceiling {
    let scale = ceiling / total_incoming;
    for each incoming edge:
        new_weight = current * scale;
}
```

This is homeostatic plasticity -- a biological mechanism that prevents individual neurons from becoming over-stimulated. In m1nd, it prevents hub nodes (like `config.py` or `main.py`) from accumulating so much incoming weight that they dominate every activation query regardless of the actual query content.

The normalization is proportional: all incoming edges are scaled by the same factor. This preserves relative strengths while enforcing an absolute ceiling. A node with 10 incoming edges at weight 1.0 each (total 10.0) would have all edges scaled to 0.5, bringing the total to 5.0.

### Step 5: Query memory recording

The query, its seeds, and its activated nodes are recorded in a bounded ring buffer (capacity: 1000 queries). This memory serves two purposes:

1. **Priming signal**: future queries that share seeds with past queries get a boost from nodes that frequently appeared in those past results. This implements a form of associative memory -- "things I looked at near authentication tend to be relevant when I look at authentication again."

2. **Seed bigrams**: pairs of seeds that co-occur across multiple queries are tracked. This supports the `m1nd.warmup` tool, which uses query memory to pre-activate frequently queried paths.

## How m1nd.learn works

The automatic plasticity cycle runs on every `m1nd.activate` call. But agents can also provide explicit feedback via `m1nd.learn`:

### Positive feedback: `m1nd.learn(feedback="correct", node_ids=[...])`

When an agent confirms that specific nodes were useful:

1. The edges connecting those nodes are strengthened with an **amplified** Hebbian update (the activation values are set to 1.0 for the confirmed nodes, producing maximum `delta_w`).
2. The strengthen counters increment, moving edges closer to the LTP threshold.
3. Query memory records the confirmed nodes with high weight, boosting them in future priming signals.

### Negative feedback: `m1nd.learn(feedback="wrong", node_ids=[...])`

When an agent marks results as irrelevant:

1. The edges connecting those nodes receive **decay** as if they were inactive, even though they were activated.
2. The weaken counters increment, moving edges closer to the LTD threshold.
3. Query memory records the rejection, reducing the priming signal for those nodes.

This feedback loop is what makes m1nd adaptive. A team that mostly works on the payment system will gradually strengthen all paths around payment-related modules. An agent investigating authentication will produce different results than an agent investigating billing -- even on the same codebase -- because their feedback histories have shaped different edge weight landscapes.

## Plasticity state persistence

The learned weights are valuable. Losing them means losing the graph's adaptation to your workflow. m1nd persists plasticity state in two ways:

### Per-edge state (SynapticState)

Each edge's plasticity state is captured as a serializable record:

```rust
pub struct SynapticState {
    pub source_label: String,
    pub target_label: String,
    pub relation: String,
    pub original_weight: f32,
    pub current_weight: f32,
    pub strengthen_count: u16,
    pub weaken_count: u16,
    pub ltp_applied: bool,
    pub ltd_applied: bool,
}
```

This is exported via `PlasticityEngine::export_state()` and persisted to `M1ND_PLASTICITY_STATE` (a JSON file). The export includes a NaN firewall (FM-PL-001): any non-finite weight falls back to the original weight. The write is atomic (temp file + rename, FM-PL-008) to prevent corruption on crash.

### Importing state

When m1nd restarts, `import_state` restores learned weights. Edge identity matching uses **(source_label, target_label, relation)** triples -- not numeric indices -- because re-ingesting the codebase may produce different node numbering. This means plasticity survives codebase re-ingestion: if `auth.py -> session.py` was strengthened, that strengthening persists even if `auth.py` gets a different NodeId after re-ingest.

Weights are clamped to `[weight_floor, weight_cap]` on import. Invalid JSON triggers a schema validation error (FM-PL-007) rather than corrupting the graph.

### Persistence frequency

The graph auto-persists every 50 queries and on server shutdown. This is a balance between durability (don't lose too much learning) and disk I/O (don't write on every query).

## The drift tool

After persistence, the natural question is: *how much has the graph changed?* The `m1nd.drift` tool answers this.

`m1nd.drift` compares the current edge weights against their original (ingest-time) baselines and reports:

- **Total edges changed**: how many edges have weights different from their original values.
- **Average weight change**: the mean absolute delta across all modified edges.
- **Top strengthened edges**: the edges that have grown the most relative to their baseline.
- **Top weakened edges**: the edges that have decayed the most.
- **LTP/LTD counts**: how many edges have crossed the long-term potentiation or depression thresholds.

This is designed for session recovery. When an agent starts a new session, `m1nd.drift` tells it what has changed since the graph was last loaded. The agent can see that "paths around the payment module strengthened significantly since yesterday" and adjust its investigation accordingly.

```
Session 1:
  ingest -> activate("auth") -> agent uses results -> learn(correct)
  → 740 edges strengthened, 12,340 edges decayed slightly

Session 2:
  drift(since=session_1) -> auth paths now 15% stronger on average
  activate("auth") -> better results, faster convergence to useful nodes

Session N:
  the graph has adapted to how your team thinks about auth
```

## How this makes the graph adapt

The combination of automatic Hebbian updates, explicit feedback, LTP/LTD thresholds, and homeostatic normalization creates a self-tuning system:

1. **Short-term adaptation** (within a session): edges on frequently queried paths strengthen immediately. The next query about the same topic converges faster.

2. **Long-term memory** (across sessions): edges that cross the LTP threshold receive a permanent bonus. Persistent investigation patterns are encoded in the graph structure.

3. **Forgetting** (controlled decay): paths that are never queried slowly fade toward the weight floor. This prevents the graph from becoming saturated with historical patterns that no longer reflect the codebase's structure.

4. **Stability** (homeostatic normalization): no node can accumulate unbounded incoming weight. Hub nodes stay important but do not become black holes that absorb all activation energy.

The result is a graph that starts generic (all edges at their ingest-time weights, reflecting code structure) and gradually becomes specific (edges weighted by how *you* use the codebase). Two teams working on the same repository will develop different plasticity landscapes. This is a feature: the graph models the team's mental model, not just the code's static structure.

## Constants reference

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `DEFAULT_LEARNING_RATE` | 0.08 | Hebbian delta_w scaling |
| `DEFAULT_DECAY_RATE` | 0.005 | Per-query inactive edge decay |
| `LTP_THRESHOLD` | 5 | Consecutive strengthens for long-term bonus |
| `LTD_THRESHOLD` | 5 | Consecutive weakens for long-term penalty |
| `LTP_BONUS` | 0.15 | One-time weight bonus at LTP threshold |
| `LTD_PENALTY` | 0.15 | One-time weight penalty at LTD threshold |
| `HOMEOSTATIC_CEILING` | 5.0 | Max total incoming weight per node |
| `WEIGHT_FLOOR` | 0.05 | Minimum edge weight (never decays below) |
| `WEIGHT_CAP` | 3.0 | Maximum edge weight (never strengthens above) |
| `DEFAULT_MEMORY_CAPACITY` | 1000 | Ring buffer size for query memory |
| `CAS_RETRY_LIMIT` | 64 | Atomic weight update retries |
