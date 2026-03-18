# SURGICAL_CONTEXT_V2 + APPLY_BATCH — Adversarial Hardening Report

**Agent**: ADVERSARY-V2
**Date**: 2026-03-15
**Grounding**: surgical_handlers.rs, protocol/surgical.rs, protocol/layers.rs, tools.rs (handle_ingest, finalize_ingest), session.rs (rebuild_engines, invalidate_all_perspectives, notify_watchers), graph.rs (CSR, SharedGraph, resolve_id, merge_node_provenance), merge.rs (merge_graphs), diff.rs (GraphDiff, DiffAction), lock_handlers.rs (require_lock, capture_baseline), test_surgical.rs
**Predecessor**: SURGICAL_CONTEXT_HARDENING.md (V1 report, same date)

---

## EXECUTIVE SUMMARY

surgical_context_v2 and apply_batch extend V1 with multi-file semantics. V2 amplifies every V1 risk by a factor proportional to the number of connected files. This report identifies **28 failure modes** across **10 categories**, with **7 that block shipment**.

Key difference from V1:
- **surgical_context_v2**: Returns source code of ALL connected files (callers + callees + imports), not just the target. One call = complete dependency context.
- **apply_batch**: Accepts an array of file edits, writes ALL atomically, re-ingests ONCE, returns diffs for all files.

---

## STEP 1: V2 ARCHITECTURE ANALYSIS

### surgical_context_v2 data flow (proposed)

```
INPUT: { file_path, agent_id, radius, include_tests }

1. Read target file from disk (same as V1)
2. Find graph nodes for this file (same as V1 via find_nodes_for_file)
3. BFS to radius N to gather callers / callees / tests (same as V1 via collect_neighbours)
4. FOR EACH unique file_path in neighbours:
     a. Read that file from disk
     b. Extract symbols from that file
     c. Package as ConnectedFileContext
5. Return target file + ALL connected file contents
```

### apply_batch data flow (proposed)

```
INPUT: { edits: [{ file_path, new_content, description }], agent_id, reingest }

1. FOR EACH edit in edits:
     a. Resolve and validate path (same as V1 via validate_path_safety)
     b. Read old content for diff
     c. Atomic write (temp + rename)
2. IF reingest:
     a. Re-ingest ALL modified files (single ingest call)
     b. Rebuild engines ONCE
3. Return array of per-file diffs + updated node IDs
```

### What V1 already has (implemented, grounded)

From `surgical_handlers.rs`:
- `resolve_file_path()`: Handles absolute + workspace-relative paths (line 28-37)
- `validate_path_safety()`: Canonicalize + ingest_roots check (line 41-72)
- `diff_summary()`: Line-based added/removed count (line 75-84)
- `extract_symbols()`: Heuristic parser for Rust/Python/TS/Go (line 88-106)
- `collect_neighbours()`: BFS with radius, visited set, caller/callee/test classification (line 346-482)
- `find_nodes_for_file()`: O(n) scan of all nodes for path match (line 495-523)
- `handle_surgical_context()`: Full pipeline for single file (line 543-633)
- `handle_apply()`: Atomic write + optional re-ingest for single file (line 650-771)

### What V1 does NOT have (gaps for V2)

1. No connected-file content retrieval
2. No batch write orchestration
3. No per-file rollback on batch failure
4. No memory budget for aggregate response size
5. No de-duplication of connected files across multiple BFS roots

---

## STEP 2: EVERY FAILURE MODE

### CATEGORY A: Memory Explosion (V2-specific)

#### A1: Connected file count explosion
**Trigger**: Target file is a hub node. Example: `chat_handler.py` (3810 lines, highest PageRank 0.449) has dozens of callers and callees. At radius=2, BFS walks into hundreds of neighbours.
**Current V1 state**: `collect_neighbours()` returns Vec with no size cap. V1 returns only metadata (node_id, label, file_path, relation, weight). V2 would return full source code for each.
**Quantified risk**: 60 connected files at 500 lines average = 30,000 lines of source code in one response. At ~50 bytes/line = 1.5MB payload. Not catastrophic, but with radius=2 on a dense graph, this can reach 200+ files = 10MB+.
**Impact**: Response payload exceeds LLM context window. Agent burns tokens on irrelevant files. Network latency. Memory pressure on m1nd server (string allocation for all file reads).
**Mitigation**:
  1. HARD CAP: `max_connected_files: u32` parameter (default: 10, max: 30). Return only the top-N by edge weight.
  2. HARD CAP: `max_total_lines: u32` parameter (default: 5000, max: 20000). Truncate aggregate response.
  3. Per-connected-file truncation: return only the symbols referenced by the edge relation, not the full file. Example: if the edge is `calls::validate_session`, return only the `validate_session` function body.
  4. Response metadata: always include `"truncated_files": N, "total_available": M` so the agent knows context is incomplete.

#### A2: Duplicate file reads across BFS roots
**Trigger**: When `primary_node` is None and the fallback iterates all `file_nodes` (line 596-605), each node's BFS may visit the same neighbour files. V2 would read the same file from disk multiple times.
**Current V1 state**: Duplicates appear in callers/callees lists (no dedup in the fallback path).
**Impact**: Wasted disk I/O. Duplicate entries in response.
**Mitigation**: Build a `HashSet<PathBuf>` of files already read. Skip duplicates. The BFS visited set prevents node-level duplication but not file-level (multiple nodes can map to the same file).

#### A3: Binary or non-UTF-8 files in connected set
**Trigger**: A Rust file depends on a .wasm, .so, or generated binary file. BFS reaches a node whose provenance points to a binary.
**Current V1 state**: `handle_surgical_context()` uses `std::fs::read_to_string()` which returns `Err` for non-UTF-8 files (line 551). V1 fails entirely on this error.
**Impact for V2**: One binary file in the connected set fails the entire V2 call.
**Mitigation**: V2 must skip non-UTF-8 files gracefully. Record `"skipped_files": [{ "path": ..., "reason": "non-utf8" }]` in response.

### CATEGORY B: Partial Write Corruption (apply_batch-specific)

#### B1: File N of M fails to write — no rollback of files 1..N-1
**Trigger**: apply_batch writes files sequentially. File 3 of 5 hits a permission error, disk full, or path validation failure.
**Current V1 state**: V1 `handle_apply()` writes one file atomically (temp+rename). No multi-file coordination.
**Impact**: Files 1-2 have new content on disk. Files 3-5 still have old content. If re-ingest runs on the partial set, the graph reflects an inconsistent state. Agents that read from the graph see a frankenstate.
**Severity**: CRITICAL. This is the fundamental atomicity problem of multi-file writes.
**Mitigation**:
  1. **PRE-FLIGHT VALIDATION**: Before writing ANY file, validate ALL paths, check ALL permissions, verify ALL ingest roots. Fail fast before the first write.
  2. **BACKUP-AND-RESTORE**: Before writing file N, copy original content to `{path}.m1nd-batch-backup`. On ANY failure, restore ALL backup files, then delete backups. This is the minimum viable rollback.
  3. **TWO-PHASE COMMIT**: Phase 1: Write all files to temp paths (`.m1nd_batch_{pid}_{idx}.tmp`). Phase 2: Rename all temp files to targets atomically. If any rename fails, delete all temp files that were renamed and restore the originals.
  4. **RESPONSE STATUS**: Return per-file status in the response: `"edits": [{ "file_path": ..., "status": "ok|failed|rolled_back", "error": ... }]`.
  5. **NO PARTIAL RE-INGEST**: Re-ingest only runs if ALL writes succeed. Never re-ingest a partial state.

#### B2: Temp file collision in batch
**Trigger**: Current V1 temp path is `.m1nd_apply_{pid}.tmp` (line 667-669). In a batch, ALL files in the batch would use the same temp path because pid is constant within a process.
**Impact**: File 2's temp write overwrites file 1's temp file before file 1 is renamed.
**Severity**: CRITICAL. Data loss.
**Mitigation**: Include a per-edit index in the temp path: `.m1nd_batch_{pid}_{idx}.tmp`. Or use a unique random suffix per edit.

#### B3: Rename fails across filesystem boundaries
**Trigger**: Two edited files live on different mount points. `std::fs::rename()` cannot atomic-rename across filesystems.
**Current V1 state**: V1 already has this risk (line 689) but it's single-file. Batch amplifies it because mixed-filesystem edits are more likely when editing files from multiple connected modules.
**Impact**: Rename returns EXDEV. Cleanup code removes the temp file (line 691) but the target was never updated.
**Mitigation**: Fall back to copy+delete when rename fails with EXDEV. Document that cross-filesystem atomicity is best-effort.

### CATEGORY C: Graph Consistency During Batch Re-ingest

#### C1: rebuild_engines() invalidates ALL perspectives
**Trigger**: `apply_batch` calls re-ingest, which calls `finalize_ingest()` -> `rebuild_engines()` -> `invalidate_all_perspectives()` (session.rs line 263). Every perspective for every agent is marked stale.
**Current V1 state**: Same issue in V1 but one file write is fast. Batch write + batch re-ingest takes longer, extending the window during which all perspectives are invalid.
**Impact**: Agent A is mid-investigation using perspective navigation. Agent B does an `apply_batch` on unrelated files. Agent A's perspective is invalidated. All accumulated navigation state (visited nodes, route caches) is lost.
**Severity**: HIGH for multi-agent deployments.
**Mitigation**:
  1. **SCOPED INVALIDATION**: Instead of `invalidate_all_perspectives()`, compute the set of affected node IDs from the modified files. Only invalidate perspectives that have visited any of those nodes.
  2. **IMPLEMENTATION**: After re-ingest, diff pre/post node sets. For each changed node, check `perspective.visited_nodes`. Only mark perspectives that intersect the changed set as stale.
  3. **FALLBACK**: If the diff is too expensive, maintain the current behavior but document it. Agents should `trail_save` before calling `apply_batch`.

#### C2: Incremental ingest path is broken
**Trigger**: `handle_apply()` calls `handle_ingest()` with `incremental: true` (line 717-723). But `handle_ingest()` incremental path (tools.rs line 1336-1349) returns raw JSON without calling `finalize_ingest()`.
**Grounded evidence**: tools.rs line 1340 calls `ingestor.ingest_incremental()` which returns `(diff, stats)`. The `diff` (GraphDiff) is NEVER applied to the main graph. It just returns the stats as JSON.
**Impact**: After `apply` writes a file and "re-ingests", the graph is NOT updated. The graph still reflects the old file content. All subsequent queries return stale data.
**Severity**: CRITICAL. **This is a BUG in the current V1 implementation.**
**Mitigation**:
  1. Fix `handle_ingest()` incremental path to: (a) apply the diff to the graph via `diff.apply(&mut graph)`, (b) call `graph.finalize()`, (c) call `state.rebuild_engines()`.
  2. Alternatively, for `apply` specifically, use the non-incremental path but scoped to only the modified file(s). Read the current graph, extract nodes from the modified file, re-parse just that file, merge the result.

#### C3: Node removal is a no-op in diff application
**Trigger**: `DiffAction::RemoveNode` in diff.rs (line 191-194) does nothing: `// Graph doesn't support node removal in CSR — mark as removed via tag or skip. For now, count it.`
**Impact**: When `apply_batch` modifies a file that removed a function, the old function's node persists as a ghost in the graph. Queries still find it. Impact analysis includes it. Predictions reference it.
**Severity**: HIGH. Accumulates over time. The graph becomes increasingly inaccurate.
**Mitigation**:
  1. Implement soft-delete: add a `deleted: bool` field to NodeProvenance. Filter deleted nodes from ALL query paths.
  2. Or: for the specific case of apply re-ingest, use full (non-incremental) re-ingest for just the modified files. This replaces all nodes from those files with fresh ones. But requires the merge path to handle "overlay replaces base nodes from same file".

#### C4: Graph write lock held during entire batch re-ingest
**Trigger**: `finalize_ingest()` acquires `state.graph.write()` (tools.rs line 53) for the entire merge + finalize operation. For a batch of N files, this lock is held for N * parse_time + merge_time + finalize_time.
**Impact**: ALL other queries block. In a multi-agent system, this stalls every agent.
**Severity**: MEDIUM for small batches (5 files, ~200ms total). HIGH for large batches (20+ files).
**Mitigation**:
  1. Build the new merged graph outside the write lock. Only hold the write lock for the final swap (`*graph = combined_graph`).
  2. Current code already does this partially: `merge_graphs()` creates a new graph from base + overlay. The write lock is only for the assignment. BUT: `graph.finalize()` is called inside the write lock (line 55-57). Move finalization outside.

### CATEGORY D: Circular Dependencies and Infinite Expansion

#### D1: Circular import chains in connected file collection
**Trigger**: File A imports File B, File B imports File A. V2 BFS walks: A -> B -> A (but A is in visited set, so stops). However, if `find_nodes_for_file()` returns MULTIPLE nodes per file (function-level nodes), the visited set is per-node, not per-file. Node `fn::A::foo` visits Node `fn::B::bar`, which visits Node `fn::A::baz` (different node, same file).
**Current V1 state**: `collect_neighbours()` uses a `visited: HashSet<NodeId>` (line 369). This prevents node-level cycles. But V2's "read all connected files" would read File A's content once for `fn::A::foo`'s discovery and potentially again for `fn::A::baz`'s discovery.
**Impact**: Not infinite (visited set prevents node revisits), but redundant file reads and duplicate file entries in the response.
**Mitigation**: V2 must maintain a `files_read: HashSet<PathBuf>` in addition to the node-level visited set. Skip file reads for already-read paths.

#### D2: Star topology explosion
**Trigger**: A utility module imported by 100+ files. At radius=1, V2 returns the utility module + all 100 importers' source code.
**Impact**: Massive response. Most importers are irrelevant to the edit task.
**Mitigation**: The `max_connected_files` cap from A1 handles this. Additionally, prioritize files by edge weight (already sorted in V1, line 477-479). Return only the highest-weight connections.

#### D3: Radius > 1 causes combinatorial explosion
**Trigger**: At radius=1, a node with degree 20 returns 20 files. At radius=2, each of those 20 has degree 20, returning 400 files (minus visited). At radius=3, potentially 8000.
**Current V1 state**: Default radius is 1 (protocol/surgical.rs line 39). No hard cap on radius.
**Impact**: radius=3 on a dense graph could attempt to read thousands of files.
**Mitigation**:
  1. HARD CAP radius to 2 maximum. Reject radius > 2 with error.
  2. Apply `max_connected_files` cap regardless of radius.
  3. Document that radius=2 is rarely needed for surgical editing.

### CATEGORY E: Security in Batch Operations

#### E1: Path validation bypass through batch composition
**Trigger**: An agent crafts a batch where edit 1 creates a symlink inside the workspace pointing to `/etc/passwd`, and edit 2 writes to that symlink path. Edit 2 passes `validate_path_safety()` because the symlink is within the workspace, but the canonical target is outside.
**Current V1 state**: `validate_path_safety()` calls `canonicalize()` which resolves symlinks (line 46-49). This DOES protect against this attack because the canonical path would be `/etc/passwd`, which is outside ingest_roots.
**Impact**: LOW. V1's canonicalization already handles this.
**Finding**: V1's security is adequate for this vector. No V2-specific mitigation needed.

#### E2: TOCTOU between validation and write in batch
**Trigger**: Between `validate_path_safety()` and `std::fs::write()`, another process creates a symlink at the validated path pointing outside the workspace.
**Impact**: The validated canonical path is correct at check time but the actual write goes elsewhere.
**Severity**: LOW. This is a fundamental OS-level TOCTOU issue. Mitigation would require O_NOFOLLOW or tmpfile+linkat, which is platform-specific.
**Mitigation**: Document as known limitation. The temp-file-then-rename pattern partially mitigates this (rename replaces the symlink itself, not its target — but this depends on filesystem semantics).

#### E3: Batch allows writing to m1nd state files
**Trigger**: A batch edit targets `graph_snapshot.json`, `antibodies.json`, or other m1nd persistence files.
**Current V1 state**: V1 has NO deny-list for m1nd state files. `validate_path_safety()` only checks ingest_roots allow-list. If the m1nd data directory is within an ingest root (which it often is), these files are writable.
**Impact**: Corrupted m1nd state. Requires manual recovery.
**Severity**: HIGH.
**Mitigation**: Add a deny-list in `validate_path_safety()`: reject any path matching `**/graph_snapshot.json`, `**/plasticity_state.json`, `**/antibodies.json`, `**/tremor_state.json`, `**/trust_state.json`, `**/trails/*.json`.

#### E4: Empty ingest_roots allows writes anywhere
**Trigger**: `validate_path_safety()` (line 52-54): `if ingest_roots.is_empty() { return Ok(canonical); }`. If no ingest has been done yet, ANY path is allowed.
**Current V1 state**: This is already in the V1 hardening report (G1). Still unfixed.
**Impact**: Before the first ingest, `apply` and `apply_batch` can write to any path on the filesystem.
**Severity**: CRITICAL.
**Mitigation**: If ingest_roots is empty, REFUSE all writes. Require at least one ingest before any apply operation.

### CATEGORY F: Concurrency and Locking

#### F1: apply_batch does not check lock registry
**Trigger**: Agent A holds a lock on `chat_handler.py`. Agent B calls `apply_batch` with an edit to `chat_handler.py`.
**Current V1 state**: V1 `handle_apply()` does NOT check the lock registry (line 650-771). The lock system is purely advisory and only used by `lock_handlers.rs`.
**Impact**: Lock system is meaningless if apply ignores it. Multi-agent coordination breaks down.
**Severity**: HIGH.
**Mitigation**:
  1. Before any write, scan `state.locks` for locks whose scope includes the target file.
  2. If a lock exists and the lock owner is not the calling agent, return error with the lock details.
  3. For `apply_batch`, check ALL files against ALL locks before writing ANY file. Fail the entire batch if any file is locked.

#### F2: Concurrent apply_batch calls from different agents
**Trigger**: Agent A calls `apply_batch` on files [X, Y]. Agent B simultaneously calls `apply_batch` on files [Y, Z]. File Y is written by both.
**Impact**: Last write wins. The graph reflects only the last writer's content for Y.
**Severity**: CRITICAL.
**Mitigation**:
  1. Acquire an exclusive process-level mutex before the batch write loop. Only one apply_batch at a time.
  2. Or: acquire per-file advisory locks (flock) for all files in the batch before writing. Release after re-ingest.
  3. Document that apply_batch is NOT safe for concurrent use on overlapping file sets without external coordination.

#### F3: Re-ingest during batch invalidates the calling agent's perspective
**Trigger**: The agent that calls `apply_batch` has an active perspective. The re-ingest invalidates its own perspective.
**Impact**: The agent loses its navigation state immediately after applying changes.
**Severity**: MEDIUM.
**Mitigation**: Exempt the calling agent's perspectives from invalidation. Or: return the perspective state in the apply_batch response so the agent can resume without a new `perspective_start`.

### CATEGORY G: Response Quality and Coherence

#### G1: Connected file content becomes stale during the call
**Trigger**: V2 reads file A, then file B, then file C. While reading C, another process modifies A. The response contains stale content for A.
**Impact**: LOW. This is inherent to any multi-file read. The time window is milliseconds.
**Mitigation**: Include `"content_hash": sha256(content)` for each file in the response. Agents can use this for optimistic concurrency checks in subsequent apply calls.

#### G2: Symbol extraction is heuristic and misses patterns
**Trigger**: `extract_symbols()` uses regex-style line matching (line 88-106). It misses: nested functions, lambda assignments, decorator-heavy Python, macro-generated Rust code, JSX components.
**Current V1 state**: This is already a V1 limitation. V2 amplifies it because connected files also get symbol extraction.
**Impact**: Missing symbols in the response. Agent doesn't see important code structures.
**Mitigation**: Document the limitations. For V2, symbol extraction on connected files is OPTIONAL — the primary value is the full source code. Symbol extraction is a bonus for navigation.

#### G3: Diff summary is imprecise
**Trigger**: `diff_summary()` uses set-based line comparison (line 75-84). Reordered but identical lines show as 0 changes. Duplicated lines undercount changes.
**Impact**: LOW. The diff is informational, not used for logic.
**Mitigation**: Consider using a proper LCS-based diff for accuracy. For V2 batch, per-file diffs are critical for agent decision-making.

### CATEGORY H: V1/V2 Protocol Divergence

#### H1: Two incompatible SurgicalContextInput types exist
**Current state**: `protocol/surgical.rs` defines `SurgicalContextInput` with `file_path` as the key field (line 22-37). `protocol/layers.rs` defines a DIFFERENT `SurgicalContextInput` with `node_id` as the key field (line 1280-1303). The golden tests import from `layers.rs`. The actual handler uses `surgical.rs`.
**Impact**: The golden tests (test_surgical.rs) test a DIFFERENT protocol than what's implemented. They are structurally valid but semantically disconnected from the actual handler.
**Severity**: HIGH for test confidence. The tests pass but don't test the real code.
**Mitigation**: V2 must resolve this split. Either:
  1. Converge to one protocol (prefer `surgical.rs` since it's what's implemented), update golden tests.
  2. Or implement both as separate tools: `m1nd.surgical_context` (by file_path, implemented) and `m1nd.surgical_context.node` (by node_id, from layers.rs, not yet implemented).

#### H2: V1 apply takes file_path + new_content (full replacement). Layers.rs apply takes node_id + line range.
**Current state**: The implemented `handle_apply()` in `surgical_handlers.rs` takes `file_path` + `new_content` (full file replacement, line 650). The `ApplyInput` in `protocol/layers.rs` takes `node_id` + `line_start` + `line_end` + `new_content` (line-range replacement, line 1383-1405).
**Impact**: These are fundamentally different operations. Full-file replacement is safer (no line-number drift) but heavier. Line-range replacement is surgical but fragile.
**Severity**: MEDIUM. Both approaches have valid use cases.
**Mitigation for V2**: `apply_batch` should support BOTH modes:
  1. `mode: "full"` — full file replacement (current implementation).
  2. `mode: "range"` — line-range replacement (layers.rs spec). Requires line-range splicing logic.
  Default to `"full"` for safety.

### CATEGORY I: Performance

#### I1: O(n) scan in find_nodes_for_file()
**Trigger**: `find_nodes_for_file()` (line 495-523) iterates ALL nodes in the graph to find matches by source_path. Called once per file in V1. In V2, called once for the target file + once for each connected file.
**Current V1 state**: For a graph with 5000 nodes, each call scans 5000 nodes. V2 with 10 connected files = 50,000 node scans.
**Impact**: MEDIUM. Each scan is O(n) string comparison. For graphs under 10K nodes, this is <1ms total. For 100K+ node graphs, this becomes noticeable.
**Mitigation**: Build a reverse index `HashMap<PathBuf, Vec<NodeId>>` during ingest. Store in SessionState. O(1) lookup per file.

#### I2: resolve_external_id() is O(n) on id_to_node
**Trigger**: `resolve_external_id()` (line 485-492) iterates ALL entries in `id_to_node` HashMap to find a NodeId -> String reverse mapping. Called once per neighbour node.
**Current V1 state**: V1 calls this for each caller/callee. With 30 neighbours, that's 30 * map_size lookups. Each lookup is O(map_size) because it iterates.
**Impact**: For a graph with 5000 nodes and 30 neighbours: 150,000 comparisons. Measurable but not catastrophic.
**Mitigation**: Build a reverse map `HashMap<NodeId, InternedStr>` (or `Vec<InternedStr>` indexed by NodeId) during graph finalization. O(1) reverse lookup.

#### I3: Connected file reads are sequential
**Trigger**: V2 reads connected files one by one from disk.
**Impact**: 10 files * ~1ms per file read = 10ms. Acceptable.
**Mitigation**: Not needed for <30 files. For larger connected sets, consider parallel file reads via thread pool.

### CATEGORY J: Test Coverage Gaps (V2-specific)

#### J1: No test for batch write rollback
**Current state**: No test exists for the scenario where file 3 of 5 fails to write.
**Required test**: Create 5 temp files, mock a permission error on the 3rd, verify ALL files have original content after rollback.

#### J2: No test for connected file memory cap
**Required test**: Create a graph with a hub node connected to 100+ files. Call V2. Verify response respects `max_connected_files` cap.

#### J3: No test for incremental ingest integration with apply
**Current state**: The incremental ingest path doesn't update the graph (C2 finding). No test catches this because there's no integration test that writes a file, re-ingests, and verifies the graph changed.
**Required test**: Write a file adding a new function. Call apply with reingest=true. Query the graph for the new function's node. Verify it exists.

#### J4: No test for concurrent apply_batch from two agents
**Required test**: Two threads call apply_batch simultaneously on overlapping file sets. Verify either (a) one fails with a lock error, or (b) the final state is consistent.

#### J5: No test for binary file in connected set
**Required test**: Create a graph where a code file depends on a binary file. Call V2. Verify binary is skipped gracefully, not error.

#### J6: No test for empty ingest_roots blocking writes
**Required test**: Call apply without prior ingest (ingest_roots is empty). Verify the write is REFUSED.

#### J7: No test for m1nd state file deny-list
**Required test**: Call apply with file_path pointing to `graph_snapshot.json`. Verify rejection.

---

## BUGS FOUND DURING ANALYSIS (existing V1 code)

### BUG 1: Incremental ingest does not update the graph
**File**: `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-mcp/src/tools.rs`
**Lines**: 1336-1349
**Code path**: `handle_ingest()` with `incremental: true`
**Issue**: Calls `ingestor.ingest_incremental()` which returns `(diff, stats)`. The `diff: GraphDiff` is NEVER applied to the graph. The function returns stats JSON immediately. No call to `diff.apply()`, no `finalize_ingest()`, no `rebuild_engines()`.
**Impact**: Every `apply` call with `reingest: true` that takes the incremental path leaves the graph unchanged. The file on disk has new content; the graph still reflects old content. All subsequent queries are stale.
**Root cause**: The incremental path was "simplified" (comment on line 1338: `// For incremental, we'd need changed files list -- simplified here`). The simplification omitted the most critical step.
**Fix**: Apply the diff to the graph, finalize, rebuild:
```rust
let mut graph = state.graph.write();
diff.apply(&mut graph)?;
graph.finalize()?;
drop(graph);
state.rebuild_engines()?;
```

### BUG 2: Node removal is a no-op
**File**: `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-ingest/src/diff.rs`
**Lines**: 191-194
**Code**: `DiffAction::RemoveNode(_ext_id) => { applied += 1; }` (just counts, doesn't remove)
**Comment**: `// Graph doesn't support node removal in CSR — mark as removed via tag or skip. For now, count it.`
**Impact**: Deleted symbols remain as ghost nodes indefinitely. Graph accuracy degrades with every edit cycle.

### BUG 3: Staleness detection dead code (already reported in V1 report)
**File**: `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-mcp/src/perspective_handlers.rs`
**Line**: 606
**Status**: Still unfixed. V2 inherits this bug.

---

## RISK MATRIX

| ID | Category | Severity | Likelihood | Blocks V2 Ship? |
|----|----------|----------|------------|-----------------|
| A1 | Memory explosion | HIGH | HIGH | YES |
| A2 | Duplicate file reads | LOW | MEDIUM | No |
| A3 | Binary file crash | MEDIUM | MEDIUM | No |
| B1 | Partial write, no rollback | CRITICAL | HIGH | **YES** |
| B2 | Temp file collision | CRITICAL | HIGH | **YES** |
| B3 | Cross-filesystem rename | LOW | LOW | No |
| C1 | Perspective invalidation blast | HIGH | HIGH | No (document) |
| C2 | **Incremental ingest broken (BUG)** | CRITICAL | CERTAIN | **YES** |
| C3 | Node removal no-op (BUG) | HIGH | CERTAIN | No (pre-existing) |
| C4 | Graph write lock duration | MEDIUM | MEDIUM | No |
| D1 | Circular file reads | LOW | MEDIUM | No |
| D2 | Star topology explosion | HIGH | MEDIUM | Covered by A1 |
| D3 | Radius explosion | HIGH | LOW | No (cap radius) |
| E3 | m1nd state file writes | HIGH | LOW | No (add deny-list) |
| E4 | **Empty ingest_roots = write anywhere** | CRITICAL | MEDIUM | **YES** |
| F1 | Lock registry ignored | HIGH | MEDIUM | No (advisory) |
| F2 | Concurrent batch race | CRITICAL | LOW | No (document) |
| F3 | Self-invalidation | MEDIUM | HIGH | No |
| H1 | Protocol divergence (two Input types) | HIGH | CERTAIN | **YES** |
| H2 | Full vs range apply mismatch | MEDIUM | CERTAIN | No (support both) |
| I1 | O(n) file lookup | MEDIUM | MEDIUM | No |
| I2 | O(n) reverse ID lookup | MEDIUM | MEDIUM | No |

---

## TOP 7 MANDATORY FIXES (Block V2 Shipment)

### 1. B1 — Batch write rollback mechanism
**Requirement**: apply_batch MUST NOT leave partially-written state on failure.
**Implementation**: Pre-flight validate all paths. Backup all originals. Write all to temp files. Rename all atomically. On any failure, restore all backups.

### 2. B2 — Unique temp file paths per batch edit
**Requirement**: Each edit in a batch MUST use a unique temp file path.
**Implementation**: `.m1nd_batch_{pid}_{uuid}_{idx}.tmp` or use `tempfile::NamedTempFile`.

### 3. C2 — Fix incremental ingest to actually update the graph
**Requirement**: `handle_ingest()` incremental path MUST apply the diff, finalize the graph, and rebuild engines.
**Implementation**: 4 lines of code (see BUG 1 fix above). This also fixes V1 apply's re-ingest.

### 4. E4 — Block writes when ingest_roots is empty
**Requirement**: `validate_path_safety()` MUST reject ALL writes when `ingest_roots` is empty.
**Implementation**: Change line 52-54 from `return Ok(canonical)` to `return Err(...)`.

### 5. A1 — Hard cap on connected file count and total lines
**Requirement**: V2 MUST cap the number of connected files and total lines returned.
**Implementation**: Add `max_connected_files: u32` (default 10) and `max_total_lines: u32` (default 5000) to input. Enforce during connected file collection.

### 6. H1 — Resolve protocol divergence
**Requirement**: V2 MUST have ONE canonical protocol definition for each tool.
**Implementation**: If V2 uses file_path as primary key (current implementation), update layers.rs to match surgical.rs. If V2 uses node_id, update surgical.rs and the handler. Do NOT ship with two incompatible definitions.

### 7. E3 — Add deny-list for m1nd state files
**Requirement**: apply and apply_batch MUST refuse to write to m1nd's own persistence files.
**Implementation**: Add pattern matching in `validate_path_safety()` for known state file patterns.

---

## DESIGN CONTRACTS FOR V2

### surgical_context_v2 contract
```
INPUT: {
  file_path: String,          // Target file
  agent_id: String,
  symbol?: String,            // Optional: focus on specific symbol
  radius: u32,                // Default: 1, max: 2
  include_tests: bool,        // Default: true
  max_connected_files: u32,   // Default: 10, max: 30
  max_lines_per_file: u32,    // Default: 500, max: 2000
  max_total_lines: u32,       // Default: 5000, max: 20000
  include_source: bool,       // Default: true (return file content for connected files)
}

OUTPUT: {
  // Target file (same as V1)
  file_path: String,
  file_contents: String,
  line_count: u32,
  node_id: String,
  symbols: [SurgicalSymbol],
  focused_symbol?: SurgicalSymbol,
  content_hash: String,       // SHA-256 for optimistic concurrency

  // Connected files (V2-new)
  connected_files: [{
    file_path: String,
    content: String,           // Full or truncated source
    content_hash: String,
    line_count: u32,
    symbols: [SurgicalSymbol],
    relation: String,          // How this file connects to target
    edge_weight: f32,
    truncated: bool,
  }],

  // Graph context
  callers: [SurgicalNeighbour],
  callees: [SurgicalNeighbour],
  tests: [SurgicalNeighbour],

  // Metadata
  truncated_files: u32,        // Files skipped due to caps
  total_available_files: u32,  // Total connected files before cap
  skipped_files: [{ path, reason }],  // Binary, non-UTF-8, etc.
  elapsed_ms: f64,
}

INVARIANTS:
  - len(connected_files) <= max_connected_files
  - sum(connected_files[*].line_count) + line_count <= max_total_lines
  - radius <= 2
  - Binary/non-UTF-8 files: skipped, listed in skipped_files
  - No duplicate file paths in connected_files
  - connected_files sorted by edge_weight descending
```

### apply_batch contract
```
INPUT: {
  edits: [{
    file_path: String,
    new_content: String,        // Full file replacement
    description?: String,
  }],
  agent_id: String,
  reingest: bool,               // Default: true
  fail_fast: bool,              // Default: true (abort batch on first failure)
}

OUTPUT: {
  success: bool,                // True only if ALL edits succeeded
  edits: [{
    file_path: String,
    status: "ok" | "failed" | "skipped" | "rolled_back",
    error?: String,
    bytes_written: usize,
    lines_added: i32,
    lines_removed: i32,
    content_hash: String,       // Hash of new content
  }],
  reingested: bool,
  updated_node_ids: [String],
  elapsed_ms: f64,
}

INVARIANTS:
  - ALL paths validated BEFORE any write begins
  - Writes rejected when ingest_roots is empty
  - Writes rejected for m1nd state files (deny-list)
  - Lock registry checked for ALL files before ANY write
  - Each edit uses a unique temp file
  - On ANY failure: ALL written files restored from backup
  - Re-ingest ONLY runs when ALL writes succeed
  - Re-ingest calls finalize_ingest() (not the broken incremental path)
  - MUST NOT call learn() internally
  - MUST notify watchers after successful re-ingest
```

---

## V1 HARDENING REPORT DELTA

The V1 report (SURGICAL_CONTEXT_HARDENING.md) remains valid. V2 inherits ALL V1 findings plus the additional 28 findings in this report. Cross-reference:

| V1 Finding | V2 Status |
|------------|-----------|
| A1: node_id not found | Inherited (V2 uses file_path, less prone) |
| B1: Stale graph (BUG) | Inherited + amplified by batch |
| C1: Concurrent writes | Amplified by batch (F2 in this report) |
| G1: Path escape | Inherited + E4 empty roots (this report) |
| G2: State file corruption | E3 in this report |
| H3: Slow re-ingest | Amplified by batch (C4 in this report) |
| I1: Write+ingest atomicity | B1 in this report (multi-file version) |
