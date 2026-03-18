# SURGICAL_CONTEXT + APPLY — Adversarial Hardening Report

**Agent**: ADVERSARY-HARDENING
**Date**: 2026-03-15
**Grounding**: server.rs, tools.rs, engine_ops.rs, perspective_handlers.rs, peek_security.rs, session.rs, graph.rs, antibody.rs, trust.rs, temporal.rs, lib.rs (m1nd-ingest)

---

## STEP 1: VISION — What These Tools Do

### m1nd.surgical_context(node_id)

**Purpose**: Single-call omniscient context retrieval for a graph node. Returns everything an agent needs to understand and modify a code entity without multiple round-trips.

**Composite output** (assembled from existing engine queries):

| Sub-query | Source | Purpose |
|-----------|--------|---------|
| peek | perspective_handlers.rs `handle_perspective_peek` -> peek_security.rs `secure_peek` | File content at node's source location |
| impact | tools.rs `handle_impact` -> temporal.rs `impact_calculator.compute` | Blast radius — what breaks if this changes |
| deps | graph CSR forward edges (imports/calls/references) | Direct dependencies |
| tests | graph query for nodes tagged `test` linked to this node | Associated test files/functions |
| antibodies | antibody.rs `scan_graph` filtered to this node | Known bug patterns involving this node |
| trust | trust.rs `TrustLedger::compute_trust_score` for this node | Defect history risk assessment |
| ghost_edges | activation with ghost edge detection | Latent/inferred connections |

**Flow**: `resolve node_id` -> parallel sub-queries -> assemble composite JSON -> return.

### m1nd.apply(node_id, new_content)

**Purpose**: Write code to the file at a node's provenance location, re-ingest the modified file, and predict downstream impact.

**Flow**: `resolve node_id` -> extract provenance (source_path, line_start, line_end) -> write new_content to file at those lines -> re-ingest the file -> run predict on the modified node -> return diff + predictions.

---

## STEP 2: FEATURES — Internal Sub-query Decomposition

### surgical_context internal operations (minimum 7 engine calls)

1. **graph.resolve_id(node_id)** — Resolve external ID string to internal NodeId
2. **graph.resolve_node_provenance(node)** — Get source_path, line_start, line_end
3. **secure_peek(source_path, ...)** — Read file content with full security pipeline (9 steps)
4. **temporal.impact_calculator.compute(graph, node, Forward)** — Blast radius
5. **CSR traversal: out_range(node) + in_range(node)** — Direct dependencies (imports/calls/references)
6. **Query for test nodes** — Either graph query or tag-filtered traversal for test:* tags
7. **antibody scan (filtered)** — Pattern match against this node's subgraph
8. **trust_ledger.compute_trust_score(node_id)** — Actuarial risk
9. **ghost edge detection** — From activation or dedicated scan

### apply internal operations (minimum 5 steps, each dangerous)

1. **graph.resolve_id(node_id)** — Resolve
2. **graph.resolve_node_provenance(node)** — Get file path + line range
3. **fs::write() or line-level splice** — Destructive file modification
4. **re-ingest(file_path, incremental)** — Rebuild graph for modified file
5. **predict(changed_node)** — Co-change prediction on re-ingested graph

---

## STEP 3: HARDENING — Every Way This Breaks

### CATEGORY A: Node Resolution Failures

#### A1: node_id does not exist in graph
**Trigger**: Typo in node_id, or node was from a previous ingest that got replaced.
**Current behavior** (handle_impact pattern): Returns empty/error JSON. No crash.
**Risk**: LOW but misleading — agents may think the node has zero impact when it simply doesn't exist.
**Mitigation**: Return explicit `"error": "node_not_found"` with `"node_id": <input>` and `"suggestion": "run m1nd.seek to find the correct node_id"`. Do NOT return a partial result with empty arrays — that looks like success.

#### A2: Ambiguous node_id (multiple matches)
**Trigger**: `resolve_id` uses exact string match on interned IDs. If the user passes "lib.rs" instead of "file::src/lib.rs", no match. But if they pass a label that happens to match a different node type...
**Risk**: MEDIUM. The external ID format (`file::path`, `fn::name`, `mod::name`) is not enforced at the API boundary. Users might pass bare names.
**Mitigation**: If `resolve_id` fails, fall back to `SeedFinder::find_seeds` with the input as query text, take top-1 if confidence > 0.9. Return the resolved ID so the agent can verify. Log a warning about ambiguity.

#### A3: Node exists in graph but provenance is empty
**Trigger**: Nodes created by cross-reference resolution, ghost edges, or from non-file sources (memory adapter, JSON adapter) have no source_path.
**Risk**: HIGH for apply (cannot write to file), MEDIUM for surgical_context (peek fails, but other sub-queries work).
**Mitigation for surgical_context**: Return `"peek": null` with `"peek_unavailable_reason": "no source provenance"`. Other sub-queries proceed normally.
**Mitigation for apply**: HARD FAIL. Return error: `"cannot apply: node has no file provenance"`. Do NOT attempt to guess the file.

### CATEGORY B: Stale Graph (File Modified Since Last Ingest)

#### B1: File was modified since last ingest
**Trigger**: Another agent (or human) edited the file. The graph's line_start/line_end are now wrong.
**Risk**: CRITICAL for apply — writing to wrong lines. MEDIUM for surgical_context — peek shows stale content but peek_security already detects this via mtime check.
**Current behavior**: `peek_security.rs` `check_staleness()` compares file mtime against `last_ingest_ms`. Sets `provenance_stale: true` in output. BUT: `last_ingest_ms` is passed as `None` by `handle_perspective_peek` (line 606: `None`), so staleness is NEVER detected in practice.
**Finding**: **BUG — staleness detection is dead code for peek.** The `last_ingest_ms` parameter is always `None`.
**Mitigation**:
  1. `surgical_context` MUST pass the actual last ingest timestamp (from SessionState) to `secure_peek`.
  2. If stale, `surgical_context` SHOULD warn but still return content (with `"stale": true`).
  3. If stale, `apply` MUST refuse to write unless `force: true` is passed. Re-ingest first.
  4. Fix `handle_perspective_peek` to pass `state.last_persist_time` or a dedicated ingest timestamp.

#### B2: File was deleted since last ingest
**Trigger**: File removed from disk, node still in graph.
**Risk**: apply will fail with IO error. peek will fail at existence check.
**Mitigation**: Check `canonical.exists()` before any operation. Return `"error": "source_file_deleted"` with the stale path.

#### B3: File was renamed/moved since last ingest
**Trigger**: Refactoring moved a file. Graph has old path.
**Risk**: Same as B2 for the old path. New path has no node.
**Mitigation**: This is fundamentally unsolvable without re-ingest. Document the limitation. Recommend `ingest` before `apply` if files changed.

### CATEGORY C: Concurrent File Access (Multi-Agent Conflicts)

#### C1: apply writes to a file another agent is editing
**Trigger**: Two agents call `apply` on different nodes in the same file simultaneously. Or one agent has a lock (m1nd.lock.create) on the region.
**Risk**: CRITICAL. Last write wins. Intermediate changes lost. Graph becomes inconsistent.
**Current state**: The lock system (`lock_handlers.rs`, `LockState` in session.rs) exists but is advisory — it doesn't prevent file writes.
**Mitigation**:
  1. `apply` MUST check the lock registry before writing. If any lock covers the target file/region, return `"error": "locked_by_agent"` with the lock owner and lock_id.
  2. `apply` SHOULD acquire an exclusive ephemeral lock for the duration of the write+reingest cycle. Release on completion or error.
  3. Consider file-level flock (OS-level) as a defense-in-depth measure against non-m1nd processes.

#### C2: Re-ingest during apply invalidates other agents' perspectives
**Trigger**: `finalize_ingest` calls `rebuild_engines` which calls `invalidate_all_perspectives`. Every active perspective across all agents becomes stale.
**Risk**: HIGH. An agent mid-investigation gets their perspective invalidated because a different agent did an `apply` on an unrelated file.
**Mitigation**:
  1. `apply` should use **incremental ingest** (single file), not full re-ingest. This limits the blast radius.
  2. If incremental ingest is not available for the adapter, document this as a known limitation.
  3. Consider a `PerspectiveInvalidationScope` that only invalidates perspectives touching the modified file's subgraph.

#### C3: Two agents apply to the same file at the same line range
**Trigger**: Race condition. Both read the same surgical_context, both generate patches, both apply.
**Risk**: CRITICAL. Second apply overwrites first. Graph reflects only the last write.
**Mitigation**: Optimistic concurrency with content hash. `apply` should:
  1. Read the current file content hash before writing.
  2. Compare against a `content_hash` field from the prior `surgical_context` call.
  3. If hash mismatch, refuse with `"error": "content_changed_since_context"`.

### CATEGORY D: Content Quality and Safety

#### D1: new_content has syntax errors
**Trigger**: LLM generates broken code. `apply` writes it, re-ingest proceeds (ingest doesn't validate syntax — it's a parser, not a compiler).
**Risk**: HIGH. The file is now broken. All downstream consumers fail. The graph reflects the broken structure.
**Mitigation**:
  1. Post-write validation: run a lightweight syntax check (tree-sitter parse or language-specific linter) before re-ingest. If syntax invalid, revert the file (restore from backup), return `"error": "syntax_error"` with the parser output.
  2. `apply` MUST create a backup of the original file content before writing. Store as `{path}.m1nd-backup` or in-memory.
  3. The re-ingest step should detect parse failures and report them.

#### D2: new_content breaks tests
**Trigger**: Code compiles but is semantically wrong. Tests that were passing now fail.
**Risk**: MEDIUM. This is expected in normal development — not all changes pass tests immediately.
**Mitigation**: `apply` response SHOULD include the `predict` output showing which test files are likely impacted. The agent decides whether to run tests. `apply` itself should NOT run tests (that's the agent's job, and test execution is outside m1nd's scope).

#### D3: new_content is much larger or smaller than the original
**Trigger**: Agent replaces 3 lines with 300 lines, or deletes most of a function.
**Risk**: MEDIUM. Line numbers for all nodes below the edit point in the same file become stale.
**Mitigation**:
  1. After write, re-ingest must update line numbers for ALL nodes in the same file. Incremental ingest already handles this if implemented correctly.
  2. Return `"lines_delta": <new_lines - old_lines>` in the response so agents know the magnitude of the shift.

#### D4: Prompt injection via new_content
**Trigger**: Adversarial input in `new_content` containing MCP directives, system prompt fragments, or shell commands.
**Risk**: LOW (m1nd only writes to files, doesn't execute them). But if the file is later `eval`ed or `source`d, it could be a vector.
**Mitigation**: This is out of m1nd's scope. Document that `apply` writes content verbatim. Agents are responsible for content safety.

### CATEGORY E: Node ID Stability After Re-ingest

#### E1: Re-ingest changes node IDs (dangling references)
**Trigger**: Ingest generates node IDs from file paths + structure. If the internal structure changes (e.g., a function is renamed inside new_content), the old node ID disappears and a new one is created.
**Risk**: CRITICAL. The agent's `node_id` from surgical_context is now stale. Subsequent calls using that ID will fail or hit wrong nodes.
**Current behavior**: `finalize_ingest` replaces the entire graph (or merges). Node IDs are external strings like `file::src/lib.rs` or `fn::Graph::resolve_id`. If the function is renamed, the old `fn::` ID vanishes.
**Mitigation**:
  1. `apply` response MUST include `"old_node_id"` and `"new_node_id"` (if they differ). If the node was renamed/split, return all resulting node IDs.
  2. For simple content changes (same function name, same file), the node ID should remain stable. Document this guarantee.
  3. For structural changes (rename, split, merge), return a mapping: `{"id_changes": [{"old": "fn::foo", "new": "fn::bar"}]}`.
  4. Implementation: Compare pre-ingest and post-ingest node sets for the modified file. Diff them.

#### E2: Re-ingest of one file creates new edges to other files
**Trigger**: new_content adds an `import` statement. The re-ingest creates a new edge from this node to the imported module.
**Risk**: LOW. This is expected behavior. But it can trigger cascade effects in predict/impact.
**Mitigation**: Return the new edges in the `apply` response: `"new_edges": [...]`.

### CATEGORY F: Facade / Multi-file Node Ambiguity

#### F1: apply is called on a node in core/ that has a facade in api/
**Trigger**: A module `core::auth::validate` is re-exported by `api::auth`. The graph may have separate nodes for both, or a single node with the core path as provenance. Which file gets edited?
**Risk**: MEDIUM. Agent may intend to edit the core implementation but the node resolves to the facade, or vice versa.
**Current behavior**: Provenance stores a single `source_path`. For re-exported nodes, the provenance is set by ingest order — typically the first file where the symbol is defined.
**Mitigation**:
  1. `surgical_context` SHOULD return `"facades": [list of re-export paths]` if the node has incoming `re_exports` or `exports` edges.
  2. `apply` SHOULD validate: if the node has facades, warn the agent and require explicit confirmation (`"target_path": "..."` override).
  3. The existing `merge_node_provenance` in graph.rs already handles provenance merge, but it takes the FIRST source_path. Document this.

#### F2: Node spans multiple files (partial classes, extension traits)
**Trigger**: Some languages allow type definitions spread across files (C# partial classes, Rust trait impls in different modules).
**Risk**: LOW for m1nd's current Rust/Python focus. HIGH for future language support.
**Mitigation**: Document the limitation. `apply` writes to the single provenance source_path. If the concept spans files, the agent must call `apply` multiple times.

### CATEGORY G: Security

#### G1: apply writes to files outside the project
**Trigger**: Malicious or buggy node_id provenance points to `/etc/passwd`, `~/.ssh/authorized_keys`, or any system file.
**Risk**: CRITICAL. Arbitrary file write vulnerability.
**Current protection**: `peek_security.rs` has an allow-list (`validate_allow_list`). BUT: `apply` is a NEW tool — it would bypass the peek security pipeline unless explicitly integrated.
**Mitigation**:
  1. `apply` MUST run the same allow-list validation as peek_security BEFORE writing. Same `PeekSecurityConfig.allow_roots` check.
  2. The allow-list should be populated from `ingest_roots` (set during ingest, stored in SessionState).
  3. If `allow_roots` is empty (default), `apply` MUST refuse ALL writes. Fail-safe default. `peek_security.rs` line 102-105 currently allows all paths when allow_roots is empty — this is WRONG for writes.
  4. Canonicalize the target path. Reject symlinks that resolve outside allowed roots.
  5. Reject paths containing `..` segments before canonicalization (defense in depth).

#### G2: apply could write to m1nd's own state files
**Trigger**: `graph_snapshot.json`, `plasticity_state.json`, `antibodies.json` are in the same directory tree as the ingested codebase.
**Risk**: HIGH. Writing garbage to these files corrupts m1nd's state permanently.
**Mitigation**: Hardcode a deny-list of m1nd state file patterns: `graph_snapshot.json`, `plasticity_state.json`, `antibodies.json`, `tremor_state.json`, `trust_state.json`, `trails/*.json`. Reject writes to any of these.

#### G3: Path traversal via crafted node_id
**Trigger**: A crafted external ID like `file::../../etc/passwd` could resolve to a node whose provenance points outside the project.
**Risk**: MEDIUM. Provenance is set during ingest, which uses the walker's relative paths. The walker should not traverse outside the root. But if provenance is manually set (e.g., via JSON adapter), it could contain arbitrary paths.
**Mitigation**: `apply` must canonicalize and validate ALL paths, regardless of source. Never trust provenance paths blindly.

### CATEGORY H: Performance

#### H1: surgical_context runs too many internal queries
**Counting**: At minimum 7 engine operations (see Step 2). With full options: 9+ operations.
**Risk**: MEDIUM. Each sub-query acquires a graph read lock. If any sub-query is slow (e.g., antibody scan on a huge graph), the entire call blocks.
**Mitigation**:
  1. Set a total wall-clock budget (e.g., 2000ms). Use the existing `SynthesisBudget` pattern from engine_ops.rs.
  2. Execute sub-queries in priority order: resolve -> peek -> impact -> deps -> trust -> antibodies -> tests -> ghost_edges. Cut off after budget exceeded.
  3. Return `"budget_exhausted": true` and `"completed_queries": [...]` if timeout occurs.
  4. The antibody scan is the most expensive sub-query. Consider making it optional (default off) with an `include_antibodies: bool` parameter.

#### H2: peek returns >10K lines (huge files)
**Trigger**: Node provenance points to a 50K-line generated file or minified bundle.
**Risk**: MEDIUM. Response payload becomes huge. Agent context window wasted.
**Current protection**: `peek_security.rs` has `max_chars` (default from `PeekSecurityConfig`), `max_file_size` (10MB cap), and line range extraction. These are adequate.
**Mitigation**: surgical_context should use a tighter line window than perspective.peek's default. Suggest: `center_line +/- 50 lines` (100 lines total) as default, with an optional `context_lines: u32` parameter.

#### H3: Re-ingest after apply is slow for large codebases
**Trigger**: Full re-ingest of a 10K-file project after changing one line.
**Risk**: HIGH. Full ingest can take 5-10 seconds. During this time, the graph write lock is held (line 53 in tools.rs: `state.graph.write()`), blocking ALL other queries.
**Mitigation**:
  1. `apply` MUST use incremental ingest, not full ingest. The `ingest_incremental` method exists in m1nd-ingest but is currently limited (line 1336-1349 in tools.rs: simplified path passing).
  2. For incremental ingest: only re-parse the modified file, then merge changes into the existing graph.
  3. If incremental ingest is not feasible, queue the re-ingest as a background task and return immediately with `"reingest_pending": true`.

### CATEGORY I: Atomicity and Rollback

#### I1: apply writes file but re-ingest fails
**Trigger**: File is written successfully, but `finalize_ingest` or `rebuild_engines` fails (e.g., out of memory, graph corruption).
**Risk**: CRITICAL. The file on disk has new content, but the graph still reflects the old content. The system is in an inconsistent state.
**Mitigation**:
  1. Write to a temp file first. Only rename to target after ALL post-write steps succeed. (Atomic write pattern.)
  2. If re-ingest fails, restore the backup file and return error.
  3. If re-ingest succeeds but predict fails, that's non-fatal — the write is committed, prediction is optional.

#### I2: apply partially writes (disk full, permission denied)
**Trigger**: Filesystem error mid-write.
**Risk**: HIGH. Truncated file = broken code + broken graph.
**Mitigation**: Write to temp file + atomic rename (see I1). This is the standard solution.

#### I3: No undo for apply
**Trigger**: Agent calls apply, result is bad, wants to revert.
**Risk**: MEDIUM. There's no built-in undo.
**Mitigation**:
  1. `apply` response MUST include `"original_content": "..."` (the content that was replaced). This enables agent-side undo by calling `apply` again with the original content.
  2. Consider an `m1nd.apply.undo(apply_id)` tool, but this adds complexity. The original_content approach is simpler and more reliable.
  3. Alternatively, integrate with git: `apply` could create a git stash or commit before writing.

### CATEGORY J: Edge Cases in Line-Level Editing

#### J1: Node provenance has line_start but no line_end
**Trigger**: Some extractors set line_start only. `line_end` defaults to `line_start` (graph.rs line 742-743).
**Risk**: LOW for peek (shows one line). MEDIUM for apply — agent may intend to replace a multi-line block but only one line gets replaced.
**Mitigation**: If `line_start == line_end`, `apply` should warn: `"single_line_provenance": true`. Agent must explicitly provide the intended line range or use content-based matching instead of line-based splicing.

#### J2: new_content ends with newline (or not)
**Trigger**: Inconsistent trailing newline handling between the original content and new_content.
**Risk**: LOW. But can cause phantom diffs in version control.
**Mitigation**: Normalize: if original content at those lines ended with newline, ensure new_content does too. Document the behavior.

#### J3: Line encoding mismatch (CRLF vs LF)
**Trigger**: Windows-origin files with CRLF. new_content has LF.
**Risk**: LOW on macOS/Linux. Higher on Windows.
**Mitigation**: Detect the file's line ending style before writing. Normalize new_content to match.

### CATEGORY K: Interaction with Other m1nd Systems

#### K1: apply + learn feedback loop
**Trigger**: Agent calls `apply`, then `learn("correct")` if tests pass, or `learn("wrong")` if they don't. This updates plasticity weights and antibody extraction.
**Risk**: LOW. This is the intended workflow. But if `apply` auto-learns (which it shouldn't), it could create biased feedback.
**Mitigation**: `apply` must NOT call `learn` internally. Learning is the agent's decision after verifying the result.

#### K2: apply + lock.watch interaction
**Trigger**: An agent has a lock.watch on a region. Another agent's `apply` modifies that region. The watcher should be notified.
**Risk**: MEDIUM. If `apply` uses incremental ingest, `notify_watchers(WatchTrigger::Ingest)` is called. But if apply doesn't go through the standard ingest path, watchers are bypassed.
**Mitigation**: `apply` MUST call `state.notify_watchers(WatchTrigger::Ingest)` after successful write + reingest. This is already in `rebuild_engines` path but verify it fires for incremental ingest too.

#### K3: apply + perspective navigation
**Trigger**: Agent is mid-perspective-navigation. Calls `apply`. Graph changes. All perspectives invalidated.
**Risk**: HIGH (see C2). The agent loses their navigation state.
**Mitigation**: `apply` should return the current perspective state after the operation, so the agent can resume without calling perspective.start again. Or: don't invalidate the calling agent's perspective.

---

## SUMMARY: Risk Matrix

| ID | Category | Severity | Likelihood | Fix Complexity |
|----|----------|----------|------------|----------------|
| A1 | Node not found | LOW | HIGH | LOW |
| A2 | Ambiguous ID | MEDIUM | MEDIUM | MEDIUM |
| A3 | No provenance | HIGH (apply) | MEDIUM | LOW |
| B1 | Stale graph (**BUG FOUND**) | CRITICAL | HIGH | LOW |
| B2 | File deleted | MEDIUM | LOW | LOW |
| B3 | File renamed | MEDIUM | MEDIUM | N/A (limitation) |
| C1 | Concurrent writes | CRITICAL | MEDIUM | HIGH |
| C2 | Perspective invalidation | HIGH | HIGH | HIGH |
| C3 | Same-line race | CRITICAL | LOW | MEDIUM |
| D1 | Syntax errors | HIGH | HIGH | MEDIUM |
| D2 | Test failures | MEDIUM | HIGH | LOW (document) |
| D3 | Size mismatch | MEDIUM | MEDIUM | LOW |
| E1 | ID instability | CRITICAL | MEDIUM | HIGH |
| E2 | New edges | LOW | HIGH | LOW |
| F1 | Facade ambiguity | MEDIUM | LOW | MEDIUM |
| G1 | Path escape (**CRITICAL**) | CRITICAL | LOW | LOW |
| G2 | State file corruption | HIGH | LOW | LOW |
| G3 | Path traversal | MEDIUM | LOW | LOW |
| H1 | Query count perf | MEDIUM | MEDIUM | MEDIUM |
| H2 | Huge file peek | MEDIUM | LOW | Already mitigated |
| H3 | Slow re-ingest | HIGH | HIGH | HIGH |
| I1 | Write+ingest atomicity | CRITICAL | LOW | MEDIUM |
| I2 | Partial write | HIGH | LOW | LOW |
| I3 | No undo | MEDIUM | MEDIUM | LOW |
| J1 | Single-line provenance | MEDIUM | MEDIUM | LOW |
| K2 | Watcher bypass | MEDIUM | MEDIUM | LOW |
| K3 | Perspective loss | HIGH | HIGH | MEDIUM |

---

## TOP 5 MANDATORY FIXES (Block Shipment)

1. **G1 — Path escape in apply**: MUST enforce allow-list before any write. MUST fail-safe when allow_roots is empty (unlike peek which allows all). This is an arbitrary file write vulnerability.

2. **B1 — Staleness detection is dead code**: `handle_perspective_peek` passes `None` for `last_ingest_ms` (line 606). This means `check_staleness` always returns `false`. Fix: pass actual ingest timestamp from SessionState. This affects both surgical_context and all perspective.peek calls.

3. **I1 — Atomicity**: apply must use temp-file + atomic-rename pattern. If re-ingest fails after file write, the system enters an unrecoverable inconsistent state. Backup-and-restore is the minimum viable defense.

4. **C1 — Lock integration**: apply must check the lock registry before writing. Advisory locks are worthless if the write tool ignores them.

5. **E1 — Node ID mapping after re-ingest**: apply must diff pre/post node sets for the modified file and return an ID change map. Without this, agents cannot continue using the node_id they started with.

---

## DESIGN CONSTRAINTS FOR FORGE-CONTRACTS

Based on this hardening analysis, the following contracts are non-negotiable:

### surgical_context contract
```
INPUT:  { node_id: String, agent_id: String, context_lines?: u32, include_antibodies?: bool }
OUTPUT: {
  node_id: String,
  resolved: bool,
  peek: PeekContent | null,
  peek_unavailable_reason?: String,
  impact: { blast_radius: [...], total_energy: f32 },
  deps: { forward: [...], reverse: [...] },
  tests: [{ node_id, label, source_path }],
  antibodies: [{ id, name, severity, match_confidence }] | null,
  trust: { trust_score, risk_multiplier, tier },
  ghost_edges: [...],
  stale: bool,
  content_hash: String,  // for optimistic concurrency in apply
  budget_exhausted: bool,
  completed_queries: [String]
}
INVARIANTS:
  - resolve failure -> { resolved: false, error: "node_not_found" }
  - peek failure (no provenance) -> peek: null, other fields populated
  - total wall-clock < 2000ms, sub-query budget: 8 calls max
  - content_hash = SHA-256 of file content at peek time
```

### apply contract
```
INPUT:  {
  node_id: String,
  agent_id: String,
  new_content: String,
  content_hash?: String,  // from surgical_context, for optimistic concurrency
  force?: bool,           // override staleness check
  target_path?: String    // override provenance path (for facade disambiguation)
}
OUTPUT: {
  success: bool,
  old_content: String,
  new_content: String,
  lines_delta: i32,
  file_path: String,
  id_changes: [{ old: String, new: String }],
  predictions: [...],
  new_edges: [...],
  warnings: [String]
}
INVARIANTS:
  - MUST check allow-list before write (fail if path outside allowed roots)
  - MUST check lock registry before write (fail if locked)
  - MUST check content_hash if provided (fail if mismatch)
  - MUST check staleness if not force (fail if stale)
  - MUST backup original content before write
  - MUST use atomic write (temp file + rename)
  - MUST restore backup if re-ingest fails
  - MUST use incremental ingest (not full re-ingest)
  - MUST notify watchers after successful write
  - MUST NOT call learn() internally
  - MUST NOT write to m1nd state files
  - MUST return old_content for agent-side undo
```

---

## BUG FOUND DURING ANALYSIS

**File**: `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-mcp/src/perspective_handlers.rs`
**Line**: 606
**Code**: `None, // last_ingest_ms`
**Impact**: Staleness detection (`check_staleness` in peek_security.rs) is completely inert. It always receives `None`, so it always returns `false`. No peek call ever detects that the file changed since ingest.
**Fix**: Pass `state.last_persist_time` converted to unix ms, or better, track `last_ingest_time_ms` as a dedicated field in SessionState.

---

## TEST BLIND SPOTS

1. **No test for allow-list enforcement on write paths** — peek_security has tests for read paths, but apply (when built) needs its own write-path security tests.
2. **No test for concurrent apply from two agents** — lock contention under parallel writes is untested.
3. **No test for re-ingest after file modification** — the incremental ingest path for single-file changes is noted as "simplified" in tools.rs line 1339.
4. **No test for node ID stability after content change** — what happens to `fn::foo` if `foo` is renamed in new_content?
5. **No test for staleness detection** — since it's dead code, naturally there are no passing tests that verify it works.
6. **No test for atomic write + rollback on ingest failure** — the backup/restore pattern needs integration tests.
7. **No test for huge file handling in surgical_context** — what happens when a node points to a 100K-line file?
