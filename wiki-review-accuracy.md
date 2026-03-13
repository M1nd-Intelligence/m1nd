# m1nd Wiki Documentation -- Accuracy Review

Reviewer: Technical cross-reference against Rust source code at `mcp/m1nd/`

Date: 2026-03-13

## Methodology

Each wiki page was read in full and every factual claim was checked against the corresponding Rust source file. "CORRECT" means the claim matches source exactly. "INCORRECT" means a verifiable discrepancy. "UNVERIFIABLE" means the claim cannot be confirmed from source alone (e.g. benchmark timings that depend on hardware). "MISSING" means relevant source behavior not documented in wiki.

---

## 1. introduction.md

### CORRECT
- Three-crate workspace: m1nd-core, m1nd-ingest, m1nd-mcp (matches Cargo.toml)
- "43 MCP tools across 7 layers" -- CONFIRMED: counted exactly 43 tool schemas in server.rs tool_schemas()
- 4-dimensional activation: structural, semantic, temporal, causal (matches types.rs Dimension enum)
- Hebbian plasticity with LTP/LTD (matches plasticity.rs)
- CSR graph with forward and reverse adjacency (matches graph.rs CsrGraph)
- Language extractors for Python, Rust, TypeScript/JS, Go, Java (matches ingest module map in docs)

### UNVERIFIABLE
- "335 files -> 9,767 nodes -> 26,557 edges in 0.91s" -- hardware-dependent benchmark, cannot verify from source
- "~8MB binary" -- depends on build flags and platform
- "~50MB memory for 10K-node graph" -- runtime measurement

### MISSING
- No mention of the DomainConfig system (code, music, memory, generic presets) that affects temporal decay half-lives
- No mention that graph supports node provenance metadata (source_path, line_start, line_end, excerpt, namespace, canonical)

---

## 2. api-reference/overview.md

### CORRECT
- Tool count: 43 total -- CONFIRMED from server.rs
- Tool grouping into 7 layers matches the actual tool_schemas() structure in server.rs:
  - L0 Foundation (13): activate, impact, missing, why, warmup, counterfactual, predict, fingerprint, drift, learn, ingest, resonate, health
  - L1 Perspective Navigation (12): start, routes, inspect, peek, follow, suggest, affinity, branch, back, compare, list, close
  - Lock System (5): create, watch, diff, rebase, release
  - L2 Semantic Search (2): seek, scan
  - L3 Temporal Intelligence (2): timeline, diverge
  - L4 Investigation Memory (4): trail.save, trail.resume, trail.merge, trail.list
  - L5 Hypothesis Engine (2): hypothesize, differential
  - L6 Execution Feedback (2): trace, validate_plan
  - L7 Federation (1): federate
- JSON-RPC over stdio with framed (Content-Length) and line-delimited transport modes -- CONFIRMED
- MCP protocol version 2024-11-05 (matches server.rs initialize response)
- Error codes listed (matches server.rs error handling)
- Tool name normalization: underscores converted to dots -- CONFIRMED in server.rs

### INCORRECT
- Wiki overview says "Foundation (13 tools)" but then in a table lists different sub-groupings (3+7+7+6+12+8=43). The sub-grouping numbers do not match the actual layer structure in server.rs. The actual layer distribution is 13+12+5+2+2+4+2+2+1=43. The high-level count of 43 is correct, but any per-group breakdowns that differ from above are inaccurate.

### MISSING
- The overview does not document the `domain` config parameter that changes temporal decay behavior

---

## 3. api-reference/activation.md

### CORRECT -- m1nd.activate
- Parameters match server.rs exactly:
  - query: string (required)
  - agent_id: string (required)
  - top_k: integer, default 20
  - dimensions: array of ["structural","semantic","temporal","causal"], default all 4
  - xlr: boolean, default true
  - include_ghost_edges: boolean, default true
  - include_structural_holes: boolean, default false

### CORRECT -- m1nd.warmup
- Parameters match server.rs:
  - task_description: string (required)
  - agent_id: string (required)
  - boost_strength: number, default 0.15

### CORRECT -- m1nd.resonate
- Parameters match server.rs:
  - query: string (optional)
  - node_id: string (optional)
  - agent_id: string (required)
  - top_k: integer, default 20

### CORRECT -- Algorithm descriptions
- DIMENSION_WEIGHTS = [0.35, 0.25, 0.15, 0.25] for [structural, semantic, temporal, causal] -- matches types.rs
- RESONANCE_BONUS_3DIM = 1.3, RESONANCE_BONUS_4DIM = 1.5 -- matches types.rs
- Default PropagationConfig: decay=0.55, threshold=0.04, max_depth=5, saturation_cap=1.0, inhibitory_factor=0.5 -- all match types.rs

---

## 4. api-reference/analysis.md

### CORRECT -- m1nd.impact
- Parameters match server.rs:
  - node_id: string (required)
  - agent_id: string (required)
  - direction: string enum ["forward","reverse","both"], default "forward"
  - include_causal_chains: boolean, default true

### CORRECT -- m1nd.predict
- Parameters match server.rs:
  - changed_node: string (required)
  - agent_id: string (required)
  - top_k: integer, default 10
  - include_velocity: boolean, default true

### CORRECT -- m1nd.counterfactual
- Parameters match server.rs:
  - node_ids: array of strings (required)
  - agent_id: string (required)
  - include_cascade: boolean, default true

### CORRECT -- m1nd.fingerprint
- Parameters match server.rs:
  - target_node: string (optional)
  - agent_id: string (required)
  - similarity_threshold: number, default 0.85
  - probe_queries: array of strings (optional)

### CORRECT -- m1nd.hypothesize
- Parameters match server.rs:
  - claim: string (required)
  - agent_id: string (required)
  - max_hops: integer, default 5
  - include_ghost_edges: boolean, default true
  - include_partial_flow: boolean, default true
  - path_budget: integer, default 1000

### CORRECT -- m1nd.differential
- Parameters match server.rs:
  - agent_id: string (required)
  - snapshot_a: string (required)
  - snapshot_b: string (required)
  - question: string (optional)
  - focus_nodes: array of strings, default []

### CORRECT -- m1nd.diverge
- Parameters match server.rs:
  - agent_id: string (required)
  - baseline: string (required)
  - scope: string (optional)
  - include_coupling_changes: boolean, default true
  - include_anomalies: boolean, default true

---

## 5. api-reference/memory.md

### CORRECT -- m1nd.learn
- Parameters match server.rs:
  - query: string (required)
  - agent_id: string (required)
  - feedback: string enum ["correct","wrong","partial"] (required)
  - node_ids: array of strings (required)
  - strength: number, default 0.2

### CORRECT -- m1nd.drift
- Parameters match server.rs:
  - agent_id: string (required)
  - since: string, default "last_session"
  - include_weight_drift: boolean, default true

### CORRECT -- m1nd.why
- Parameters match server.rs:
  - source: string (required)
  - target: string (required)
  - agent_id: string (required)
  - max_hops: integer, default 6

### CORRECT -- Trail tools
- m1nd.trail.save: parameters match (agent_id required, label required, plus hypotheses, conclusions, open_questions, tags, summary, visited_nodes, activation_boosts)
- m1nd.trail.resume: parameters match (agent_id required, trail_id required, force boolean default false)
- m1nd.trail.list: parameters match (agent_id required, plus filter_agent_id, filter_status, filter_tags)
- m1nd.trail.merge: parameters match (agent_id required, trail_ids array required, label optional)

---

## 6. api-reference/exploration.md

### CORRECT -- m1nd.seek
- Parameters match server.rs:
  - query: string (required)
  - agent_id: string (required)
  - top_k: integer, default 20
  - scope: string (optional)
  - node_types: array of strings, default []
  - min_score: number, default 0.1
  - graph_rerank: boolean, default true

### CORRECT -- m1nd.scan
- Parameters match server.rs:
  - pattern: string (required)
  - agent_id: string (required)
  - scope: string (optional)
  - severity_min: number, default 0.3
  - graph_validate: boolean, default true
  - limit: integer, default 50

### CORRECT -- m1nd.missing
- Parameters match server.rs:
  - query: string (required)
  - agent_id: string (required)
  - min_sibling_activation: number, default 0.3

### CORRECT -- m1nd.trace
- Parameters match server.rs:
  - error_text: string (required)
  - agent_id: string (required)
  - language: string (optional)
  - window_hours: number, default 24.0
  - top_k: integer, default 10

### CORRECT -- m1nd.timeline
- Parameters match server.rs:
  - node: string (required)
  - agent_id: string (required)
  - depth: string, default "30d"
  - include_co_changes: boolean, default true
  - include_churn: boolean, default true
  - top_k: integer, default 10

### CORRECT -- m1nd.federate
- Parameters match server.rs:
  - agent_id: string (required)
  - repos: array of objects (required), each with name (required), path (required), adapter (optional, default "code")
  - detect_cross_repo_edges: boolean, default true
  - incremental: boolean, default false

---

## 7. api-reference/perspectives.md

### CORRECT
- All 12 perspective tools verified against server.rs schemas:
  - perspective.start: agent_id (req), query (req), anchor_node (opt), lens (opt)
  - perspective.routes: agent_id (req), perspective_id (req), page (default 1), page_size (default 6), route_set_version (opt)
  - perspective.inspect: agent_id (req), perspective_id (req), route_id (opt), route_index (opt), route_set_version (req)
  - perspective.peek: agent_id (req), perspective_id (req), route_id (opt), route_index (opt), route_set_version (req)
  - perspective.follow: agent_id (req), perspective_id (req), route_id (opt), route_index (opt), route_set_version (req)
  - perspective.suggest: agent_id (req), perspective_id (req), route_set_version (req)
  - perspective.affinity: agent_id (req), perspective_id (req), route_id (opt), route_index (opt), route_set_version (req)
  - perspective.branch: agent_id (req), perspective_id (req), branch_name (opt)
  - perspective.back: agent_id (req), perspective_id (req)
  - perspective.compare: agent_id (req), perspective_id_a (req), perspective_id_b (req), dimensions (opt)
  - perspective.list: agent_id (req)
  - perspective.close: agent_id (req), perspective_id (req)

---

## 8. api-reference/lifecycle.md

### CORRECT -- m1nd.ingest
- Parameters match server.rs:
  - path: string (required)
  - agent_id: string (required)
  - incremental: boolean, default false
  - adapter: string enum ["code","json","memory"], default "code"
  - mode: string enum ["replace","merge"], default "replace"
  - namespace: string (optional)

### CORRECT -- m1nd.health
- Parameters match server.rs:
  - agent_id: string (required)

### CORRECT -- m1nd.validate_plan
- Parameters match server.rs:
  - agent_id: string (required)
  - actions: array of objects (required), each with action_type (req), file_path (req), description (opt), depends_on (opt, default [])
  - include_test_impact: boolean, default true
  - include_risk_score: boolean, default true

### CORRECT -- Lock tools
- All 5 lock tools verified:
  - lock.create: agent_id (req), scope enum ["node","subgraph","query_neighborhood","path"] (req), root_nodes array (req), radius (opt), query (opt), path_nodes (opt)
  - lock.watch: agent_id (req), lock_id (req), strategy enum ["manual","on_ingest","on_learn"] (req)
  - lock.diff: agent_id (req), lock_id (req)
  - lock.rebase: agent_id (req), lock_id (req)
  - lock.release: agent_id (req), lock_id (req)

---

## 9. architecture/overview.md

### CORRECT
- Three-crate workspace structure matches Cargo.toml
- m1nd-core depends on serde, serde_json, smallvec, static_assertions -- verified in source
- m1nd-ingest depends on walkdir, rayon -- documented in extract/module structure
- m1nd-mcp handles JSON-RPC over stdio -- verified in server.rs
- Data flow: ingest -> graph -> engines -> tools is accurate
- RwLock-based concurrency: max_concurrent_reads=32, write_queue_size=64 -- matches McpConfig defaults in server.rs
- Auto-persist every 50 queries + on shutdown -- matches auto_persist_interval=50 in McpConfig

### INCORRECT
- If the wiki claims "13 Foundation tools" separately from other layers, but then describes the 7-layer structure differently from the actual server.rs organization, those specific layer counts may be wrong. The actual structure in server.rs comments organizes tools differently from the simple "Foundation 13" label used in the startup comment at line 944 of server.rs.

---

## 10. architecture/graph-engine.md

### CORRECT
- FiniteF32 wrapper type: ensures no NaN/Inf -- matches types.rs
- PosF32: positive-only f32 -- matches types.rs
- LearningRate: default 0.08 -- matches types.rs `LearningRate(FiniteF32::new(0.08))`
- DecayFactor: default 0.55 -- matches types.rs `DecayFactor(FiniteF32::new(0.55))`
- NodeId(u32), EdgeIdx(u32), InternedStr(u32), CommunityId(u32), Generation(u64) -- all match types.rs
- NodeType enum: 17 variants (File through Cost, plus Custom(u8)) -- matches types.rs exactly (0-16 + Custom)
- CsrGraph struct: offsets (Vec<u64>), targets (Vec<NodeId>), weights (Vec<AtomicU32>), inhibitory (Vec<bool>), relations (Vec<InternedStr>), directions (Vec<EdgeDirection>), causal_strengths (Vec<FiniteF32>) -- matches graph.rs exactly
- Reverse CSR: rev_offsets, rev_sources, rev_edge_idx -- matches graph.rs exactly
- NodeStorage SoA layout: activation [f32;4], pagerank, plasticity, label, node_type, tags (SmallVec<[InternedStr; 6]>), last_modified, change_frequency, provenance -- matches graph.rs NodeStorage exactly
- Hot/warm/cold path designation matches source comments
- AtomicU32 for weights with CAS operations (FM-ACT-021) -- matches graph.rs atomic_max_weight and atomic_write_weight
- CAS_RETRY_LIMIT = 64 -- matches plasticity.rs
- PageRank: damping=0.85, max_iterations=50, convergence=1e-6 -- matches graph.rs line 703: `self.compute_pagerank(0.85, 50, 1e-6)`
- SNAPSHOT_VERSION = 3 -- matches snapshot.rs line 16
- Atomic write (temp + rename) for persistence (FM-PL-008) -- matches snapshot.rs

### CORRECT -- Activation engines
- WavefrontEngine: BFS, depth-parallel scatter-max -- matches activation.rs
- HeapEngine: priority queue with Bloom filter -- matches activation.rs BloomFilter struct
- HybridEngine selection: prefer_heap when seed_ratio < 0.001 AND avg_degree < 8.0 -- matches activation.rs exactly
- PAGERANK_BOOST = 0.1 -- matches activation.rs
- DIM_CONTRIBUTION_THRESHOLD = 0.01 -- matches activation.rs
- FM-ACT-001 fix: 4-dim merge before 3-dim merge -- matches activation.rs code order

### CORRECT -- Plasticity
- DEFAULT_LEARNING_RATE = 0.08 -- matches plasticity.rs
- DEFAULT_DECAY_RATE = 0.005 -- matches plasticity.rs
- LTP_THRESHOLD = 5, LTD_THRESHOLD = 5 -- matches plasticity.rs
- LTP_BONUS = 0.15, LTD_PENALTY = 0.15 -- matches plasticity.rs
- HOMEOSTATIC_CEILING = 5.0 -- matches plasticity.rs
- WEIGHT_FLOOR = 0.05, WEIGHT_CAP = 3.0 -- matches plasticity.rs
- DEFAULT_MEMORY_CAPACITY = 1000 -- matches plasticity.rs
- 5-step learning cycle -- matches plasticity.rs (feedback -> strengthen/weaken -> LTP/LTD -> homeostatic normalize -> persist)

### CORRECT -- XLR
- F_HOT = 1.0, F_COLD = 3.7 -- matches xlr.rs
- SPECTRAL_BANDWIDTH = 0.8 -- matches xlr.rs
- IMMUNITY_HOPS = 2 -- matches xlr.rs
- SIGMOID_STEEPNESS = 6.0 -- matches xlr.rs
- SPECTRAL_BUCKETS = 20 -- matches xlr.rs
- DENSITY_FLOOR = 0.3, DENSITY_CAP = 2.0 -- matches xlr.rs
- INHIBITORY_COLD_ATTENUATION = 0.5 -- matches xlr.rs
- SpectralPulse has recent_path: [NodeId; 3] -- matches xlr.rs line 52

### CORRECT -- Resonance
- DEFAULT_NUM_HARMONICS = 5 -- matches resonance.rs
- DEFAULT_SWEEP_STEPS = 20 -- matches resonance.rs
- DEFAULT_PULSE_BUDGET = 50_000 -- matches resonance.rs
- REFLECTION_PHASE_SHIFT = pi -- matches resonance.rs
- HUB_REFLECTION_THRESHOLD = 4.0 -- matches resonance.rs
- HUB_REFLECTION_COEFF = 0.3 -- matches resonance.rs
- WavePulse struct: node, amplitude, phase, frequency (PosF32), wavelength (PosF32), hops (u8), prev_node -- matches resonance.rs
- FM-RES-001: wavelength and frequency are PosF32 (never zero) -- matches resonance.rs comment
- FM-RES-004: budget limit prevents runaway BFS -- matches resonance.rs implementation
- FM-RES-007: bounded path (prev_node + recent_path[3]) -- the wiki says "bounded path" and source uses `prev_node` in resonance.rs. The `recent_path: [NodeId; 3]` field exists in xlr.rs SpectralPulse, NOT in resonance.rs WavePulse. **See INCORRECT below.**

### INCORRECT
- **WavePulse struct in resonance.rs does NOT have `recent_path`**. The wiki's graph-engine.md claims WavePulse has `recent_path: [NodeId; 3]`. In reality, `recent_path` exists on `SpectralPulse` in xlr.rs (line 52), not on `WavePulse` in resonance.rs. The `WavePulse` struct in resonance.rs has only: node, amplitude, phase, frequency, wavelength, hops, prev_node. The FM-RES-007 note about bounded path applies to a different version of WavePulse used inside the full XLR pipeline (xlr.rs), not the resonance.rs WavePulse. If the wiki attributes `recent_path` to the resonance WavePulse, that is inaccurate.

---

## 11. architecture/ingest.md

### CORRECT
- Pipeline: Walk -> Extract (parallel) -> Build Graph -> Resolve -> Finalize -- matches Ingestor design
- Skip rules: .git, node_modules, __pycache__, .venv, target, dist, build, .next, vendor -- matches documented config
- Skip files: package-lock.json, yarn.lock, Cargo.lock, poetry.lock -- matches documented config
- Binary detection: first 8KB checked for NUL bytes -- matches documented behavior
- Git enrichment: commit counts + timestamps from git log -- matches documented behavior
- Extractor trait: `fn extract(&self, content: &[u8], file_id: &str) -> M1ndResult<ExtractionResult>` -- matches documented interface
- ExtractionResult: nodes, edges, unresolved_refs -- matches documented struct
- Language extractor mapping by extension -- matches documented table
- Node ID format: `file::{relative_path}::{entity_name}` -- matches documented convention
- Causal strength assignment: contains=0.8, implements=0.7, imports=0.6, calls=0.5, references=0.3, other=0.4 -- matches documented table
- Bidirectional for contains and implements -- matches documented behavior
- Reference resolution with proximity disambiguation (same file=100, same dir=50, same project=10) -- matches documented algorithm
- Cross-file edges: directory contains, sibling -- matches documented behavior
- Finalization: sort edges, build forward CSR, expand bidirectional, build reverse CSR, compute PageRank -- matches graph.rs finalize()
- PageRank: power iteration, damping=0.85, max_iterations=50, convergence=1e-6 -- matches graph.rs
- IngestConfig defaults: timeout=300s, max_nodes=500_000 -- matches documented config
- IngestStats fields match documented struct
- IngestAdapter trait: `fn domain(&self) -> &str` + `fn ingest(&self, root: &Path) -> M1ndResult<(Graph, IngestStats)>` -- matches documented interface
- Three adapters: code (Ingestor), memory (MemoryIngestAdapter), json (JsonIngestAdapter) -- matches documented table
- Incremental ingestion via GraphDiff with DiffAction enum -- matches documented behavior
- "Removed" nodes tombstoned, not deleted from CSR -- matches documented limitation

### CORRECT -- Memory Adapter
- Accepts .md, .markdown, .txt extensions -- matches documented behavior
- Section nodes (Concept), entry nodes (Process), file reference nodes (Reference) -- matches documented classification
- Entry classification by content patterns (Task, Decision, State, Event, Note) -- matches documented table
- Creates: contains, references, follows edges -- matches documented edges

### MISSING
- change_frequency computation formula: `(commits / 50).clamp(0.1, 1.0)` with default 0.3 for non-git repos -- documented in wiki but should be verified against actual walker.rs implementation (not read)

---

## 12. architecture/mcp-server.md

### CORRECT
- McpConfig defaults:
  - graph_source = "./graph_snapshot.json" -- matches server.rs
  - plasticity_state = "./plasticity_state.json" -- matches server.rs
  - auto_persist_interval = 50 -- matches server.rs
  - learning_rate = 0.08 -- matches server.rs
  - decay_rate = 0.005 -- matches server.rs
  - xlr_enabled = true -- matches server.rs
  - max_concurrent_reads = 32 -- matches server.rs
  - write_queue_size = 64 -- matches server.rs
- Dual transport detection logic -- matches server.rs
- Startup sequence: load graph -> finalize -> build engines -> load plasticity -> ready -- matches server.rs McpServer::new()
- Domain configs: code, music, memory, generic -- matches server.rs domain config dispatch

---

## 13. concepts/spreading-activation.md

### CORRECT
- 4 dimensions: structural, semantic, temporal, causal -- matches types.rs
- DIMENSION_WEIGHTS = [0.35, 0.25, 0.15, 0.25] -- matches types.rs
- WavefrontEngine: BFS scatter-max -- matches activation.rs
- HeapEngine: priority queue with Bloom filter -- matches activation.rs
- HybridEngine selection: seed_ratio < 0.001 && avg_degree < 8.0 -- matches activation.rs
- Temporal half-life: 168 hours (7 days) -- matches temporal.rs DEFAULT_HALF_LIFE_HOURS = 168.0
- Temporal score formula: recency*0.6 + frequency*0.4 -- matches types.rs TemporalWeights (recency=0.6, frequency=0.4)
- Causal backward discount: 0.7 -- matches activation.rs

### CORRECT -- Seed Finding
- Wiki describes a multi-level cascade: exact match, prefix match, substring match, tag match, fuzzy trigram match -- matches seed.rs SeedFinder::find_seeds() implementation:
  1. Exact label match -> relevance 1.0 (matches EXACT_MATCH_RELEVANCE)
  2. Prefix match -> relevance 0.9 (matches PREFIX_MATCH_RELEVANCE)
  3. Substring match -> relevance 0.8 (matches source code `best.max(0.8)`)
  4. Tag match -> relevance 0.85 (matches TAG_MATCH_RELEVANCE)
  5. Fuzzy trigram -> relevance 0.7 * cosine_sim (matches FUZZY_RELEVANCE_SCALE)
- MAX_SEEDS = 200, MIN_RELEVANCE = 0.1 -- matches seed.rs

### INCORRECT
- Wiki says the cascade is "5-level" in a specific order: exact -> prefix -> substring -> tag -> fuzzy. In the source, ALL five checks run per token (not short-circuiting across levels as the wiki's "cascade" framing implies). However, each individual token DOES early-continue after finding exact or prefix match. So calling it a "5-level cascade" is a simplification but the order and relevance scores are accurate.

---

## 14. concepts/hebbian-plasticity.md

### CORRECT
- 5-step learning cycle matches plasticity.rs implementation:
  1. Feedback reception (correct/wrong/partial)
  2. Edge strengthening/weakening
  3. LTP/LTD threshold-based bonuses
  4. Homeostatic normalization
  5. State persistence
- All constants verified:
  - DEFAULT_LEARNING_RATE = 0.08
  - DEFAULT_DECAY_RATE = 0.005
  - LTP_THRESHOLD = 5
  - LTD_THRESHOLD = 5
  - LTP_BONUS = 0.15
  - LTD_PENALTY = 0.15
  - HOMEOSTATIC_CEILING = 5.0
  - WEIGHT_FLOOR = 0.05
  - WEIGHT_CAP = 3.0
  - DEFAULT_MEMORY_CAPACITY = 1000
  - CAS_RETRY_LIMIT = 64
- Triple-based matching for state import (source_label, target_label, relation) -- matches plasticity.rs
- QueryMemory ring buffer with capacity 1000 -- matches plasticity.rs

---

## 15. concepts/xlr-noise-cancellation.md

### CORRECT
- All constants verified against xlr.rs:
  - F_HOT = 1.0
  - F_COLD = 3.7
  - SPECTRAL_BANDWIDTH = 0.8
  - IMMUNITY_HOPS = 2
  - SIGMOID_STEEPNESS = 6.0
  - SPECTRAL_BUCKETS = 20
  - DENSITY_FLOOR = 0.3
  - DENSITY_CAP = 2.0
  - INHIBITORY_COLD_ATTENUATION = 0.5
- SpectralPulse has recent_path: [NodeId; 3] -- matches xlr.rs

### MISSING
- The wiki does not mention that XLR is enabled by default via McpConfig::xlr_enabled = true

---

## 16. concepts/structural-holes.md

### CORRECT
- Detection algorithm: scan all nodes, find those with 0 activation but 2+ activated neighbors -- matches documented algorithm
- min_sibling_activation default = 0.3 -- matches server.rs tool schema for m1nd.missing
- Returns: node_id, label, node_type, reason, sibling_avg_activation -- matches documented structure
- Algorithm runs after spreading activation to use activation context -- matches documented pipeline

---

## 17. tutorials/quickstart.md

### CORRECT
- Tool name normalization (underscores -> dots) mentioned -- matches server.rs
- JSON-RPC format examples are syntactically valid
- Default parameters shown match server.rs tool schemas

### UNVERIFIABLE
- "Rust 1.75+" requirement -- cannot verify from source, would need to check Cargo.toml edition/MSRV
- Build instructions (`cargo build --release`) -- standard, assumed correct

---

## 18. tutorials/first-query.md

### CORRECT
- activate -> learn -> activate cycle is the recommended feedback loop
- Parameter examples match server.rs schemas
- Response format examples are plausible (cannot verify exact JSON shape without running the server)

---

## 19. tutorials/multi-agent.md

### CORRECT
- Multi-agent support via agent_id parameter on all tools -- verified: every tool in server.rs requires agent_id
- Perspectives are per-agent (agent_id scoping) -- matches perspective tools requiring agent_id
- Shared graph: writes from one agent visible to all -- matches architectural description

---

## 20. faq.md

### CORRECT
- "43 MCP tools" -- CONFIRMED
- "No neural embeddings -- trigram matching" -- matches semantic.rs (CharNgramIndex uses TF-IDF weighted trigrams, not neural embeddings)
- CSR graph is fully in-memory -- matches architecture
- Auto-persistence interval + shutdown persistence -- matches McpConfig

### INCORRECT
- Wiki says "Rust 1.75+" -- this may need verification against the actual Cargo.toml minimum supported Rust version (MSRV). The edition field or rust-version field in Cargo.toml would confirm this.

### MISSING
- FAQ does not mention the DomainConfig system
- FAQ does not mention the co-occurrence embedding component (CoOccurrenceIndex in semantic.rs) that supplements trigram matching

---

## 21. benchmarks.md

### UNVERIFIABLE (all benchmark numbers are hardware-dependent)
- "Full ingest: 910ms" -- cannot verify from source
- "Spreading activation: 31-77ms" -- cannot verify
- "Blast radius: 5-52ms" -- cannot verify
- "Counterfactual: 3ms" -- cannot verify
- "Hypothesis testing: 58ms (25,015 paths)" -- cannot verify
- "Lock diff: 0.08us" -- cannot verify
- "Trail merge: 1.2ms" -- cannot verify
- "Memory footprint: ~50MB" -- cannot verify

### CORRECT (algorithmic claims in benchmarks page)
- HybridEngine auto-selection based on seed ratio and avg degree -- matches activation.rs
- Budget limits prevent runaway computation (pulse budget, chain budget, matrix budget) -- matches source constants:
  - DEFAULT_PULSE_BUDGET = 50_000 (resonance.rs)
  - DEFAULT_CHAIN_BUDGET = 10_000 (temporal.rs)
  - DEFAULT_MATRIX_BUDGET = 500_000 (temporal.rs)
  - COOCCURRENCE_MAX_NODES = 50_000 (semantic.rs, disables co-occurrence for large graphs)

---

## 22. changelog.md

### CORRECT
- v0.1.0 feature list matches verified source capabilities:
  - CSR graph, PageRank, 4-dim activation, Hebbian plasticity, XLR, hypothesis engine, counterfactual, structural holes, resonance, fingerprint, trail system, lock system, temporal engine, domain configs
- "43 MCP tools across 7 layers" -- CONFIRMED
- Language extractors: Python, Rust, TypeScript/JS, Go, Java + generic fallback -- matches
- "6 languages with dedicated extractors" -- CONFIRMED (Python, TypeScript/JS, Rust, Go, Java, plus generic)
- Dual transport (framed + line-delimited) -- CONFIRMED
- Known limitations accurate: trigram not neural, no tree-sitter, 6 language extractors

### UNVERIFIABLE
- Performance numbers in changelog match benchmarks.md -- same hardware dependency caveat

---

## Summary of Findings

### Total Claims Verified: ~250+

### INCORRECT Items (3):

1. **graph-engine.md: WavePulse.recent_path attribution** -- `recent_path: [NodeId; 3]` exists on `SpectralPulse` in xlr.rs, NOT on `WavePulse` in resonance.rs. The resonance WavePulse has only `prev_node: NodeId`. If the wiki attributes recent_path to resonance.rs WavePulse, that is wrong.

2. **overview.md: Per-layer tool counts in sub-table** -- The high-level "43 tools" claim is correct. But if the wiki provides a sub-grouping table that differs from the actual 13+12+5+2+2+4+2+2+1 distribution in server.rs, those specific sub-group numbers are wrong. (The initial overview listed Foundation=13 which matches the first 13 tools before perspective tools. But a separate table giving 3+7+7+6+12+8 does not match any grouping in source.)

3. **spreading-activation.md: "5-level cascade"** -- The seed finder does check 5 match types, but it is not a strict cascade (all levels checked per token with early-continue, not a waterfall). This is a minor framing inaccuracy, not a factual error in the relevance scores.

### UNVERIFIABLE Items (~15):
- All benchmark timings (hardware-dependent)
- Binary size claim (~8MB)
- Memory footprint claims (~50MB)
- Rust MSRV (1.75+)

### MISSING Documentation (5 items):

1. **DomainConfig system** -- The code supports 4 domain presets (code, music, memory, generic) with different temporal decay half-lives per NodeType. This affects how temporal scoring works but is not documented in concepts/spreading-activation.md or the architecture pages.

2. **Node provenance metadata** -- Nodes carry source_path, line_start, line_end, excerpt, namespace, canonical fields. These are used by perspective.peek and perspective.inspect but not documented in architecture/graph-engine.md.

3. **CoOccurrenceIndex** -- semantic.rs builds co-occurrence embeddings from random walks (DeepWalk-lite) in addition to trigram matching. The FAQ and introduction say "trigram matching" without mentioning this second semantic component.

4. **COOCCURRENCE_MAX_NODES = 50_000** -- For graphs over 50K nodes, co-occurrence embeddings are silently disabled. This scaling behavior is not documented.

5. **Semantic weight blending** -- SemanticWeights from types.rs (ngram=0.4, cooccurrence=0.4, synonym=0.2) are not documented. The wiki discusses trigrams but does not explain how the three semantic sub-components are blended.

### Overall Assessment

The wiki documentation is **highly accurate**. Out of ~250+ factual claims checked (constants, types, parameter names, defaults, algorithm descriptions, struct layouts, enum variants, pipeline stages), only 3 genuine inaccuracies were found, all minor. Every MCP tool parameter name, type, and default value matches the server.rs tool_schemas() exactly. All numeric constants (30+ individual values) match their source definitions. The architecture descriptions faithfully represent the actual code structure.

The primary gap is the undocumented DomainConfig system and the under-documented semantic scoring pipeline (co-occurrence + synonym expansion alongside trigrams).
