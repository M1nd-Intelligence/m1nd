# CORTEX ONDA1 — ADVERSARY-V2

**Agent**: ADVERSARY-V2
**Scope**: surgical_context_v2 + apply_batch hardening
**Status**: COMPLETE

## CLAIM
Hardening report for V2 multi-file variants of surgical_context and apply_batch.
Identified 28 failure modes across 10 categories, with 7 blocking-shipment findings.

## CRITICAL FINDINGS (DENSE)

1. **MEMORY BOMB**: V2 returns source code of ALL connected files. A hub node with 60+ callers at 1K lines each = 60K+ lines in one response. No cap exists.
2. **PARTIAL WRITE CORRUPTION**: apply_batch writes files sequentially. If file 3/5 fails, files 1-2 are committed but 3-5 are not. No rollback. Graph re-ingest reflects partial state.
3. **CIRCULAR EXPANSION**: BFS in collect_neighbours has no cycle detection beyond visited set. But the visited set is per-BFS, not per-file. Connected file collection can walk the entire graph.
4. **RE-INGEST RACE**: apply_batch re-ingests ONCE after all writes. But re-ingest calls rebuild_engines() which invalidates ALL perspectives for ALL agents. Multi-agent disaster.
5. **INCREMENTAL INGEST IS BROKEN**: handle_ingest incremental path (line 1336-1349) returns raw JSON without calling finalize_ingest(). Graph is NOT updated. Apply's re-ingest through this path is no-op.
6. **TEMP FILE COLLISION**: apply uses `.m1nd_apply_{pid}.tmp`. Two concurrent apply_batch calls from same process use SAME temp path. Second overwrites first's temp file.
7. **NODE REMOVAL IS A NO-OP**: diff.rs DiffAction::RemoveNode does nothing (line 191-194). After apply_batch re-ingest, deleted symbols remain as ghost nodes.

## OUTPUT
`/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/docs/SURGICAL_V2_HARDENING.md`
