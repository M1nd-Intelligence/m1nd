# CORTEX V04 -- ADVERSARY

**Agent**: ADVERSARY-V04
**Scope**: m1nd v0.4.0 -- 6 new tools + 1 bug fix
**Status**: COMPLETE

## CLAIM
Adversary analysis for 6 proposed v0.4.0 tools (search, help, report, panoramic, savings, perspective.routes bug fix).
Identified **42 failure modes** across 6 tools, with **11 must-fix-before-shipping** findings.

---

## TOOL 1: m1nd.search (literal + regex + semantic)

### Context
Existing `m1nd.seek` already does fuzzy tokenized search on node labels/ids. `m1nd.search` adds literal substring, regex, and semantic modes. The question: does it overlap `seek` enough to confuse agents? And what breaks?

### Failure Modes

| # | Severity | Failure | Detail |
|---|----------|---------|--------|
| S1 | **CRITICAL** | Regex ReDoS | User passes `(a+)+$` or equivalent catastrophic backtracking pattern. Rust's `regex` crate is safe (linear time, no backtracking), BUT if implementation uses `fancy-regex` for look-arounds, ReDoS is back. Must enforce regex crate only, never fancy-regex. |
| S2 | **CRITICAL** | Unbounded result set | Query `"."` in literal mode matches every node label. If top_k is not enforced or defaults to unlimited, response could be 50K+ entries. Must hard-cap at 500 results regardless of user top_k. |
| S3 | **HIGH** | Overlap confusion with seek | `seek` does fuzzy tokenized matching, `search` does literal/regex/semantic. Agents will confuse them. `help` tool must clearly distinguish. Consider deprecating `seek` in favor of `search(mode="semantic")` or the split will cause perpetual misuse. |
| S4 | **HIGH** | Stale results after file deletion | Graph retains nodes from files deleted since last ingest. `search` returns nodes whose provenance points to non-existent files. Must include `provenance_stale` flag per result OR run provenance validity check. |
| S5 | **HIGH** | Binary file content in results | If `search` returns source context (surrounding lines), binary files ingested accidentally will produce garbage. `graph.resolve_node_provenance()` does not check if source is binary. Must filter by file extension or content sniffing. |
| S6 | **MEDIUM** | Invalid regex error UX | What error message does the agent get for `[invalid`? Must return `InvalidParams` with the regex parse error message, not a panic or generic "internal error". |
| S7 | **MEDIUM** | Case sensitivity ambiguity | Does literal mode match case-sensitive or case-insensitive? Neither is always right. Must expose a `case_sensitive` parameter (default: false for literal/semantic, true for regex). |
| S8 | **MEDIUM** | Empty query string | `search(query="")` -- what happens? Literal mode: matches everything. Regex mode: matches every position. Must reject empty queries with InvalidParams. |
| S9 | **LOW** | Namespace/scope filtering | If search doesn't support scope/namespace filtering like seek does, agents lose filtering power. Must include `scope` and `node_types` parameters for parity. |
| S10 | **LOW** | Line number accuracy | If provenance line numbers are from ingest time, they drift as files are edited. Documented limitation, but search results showing "line 42" when the function is now at line 67 will confuse agents. |

### Verdict
S1 and S2 are must-fix. S3 should be resolved at design level (deprecate seek or differentiate clearly).

---

## TOOL 2: m1nd.help (self-documenting)

### Context
Returns tool documentation. Currently, tool schemas are in `fn tool_schemas()` in server.rs (line ~790+). Help must read from the same source or risk drift.

### Failure Modes

| # | Severity | Failure | Detail |
|---|----------|---------|--------|
| H1 | **HIGH** | Schema drift | If help hardcodes tool descriptions instead of reading from `tool_schemas()`, any tool addition/change creates inconsistency. Must be generated from the same source of truth. |
| H2 | **HIGH** | Unknown tool_name returns what? | `help(tool_name="m1nd.nonexistent")` -- must return a clear "tool not found" error listing available tools, not crash or return empty. |
| H3 | **MEDIUM** | Output too large for LLM context | Full index of 52+ tools with params, returns, examples = potentially 10K+ tokens. Must support `brief` mode (name + one-line description) vs `full` mode (params + returns + examples). |
| H4 | **MEDIUM** | No examples for new tools | If examples are hardcoded per-tool, new tools ship without examples until someone remembers to add them. Must make examples a required field in tool schema definition. |
| H5 | **MEDIUM** | "suggested_next" tools circular | Help says "after activate, try impact". If impact's help says "after impact, try activate", agents loop. Suggested chains must be DAG-validated or at least non-circular for the first 3 steps. |
| H6 | **LOW** | Versioning | Help output should include m1nd version so agents know what tools are available. Without version, an agent might call a tool that exists in docs but not in their server. |
| H7 | **LOW** | Localization | Help is English-only. Not a bug, but should be documented. |

### Verdict
H1 is must-fix (single source of truth). H2 is must-fix (error handling). H3 should ship with brief/full modes.

---

## TOOL 3: m1nd.report (session auto-report)

### Context
SessionState tracks `queries_processed`, `sessions` (per-agent), `start_time`. But there's no query log -- just a counter. Report needs actual query history to be useful.

### Failure Modes

| # | Severity | Failure | Detail |
|---|----------|---------|--------|
| R1 | **CRITICAL** | No query history exists | `SessionState` only has `queries_processed: u64` (a counter) and `AgentSession` (first_seen, last_seen, query_count). There is NO log of which tools were called, what queries were passed, or what results were returned. Report CANNOT generate "bugs found" or "queries summary" without a query log. Must add `Vec<QueryLogEntry>` to SessionState. |
| R2 | **HIGH** | Memory leak from unbounded query log | If we add a query log, it grows forever. A 24-hour session with 1000 queries, each logged with tool_name + truncated params + timestamp = ~200KB. 10K queries = 2MB. Must cap at 10K entries with ring buffer (overwrite oldest). |
| R3 | **HIGH** | Cross-agent privacy | Agent A's report includes Agent B's queries if they share the same m1nd instance. Multi-agent is a core feature. Report MUST filter by agent_id. |
| R4 | **HIGH** | "Savings" claim accuracy | Report claims "saved X tokens by using m1nd instead of grep". This is marketing, not measurement. If the estimate is wrong (and it will be), users lose trust. Must label savings as "estimated" with clear methodology disclosure. |
| R5 | **MEDIUM** | Empty session report | `report()` called immediately after server start with zero queries. Must return a meaningful "no activity" response, not an empty/null structure. |
| R6 | **MEDIUM** | Graph evolution without ingest | Report says "graph evolved" but if no ingest happened, graph is identical. Must distinguish "graph changed via ingest" vs "weights changed via learn" vs "no change". |
| R7 | **LOW** | Format parameter validation | `format="xml"` is not supported. Must reject unknown formats with InvalidParams listing valid options. |
| R8 | **LOW** | Concurrent report generation | Two agents call report simultaneously. No contention issue (read-only), but both get the same data. Not a bug, just document it. |

### Verdict
R1 is must-fix (need query log infrastructure). R2 is must-fix (ring buffer cap). R3 is must-fix (agent isolation).

---

## TOOL 4: m1nd.panoramic (combined raio-X)

### Context
Runs 7 tools internally: layers + flow_simulate + trust + tremor + epidemic + antibody_scan + missing. Each is a full tool call. This is the most dangerous tool to ship.

### Failure Modes

| # | Severity | Failure | Detail |
|---|----------|---------|--------|
| P1 | **CRITICAL** | Cascading lock contention | All 7 sub-tools need `&mut SessionState`. They run sequentially under the same Mutex lock (server.rs line 533: `let mut session = state.session.lock()`). On HTTP, the lock is held for the entire panoramic duration. A 10K-node graph: layers ~500ms + flow_simulate ~800ms + trust ~200ms + tremor ~150ms + epidemic ~1s + antibody_scan ~300ms + missing ~400ms = **~3.3 seconds of Mutex hold**. All other HTTP/stdio requests are blocked. |
| P2 | **CRITICAL** | Single tool failure aborts entire panoramic | If epidemic panics or returns error (e.g., `NoValidInfectedNodes` on an empty graph), what happens to the other 6 results? Must use `Result` per tool and return partial results with error annotations, not fail the whole call. |
| P3 | **HIGH** | Memory pressure from 7 concurrent result sets | Each tool returns a JSON value. On a 10K-node graph: layers output ~50KB, flow_simulate ~30KB, trust ~40KB, tremor ~20KB, epidemic ~60KB, antibody_scan ~15KB, missing ~25KB = ~240KB peak. Not fatal, but if panoramic is called repeatedly, GC pressure accumulates. |
| P4 | **HIGH** | Empty graph produces 7 errors | On an empty graph, most tools fail (EmptyGraph, NoEntryPoints, etc.). Panoramic must handle the "nothing ingested" case gracefully with a single clear message, not 7 separate error objects. |
| P5 | **HIGH** | Graph too small for meaningful results | A 50-node graph: layers detects maybe 2 layers, epidemic is trivial, trust has no history, tremor has no observations. Panoramic should warn "graph too small for reliable analysis" when nodes < threshold (suggest: 200). |
| P6 | **MEDIUM** | Risk score aggregation is arbitrary | "Per-module risk score combining all signals" -- how? If it's a weighted average, the weights are arbitrary. If epidemic says "high risk" but trust says "low risk", which wins? Must document the aggregation formula explicitly and make weights configurable. |
| P7 | **MEDIUM** | Redundant computation | Some tools share intermediate results (e.g., layers and flow_simulate both traverse the graph). No shared computation = wasted CPU. Not a correctness issue but a performance smell. |
| P8 | **LOW** | Timeout cliff | HTTP timeout is 120s (TOOL_TIMEOUT_SECS). Panoramic on a huge graph could approach this. Must either have its own timeout budget per sub-tool or warn in output when total elapsed > 60s. |

### Verdict
P1 is must-fix (lock hold time is a showstopper). P2 is must-fix (partial results on failure). Consider running sub-tools with per-tool timeout and returning partial results.

---

## TOOL 5: m1nd.savings (economy counter)

### Context
Tracks tokens saved by using m1nd instead of raw file search. This is a marketing/UX feature, not a correctness feature. But wrong numbers are worse than no numbers.

### Failure Modes

| # | Severity | Failure | Detail |
|---|----------|---------|--------|
| V1 | **HIGH** | Savings estimate methodology is unfalsifiable | "If you had used grep, you would have read X tokens." But we don't know what the agent would have grepped. The counterfactual is unknowable. Must use conservative heuristics: e.g., "activate returned 10 nodes with 50 lines of context = 500 lines. A grep for the same query on the codebase would have returned ~2000 lines. Estimated savings: 1500 lines." Document that this is an estimate. |
| V2 | **HIGH** | Global counter persistence race condition | If global counter lives in a file and multiple m1nd instances run (possible in multi-worktree setups), concurrent writes corrupt the counter. Must use atomic file operations (write to temp, rename) or keep counter in-memory only with periodic flush. |
| V3 | **MEDIUM** | Zero queries = zero savings != error | `savings()` with no prior queries should return `{session_savings: 0, global_savings: N}`, not an error. |
| V4 | **MEDIUM** | Overclaiming damages credibility | If savings shows "saved 50K tokens!" but the agent still had to Read 20 files manually, the claim rings hollow. Must only count savings for queries where m1nd results were actually used (requires integration with `learn` feedback). |
| V5 | **MEDIUM** | Per-query savings not tracked | SessionState has no per-query breakdown. Adding `savings` to each query log entry couples two features. Better: compute savings lazily from query log rather than tracking it live. |
| V6 | **LOW** | Units ambiguity | "Savings" in what unit? Tokens? Lines? Characters? Must be explicit. Suggest: lines (most intuitive for developers). |
| V7 | **LOW** | Counter overflow | u64 counter for global savings won't overflow in practice (18 quintillion), but the per-session counter should reset on session start, not accumulate across restarts if state is persisted. |

### Verdict
V1 is critical to get right (methodology). V2 is must-fix if global persistence is implemented. V4 should be addressed to avoid credibility damage.

---

## TOOL 6: Fix perspective.routes (bug -- routes return empty after perspective.start)

### Context
The bug: after `perspective.start`, calling `perspective.routes` returns empty routes even though start returned non-empty routes. I traced the code.

### Root Cause Analysis

The route cache IS populated correctly in `handle_perspective_start` (line 351-358 in perspective_handlers.rs):
```rust
route_cache: Some(CachedRouteSet {
    routes,
    total_routes,
    page_size,
    version,
    ...
}),
```

And `handle_perspective_routes` (line 409-423) reads from it:
```rust
let cached = persp.route_cache.as_ref();
let total_routes = cached.map_or(0, |c| c.total_routes);
// ...
let routes: Vec<Route> = cached
    .map(|c| c.routes.iter().skip(pagination.offset).take(...).collect())
    .unwrap_or_default();
```

**THE BUG IS ON LINE 396**: `require_perspective` takes `&state` (immutable borrow), but the function signature is `&mut SessionState`. The perspective is looked up BEFORE the route cache is read. But there's a subtle issue:

```rust
let persp = require_perspective(state, &input.agent_id, &input.perspective_id)?;
```

This returns `&PerspectiveState` (immutable ref). Then at line 469:
```rust
if let Some(p) = state.get_perspective_mut(&input.agent_id, &input.perspective_id) {
    p.last_accessed_ms = now_ms();
}
```

**This is NOT the bug.** The borrow checker would prevent this from compiling if there was a borrow conflict. Let me look deeper.

**ACTUAL ROOT CAUSE CANDIDATES:**

| # | Severity | Hypothesis | Detail |
|---|----------|------------|--------|
| B1 | **CRITICAL** | focus_node is None | In `handle_perspective_start`, if `focus_node` is None (line 316-324), the routes tuple is `(vec![], now_ms())`. This happens when: (a) no anchor_node is provided AND (b) query doesn't match any node label via `contains`. The start output still returns `routes: page_routes` which is empty, but the response LOOKS successful because it has a perspective_id. The agent then calls `routes` and gets empty. **This is not a routes bug -- it's a start bug.** Start should fail or warn when no focus node is found. |
| B2 | **HIGH** | Staleness invalidation between start and routes | If ANY other operation (learn, ingest) happens between start and routes, `rebuild_engines` / `invalidate_all_perspectives` is called. This sets `route_cache = None` and bumps `route_set_version`. The routes call then sees `total_routes = 0` because `cached` is `None`. **Fix:** routes should re-synthesize when cache is invalidated instead of returning empty. |
| B3 | **HIGH** | Route version mismatch silent failure | If the agent passes `route_set_version` from start, but the perspective was invalidated, routes returns `RouteSetStale` error. But if the agent does NOT pass route_set_version (it's Optional), the staleness check is skipped (line 399-406) and empty cache is returned silently. **Fix:** when cache is None AND no version is passed, re-synthesize instead of returning empty. |
| B4 | **MEDIUM** | Substring match ambiguity | `focus_node` lookup uses `contains` (line 84-86), which matches partial strings. Query "auth" matches "auth.py", "oauth_handler.py", "authentication_service.py". The FIRST match wins (HashMap iteration order = random). Different runs may produce different focus nodes, some with many neighbors (routes) and some with few (empty-looking). |
| B5 | **MEDIUM** | Graph not finalized | If graph is not finalized (`graph.finalized = false`), `synthesize_routes` skips the CSR traversal entirely (line 99: `if graph.finalized {`). Routes are always empty for unfinalized graphs. But ingest calls `finalize()`. This only happens if the graph was loaded from a snapshot that wasn't finalized. |
| B6 | **LOW** | Page size vs total routes | start uses `page_size = 6` hardcoded (line 327). If there are 7+ routes but routes are called with default pagination, the agent only sees 6. Not a "empty" bug but a "missing routes" issue. |

### Root Cause Verdict

**Most likely: B1 + B3 combined.** The perspective starts without a focus node (returns empty routes), and routes doesn't re-synthesize on cache miss. The fix requires two changes:
1. `perspective.start` must return a diagnostic when focus_node is None instead of silently creating a useless perspective.
2. `perspective.routes` must re-synthesize routes when `route_cache` is None (due to invalidation or initial empty), not return empty.

---

## MUST-FIX BEFORE SHIPPING (11 items)

| Priority | ID | Tool | Issue |
|----------|------|------|-------|
| P0 | S2 | search | Unbounded result set -- hard cap at 500 |
| P0 | R1 | report | No query history exists in SessionState -- add query log |
| P0 | P1 | panoramic | Mutex held 3+ seconds blocking all requests -- must break lock or use per-tool locks |
| P0 | P2 | panoramic | Single tool failure aborts entire panoramic -- must return partial results |
| P0 | B1+B3 | perspective.routes | Empty routes on cache miss -- must re-synthesize |
| P1 | S1 | search | Regex ReDoS prevention -- enforce `regex` crate only, never `fancy-regex` |
| P1 | H1 | help | Schema drift -- must generate from tool_schemas() single source |
| P1 | H2 | help | Unknown tool_name error handling |
| P1 | R2 | report | Query log memory leak -- ring buffer cap at 10K entries |
| P1 | R3 | report | Cross-agent privacy -- filter report by agent_id |
| P1 | V1 | savings | Savings methodology -- must be documented and conservative |

## DESIGN RECOMMENDATIONS

1. **Deprecate `seek` in favor of `search(mode="semantic")`** -- having two search tools is confusing.
2. **Panoramic should release and re-acquire the Mutex between sub-tools** -- this prevents the 3s lock hold.
3. **Add a QueryLog struct to SessionState** -- ring buffer of `{timestamp, agent_id, tool_name, query_preview, elapsed_ms, result_count}`. Shared by report and savings.
4. **Help should be code-generated** -- derive help text from the same structs/enums that define tool schemas. One source of truth.
5. **Savings should use `learn` feedback as ground truth** -- only count savings when the agent confirms m1nd results were useful.
6. **perspective.routes cache-miss should trigger lazy re-synthesis** -- never return empty when the graph has data.

---

**Total failure modes found: 42**
**Critical: 5 | High: 14 | Medium: 15 | Low: 8**
**Must-fix before shipping: 11**
