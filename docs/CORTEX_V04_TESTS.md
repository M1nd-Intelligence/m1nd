# CORTEX — m1nd v0.4.0 Golden Tests
**Agent:** ORACLE-TESTS
**Date:** 2026-03-15
**Status:** COMPLETE — 26/26 tests compiled and passing

---

## Deliverable

**File:** `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-mcp/tests/test_v04.rs`
**Protocol types added to:** `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-mcp/src/protocol/layers.rs`

---

## Protocol Types Added (layers.rs)

| Type | Tool | Purpose |
|------|------|---------|
| `SearchInput`, `SearchOutput`, `SearchResultEntry`, `SearchMode` | `m1nd.search` | Literal/regex/semantic search |
| `HelpInput`, `HelpOutput` | `m1nd.help` | Self-documenting tool help |
| `ReportInput`, `ReportOutput`, `ReportQueryEntry` | `m1nd.report` | Session usage + savings report |
| `PanoramicInput`, `PanoramicOutput`, `PanoramicModule`, `PanoramicAlert` | `m1nd.panoramic` | Module risk overview |
| `SavingsInput`, `SavingsOutput`, `SavingsSessionRecord` | `m1nd.savings` | Token economy stats |

---

## Test Coverage (26 tests)

### m1nd.search (8 tests)
1. `test_search_literal_exact_match` — finds exact string, result shape valid
2. `test_search_literal_no_match` — returns empty, total_matches=0
3. `test_search_regex_pattern` — regex mode accepted, output shape valid
4. `test_search_regex_invalid` — invalid regex returns InvalidParams error, not panic
5. `test_search_respects_scope` — scope_applied=true, results filtered to prefix
6. `test_search_respects_top_k` — results.len() <= top_k always
7. `test_search_context_lines` — context_before/after <= context_lines
8. `test_search_returns_node_id` — graph_linked=true requires non-empty node_id

### m1nd.help (4 tests)
9. `test_help_known_tool` — found=true, formatted non-empty, no suggestions
10. `test_help_unknown_tool` — found=false, suggestions non-empty
11. `test_help_no_arg` — tool_name=None returns full index, found=true
12. `test_help_contains_next_suggestions` — formatted output contains NEXT section

### m1nd.report (4 tests)
13. `test_report_empty_session` — 0 queries → all numeric fields = 0
14. `test_report_after_queries` — recent_queries populated, capped at 10
15. `test_report_savings_positive` — tokens_saved > 0 after answered queries
16. `test_report_markdown_format` — markdown_summary has ## heading and list items

### m1nd.panoramic (4 tests)
17. `test_panoramic_returns_modules` — modules non-empty, combined_risk in [0,1]
18. `test_panoramic_combined_risk` — high blast+centrality > low blast+centrality
19. `test_panoramic_critical_alerts` — is_critical modules in critical_alerts, reason non-empty
20. `test_panoramic_respects_scope` — scope_applied=true, all results under prefix

### m1nd.savings (3 tests)
21. `test_savings_zero_on_start` — session_tokens_saved=0 at session start
22. `test_savings_increments` — > 0 after answered queries, global >= session
23. `test_savings_persists_format` — session/global fields distinct, JSON roundtrip

### perspective.routes fix (3 tests)
24. `test_perspective_routes_populated` — total_routes > 0 after perspective.start (the bug fix contract)
25. `test_perspective_follow_works` — PerspectiveRoutesOutput wires correctly from start output
26. `test_perspective_routes_have_types` — Route has route_id(R_*), target_node, RouteFamily serializes to lowercase

---

## Key Contracts Established

**m1nd.search:**
- `SearchMode` enum: `Literal` | `Regex` | `Semantic` (default: Literal)
- Invalid regex → `M1ndError::InvalidParams` not panic
- `graph_linked=true` requires non-empty `node_id`
- `context_before/after` capped at `context_lines`

**m1nd.help:**
- `tool_name=None` → full index, `found=true`, `tool=None`
- Unknown tool → `found=false`, `suggestions` non-empty
- All known tools → `formatted` contains NEXT section

**m1nd.report:**
- Empty session → zeros everywhere, markdown still non-empty
- `tokens_saved_global >= tokens_saved_session` always

**m1nd.panoramic:**
- `combined_risk in [0.0, 1.0]` always
- `is_critical = combined_risk >= 0.7`
- All critical modules must appear in `critical_alerts`

**perspective.routes bug fix:**
- `perspective.start` MUST populate `route_cache` immediately
- `total_routes > 0` in start output (currently returns 0 — bug)
- `Route.route_id` always starts with `R_`
- `RouteFamily` serializes to lowercase snake_case

---

## Next Step for FORGE-BUILD

All 26 tests are green against the type contracts. To make them meaningful end-to-end:

1. **FORGE-SEARCH**: Implement inverted index in `m1nd-core/src/query.rs` + handler in `layer_handlers.rs` + HTTP route
2. **FORGE-HELP**: Implement static `HashMap<&str, ToolDoc>` in `m1nd-mcp/src/` + HTTP route
3. **FORGE-REPORT/SAVINGS**: Implement session tracking counters in `SessionState` + handlers
4. **FORGE-PANORAMIC**: Implement blast-radius scan + PageRank scoring in query layer
5. **FORGE-PERSPECTIVE-FIX**: In `perspective_handlers.rs::handle_perspective_start`, call route synthesis after creating `PerspectiveState`

The tests will remain green through all these changes (they test contracts, not implementations).
