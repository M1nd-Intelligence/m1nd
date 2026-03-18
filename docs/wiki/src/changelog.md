# Changelog

All notable changes to m1nd are documented here. This project uses [Semantic Versioning](https://semver.org/).

## v0.1.0 -- Initial Release

The first public release of m1nd: a neuro-symbolic connectome engine with Hebbian plasticity, spreading activation, and 43 MCP tools. Built in Rust.

### Core Engine (m1nd-core)

- **Compressed Sparse Row (CSR) graph** with forward and reverse adjacency
- **PageRank computation** on ingest
- **4-dimensional spreading activation**: structural, semantic, temporal, causal
- **Hebbian plasticity**: Long-Term Potentiation (LTP), Long-Term Depression (LTD), homeostatic normalization
- **XLR differential processing**: noise cancellation inspired by balanced audio cables
- **Hypothesis engine**: claim testing with Bayesian confidence on graph paths
- **Counterfactual engine**: module removal simulation with cascade analysis
- **Structural hole detection**: topology-based gap analysis
- **Resonance analysis**: standing wave computation for structural hub identification
- **Fingerprint engine**: activation fingerprinting for structural twin detection
- **Trail system**: investigation state persistence, resume, and multi-trail merge with conflict detection
- **Lock system**: subgraph pinning with sub-microsecond diff (0.08us)
- **Temporal engine**: co-change history, velocity scoring, decay functions
- **Domain configurations**: code, music, memory, generic presets with tuned decay half-lives

### Ingest Layer (m1nd-ingest)

- **Language extractors**: Python, Rust, TypeScript/JavaScript, Go, Java
- **Generic fallback extractor**: heuristic-based for unsupported languages
- **JSON adapter**: structured data ingestion
- **Memory adapter**: text corpus ingestion
- **Reference resolver**: cross-file import and call resolution
- **Incremental ingest**: re-process only changed files
- **Multi-repo federation**: unified graph with automatic cross-repo edge detection

### MCP Server (m1nd-mcp)

- **43 MCP tools** across 7 layers:
  - Foundation (13): activate, impact, missing, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate
  - Perspective Navigation (12): start, routes, follow, back, peek, inspect, suggest, affinity, branch, compare, list, close
  - Lock System (5): create, watch, diff, rebase, release
  - Superpowers (13): hypothesize, counterfactual, predict, fingerprint, resonate, trace, validate_plan, differential, trail.save, trail.resume, trail.merge, trail.list, seek
- **JSON-RPC over stdio**: compatible with MCP protocol version 2024-11-05
- **Dual transport**: framed (Content-Length headers) and line-delimited JSON-RPC
- **Auto-persistence**: configurable interval (default: every 50 queries) + on shutdown
- **Multi-agent support**: agent ID tracking, perspective isolation, shared graph
- **Tool name normalization**: underscores automatically converted to dots (e.g., `m1nd_activate` -> `m1nd.activate`)

### Performance (measured on 335-file Python backend, ~52K lines)

- Full ingest: 910ms (9,767 nodes, 26,557 edges)
- Spreading activation: 31-77ms
- Blast radius: 5-52ms
- Counterfactual: 3ms
- Hypothesis testing: 58ms (25,015 paths)
- Lock diff: 0.08us
- Trail merge: 1.2ms
- Memory footprint: ~50MB typical

### Known Limitations

- Semantic scoring uses trigram matching, not neural embeddings (planned for v0.2)
- No tree-sitter integration yet (planned for v0.2)
- 6 languages with dedicated extractors; others use generic fallback
- Graph is fully in-memory; very large codebases (400K+ files) need ~80MB
- No dataflow or taint analysis (out of scope; use dedicated SAST tools)

---

## Planned: v0.2.0

- Tree-sitter integration for 64+ language support
- Optional embedding-based semantic scoring
- Graph partitioning for very large codebases
- Community detection algorithms
- Performance optimizations for 100K+ node graphs
- MCP Streamable HTTP transport (in addition to stdio)

---

## Planned: v0.3.0

- Distributed graph (multi-machine federation)
- Real-time file watcher integration
- Plugin system for custom extractors and tools
- Graph visualization export (DOT, D3.js, Mermaid)
- Metrics and observability (Prometheus, OpenTelemetry)
