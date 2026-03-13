# Benchmarks

All numbers in this document are from real execution against a production Python backend: 335 files, approximately 52,000 lines of code, producing a graph of 9,767 nodes and 26,557 edges.

No synthetic benchmarks. No cherry-picked runs. These are the numbers you get when you run m1nd against a real codebase.

## Core Operations

| Operation | Time | Scale |
|-----------|------|-------|
| **Full ingest** | 910ms | 335 files -> 9,767 nodes, 26,557 edges |
| **Spreading activation** | 31-77ms | 15-20 results from 9,767 nodes |
| **Blast radius** (depth=3) | 5-52ms | Up to 4,271 affected nodes |
| **Stacktrace analysis** | 3.5ms | 5 frames -> 4 suspects ranked |
| **Plan validation** | 10ms | 7 files -> 43,152 blast radius |
| **Counterfactual cascade** | 3ms | Full BFS on 26,557 edges |
| **Hypothesis testing** | 58ms | 25,015 paths explored |
| **Pattern scan** (all 8 patterns) | 38ms | 335 files, 50 findings per pattern |
| **Multi-repo federation** | 1.3s | 11,217 nodes, 18,203 cross-repo edges |
| **Lock diff** | 0.08us | 1,639-node subgraph comparison |
| **Trail merge** | 1.2ms | 5 hypotheses, 3 conflicts detected |
| **Hebbian learn** | <1ms | 740 edges adjusted |
| **Health check** | <1ms | Statistics only |
| **Seek** (semantic search) | 10-15ms | 20 results |
| **Warmup** (task priming) | 82-89ms | 50 seed nodes primed |
| **Resonate** (standing wave) | 37-52ms | Harmonic analysis |
| **Fingerprint** (twin detection) | 1-107ms | Topology comparison |
| **Why** (path explanation) | 5-6ms | Shortest path between two nodes |
| **Drift** (weight changes) | 23ms | Since last session |
| **Timeline** (temporal history) | ~1ms | Node change history |

## Comparison Table

### m1nd vs grep / ripgrep

| Dimension | ripgrep | m1nd |
|-----------|---------|------|
| **Query type** | Text pattern (regex) | Natural language intent |
| **Returns** | Lines matching pattern | Ranked nodes with 4D scores |
| **Relationships** | None | Full graph traversal |
| **Learning** | None | Hebbian plasticity |
| **"What if?" queries** | Not possible | Counterfactual, hypothesis, impact |
| **Speed (simple query)** | ~5ms | 31-77ms |
| **Speed (structural query)** | Not possible | 3-58ms |
| **Memory** | ~10MB | ~50MB |
| **Cost per query** | Zero | Zero |

ripgrep is faster for simple text matching and always will be. m1nd answers questions ripgrep cannot ask.

### m1nd vs RAG

| Dimension | RAG | m1nd |
|-----------|-----|------|
| **Retrieval method** | Embedding similarity (top-K) | Spreading activation (4D) |
| **Statefulness** | Stateless per query | Persistent graph + learning |
| **Relationships** | Not tracked | First-class edges |
| **Learning** | None | Hebbian feedback loop |
| **Investigation memory** | None | Trail save/resume/merge |
| **Structural queries** | Not possible | Impact, counterfactual, hypothesis |
| **Setup cost** | Embedding computation | 910ms ingest |
| **Cost per query** | LLM tokens for embedding | Zero |
| **Typical query latency** | 200-500ms (includes API call) | 31-77ms (local) |

### m1nd vs Static Analysis (Sourcegraph, SCIP, LSP)

| Dimension | Static Analysis | m1nd |
|-----------|----------------|------|
| **Accuracy** | Language-server precise | Structural heuristic |
| **Learning** | None | Hebbian plasticity |
| **Temporal intelligence** | git blame only | Co-change velocity + decay |
| **"What if?" simulation** | Not possible | Counterfactual cascade |
| **Hypothesis testing** | Not possible | Bayesian path analysis |
| **Investigation state** | Not tracked | Trail system |
| **Multi-agent** | Read-only sharing | Shared graph + isolated perspectives |
| **Cost** | Hosted SaaS or self-hosted infra | Single binary, zero cost |
| **Setup** | Minutes to hours (indexing) | 910ms (ingest) |

### Summary: Use the Right Tool

| Task | Best Tool |
|------|-----------|
| Find text in files | ripgrep |
| Find semantically similar code | RAG |
| Go-to-definition, find references | LSP / Sourcegraph |
| Understand blast radius of a change | **m1nd** |
| Simulate module removal | **m1nd** |
| Test structural hypotheses | **m1nd** |
| Learn from agent feedback | **m1nd** |
| Persist investigation state | **m1nd** |
| Multi-agent shared code intelligence | **m1nd** |

## Cost Comparison: Tokens Saved

In a typical AI agent coding session, the agent calls grep/ripgrep 20-50 times and reads 10-30 files to navigate a codebase. Each file read costs tokens (the file content is sent to the LLM).

m1nd replaces many of these exploratory reads with graph queries that return *ranked results* rather than raw file contents. The agent reads fewer files because it reads the *right* files first.

### Estimated Token Savings

Assumptions: 335-file Python backend, 8-hour agent workday, agent using Claude Opus.

| Without m1nd | With m1nd |
|-------------|-----------|
| ~40 grep calls/hour | ~15 grep calls/hour |
| ~20 file reads/hour | ~8 file reads/hour |
| ~150K tokens/hour (context) | ~60K tokens/hour (context) |
| ~1.2M tokens/day | ~480K tokens/day |

**Estimated savings: ~720K tokens/day** (60% reduction in context tokens).

These are estimates based on production usage in the ROOMANIZER OS multi-agent system. Your mileage varies with codebase size, task type, and agent behavior.

The key insight: m1nd does not replace search. It *focuses* search. The agent still uses grep and reads files, but it starts from a much better position because m1nd told it where to look.

## Memory and CPU Usage

### Memory

| Component | Size |
|-----------|------|
| Graph (9,767 nodes, 26,557 edges) | ~2MB |
| Plasticity state | ~500KB |
| Perspective state (per active perspective) | ~100KB |
| Lock baselines (per lock) | ~200KB |
| Trail storage (per saved trail) | ~50KB |
| JSON-RPC server overhead | ~5MB |
| **Typical total** | **~50MB** |

Memory scales linearly with graph size. A 100K-node graph would use approximately 20MB for the graph alone, with similar overhead for the server.

### CPU

m1nd is single-threaded for graph operations (no lock contention, deterministic results). Ingest uses Rayon for parallel file parsing. During query serving, CPU usage is negligible between queries and spikes briefly during activation (31-77ms of computation).

On an Apple M2, the server at idle uses <0.1% CPU. During a burst of queries, it peaks at ~5% of a single core.

## Scaling Characteristics

### Ingest Time vs Codebase Size

Ingest scales linearly with file count. Reference resolution is roughly O(n log n) where n is the number of cross-file references.

| Files | Estimated Ingest Time | Estimated Nodes |
|-------|-----------------------|-----------------|
| 100 | ~270ms | ~3,000 |
| 335 | 910ms (measured) | 9,767 (measured) |
| 1,000 | ~2.7s | ~29,000 |
| 10,000 | ~27s | ~290,000 |
| 100,000 | ~4.5min | ~2,900,000 |

### Activation Time vs Graph Size

Spreading activation is bounded by the number of edges traversed, which depends on graph density and query specificity rather than total graph size. Activation in a 100K-node graph is estimated at 100-200ms.

### Persistence Time vs State Size

JSON serialization scales linearly with state size. A 10K-node graph persists in under 100ms. A 100K-node graph would take approximately 1 second.

## Reproducibility

To reproduce these benchmarks:

```bash
git clone https://github.com/cosmophonix/m1nd.git
cd m1nd
cargo build --release

# Start the server
./target/release/m1nd-mcp
```

Then send the JSON-RPC calls from the [Examples](../EXAMPLES.md) document against your own codebase. Times will vary based on:

- Hardware (CPU speed, memory bandwidth)
- Codebase size and language
- Graph density (codebases with many cross-references produce denser graphs)
- Plasticity state (learned weights affect activation propagation paths)

Report your benchmarks via GitHub Issues with the `benchmark` label.
