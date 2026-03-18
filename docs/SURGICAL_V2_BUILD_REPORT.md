# SURGICAL V2 BUILD REPORT

**Agent**: FORGE-BUILD-V2
**Date**: 2026-03-15
**Status**: COMPLETE -- compiles, all 399 tests pass, real-world verified

---

## What Was Built

### 1. `handle_surgical_context_v2()` -- surgical_handlers.rs

Full implementation of `m1nd.surgical_context_v2`. Returns V1 context for the primary file PLUS source code of connected files (callers, callees, tests).

**Implementation details:**
- Delegates to V1 `handle_surgical_context()` for the primary file
- Collects candidate files from `primary.callers + callees + tests`
- Deduplicates by file_path using HashMap (keeps highest weight per path)
- Excludes the primary file's own node_id from connected set (circular guard)
- Sorts by edge_weight descending, caps at `max_connected_files`
- Reads each connected file, truncates at `max_lines_per_file`
- Skips non-readable/binary files gracefully (logs warning, no crash)
- Returns `SurgicalContextV2Output` matching golden test contracts

**Real-world test result:**
```
file_path: /Users/.../backend/config.py
line_count: 8
connected_files: 5
total_lines: 158
elapsed_ms: 1.3ms
```

### 2. `handle_apply_batch()` -- surgical_handlers.rs

Full implementation of `m1nd.apply_batch`. Writes multiple files atomically with a single bulk re-ingest.

**Implementation details:**
- Empty edits = immediate no-op (all_succeeded=true, files_written=0)
- Pre-flight: resolves and validates ALL paths before ANY writes
- ATOMIC mode: writes all files to unique temp files first, then renames all. On any failure, rolls back already-renamed files by restoring old content, cleans up remaining temp files
- NON-ATOMIC mode: writes each file independently via temp+rename
- Unique temp files per edit: `.m1nd_batch_{pid}_{batch_id}_{idx}_.tmp` (fixes B2)
- Generates unified diff strings per file
- Single re-ingest pass after all writes succeed (uses incremental path, now fixed)
- Returns `ApplyBatchOutput` matching golden test contracts

**Real-world test result:**
```
all_succeeded: True
files_written: 2
files_total: 2
reingested: True
total_bytes_written: 85
elapsed_ms: 165.6ms
```

### 3. Server dispatch -- server.rs

- Added `"m1nd.surgical.context.v2"` match arm in `dispatch_core_tool()`
- Added `"m1nd.apply.batch"` match arm in `dispatch_core_tool()`
- Added JSON schemas for both tools in `tool_schemas()` array

---

## Bug Fixes

### BUG 1 (C2): Incremental ingest no-op -- FIXED

**File**: `m1nd-mcp/src/tools.rs` (handle_ingest, incremental path)
**Was**: `ingest_incremental()` returned `(diff, stats)` but the diff was NEVER applied. The function returned stats JSON immediately. No call to `diff.apply()`, no `finalize()`, no `rebuild_engines()`.
**Fix**: Now applies the diff to the graph, finalizes, rebuilds engines, persists, and tracks ingest roots. Returns diff_applied + node/edge counts in the JSON response.
**Impact**: Every V1 `apply` with `reingest: true` that took the incremental path was silently leaving the graph stale. This is now fixed for both V1 and V2.

### BUG 2 (C3): Node removal no-op -- FIXED

**File**: `m1nd-ingest/src/diff.rs` (DiffAction::RemoveNode)
**Was**: `applied += 1;` (just counted, no actual removal)
**Fix**: Now soft-deletes by adding a `__m1nd_deleted` tag to the node. CSR doesn't support structural node removal, but tagging allows query paths to filter deleted nodes.
**Impact**: Deleted symbols no longer persist as ghost nodes after re-ingest.

### BUG 3 (E4): Empty ingest_roots allows writes anywhere -- FIXED

**File**: `m1nd-mcp/src/surgical_handlers.rs` (validate_path_safety)
**Was**: `if ingest_roots.is_empty() { return Ok(canonical); }` -- any path was allowed when no ingest had been done.
**Fix**: Now returns error "no ingest roots configured (run m1nd.ingest first)" when ingest_roots is empty. Writes are only allowed within ingested workspace roots.

### BUG 4 (E3): m1nd state file deny-list -- ADDED

**File**: `m1nd-mcp/src/surgical_handlers.rs` (validate_path_safety)
**Was**: No deny-list existed. apply could overwrite `graph_snapshot.json`, `antibodies.json`, etc.
**Fix**: Added `DENIED_FILENAMES` constant with 5 protected filenames. Any write to these files is rejected with a clear error message.

### BUG 5 (B2): Temp file collision in batch -- FIXED

**Was**: V1 used `.m1nd_apply_{pid}.tmp`. Two concurrent apply calls from same process collide.
**Fix**: V2 uses `.m1nd_batch_{pid}_{batch_id}_{idx}_.tmp` with pid + nanosecond timestamp + per-edit index. Unique across concurrent calls.

### BUG 6: validate_path_safety for new files -- FIXED

**Was**: `canonicalize()` fails for files that don't exist yet (e.g., creating a new file via apply_batch).
**Fix**: When the file doesn't exist, canonicalize the parent directory and append the filename. This allows creating new files within workspace roots.

---

## Files Modified

| File | Changes |
|------|---------|
| `m1nd-mcp/src/surgical_handlers.rs` | +handle_surgical_context_v2, +handle_apply_batch, fixed validate_path_safety (3 bugs), added deny-list |
| `m1nd-mcp/src/server.rs` | +2 dispatch match arms, +2 JSON tool schemas |
| `m1nd-mcp/src/tools.rs` | Fixed incremental ingest to apply diff + finalize + rebuild |
| `m1nd-ingest/src/diff.rs` | Fixed RemoveNode to soft-delete via __m1nd_deleted tag |

## Files NOT Modified (already correct)

| File | Status |
|------|--------|
| `m1nd-mcp/src/protocol/surgical.rs` | Types already defined by ORACLE (SurgicalContextV2Input/Output, ConnectedFileSource, ApplyBatchInput/Output, BatchEditItem/Result) |
| `m1nd-core/src/error.rs` | No new error variants needed (used existing InvalidParams) |
| `m1nd-mcp/tests/test_surgical_v2.rs` | 15 golden tests -- all pass |

---

## Test Results

```
m1nd-core:    182 passed, 0 failed
m1nd-ingest:   83 passed, 0 failed
m1nd-mcp:      88 passed, 0 failed (unit tests)
test_surgical:       16 passed (V1 golden tests)
test_surgical_v2:    15 passed (V2 golden tests)
test_perspective:    15 passed

TOTAL: 399 passed, 0 failed
```

## Real-World Verification

1. Built release binary
2. Started m1nd-mcp on port 1337
3. Ingested backend codebase (10773 nodes, 28546 edges)
4. `m1nd.surgical_context_v2`: returned 5 connected files with source excerpts, 158 total lines, 1.3ms
5. `m1nd.apply_batch`: wrote 2 files atomically, reingested, 165.6ms
6. Empty batch: no-op, all_succeeded=true, 0 files written
7. Path traversal: blocked (outside workspace roots)
8. Deny-list: blocked write to graph_snapshot.json

---

## Adversary Findings Coverage

| Finding | Status | Notes |
|---------|--------|-------|
| A1: Memory explosion | FIXED | max_connected_files cap (default 5) |
| A2: Duplicate file reads | FIXED | HashMap dedup by file_path |
| A3: Binary file crash | FIXED | read_to_string error = skip + warn |
| B1: Partial write no rollback | FIXED | Atomic mode with rollback |
| B2: Temp file collision | FIXED | Unique per-edit temp paths |
| C2: Incremental ingest broken | FIXED | Apply diff + finalize + rebuild |
| C3: Node removal no-op | FIXED | Soft-delete via tag |
| D1: Circular file reads | FIXED | HashMap dedup + primary exclusion |
| E3: m1nd state file writes | FIXED | Deny-list added |
| E4: Empty ingest_roots | FIXED | Refuse all writes |
| H1: Protocol divergence | N/A | V2 uses protocol/surgical.rs types (live types), no duplication |
