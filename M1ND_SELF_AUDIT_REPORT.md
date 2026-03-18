# m1nd Self-Audit — Bugs Found via m1nd on m1nd

**Date:** 2025-03-15  
**Method:** m1nd ingest + activate, flow_simulate, missing, impact, fingerprint  
**Graph:** 2196 nodes, 6061 edges (m1nd codebase)

---

## Executive Summary

m1nd was used to analyze its own codebase. Findings:

| Category | Count | Severity |
|----------|-------|----------|
| **Turbulence (race hotspots)** | 64 | Medium |
| **Valve (lock contention)** | 16 | Info |
| **Structural holes** | 10 | Medium |
| **API inconsistency** | 1 | Medium |
| **Duplicates** | 0 | — |

---

## 1. Flow Simulate — Race Condition Hotspots

**64 turbulence points** where multiple entry paths converge without upstream lock.

### 1.1 Critical: `perspective/confidence.rs`

**Node:** `m1nd-mcp/src/perspective/confidence.rs`  
**Entry pairs:** `normalize_route_path_neighborhood`, `resonant_normalization_zero_max`, `route_path_neighborhood`  
**has_lock:** false | **is_read_only:** false

Three independent entry points converge on the same file. All normalization helpers (`normalize_ghost_edge`, `normalize_structural_hole`, `normalize_resonant_amplitude`, `normalize_semantic_overlap`, `normalize_provenance_overlap`, `normalize_route_path_neighborhood`, `compute_combined_confidence`, etc.) are reachable from multiple paths with no lock.

**Recommendation:** If `confidence.rs` is called from concurrent handlers (e.g. perspective routes + affinity), add synchronization or document single-threaded guarantee.

### 1.2 API Inconsistency: `inspect_input_requires_route_ref` vs `follow_input_accepts_route_id`

**Entry pairs:** `inspect_input_requires_route_ref` / `follow_input_accepts_route_id`  
**Affected nodes:** `PerspectiveStartInput`, `PerspectiveRoutesInput`, `PerspectiveInspectInput`, `PerspectiveFollowInput`, `PerspectivePeekInput`, `PerspectiveSuggestInput`, `PerspectiveAffinityInput`, `PerspectiveBranchInput`, `PerspectiveBackInput`, `PerspectiveCompareInput`, `PerspectiveListInput`, `PerspectiveCloseInput`, and all corresponding Output types.

Two different input schemas feed the same perspective flow. `inspect` requires `route_ref`; `follow` accepts `route_id`. This may cause:
- Client confusion (which param to pass?)
- Handler logic branching on missing/conflicting fields
- Potential deserialization or validation gaps

**Recommendation:** Unify route reference semantics across perspective tools. Add integration tests for both `route_id` and `route_index` paths.

### 1.3 Session State Convergence

**Paths:** `detect_route_registrations` → `session_summary`  
**Paths:** `detect_route_registrations` → `observation_count` (via TremorRegistry)  
**Paths:** `detect_route_registrations` → `record_observation_increments_count`, `acceleration_detected_for_rapidly_changing_node`, `deceleration_produces_decelerating_direction`, `linear_regression_slope`

`SessionState` is a central hub. Multiple ingest/cross-file flows converge on it. TremorRegistry (`observation_count`, etc.) is downstream of SessionState. If ingest and tremor run concurrently, shared state access may need review.

---

## 2. Structural Holes (missing)

Nodes with many activated neighbors but **inactive** — potential gaps in error handling or test coverage.

| Node | Neighbors | Reason |
|------|-----------|--------|
| `test_e2e.sh` | 14 | Test script not in activation cone |
| `test_perspective_e2e.sh` | 15 | Same |
| `test_perspective_usecases.sh` | 2 | Same |
| `m1nd-mcp/src/tools.rs` | 2 | Core dispatch — verify error propagation |
| `m1nd-mcp/src/engine_ops.rs` | 2 | Engine ops — verify error propagation |
| `m1nd-mcp/src/http_server.rs` | 3 | HTTP layer — verify error propagation |
| `m1nd-mcp/src/lock_handlers.rs` | 3 | Lock handlers — verify error propagation |
| `m1nd-mcp/src/perspective_handlers.rs` | 3 | Perspective handlers — verify error propagation |
| `m1nd-ingest/src/walker.rs` | 2 | File walker — verify error propagation |
| `m1nd-ingest/src/lib.rs` | 2 | Ingest entry — verify error propagation |

**Recommendation:** Audit these files for unhandled `M1ndError` or `Result` propagation. Add tests that exercise error paths.

---

## 3. Impact Analysis

### 3.1 `m1nd-core/src/error.rs`

**Blast radius:** 50+ nodes (all `M1ndError` variants + types)  
**total_energy:** 2.55

Changing `error.rs` cascades to every tool and handler. Any new error variant or signature change requires coordinated updates across m1nd-mcp and m1nd-ingest.

### 3.2 `m1nd-mcp/src/session.rs`

**Blast radius:** Large (SessionState is central).  
Changing session layout affects: engine_ops, lock_handlers, perspective_handlers, tremor, ingest callbacks.

---

## 4. Valve Points (Lock Contention)

16 valve points detected — nodes that serialize flow. Notable:

- `mark_all_lock_baselines_stale`, `next_lock_id`, `agent_lock_count`, `perspective_and_lock_memory_bytes` — lock-related
- `LockScope`, `LockScopeConfig`, `LockSnapshot`, `LockState`, `LockDiffResult` — lock state types
- `valve_detected_on_lock_node` — test confirms valve detection

These are expected (lock subsystem). No action unless profiling shows contention.

---

## 5. Fingerprint — Duplicates

**Result:** 0 equivalent pairs at 0.85 similarity. No obvious code duplication.

---

## 6. Activate — Error Handling Map

**Seeds:** error, M1ndError, error_display_empty_graph, error_display_dangling_edge, louvain_empty_graph_error, IngestError, etc.

**Ghost edges:** error → M1ndError, error → ErrorResponse, error → is_error (tests), error → ApiError, AppErrorBoundary, NodeErrorBoundary, CanvasErrorBoundary.

Error handling is well-connected. `l6_extract_error_info` in layer_handlers.rs is a single point for error extraction — verify it covers all M1ndError variants.

---

## 7. Recommended Actions

1. **confidence.rs:** Document or add sync for concurrent perspective/affinity calls.
2. **Perspective API:** Unify `route_ref` vs `route_id` semantics; add tests.
3. **Structural holes:** Add error-path tests for tools.rs, engine_ops, http_server, lock_handlers, perspective_handlers, walker, lib.
4. **SessionState:** Review TremorRegistry + ingest concurrent access if both can run during same session.

---

*Generated by m1nd self-audit. Query time: ~2.5s. Zero LLM tokens.*
