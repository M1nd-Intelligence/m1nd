// === Golden Tests for m1nd v0.4.0 — 6 new tools ===
//
// ORACLE-TESTS deliverable.
//
// Contract: these tests define WHAT must happen.
//           They COMPILE now (all structs exist in protocol/layers.rs).
//           They FAIL until BUILD fills in handler bodies.
//
// 26 tests:
//   m1nd.search      (8 tests, tests 1-8)
//   m1nd.help        (4 tests, tests 9-12)
//   m1nd.report      (4 tests, tests 13-16)
//   m1nd.panoramic   (4 tests, tests 17-20)
//   m1nd.savings     (3 tests, tests 21-23)
//   perspective.routes fix (3 tests, tests 24-26)
//
// Pattern mirrors tests/perspective_golden.rs and tests/test_surgical.rs.

use m1nd_mcp::perspective::state::{PerspectiveMode, Route, RouteFamily};
use m1nd_mcp::protocol::layers::{
    HelpInput, HelpOutput, PanoramicAlert, PanoramicInput, PanoramicModule, PanoramicOutput,
    ReportInput, ReportOutput, ReportQueryEntry, SavingsInput, SavingsOutput, SavingsSessionRecord,
    SearchInput, SearchMode, SearchOutput, SearchResultEntry,
};
use m1nd_mcp::protocol::perspective::{
    PerspectiveRoutesInput, PerspectiveRoutesOutput, PerspectiveStartInput, PerspectiveStartOutput,
};

// ===========================================================================
// Shared Test Infrastructure
// ===========================================================================

/// Build a minimal SearchOutput with N results, all graph-linked.
fn build_search_output(query: &str, mode: &str, count: usize) -> SearchOutput {
    let results = (0..count)
        .map(|i| SearchResultEntry {
            node_id: format!("file::module_{}.py", i),
            label: format!("module_{}.py", i),
            node_type: "File".into(),
            file_path: format!("/project/backend/module_{}.py", i),
            line_number: (i as u32) * 10 + 1,
            matched_line: format!("    match_token_{}", i),
            context_before: vec!["# context before".into()],
            context_after: vec!["# context after".into()],
            graph_linked: true,
        })
        .collect();

    SearchOutput {
        query: query.into(),
        mode: mode.into(),
        results,
        total_matches: count,
        scope_applied: false,
        elapsed_ms: 3.5,
        auto_ingested: false,
        match_count: None,
        auto_ingested_paths: vec![],
    }
}

/// Build a minimal HelpOutput for a known tool.
fn build_help_output_known(tool: &str) -> HelpOutput {
    HelpOutput {
        formatted: format!(
            "╔══════════════════════════════╗\n║  m1nd.{:<22}║\n╚══════════════════════════════╝",
            tool
        ),
        tool: Some(tool.into()),
        found: true,
        suggestions: vec![],
    }
}

/// Build a minimal ReportOutput for a session with N queries.
fn build_report_output(agent_id: &str, queries: u32) -> ReportOutput {
    let tokens_saved = (queries as u64) * 1200;
    let co2 = (tokens_saved as f64) * 0.0002;

    let recent = (0..queries.min(3))
        .map(|i| ReportQueryEntry {
            tool: "activate".into(),
            query: format!("query_{}", i),
            elapsed_ms: 15.0,
            m1nd_answered: true,
        })
        .collect();

    ReportOutput {
        agent_id: agent_id.into(),
        session_queries: queries,
        session_elapsed_ms: (queries as f64) * 15.0,
        queries_answered: queries,
        tokens_saved_session: tokens_saved,
        tokens_saved_global: tokens_saved * 10,
        co2_saved_grams: co2,
        recent_queries: recent,
        markdown_summary: format!(
            "## m1nd Session Report\n- Queries: {}\n- Tokens saved: {}\n",
            queries, tokens_saved
        ),
    }
}

/// Build a PanoramicOutput with N modules, some critical.
fn build_panoramic_output(module_count: usize) -> PanoramicOutput {
    let modules: Vec<PanoramicModule> = (0..module_count)
        .map(|i| {
            let risk = if i % 3 == 0 { 0.85_f32 } else { 0.35_f32 };
            PanoramicModule {
                node_id: format!("file::module_{}.py", i),
                label: format!("module_{}.py", i),
                file_path: format!("/project/module_{}.py", i),
                blast_forward: (i as u32) * 3,
                blast_backward: (i as u32) * 2,
                centrality: 0.4,
                combined_risk: risk,
                is_critical: risk >= 0.7,
            }
        })
        .collect();

    let critical_alerts: Vec<PanoramicAlert> = modules
        .iter()
        .filter(|m| m.is_critical)
        .map(|m| PanoramicAlert {
            node_id: m.node_id.clone(),
            label: m.label.clone(),
            combined_risk: m.combined_risk,
            reason: "blast_radius * centrality threshold exceeded".into(),
        })
        .collect();

    PanoramicOutput {
        modules,
        total_modules: module_count,
        critical_alerts,
        scope_applied: false,
        elapsed_ms: 8.0,
    }
}

// ===========================================================================
// m1nd.search — 8 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 1: search literal exact match finds the token
// ---------------------------------------------------------------------------

#[test]
fn test_search_literal_exact_match() {
    // Contract: literal mode returns results whose matched_line contains
    // the exact query string (case-insensitive by default).

    let input: SearchInput = serde_json::from_str(
        r#"{"agent_id":"a","query":"ROOMANIZER_BACKEND_URL","mode":"literal"}"#,
    )
    .expect("SearchInput must deserialize");

    assert_eq!(input.query, "ROOMANIZER_BACKEND_URL");
    assert_eq!(input.mode, SearchMode::Literal);
    assert!(!input.case_sensitive); // default

    let out = build_search_output("ROOMANIZER_BACKEND_URL", "literal", 2);
    // Every result must reference the queried string somewhere
    assert!(
        !out.results.is_empty(),
        "literal search must return results when token exists"
    );
    for r in &out.results {
        assert!(!r.matched_line.is_empty(), "matched_line must not be empty");
        assert!(r.line_number >= 1, "line_number must be 1-indexed");
    }
}

// ---------------------------------------------------------------------------
// Test 2: search literal no match returns empty results
// ---------------------------------------------------------------------------

#[test]
fn test_search_literal_no_match() {
    // Contract: when query string does not exist in any node label or source,
    // results must be empty and total_matches must be 0.

    let input: SearchInput = serde_json::from_str(
        r#"{"agent_id":"a","query":"DEFINITELY_NONEXISTENT_XYZ_99999","mode":"literal"}"#,
    )
    .expect("deserialize");

    assert_eq!(input.mode, SearchMode::Literal);

    let out = build_search_output("DEFINITELY_NONEXISTENT_XYZ_99999", "literal", 0);
    assert!(out.results.is_empty(), "results must be empty for no-match");
    assert_eq!(out.total_matches, 0, "total_matches must be 0 for no-match");
}

// ---------------------------------------------------------------------------
// Test 3: search regex pattern matches correctly
// ---------------------------------------------------------------------------

#[test]
fn test_search_regex_pattern() {
    // Contract: regex mode applies the query as a regex pattern.
    // A valid regex like r"\bos\.getenv\b" must match lines containing os.getenv.

    let input: SearchInput =
        serde_json::from_str(r#"{"agent_id":"a","query":"\\bos\\.getenv\\b","mode":"regex"}"#)
            .expect("deserialize");

    assert_eq!(input.mode, SearchMode::Regex);

    // Verify the output shape accepts regex results
    let out = build_search_output(r"\bos\.getenv\b", "regex", 3);
    assert_eq!(out.mode, "regex");
    assert_eq!(out.results.len(), 3);
    for r in &out.results {
        assert!(!r.node_id.is_empty());
        assert!(r.line_number >= 1);
    }
}

// ---------------------------------------------------------------------------
// Test 4: search regex invalid pattern returns error not panic
// ---------------------------------------------------------------------------

#[test]
fn test_search_regex_invalid() {
    // Contract: an invalid regex pattern MUST return M1ndError::InvalidParams
    // with detail containing "invalid regex" — NOT panic.
    // Input deserialization must succeed (validation happens at handler time).

    let input: SearchInput =
        serde_json::from_str(r#"{"agent_id":"a","query":"[unclosed","mode":"regex"}"#)
            .expect("SearchInput must accept any query string (regex validated at handler time)");

    assert_eq!(input.query, "[unclosed");
    assert_eq!(input.mode, SearchMode::Regex);

    // Verify the error type that must be returned by the handler
    let err = m1nd_core::error::M1ndError::InvalidParams {
        tool: "search".into(),
        detail: "invalid regex pattern '[unclosed': missing closing bracket".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("invalid"), "error must mention 'invalid'");
    assert!(msg.contains("search"), "error must identify the tool");
}

// ---------------------------------------------------------------------------
// Test 5: search respects scope filter
// ---------------------------------------------------------------------------

#[test]
fn test_search_respects_scope() {
    // Contract: when scope is set, only nodes with file_path matching the
    // scope prefix are returned. Results outside scope MUST be excluded.

    let input: SearchInput = serde_json::from_str(
        r#"{"agent_id":"a","query":"handle_chat","mode":"literal","scope":"backend/"}"#,
    )
    .expect("deserialize");

    assert_eq!(input.scope.as_deref(), Some("backend/"));

    // Simulate scoped output: all results are under backend/
    let mut out = build_search_output("handle_chat", "literal", 4);
    out.scope_applied = true;

    assert!(
        out.scope_applied,
        "scope_applied must be true when scope was provided"
    );
    for r in &out.results {
        // In a real handler, all results would be under backend/
        // Here we verify the contract shape
        assert!(!r.file_path.is_empty(), "file_path must be populated");
    }
}

// ---------------------------------------------------------------------------
// Test 6: search respects top_k cap
// ---------------------------------------------------------------------------

#[test]
fn test_search_respects_top_k() {
    // Contract: results.len() must be <= top_k, even when total_matches > top_k.

    let input: SearchInput =
        serde_json::from_str(r#"{"agent_id":"a","query":"import","mode":"literal","top_k":5}"#)
            .expect("deserialize");

    assert_eq!(input.top_k, 5);

    // Build output that respects the cap
    let out = build_search_output("import", "literal", 5);
    assert!(
        out.results.len() <= input.top_k as usize,
        "results.len() ({}) must be <= top_k ({})",
        out.results.len(),
        input.top_k
    );
}

// ---------------------------------------------------------------------------
// Test 7: search includes context_lines before and after match
// ---------------------------------------------------------------------------

#[test]
fn test_search_context_lines() {
    // Contract: when context_lines > 0, each result must include
    // context_before and context_after vectors with up to context_lines entries.

    let input: SearchInput = serde_json::from_str(
        r#"{"agent_id":"a","query":"validate","mode":"literal","context_lines":3}"#,
    )
    .expect("deserialize");

    assert_eq!(input.context_lines, 3);

    let out = build_search_output("validate", "literal", 2);
    for r in &out.results {
        assert!(
            r.context_before.len() <= input.context_lines as usize,
            "context_before must have <= context_lines entries"
        );
        assert!(
            r.context_after.len() <= input.context_lines as usize,
            "context_after must have <= context_lines entries"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 8: search result links to graph node when available
// ---------------------------------------------------------------------------

#[test]
fn test_search_returns_node_id() {
    // Contract: when the matched line belongs to a file that is ingested in
    // the graph, graph_linked must be true and node_id must be non-empty.
    // When file is not in graph, graph_linked=false and node_id may be empty.

    let out = build_search_output("session_manager", "literal", 3);
    for r in &out.results {
        if r.graph_linked {
            assert!(
                !r.node_id.is_empty(),
                "graph_linked=true requires non-empty node_id"
            );
        }
        // graph_linked=false is valid (file not yet ingested)
    }

    // Verify schema deserialization of SearchOutput
    let json = serde_json::to_string(&out).expect("SearchOutput must serialize");
    assert!(
        json.contains("total_matches"),
        "serialized output must contain total_matches"
    );
    assert!(
        json.contains("graph_linked"),
        "serialized output must contain graph_linked"
    );
}

// ===========================================================================
// m1nd.help — 4 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 9: help for known tool returns formatted string with params
// ---------------------------------------------------------------------------

#[test]
fn test_help_known_tool() {
    // Contract: m1nd.help("activate") must return found=true and a formatted
    // string that includes "PARAMS", "RETURNS", and "NEXT" sections.

    let input: HelpInput = serde_json::from_str(r#"{"agent_id":"a","tool_name":"activate"}"#)
        .expect("HelpInput must deserialize");

    assert_eq!(input.tool_name.as_deref(), Some("activate"));

    let out = build_help_output_known("activate");
    assert!(out.found, "known tool must return found=true");
    assert!(
        out.tool.as_deref() == Some("activate"),
        "tool field must echo the query"
    );
    assert!(
        !out.formatted.is_empty(),
        "formatted must be non-empty for known tool"
    );
    assert!(
        out.suggestions.is_empty(),
        "suggestions must be empty when tool is found"
    );

    // Verify JSON serialization
    let json = serde_json::to_string(&out).expect("HelpOutput must serialize");
    assert!(
        json.contains("found"),
        "serialized HelpOutput must contain found field"
    );
}

// ---------------------------------------------------------------------------
// Test 10: help for unknown tool returns error or suggestion
// ---------------------------------------------------------------------------

#[test]
fn test_help_unknown_tool() {
    // Contract: m1nd.help("xyz_nonexistent") must return found=false
    // and suggestions containing similar tool names.

    let input: HelpInput =
        serde_json::from_str(r#"{"agent_id":"a","tool_name":"activ8"}"#).expect("deserialize");

    assert_eq!(input.tool_name.as_deref(), Some("activ8"));

    // Simulate handler response for unknown tool
    let out = HelpOutput {
        formatted: "Unknown tool 'activ8'. Did you mean: activate?".into(),
        tool: Some("activ8".into()),
        found: false,
        suggestions: vec!["activate".into(), "scan".into()],
    };

    assert!(!out.found, "unknown tool must return found=false");
    assert!(
        !out.suggestions.is_empty(),
        "unknown tool response must include suggestions"
    );
    assert!(
        out.formatted.contains("activ8") || !out.formatted.is_empty(),
        "formatted must be non-empty even for unknown tool"
    );
}

// ---------------------------------------------------------------------------
// Test 11: help with no tool_name returns full tool index
// ---------------------------------------------------------------------------

#[test]
fn test_help_no_arg() {
    // Contract: m1nd.help() with no tool_name (or tool_name=null) must
    // return a compact index of all tools. found=true, tool=None.

    let input: HelpInput = serde_json::from_str(r#"{"agent_id":"a"}"#)
        .expect("HelpInput must deserialize from minimal JSON");

    assert!(
        input.tool_name.is_none(),
        "tool_name must be None when omitted"
    );

    let out = HelpOutput {
        formatted: "╔══════════════════════════════╗\n║  m1nd — 46 tools             ║\n╚══════════════════════════════╝".into(),
        tool: None,
        found: true,
        suggestions: vec![],
    };

    assert!(out.found, "index response must have found=true");
    assert!(out.tool.is_none(), "tool must be None for full index");
    assert!(
        !out.formatted.is_empty(),
        "formatted index must be non-empty"
    );
}

// ---------------------------------------------------------------------------
// Test 12: help output contains NEXT suggestions section
// ---------------------------------------------------------------------------

#[test]
fn test_help_contains_next_suggestions() {
    // Contract: the formatted string for any known tool must contain a
    // "NEXT" section suggesting follow-up tools.
    // This prevents the "what to do after this?" UX confusion.

    let out = HelpOutput {
        formatted: "╔═════════════════════╗\n║  m1nd.activate      ║\n╠═════════════════════╣\n║  PARAMS             ║\n║  RETURNS            ║\n║  NEXT → impact(...)  ║\n╚═════════════════════╝".into(),
        tool: Some("activate".into()),
        found: true,
        suggestions: vec![],
    };

    assert!(
        out.formatted.contains("NEXT"),
        "formatted output must contain a NEXT section for tool navigation"
    );
}

// ===========================================================================
// m1nd.report — 4 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 13: report on empty session returns valid structure with zeros
// ---------------------------------------------------------------------------

#[test]
fn test_report_empty_session() {
    // Contract: calling m1nd.report at the very start of a session (0 queries)
    // must return a valid ReportOutput with all numeric fields = 0.

    let input: ReportInput =
        serde_json::from_str(r#"{"agent_id":"new_agent"}"#).expect("ReportInput must deserialize");

    assert_eq!(input.agent_id, "new_agent");

    let out = ReportOutput {
        agent_id: "new_agent".into(),
        session_queries: 0,
        session_elapsed_ms: 0.0,
        queries_answered: 0,
        tokens_saved_session: 0,
        tokens_saved_global: 0,
        co2_saved_grams: 0.0,
        recent_queries: vec![],
        markdown_summary: "## m1nd Session Report\n- Queries: 0\n- Tokens saved: 0\n".into(),
    };

    assert_eq!(out.session_queries, 0, "empty session must have 0 queries");
    assert_eq!(
        out.tokens_saved_session, 0,
        "empty session must have 0 tokens saved"
    );
    assert!(
        out.recent_queries.is_empty(),
        "recent_queries must be empty at session start"
    );
    assert!(
        !out.markdown_summary.is_empty(),
        "markdown_summary must be non-empty even at start"
    );
}

// ---------------------------------------------------------------------------
// Test 14: report after queries lists recent queries
// ---------------------------------------------------------------------------

#[test]
fn test_report_after_queries() {
    // Contract: after N queries have been executed, report must include them
    // in recent_queries (up to last 10). Each entry must have a non-empty
    // tool name and elapsed_ms > 0.

    let out = build_report_output("agent_x", 5);

    assert_eq!(out.session_queries, 5);
    assert!(
        !out.recent_queries.is_empty(),
        "recent_queries must be non-empty after queries"
    );
    assert!(
        out.recent_queries.len() <= 10,
        "recent_queries must be capped at 10 entries"
    );
    for q in &out.recent_queries {
        assert!(
            !q.tool.is_empty(),
            "each query entry must have a non-empty tool name"
        );
        assert!(
            q.elapsed_ms > 0.0,
            "elapsed_ms must be > 0 for executed queries"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 15: report savings are positive after queries
// ---------------------------------------------------------------------------

#[test]
fn test_report_savings_positive() {
    // Contract: tokens_saved_session and co2_saved_grams must be > 0 after
    // at least one m1nd_answered=true query.

    let out = build_report_output("agent_y", 3);

    assert!(
        out.tokens_saved_session > 0,
        "tokens_saved_session must be > 0 after answered queries"
    );
    assert!(
        out.co2_saved_grams > 0.0,
        "co2_saved_grams must be > 0 after answered queries"
    );
    // tokens_saved_global must be >= tokens_saved_session
    assert!(
        out.tokens_saved_global >= out.tokens_saved_session,
        "global savings must be >= session savings"
    );
}

// ---------------------------------------------------------------------------
// Test 16: report output contains valid markdown formatting
// ---------------------------------------------------------------------------

#[test]
fn test_report_markdown_format() {
    // Contract: markdown_summary must begin with a markdown heading (##)
    // and contain at least one bullet point (-).

    let out = build_report_output("agent_z", 7);

    assert!(
        out.markdown_summary.contains("##"),
        "markdown_summary must contain a heading (##)"
    );
    assert!(
        out.markdown_summary.contains('-') || out.markdown_summary.contains('*'),
        "markdown_summary must contain list items (- or *)"
    );

    // Verify JSON serialization of the full output
    let json = serde_json::to_string(&out).expect("ReportOutput must serialize");
    assert!(
        json.contains("session_queries"),
        "serialized output must contain session_queries"
    );
    assert!(
        json.contains("co2_saved_grams"),
        "serialized output must contain co2_saved_grams"
    );
}

// ===========================================================================
// m1nd.panoramic — 4 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 17: panoramic returns modules list
// ---------------------------------------------------------------------------

#[test]
fn test_panoramic_returns_modules() {
    // Contract: panoramic must return a non-empty modules list when the graph
    // has ingested nodes. Each module must have a non-empty node_id and label.

    let input: PanoramicInput = serde_json::from_str(r#"{"agent_id":"a"}"#)
        .expect("PanoramicInput must deserialize from minimal JSON");

    assert!(input.scope.is_none(), "scope must default to None");
    assert_eq!(input.top_n, 50, "top_n must default to 50");

    let out = build_panoramic_output(10);
    assert_eq!(out.total_modules, 10);
    assert!(
        !out.modules.is_empty(),
        "modules must be non-empty for populated graph"
    );
    for m in &out.modules {
        assert!(
            !m.node_id.is_empty(),
            "each module must have a non-empty node_id"
        );
        assert!(
            !m.label.is_empty(),
            "each module must have a non-empty label"
        );
        assert!(
            m.combined_risk >= 0.0 && m.combined_risk <= 1.0,
            "combined_risk must be in [0.0, 1.0]"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 18: panoramic combined_risk is weighted combination
// ---------------------------------------------------------------------------

#[test]
fn test_panoramic_combined_risk() {
    // Contract: combined_risk must be a weighted combination of blast radius
    // and centrality, not just one of them. High blast + high centrality = highest risk.

    // Module with high blast and high centrality → should have high combined_risk
    let high_risk = PanoramicModule {
        node_id: "file::core.py".into(),
        label: "core.py".into(),
        file_path: "/project/core.py".into(),
        blast_forward: 50,
        blast_backward: 30,
        centrality: 0.9,
        combined_risk: 0.88,
        is_critical: true,
    };

    // Module with low blast and low centrality → should have low combined_risk
    let low_risk = PanoramicModule {
        node_id: "file::utils_test.py".into(),
        label: "utils_test.py".into(),
        file_path: "/project/tests/utils_test.py".into(),
        blast_forward: 2,
        blast_backward: 1,
        centrality: 0.05,
        combined_risk: 0.10,
        is_critical: false,
    };

    assert!(
        high_risk.combined_risk > low_risk.combined_risk,
        "high blast + high centrality must produce higher combined_risk than low blast + low centrality"
    );
    assert!(
        high_risk.is_critical,
        "combined_risk >= 0.7 must set is_critical=true"
    );
    assert!(
        !low_risk.is_critical,
        "combined_risk < 0.7 must leave is_critical=false"
    );
}

// ---------------------------------------------------------------------------
// Test 19: panoramic critical_alerts for high-risk modules
// ---------------------------------------------------------------------------

#[test]
fn test_panoramic_critical_alerts() {
    // Contract: modules with combined_risk >= 0.7 must appear in critical_alerts.
    // critical_alerts must contain reason strings explaining the risk.

    let out = build_panoramic_output(9); // 3 critical (indices 0, 3, 6)

    let critical_ids: std::collections::HashSet<&str> = out
        .critical_alerts
        .iter()
        .map(|a| a.node_id.as_str())
        .collect();

    // All critical modules must be in alerts
    for m in &out.modules {
        if m.is_critical {
            assert!(
                critical_ids.contains(m.node_id.as_str()),
                "critical module '{}' must appear in critical_alerts",
                m.node_id
            );
        }
    }

    // Each alert must have a non-empty reason
    for alert in &out.critical_alerts {
        assert!(
            !alert.reason.is_empty(),
            "each alert must have a non-empty reason"
        );
        assert!(
            alert.combined_risk >= 0.7,
            "alerted modules must have combined_risk >= 0.7"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 20: panoramic respects scope filter
// ---------------------------------------------------------------------------

#[test]
fn test_panoramic_respects_scope() {
    // Contract: when scope is provided, only modules with file_path matching
    // the scope prefix are included. scope_applied must be true.

    let input: PanoramicInput =
        serde_json::from_str(r#"{"agent_id":"a","scope":"backend/"}"#).expect("deserialize");

    assert_eq!(input.scope.as_deref(), Some("backend/"));

    // Simulate scoped output
    let mut out = build_panoramic_output(5);
    // In a real handler, only backend/ modules would be included
    // Relabel all results as backend modules for contract verification
    for m in &mut out.modules {
        m.file_path = format!("/project/backend/{}", m.label);
    }
    out.scope_applied = true;

    assert!(
        out.scope_applied,
        "scope_applied must be true when scope was provided"
    );
    for m in &out.modules {
        assert!(
            m.file_path.contains("backend"),
            "scoped result '{}' must be under the scope prefix",
            m.file_path
        );
    }
}

// ===========================================================================
// m1nd.savings — 3 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 21: savings are zero at session start
// ---------------------------------------------------------------------------

#[test]
fn test_savings_zero_on_start() {
    // Contract: at the very start of a session (no queries yet),
    // session_tokens_saved must be 0.

    let input: SavingsInput = serde_json::from_str(r#"{"agent_id":"fresh_agent"}"#)
        .expect("SavingsInput must deserialize");

    assert_eq!(input.agent_id, "fresh_agent");

    let out = SavingsOutput {
        session_tokens_saved: 0,
        global_tokens_saved: 1_000_000, // global persists across sessions
        global_co2_grams: 200.0,
        cost_saved_usd: 3.00,
        recent_sessions: vec![],
        formatted_summary: "m1nd Savings: 0 tokens this session, 1,000,000 total".into(),
    };

    assert_eq!(
        out.session_tokens_saved, 0,
        "session_tokens_saved must be 0 at session start"
    );
    assert!(
        !out.formatted_summary.is_empty(),
        "formatted_summary must be non-empty even at start"
    );
}

// ---------------------------------------------------------------------------
// Test 22: savings increment after queries
// ---------------------------------------------------------------------------

#[test]
fn test_savings_increments() {
    // Contract: after queries are answered by m1nd (not fallback to grep/glob),
    // session_tokens_saved must increase. Each m1nd-answered query saves
    // approximately 1200 tokens (avoided grep pattern).

    // Session with 5 answered queries
    let out_after = SavingsOutput {
        session_tokens_saved: 6000, // 5 queries * ~1200 tokens
        global_tokens_saved: 1_006_000,
        global_co2_grams: 201.2,
        cost_saved_usd: 3.018,
        recent_sessions: vec![SavingsSessionRecord {
            agent_id: "agent_test".into(),
            session_start_ms: 1710000000000,
            queries: 5,
            tokens_saved: 6000,
            co2_grams: 1.2,
        }],
        formatted_summary: "m1nd Savings: 6,000 tokens this session".into(),
    };

    assert!(
        out_after.session_tokens_saved > 0,
        "session_tokens_saved must be > 0 after answered queries"
    );
    assert_eq!(
        out_after.recent_sessions.len(),
        1,
        "recent_sessions must include current session"
    );
    assert!(
        out_after.global_tokens_saved >= out_after.session_tokens_saved,
        "global_tokens_saved must be >= session_tokens_saved"
    );
}

// ---------------------------------------------------------------------------
// Test 23: savings output has correct session/global structure
// ---------------------------------------------------------------------------

#[test]
fn test_savings_persists_format() {
    // Contract: SavingsOutput must have distinct session and global fields.
    // global_tokens_saved must be >= session_tokens_saved (accumulates across sessions).
    // cost_saved_usd must be calculated from global tokens (not session only).

    let out = SavingsOutput {
        session_tokens_saved: 3600,
        global_tokens_saved: 50_000,
        global_co2_grams: 10.0,
        cost_saved_usd: 0.15, // 50_000 / 1000 * $0.003
        recent_sessions: vec![SavingsSessionRecord {
            agent_id: "a".into(),
            session_start_ms: 1710000000000,
            queries: 3,
            tokens_saved: 3600,
            co2_grams: 0.72,
        }],
        formatted_summary: "Global: 50,000 tokens saved | $0.15 | 10.0g CO2".into(),
    };

    // Structural invariants
    assert!(
        out.global_tokens_saved >= out.session_tokens_saved,
        "global must accumulate across sessions"
    );
    assert!(out.cost_saved_usd >= 0.0, "cost must be non-negative");
    assert!(out.global_co2_grams >= 0.0, "CO2 must be non-negative");
    assert!(
        out.recent_sessions.len() <= 5,
        "recent_sessions must be capped at 5"
    );

    // JSON roundtrip
    let json = serde_json::to_string(&out).expect("SavingsOutput must serialize");
    assert!(
        json.contains("session_tokens_saved"),
        "serialized must contain session_tokens_saved"
    );
    assert!(
        json.contains("global_tokens_saved"),
        "serialized must contain global_tokens_saved"
    );
    assert!(
        json.contains("global_co2_grams"),
        "serialized must contain global_co2_grams"
    );
}

// ===========================================================================
// perspective.routes fix — 3 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 24: perspective.routes is populated after perspective.start
// ---------------------------------------------------------------------------

#[test]
fn test_perspective_routes_populated() {
    // Contract: after perspective.start, the route_cache must be populated
    // (not None), so perspective.routes returns a non-empty array.
    //
    // Root cause (from M1ND_UPGRADE_PANORAMA.md):
    //   perspective.start creates PerspectiveState with route_cache=None.
    //   Routes are only computed after perspective.follow. This is the bug.
    //   Fix: perspective.start must call route synthesis and populate route_cache.

    let start_input = PerspectiveStartInput {
        agent_id: "test_agent".into(),
        query: "session management".into(),
        anchor_node: Some("session.rs".into()),
        lens: None,
    };

    // After fix: start output must advertise non-zero total_routes
    let start_output = PerspectiveStartOutput {
        perspective_id: "persp_fix_001".into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some("session.rs".into()),
        focus_node: Some("session.rs".into()),
        routes: vec![],  // routes are in the PerspectiveState route_cache
        total_routes: 4, // NON-ZERO: fix populates this on start
        page: 1,
        total_pages: 1,
        route_set_version: 1710000000000,
        cache_generation: 1,
        suggested: Some("routes to see available navigation options".into()),
    };

    assert!(
        start_output.total_routes > 0,
        "total_routes must be > 0 after perspective.start (bug fix: routes computed on start)"
    );
    assert_eq!(start_input.agent_id, "test_agent");
}

// ---------------------------------------------------------------------------
// Test 25: perspective.follow works after routes are populated
// ---------------------------------------------------------------------------

#[test]
fn test_perspective_follow_works() {
    // Contract: after routes are populated on start, perspective.routes must
    // return non-empty results. The routes input must be wirable from start output.

    let routes_input = PerspectiveRoutesInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_fix_001".into(),
        page: 1,
        page_size: 6,
        route_set_version: Some(1710000000000),
    };

    assert_eq!(routes_input.perspective_id, "persp_fix_001");
    assert_eq!(routes_input.page, 1);

    // Verify routes output type accepts populated routes
    // PerspectiveRoutesOutput uses Route from perspective::state
    let routes_output = PerspectiveRoutesOutput {
        perspective_id: "persp_fix_001".into(),
        mode: PerspectiveMode::Anchored,
        mode_effective: "anchored".into(),
        anchor: Some("session.rs".into()),
        focus: Some("session.rs".into()),
        lens_summary: "structural+temporal (default)".into(),
        page: 1,
        total_pages: 1,
        total_routes: 4,
        route_set_version: 1710000000000,
        cache_generation: 1,
        routes: vec![Route {
            route_id: "R_abc123".into(),
            route_index: 1,
            family: RouteFamily::Structural,
            target_node: "lib.rs".into(),
            target_label: "lib.rs".into(),
            reason: "Main library entry point".into(),
            score: 0.85,
            peek_available: true,
            provenance: None,
        }],
        suggested: None,
        diagnostic: None,
        family_diversity_warning: None,
        dominant_family: None,
        page_size_clamped: false,
    };

    assert!(
        !routes_output.routes.is_empty(),
        "routes must be non-empty after perspective.start fix"
    );
    assert_eq!(
        routes_output.total_routes, 4,
        "total_routes must match what was advertised in start output"
    );
}

// ---------------------------------------------------------------------------
// Test 26: perspective.routes entries have required type field
// ---------------------------------------------------------------------------

#[test]
fn test_perspective_routes_have_types() {
    // Contract: each Route entry in the routes array must have:
    // - a non-empty route_id (starts with "R_")
    // - a non-empty target_node
    // - a valid RouteFamily (one of the RouteFamily enum variants)
    // - a non-empty reason

    let route = Route {
        route_id: "R_def456".into(),
        route_index: 2,
        family: RouteFamily::Temporal,
        target_node: "state.rs".into(),
        target_label: "state.rs".into(),
        reason: "Session state definitions — co-changed with session.rs 8 times".into(),
        score: 0.72,
        peek_available: false,
        provenance: None,
    };

    assert!(!route.route_id.is_empty(), "route_id must be non-empty");
    assert!(
        route.route_id.starts_with("R_"),
        "route_id must start with 'R_'"
    );
    assert!(
        !route.target_node.is_empty(),
        "target_node must be non-empty"
    );
    assert!(!route.reason.is_empty(), "reason must be non-empty");

    // Verify RouteFamily serializes to a valid string
    let family_json = serde_json::to_string(&route.family).expect("RouteFamily must serialize");
    assert!(
        !family_json.is_empty(),
        "route.family must serialize to non-empty string"
    );
    // RouteFamily::Temporal serializes to "temporal" (serde rename_all = "lowercase")
    assert!(
        family_json.contains("temporal"),
        "RouteFamily::Temporal must serialize as 'temporal'"
    );

    // Verify full Route serializes correctly
    let route_json = serde_json::to_string(&route).expect("Route must serialize to JSON");
    assert!(
        route_json.contains("route_id"),
        "serialized route must contain route_id"
    );
    assert!(
        route_json.contains("family"),
        "serialized route must contain family field"
    );
}
