# CORTEX — ONDA 1 CONTRACTS DELIVERABLE

> Agent: FORGE-CONTRACTS-V2
> Task: Design V2 contracts for surgical_context_v2 + apply_batch
> Status: COMPLETE

---

## What Was Done

1. **Read all existing surgical contract implementations** (3 separate type sets):
   - `protocol/surgical.rs` — live V1 types (file-oriented: `SurgicalContextInput{file_path}`, `ApplyInput{file_path, new_content}`)
   - `protocol/layers.rs` — TEMPESTA golden-test types (node-oriented: `SurgicalContextInput{node_id}`, `ApplyInput{node_id, line_start, line_end}`)
   - `docs/SURGICAL_CONTEXT_CONTRACTS.md` — doc-only types with deeper features (antibodies, ghost edges, content hash)

2. **Designed V2 contracts** that extend the LIVE V1 types (from `protocol/surgical.rs`), not the doc-only types:

   - **`SurgicalContextV2Input`** — adds `include_connected_sources`, `max_connected_files`, `max_lines_per_file` to V1 input
   - **`SurgicalContextV2Output`** — wraps V1 output as `primary`, adds `connected_files: Vec<ConnectedFileContext>`, `total_lines`, `total_files`
   - **`ConnectedFileContext`** — `file_path`, `source_code`, `relation`, `relevance_score`, `line_count`, `truncated`, `node_id`
   - **`ApplyBatchInput`** — `edits: Vec<SingleEdit>`, `atomic: bool`, `reingest: bool`
   - **`SingleEdit`** — `file_path`, `new_content`, `label`
   - **`ApplyBatchOutput`** — `results: Vec<SingleEditResult>`, `total_files`, `total_lines_added/removed`, `reingested`, `updated_node_ids`
   - **`SingleEditResult`** — per-file success/failure with `bytes_written`, `lines_added/removed`, `error`

3. **Wrote complete handler implementations** for both tools, reusing all existing V1 helpers (`resolve_file_path`, `validate_path_safety`, `diff_summary`, `find_nodes_for_file`).

4. **Wrote dispatch integration code** — match arms for `dispatch_core_tool()` in `server.rs`.

5. **Wrote JSON schemas** for MCP tool registration.

6. **Wrote error variants** for batch failure modes.

## Key Design Decisions

| Decision | Why |
|----------|-----|
| V2 output embeds V1 `SurgicalContextOutput` as `primary` | Zero duplication, strict superset, existing consumers unaffected |
| Connected sources ranked by edge_weight | Structural signal is the most reliable ranking; temporal/co-change scoring can be added in BUILD without contract change |
| Batch uses temp-then-rename per file (not a single tarball) | Same-filesystem rename is atomic on POSIX; per-file granularity gives better error reporting |
| Batch re-ingest is sequential, not parallel | Same constraint as V1: `&mut SessionState` is not `Send`, no benefit from parallelism |
| No new Cargo.toml dependencies | V2 reuses all existing std/crate imports |

## Output File

`/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/docs/SURGICAL_V2_CONTRACTS.md`

Contains: compilable Rust structs, JSON schemas, dispatch code, error types, handler implementations, and build checklist.

## Consistency Notes

The codebase has **three** sets of surgical types in different locations. The V2 contracts extend the LIVE types from `protocol/surgical.rs` (the ones actually used by `surgical_handlers.rs`). The TEMPESTA golden-test types in `protocol/layers.rs` are a separate contract set used only by `tests/test_surgical.rs` — V2 does not touch those.
