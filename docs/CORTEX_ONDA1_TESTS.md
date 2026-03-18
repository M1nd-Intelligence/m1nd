# CORTEX — ONDA 1: Golden Tests for surgical_context_v2 + apply_batch

**Agent:** ORACLE-TESTS-V2
**File:** `mcp/m1nd/m1nd-mcp/tests/test_surgical_v2.rs`
**Status:** ALL 15 TESTS COMPILE AND PASS (contract tests — handlers are stubs until FORGE-BUILD)
**Date:** 2026-03-15

---

## What was done

### Protocol types added (surgical.rs)
Two new tool contracts defined in `mcp/m1nd/m1nd-mcp/src/protocol/surgical.rs`:

**`surgical_context_v2`**
- `SurgicalContextV2Input` — extends v1 with `max_connected_files` (default 5) and `max_lines_per_file` (default 60)
- `SurgicalContextV2Output` — adds `connected_files: Vec<ConnectedFileSource>` and `total_lines: usize`
- `ConnectedFileSource` — per-neighbour: `node_id`, `label`, `file_path`, `relation_type` ("caller"/"callee"/"test"), `edge_weight`, `source_excerpt`, `excerpt_lines`, `truncated`

**`apply_batch`**
- `ApplyBatchInput` — `edits: Vec<BatchEditItem>`, `atomic: bool` (default true), `reingest: bool` (default true)
- `BatchEditItem` — `file_path`, `new_content`, `description?`
- `ApplyBatchOutput` — `all_succeeded`, `files_written`, `files_total`, `results`, `reingested`, `total_bytes_written`, `elapsed_ms`
- `BatchEditResult` — per-file: `file_path`, `success`, `diff`, `lines_added`, `lines_removed`, `error?`

---

## Test contracts (15 tests)

### surgical_context_v2 (6 tests)

| # | Test | Contract |
|---|------|---------|
| 1 | `test_v2_returns_connected_file_sources` | connected_files has non-empty source_excerpt, non-empty node_id/file_path/relation_type, edge_weight in (0,1] |
| 2 | `test_v2_respects_max_connected_files` | len(connected_files) <= max_connected_files; sorted edge_weight descending; default=5 |
| 3 | `test_v2_respects_max_lines_per_file` | when file > cap: truncated=true, excerpt_lines <= max_lines_per_file; default=60 |
| 4 | `test_v2_includes_relation_type` | each entry has relation_type in {"caller","callee","test"} |
| 5 | `test_v2_no_circular_expansion` | no duplicate node_ids; target file not in its own connected_files |
| 6 | `test_v2_total_lines_accurate` | total_lines == line_count + sum(excerpt_lines); total_lines >= line_count |

### apply_batch (6 tests)

| # | Test | Contract |
|---|------|---------|
| 7 | `test_batch_writes_multiple_files` | all_succeeded=true; files_written==len(edits); results in input order |
| 8 | `test_batch_atomic_rollback` | one failure → all_succeeded=false, files_written=0, reingested=false |
| 9 | `test_batch_returns_per_file_diff` | each result has non-empty diff with "@@"; lines_added/removed >= 0; results in order |
| 10 | `test_batch_reingests_once` | single reingested bool covers all files; reingested=false when reingest=false |
| 11 | `test_batch_path_traversal_blocked` | path outside workspace → success=false, error contains "outside" |
| 12 | `test_batch_empty_edits_noop` | empty edits → all_succeeded=true, files_written=0, reingested=false, elapsed_ms>=0 |

### Schema parity (3 tests)

- `schema_parity_surgical_context_v2_minimal` — minimal JSON deserializes
- `schema_parity_apply_batch_minimal` — minimal JSON deserializes
- `schema_parity_output_types_serialize` — both outputs serialize to valid JSON with required fields

---

## Files modified

- `mcp/m1nd/m1nd-mcp/src/protocol/surgical.rs` — added 7 new types
- `mcp/m1nd/m1nd-mcp/tests/test_surgical_v2.rs` — 15 golden tests (NEW)

## For FORGE-BUILD

Implement handlers in `surgical_handlers.rs`:
- `handle_surgical_context_v2(state, SurgicalContextV2Input) -> M1ndResult<SurgicalContextV2Output>`
- `handle_apply_batch(state, ApplyBatchInput) -> M1ndResult<ApplyBatchOutput>`

Wire via `tools.rs` dispatch (same pattern as `handle_surgical_context` + `handle_apply`).

Key implementation constraints from the tests:
1. v2: fetch source for each neighbour, cap at max_connected_files (by weight), truncate at max_lines_per_file
2. v2: use visited set in BFS to prevent circular expansion
3. batch: all-or-nothing writes when atomic=true — write to temp files, rename all at end, rollback on any failure
4. batch: single re-ingest call covering all modified files (not one per file)
5. batch: validate all paths against ingest_roots before any writes
6. batch: empty edits → fast path, return immediately
