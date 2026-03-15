# Changelog

All notable changes to m1nd are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-03-15

### Added

- **`surgical_context_v2`** — Returns source code of all connected files (callers + callees + tests) in a single call. The agent gets complete dependency context without multiple round-trips. Params: `file_path`, `include_connected_sources` (default: true), `max_connected_files` (default: 5), `max_lines_per_file` (default: 500). Measured at 1.3ms.

- **`apply_batch`** — Accepts an array of file edits and writes all of them atomically (temp + rename per file), then re-ingests the graph once at the end. Returns per-file diffs. Params: `edits[]` (each with `file_path` + `new_content`), `atomic` (default: true). Measured at 165ms for typical multi-file changesets.

### Fixed

- **#15** — `epidemic` tool: SIR propagation returned incorrect node counts when the graph had zero-degree nodes. Fixed normalization in the epidemic kernel.

- **#10** — `flow_simulate`: race condition false positives on single-threaded execution paths. Added thread-count guard before flagging concurrent-access patterns.

### Changed

- Tool count: 54 → 56
- Version bump: 0.2.1 → 0.3.0 across all three crates (m1nd-core, m1nd-ingest, m1nd-mcp)

---

## [0.2.1] - 2026-03-06

### Added

- `surgical_context` — Complete context for a single code node in one call: source code, callers, callees, tests, trust score, blast radius.
- `apply` — Write edited code back to file atomically, re-ingest graph, run co-change prediction.
- Surgical tools category (2 tools at launch).

### Changed

- Tool count: 52 → 54
- `layers` and `layer_inspect` promoted from experimental to stable.

---

## [0.2.0] - 2026-02-20

### Added

- Perspective Navigation system (12 tools): start, routes, follow, back, peek, inspect, suggest, affinity, branch, compare, list, close.
- Lock System (5 tools): create, watch, diff, rebase, release. `lock.diff` at 0.08μs.
- Superpowers Extended (9 tools): antibody_scan, antibody_list, antibody_create, flow_simulate, epidemic, tremor, trust, layers, layer_inspect.

### Changed

- Tool count: 13 → 52 (Foundation + Perspectives + Locks + Superpowers + Extended)

---

## [0.1.0] - 2026-01-15

### Added

- Foundation (13 tools): ingest, activate, impact, why, learn, drift, health, seek, scan, timeline, diverge, warmup, federate.
- Superpowers (13 tools): hypothesize, counterfactual, missing, resonate, fingerprint, trace, validate_plan, predict, trail.save, trail.resume, trail.merge, trail.list, differential.
- MCP server (JSON-RPC over stdio + HTTP).
- GUI (auto-launch on MCP start).
- Hebbian plasticity — graph learns from every query.
- Multi-agent support — single m1nd instance serves all agents.
