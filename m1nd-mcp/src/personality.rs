// === m1nd-mcp/src/personality.rs ===
//
// v0.4.0: _m1nd metadata builder, suggest_next mapping, personality templates,
// ANSI formatting, visual identity glyphs.

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Visual Identity — glyphs and ANSI colors
// ---------------------------------------------------------------------------

/// m1nd logo glyphs with their semantic meanings.
pub const GLYPH_SIGNAL: &str = "\u{234C}"; // ⍌ — spreading activation signal
pub const GLYPH_PATH: &str = "\u{2350}"; // ⍐ — paths through the graph
pub const GLYPH_STRUCTURE: &str = "\u{2342}"; // ⍂ — structural analysis
pub const GLYPH_DIMENSION: &str = "\u{1D53B}"; // 𝔻 — 4D dimensional scoring
pub const GLYPH_CONNECTION: &str = "\u{27C1}"; // ⟁ — graph connections, edges

/// ANSI escape codes for m1nd's color palette.
pub const ANSI_CYAN: &str = "\x1b[38;2;0;212;255m";
pub const ANSI_GOLD: &str = "\x1b[38;2;255;215;0m";
pub const ANSI_MAGENTA: &str = "\x1b[38;2;255;0;255m";
pub const ANSI_BLUE: &str = "\x1b[38;2;65;105;225m";
pub const ANSI_GREEN: &str = "\x1b[38;2;0;255;136m";
pub const ANSI_RED: &str = "\x1b[38;2;255;71;87m";
pub const ANSI_DIM: &str = "\x1b[38;2;90;101;119m";
pub const ANSI_RESET: &str = "\x1b[0m";
pub const ANSI_BOLD: &str = "\x1b[1m";

// ---------------------------------------------------------------------------
// Gradient border builder
// ---------------------------------------------------------------------------

/// Build a gradient top border (cyan -> magenta -> blue -> green).
pub fn gradient_top_border(width: usize) -> String {
    let colors = [ANSI_CYAN, ANSI_MAGENTA, ANSI_BLUE, ANSI_GREEN];
    let segment = width / colors.len();
    let mut border = String::new();
    for (i, color) in colors.iter().enumerate() {
        let len = if i == colors.len() - 1 {
            width - segment * i
        } else {
            segment
        };
        border.push_str(color);
        for _ in 0..len {
            border.push('\u{2550}'); // ═
        }
    }
    border.push_str(ANSI_RESET);
    border
}

/// Build a gradient bottom border.
pub fn gradient_bottom_border(width: usize) -> String {
    let colors = [ANSI_GREEN, ANSI_BLUE, ANSI_MAGENTA, ANSI_CYAN];
    let segment = width / colors.len();
    let mut border = String::new();
    for (i, color) in colors.iter().enumerate() {
        let len = if i == colors.len() - 1 {
            width - segment * i
        } else {
            segment
        };
        border.push_str(color);
        for _ in 0..len {
            border.push('\u{2550}'); // ═
        }
    }
    border.push_str(ANSI_RESET);
    border
}

// ---------------------------------------------------------------------------
// suggest_next mapping (D5 from synthesis)
// ---------------------------------------------------------------------------

/// Returns suggested next tool calls based on the tool just executed.
pub fn suggest_next(tool_name: &str) -> Vec<String> {
    match tool_name {
        "m1nd_activate" | "m1nd_seek" | "m1nd_search" => vec![
            "impact(top_result) to check blast radius".into(),
            "learn(feedback) to strengthen edges".into(),
            "hypothesize(claim) to test a theory".into(),
        ],
        "m1nd_impact" => vec![
            "validate_plan(files) to verify changes".into(),
            "counterfactual(node) to simulate removal".into(),
            "predict(changed_node) for co-change likelihood".into(),
        ],
        "m1nd_hypothesize" => vec![
            "missing(query) to find structural holes".into(),
            "trace(error) to follow dependency chain".into(),
            "learn(feedback) to confirm/deny".into(),
        ],
        "m1nd_surgical_context" | "m1nd_surgical_context_v2" => vec![
            "apply(file, content) to make changes".into(),
            "apply_batch(edits) for multiple files".into(),
        ],
        "m1nd_apply" | "m1nd_apply_batch" => vec![
            "predict(changed_node) for ripple effects".into(),
            "learn(feedback) to update graph".into(),
        ],
        "m1nd_missing" => vec![
            "activate(topic) to explore the gap".into(),
            "hypothesize(claim) about the missing piece".into(),
        ],
        "m1nd_predict" => vec![
            "impact(predicted_node) to verify".into(),
            "learn(feedback) to calibrate".into(),
        ],
        "m1nd_panoramic" => vec![
            "impact(critical_module) for deep dive".into(),
            "antibody_scan to check for patterns".into(),
        ],
        "m1nd_ingest" => vec![
            "activate(topic) to explore ingested code".into(),
            "layers to detect architecture".into(),
            "panoramic for full health scan".into(),
        ],
        "m1nd_layers" => vec![
            "layer_inspect(layer_name) for details".into(),
            "panoramic for risk analysis".into(),
        ],
        "m1nd_trust" => vec![
            "tremor(node) to check volatility".into(),
            "panoramic for combined view".into(),
        ],
        _ => vec![
            "activate(query) for exploration".into(),
            "help for tool reference".into(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Personality templates (D2 from synthesis)
// ---------------------------------------------------------------------------

/// Generate a personality one-liner based on tool and result.
pub fn personality_line(tool_name: &str, result: &Value) -> String {
    match tool_name {
        "m1nd_activate" => {
            let count = result
                .get("results")
                .and_then(|v| v.as_array())
                .map_or(0, |a| a.len());
            let query = result.get("query").and_then(|v| v.as_str()).unwrap_or("?");
            if count == 0 {
                format!("no results for '{}'. try ingest first, or rephrase.", query)
            } else {
                let top = result
                    .get("results")
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.get("label"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                format!("found {} results for '{}'. top hit: {}.", count, query, top)
            }
        }
        "m1nd_impact" => {
            let total = result
                .get("blast_radius")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            format!("{} nodes in blast radius. careful here.", total)
        }
        "m1nd_search" => {
            let count = result
                .get("total_matches")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let query = result.get("query").and_then(|v| v.as_str()).unwrap_or("?");
            let mode = result
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("literal");
            format!("{} matches for '{}' ({})", count, query, mode)
        }
        "m1nd_panoramic" => {
            let total = result
                .get("total_modules")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let alerts = result
                .get("critical_alerts")
                .and_then(|v| v.as_array())
                .map_or(0, |a| a.len());
            format!("{} modules scanned. {} critical alerts.", total, alerts)
        }
        "m1nd_hypothesize" => {
            let verdict = result
                .get("verdict")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let confidence = result
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!(
                "verdict: {} ({:.0}% confidence).",
                verdict,
                confidence * 100.0
            )
        }
        "m1nd_ingest" => {
            let nodes = result
                .get("node_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let edges = result
                .get("edge_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            format!("ingested: {} nodes, {} edges. graph ready.", nodes, edges)
        }
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// _m1nd metadata builder
// ---------------------------------------------------------------------------

/// Build the `_m1nd` metadata envelope to wrap every tool response.
pub fn build_m1nd_meta(
    tool_name: &str,
    result: &Value,
    session_tokens_saved: u64,
    global_tokens_saved: u64,
) -> Value {
    let suggestions = suggest_next(tool_name);
    let personality = personality_line(tool_name, result);

    let mut meta = json!({
        "suggest_next": suggestions,
        "savings": {
            "query_tokens_saved": estimate_query_savings(tool_name),
            "session_total": session_tokens_saved,
        },
        "gaia": {
            "global_tokens_never_burned": global_tokens_saved,
        },
    });

    if !personality.is_empty() {
        meta["personality"] = Value::String(personality);
    }

    meta
}

/// Estimate tokens saved for a single query.
fn estimate_query_savings(tool_name: &str) -> u64 {
    match tool_name {
        "m1nd_activate" | "m1nd_seek" | "m1nd_search" => 750,
        "m1nd_impact" | "m1nd_predict" | "m1nd_counterfactual" => 1000,
        "m1nd_surgical_context" => 3200,
        "m1nd_surgical_context_v2" => 4800,
        "m1nd_hypothesize" | "m1nd_missing" => 1000,
        "m1nd_apply" | "m1nd_apply_batch" => 900,
        "m1nd_scan" => 1000,
        _ => 500,
    }
}

// ---------------------------------------------------------------------------
// Help tool content
// ---------------------------------------------------------------------------

/// Tool documentation entry for the help system.
pub struct ToolDoc {
    pub name: &'static str,
    pub category: &'static str,
    pub glyph: &'static str,
    pub one_liner: &'static str,
    pub params: &'static [(&'static str, &'static str, bool)], // (name, description, required)
    pub returns: &'static str,
    pub example: &'static str,
    pub next: &'static [&'static str],
}

/// Get all tool documentation entries.
pub fn tool_docs() -> Vec<ToolDoc> {
    vec![
        ToolDoc {
            name: "m1nd_activate",
            category: "Foundation",
            glyph: GLYPH_SIGNAL,
            one_liner: "Spreading activation query -- fire signal into the graph",
            params: &[
                ("query", "Search query for spreading activation", true),
                ("agent_id", "Calling agent identifier", true),
                ("top_k", "Number of top results (default: 20)", false),
                (
                    "dimensions",
                    "Activation dimensions (structural, semantic, temporal, causal)",
                    false,
                ),
                (
                    "xlr",
                    "Enable XLR noise cancellation (default: true)",
                    false,
                ),
            ],
            returns: "Ranked list of activated nodes with scores, dimensions, ghost edges",
            example: r#"{"query": "rate limiting", "agent_id": "jimi", "top_k": 10}"#,
            next: &["impact", "learn", "hypothesize"],
        },
        ToolDoc {
            name: "m1nd_impact",
            category: "Foundation",
            glyph: GLYPH_SIGNAL,
            one_liner: "Blast radius analysis -- who gets hit when this changes",
            params: &[
                ("node_id", "Target node identifier", true),
                ("agent_id", "Calling agent identifier", true),
                (
                    "direction",
                    "forward | reverse | both (default: forward)",
                    false,
                ),
            ],
            returns: "Blast radius, depth distribution, causal chains",
            example: r#"{"node_id": "file::backend/chat_handler.py", "agent_id": "jimi"}"#,
            next: &["validate_plan", "counterfactual", "predict"],
        },
        ToolDoc {
            name: "m1nd_missing",
            category: "Foundation",
            glyph: GLYPH_SIGNAL,
            one_liner: "Find structural holes -- what connections SHOULD exist but don't",
            params: &[
                ("query", "Topic to find structural holes around", true),
                ("agent_id", "Calling agent identifier", true),
            ],
            returns: "Missing edges, ghost edges, structural holes",
            example: r#"{"query": "authentication", "agent_id": "jimi"}"#,
            next: &["activate", "hypothesize"],
        },
        ToolDoc {
            name: "m1nd_why",
            category: "Foundation",
            glyph: GLYPH_PATH,
            one_liner: "Path explanation -- how are two nodes connected?",
            params: &[
                ("source", "Source node", true),
                ("target", "Target node", true),
                ("agent_id", "Calling agent identifier", true),
                ("max_hops", "Maximum hops (default: 6)", false),
            ],
            returns: "Shortest path with edge weights and relation types",
            example: r#"{"source": "file::auth.py", "target": "file::db.py", "agent_id": "jimi"}"#,
            next: &["trace", "impact"],
        },
        ToolDoc {
            name: "m1nd_warmup",
            category: "Foundation",
            glyph: GLYPH_SIGNAL,
            one_liner: "Task-based priming -- prepare the graph for focused work",
            params: &[
                ("task_description", "Description of the task", true),
                ("agent_id", "Calling agent identifier", true),
            ],
            returns: "Primed node count, boost summary",
            example: r#"{"task_description": "fix rate limiting in smart_router", "agent_id": "jimi"}"#,
            next: &["activate", "impact"],
        },
        ToolDoc {
            name: "m1nd_counterfactual",
            category: "Foundation",
            glyph: GLYPH_STRUCTURE,
            one_liner: "What-if simulation -- what breaks if we remove these nodes?",
            params: &[
                ("node_ids", "Nodes to simulate removal of", true),
                ("agent_id", "Calling agent identifier", true),
            ],
            returns: "Orphaned nodes, broken paths, cascade impact",
            example: r#"{"node_ids": ["file::legacy.py"], "agent_id": "jimi"}"#,
            next: &["impact", "predict"],
        },
        ToolDoc {
            name: "m1nd_predict",
            category: "Foundation",
            glyph: GLYPH_DIMENSION,
            one_liner: "Co-change prediction -- what else needs to change?",
            params: &[
                ("changed_node", "Node that was changed", true),
                ("agent_id", "Calling agent identifier", true),
                ("top_k", "Number of predictions (default: 10)", false),
            ],
            returns: "Predicted co-change nodes with probability scores",
            example: r#"{"changed_node": "file::session.py", "agent_id": "jimi"}"#,
            next: &["impact", "learn"],
        },
        ToolDoc {
            name: "m1nd_fingerprint",
            category: "Foundation",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Activation fingerprint -- find duplicate/equivalent code",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("target_node", "Node to find equivalents for", false),
                (
                    "similarity_threshold",
                    "Cosine similarity threshold (default: 0.85)",
                    false,
                ),
            ],
            returns: "Equivalent node pairs with similarity scores",
            example: r#"{"target_node": "file::utils.py", "agent_id": "jimi"}"#,
            next: &["counterfactual", "differential"],
        },
        ToolDoc {
            name: "m1nd_drift",
            category: "Foundation",
            glyph: GLYPH_DIMENSION,
            one_liner: "Weight drift since last session -- what changed in the graph?",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                (
                    "since",
                    "Baseline: last_session (default) or ISO date",
                    false,
                ),
            ],
            returns: "Edge weight changes, node additions/removals",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["activate", "ingest"],
        },
        ToolDoc {
            name: "m1nd_learn",
            category: "Foundation",
            glyph: GLYPH_CONNECTION,
            one_liner: "Hebbian feedback -- correct/wrong/partial strengthens edges",
            params: &[
                ("query", "Original query", true),
                ("agent_id", "Calling agent identifier", true),
                ("feedback", "correct | wrong | partial", true),
                ("node_ids", "Nodes to apply feedback to", true),
            ],
            returns: "Updated edge weights, plasticity state",
            example: r#"{"query": "auth flow", "feedback": "correct", "node_ids": ["file::auth.py"], "agent_id": "jimi"}"#,
            next: &["activate", "predict"],
        },
        ToolDoc {
            name: "m1nd_ingest",
            category: "Foundation",
            glyph: GLYPH_CONNECTION,
            one_liner: "Load codebase into the graph -- the foundation of everything",
            params: &[
                ("path", "Filesystem path to source root", true),
                ("agent_id", "Calling agent identifier", true),
                ("adapter", "code | json | memory (default: code)", false),
                ("mode", "replace | merge (default: replace)", false),
            ],
            returns: "Node/edge counts, ingest stats",
            example: r#"{"path": "/project/backend", "agent_id": "jimi"}"#,
            next: &["activate", "layers", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_resonate",
            category: "Foundation",
            glyph: GLYPH_SIGNAL,
            one_liner: "Standing wave harmonics -- find resonant patterns in the graph",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("query", "Seed query", false),
                ("node_id", "Specific seed node", false),
            ],
            returns: "Harmonics, sympathetic pairs, resonant frequencies",
            example: r#"{"query": "error handling", "agent_id": "jimi"}"#,
            next: &["activate", "fingerprint"],
        },
        ToolDoc {
            name: "m1nd_health",
            category: "Foundation",
            glyph: GLYPH_DIMENSION,
            one_liner: "Server health -- graph size, uptime, sessions",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Status, node/edge counts, uptime, active sessions",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["ingest", "drift"],
        },
        // --- Superpowers ---
        ToolDoc {
            name: "m1nd_seek",
            category: "Superpowers",
            glyph: GLYPH_PATH,
            one_liner: "Intent-aware semantic search -- find code by PURPOSE",
            params: &[
                ("query", "Natural language query", true),
                ("agent_id", "Calling agent identifier", true),
                ("top_k", "Max results (default: 20)", false),
                ("scope", "File path prefix filter", false),
            ],
            returns: "Ranked results with trigram + PageRank scoring",
            example: r#"{"query": "rate limit retry logic", "agent_id": "jimi"}"#,
            next: &["impact", "learn"],
        },
        ToolDoc {
            name: "m1nd_scan",
            category: "Superpowers",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Pattern-based structural analysis with graph validation",
            params: &[
                (
                    "pattern",
                    "Pattern ID (error_handling, concurrency, auth_boundary, etc.)",
                    true,
                ),
                ("agent_id", "Calling agent identifier", true),
                ("scope", "File path prefix", false),
            ],
            returns: "Findings with severity, graph-validated",
            example: r#"{"pattern": "error_handling", "agent_id": "jimi"}"#,
            next: &["hypothesize", "impact"],
        },
        ToolDoc {
            name: "m1nd_search",
            category: "Superpowers",
            glyph: GLYPH_PATH,
            one_liner: "Literal/regex/semantic code search with graph context",
            params: &[
                ("query", "Search term or regex pattern", true),
                ("agent_id", "Calling agent identifier", true),
                (
                    "mode",
                    "literal | regex | semantic (default: literal)",
                    false,
                ),
                ("scope", "File path prefix filter", false),
                ("top_k", "Max results (default: 50, max: 500)", false),
                (
                    "context_lines",
                    "Lines of context (default: 2, max: 10)",
                    false,
                ),
                (
                    "case_sensitive",
                    "Case-sensitive matching (default: false)",
                    false,
                ),
            ],
            returns: "File matches with context lines and graph node cross-references",
            example: r#"{"query": "ANTHROPIC_API_KEY", "agent_id": "jimi", "mode": "literal"}"#,
            next: &["impact", "learn"],
        },
        // --- Extended ---
        ToolDoc {
            name: "m1nd_hypothesize",
            category: "Extended",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Test a structural claim about the codebase",
            params: &[
                ("claim", "Natural language claim", true),
                ("agent_id", "Calling agent identifier", true),
            ],
            returns: "Verdict (likely_true/likely_false/inconclusive), confidence, evidence",
            example: r#"{"claim": "chat_handler validates session tokens", "agent_id": "jimi"}"#,
            next: &["missing", "learn"],
        },
        ToolDoc {
            name: "m1nd_trace",
            category: "Extended",
            glyph: GLYPH_PATH,
            one_liner: "Dependency chain tracing -- follow imports and calls",
            params: &[
                ("query", "Start node or query", true),
                ("agent_id", "Calling agent identifier", true),
            ],
            returns: "Dependency chains with edge types",
            example: r#"{"query": "file::auth.py", "agent_id": "jimi"}"#,
            next: &["impact", "hypothesize"],
        },
        ToolDoc {
            name: "m1nd_differential",
            category: "Extended",
            glyph: GLYPH_DIMENSION,
            one_liner: "Compare two subgraphs -- structural diff",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("group_a", "First set of nodes", true),
                ("group_b", "Second set of nodes", true),
            ],
            returns: "Shared/unique nodes, edge deltas, structural similarity",
            example: r#"{"group_a": ["file::v1.py"], "group_b": ["file::v2.py"], "agent_id": "jimi"}"#,
            next: &["fingerprint", "counterfactual"],
        },
        ToolDoc {
            name: "m1nd_validate_plan",
            category: "Extended",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Validate a multi-step code change plan against the graph",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("plan", "Array of file changes", true),
            ],
            returns: "Validation result: conflicts, missing deps, risk assessment",
            example: r#"{"plan": [{"file": "auth.py", "action": "modify"}], "agent_id": "jimi"}"#,
            next: &["impact", "apply"],
        },
        ToolDoc {
            name: "m1nd_federate",
            category: "Extended",
            glyph: GLYPH_CONNECTION,
            one_liner: "Query across graph namespaces",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("query", "Search query", true),
            ],
            returns: "Results from all namespaces with provenance",
            example: r#"{"query": "authentication", "agent_id": "jimi"}"#,
            next: &["activate", "why"],
        },
        // --- Superpowers: Immunology, Seismology, etc. ---
        ToolDoc {
            name: "m1nd_antibody_scan",
            category: "Superpowers",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Immune system -- scan for known bug patterns",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Antibody matches with severity and affected nodes",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["antibody_create", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_antibody_list",
            category: "Superpowers",
            glyph: GLYPH_STRUCTURE,
            one_liner: "List all stored antibody patterns",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "All antibodies with patterns and match counts",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["antibody_create", "antibody_scan"],
        },
        ToolDoc {
            name: "m1nd_antibody_create",
            category: "Superpowers",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Create a new antibody bug pattern",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("pattern", "Bug pattern to detect", true),
                ("description", "Human-readable description", true),
            ],
            returns: "Created antibody with ID",
            example: r#"{"pattern": "unwrap().unwrap()", "description": "double unwrap", "agent_id": "jimi"}"#,
            next: &["antibody_scan"],
        },
        ToolDoc {
            name: "m1nd_flow_simulate",
            category: "Superpowers",
            glyph: GLYPH_PATH,
            one_liner: "Fluid dynamics -- simulate data flow and detect bottlenecks",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Flow paths, bottlenecks, race condition candidates",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["epidemic", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_epidemic",
            category: "Superpowers",
            glyph: GLYPH_CONNECTION,
            one_liner: "SIR model -- predict how bugs spread through the codebase",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Infection spread prediction, SIR curves",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["antibody_scan", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_tremor",
            category: "Superpowers",
            glyph: GLYPH_DIMENSION,
            one_liner: "Seismology -- detect change acceleration and volatility",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Tremor magnitude, frequency, affected regions",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["trust", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_trust",
            category: "Superpowers",
            glyph: GLYPH_DIMENSION,
            one_liner: "Actuarial trust scoring -- per-node defect risk assessment",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Trust scores per node with defect history",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["tremor", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_layers",
            category: "Superpowers",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Detect architectural layers and violations",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Detected layers, layer assignments, violations",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["layer_inspect", "panoramic"],
        },
        ToolDoc {
            name: "m1nd_layer_inspect",
            category: "Superpowers",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Inspect a specific architectural layer",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("layer", "Layer name to inspect", true),
            ],
            returns: "Layer members, statistics, violations",
            example: r#"{"layer": "api", "agent_id": "jimi"}"#,
            next: &["layers", "impact"],
        },
        // --- Surgical ---
        ToolDoc {
            name: "m1nd_surgical_context",
            category: "Surgical",
            glyph: GLYPH_CONNECTION,
            one_liner: "Targeted code context extraction -- read only what matters",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("query", "What you need context for", true),
            ],
            returns: "Relevant code snippets with provenance",
            example: r#"{"query": "session pool initialization", "agent_id": "jimi"}"#,
            next: &["apply", "apply_batch"],
        },
        ToolDoc {
            name: "m1nd_surgical_context_v2",
            category: "Surgical",
            glyph: GLYPH_CONNECTION,
            one_liner: "Enhanced context extraction with dependency resolution",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("query", "What you need context for", true),
            ],
            returns: "Code context with imports, dependencies, and types resolved",
            example: r#"{"query": "chat handler message routing", "agent_id": "jimi"}"#,
            next: &["apply", "apply_batch"],
        },
        ToolDoc {
            name: "m1nd_apply",
            category: "Surgical",
            glyph: GLYPH_CONNECTION,
            one_liner: "Apply a code change and update the graph",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("file", "Target file path", true),
                ("content", "New content", true),
            ],
            returns: "Apply result with graph update status",
            example: r#"{"file": "backend/auth.py", "content": "...", "agent_id": "jimi"}"#,
            next: &["predict", "learn"],
        },
        ToolDoc {
            name: "m1nd_apply_batch",
            category: "Surgical",
            glyph: GLYPH_CONNECTION,
            one_liner: "Apply multiple code changes atomically",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("edits", "Array of file edits", true),
            ],
            returns: "Batch result with per-file status",
            example: r#"{"edits": [{"file": "a.py", "content": "..."}], "agent_id": "jimi"}"#,
            next: &["predict", "learn"],
        },
        // --- Perspective ---
        ToolDoc {
            name: "m1nd_perspective_start",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Enter a perspective -- create a navigable route surface from a query",
            params: &[
                ("query", "Seed query for route synthesis", true),
                ("agent_id", "Calling agent identifier", true),
                (
                    "anchor_node",
                    "Anchor to a specific node (activates anchored mode)",
                    false,
                ),
                ("lens", "Starting lens configuration", false),
            ],
            returns: "Perspective ID, initial route set, focus node",
            example: r#"{"query": "authentication flow", "agent_id": "jimi"}"#,
            next: &[
                "perspective_routes",
                "perspective_follow",
                "perspective_suggest",
            ],
        },
        ToolDoc {
            name: "m1nd_perspective_routes",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Browse the current route set with pagination",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("page", "Page number, 1-based (default: 1)", false),
                (
                    "page_size",
                    "Routes per page, clamped 1-10 (default: 6)",
                    false,
                ),
                (
                    "route_set_version",
                    "Version from previous response for staleness check",
                    false,
                ),
            ],
            returns: "Paginated routes with labels, scores, and route IDs",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123"}"#,
            next: &[
                "perspective_inspect",
                "perspective_follow",
                "perspective_peek",
            ],
        },
        ToolDoc {
            name: "m1nd_perspective_inspect",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Expand a route with full path, metrics, provenance, and affinity",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("route_set_version", "Version for staleness check", true),
                ("route_id", "Stable content-addressed route ID", false),
                ("route_index", "1-based page-local position", false),
            ],
            returns: "Full route path with edge weights, provenance, affinity scores",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123", "route_id": "r-def456", "route_set_version": 1}"#,
            next: &[
                "perspective_follow",
                "perspective_peek",
                "perspective_affinity",
            ],
        },
        ToolDoc {
            name: "m1nd_perspective_peek",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Extract a small relevant code/doc slice from a route target",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("route_set_version", "Version for staleness check", true),
                ("route_id", "Stable route ID", false),
                ("route_index", "1-based page-local position", false),
            ],
            returns: "Code snippet from target node with line numbers",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123", "route_id": "r-def456", "route_set_version": 1}"#,
            next: &["perspective_follow", "perspective_inspect"],
        },
        ToolDoc {
            name: "m1nd_perspective_follow",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Follow a route -- move focus to target, synthesize new routes",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("route_set_version", "Version for staleness check", true),
                ("route_id", "Stable route ID", false),
                ("route_index", "1-based page-local position", false),
            ],
            returns: "New focus node, new route set, navigation depth",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123", "route_id": "r-def456", "route_set_version": 1}"#,
            next: &[
                "perspective_routes",
                "perspective_back",
                "perspective_suggest",
            ],
        },
        ToolDoc {
            name: "m1nd_perspective_suggest",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Get next best move suggestion based on navigation history",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("route_set_version", "Version for staleness check", true),
            ],
            returns: "Suggested route with reasoning based on coverage gaps",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123", "route_set_version": 1}"#,
            next: &["perspective_follow", "perspective_inspect"],
        },
        ToolDoc {
            name: "m1nd_perspective_affinity",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Discover probable connections a route target might have",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("route_set_version", "Version for staleness check", true),
                ("route_id", "Stable route ID", false),
                ("route_index", "1-based page-local position", false),
            ],
            returns: "Affinity edges with strength scores and shared dimensions",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123", "route_id": "r-def456", "route_set_version": 1}"#,
            next: &["perspective_follow", "perspective_inspect"],
        },
        ToolDoc {
            name: "m1nd_perspective_branch",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Fork the current navigation state into a new branch",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
                ("branch_name", "Optional name for the branch", false),
            ],
            returns: "New perspective ID for the branch, inherited state",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123", "branch_name": "auth-deep-dive"}"#,
            next: &["perspective_routes", "perspective_compare"],
        },
        ToolDoc {
            name: "m1nd_perspective_back",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Navigate back to previous focus, restoring checkpoint state",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Active perspective ID", true),
            ],
            returns: "Restored focus node, previous route set",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123"}"#,
            next: &["perspective_routes", "perspective_follow"],
        },
        ToolDoc {
            name: "m1nd_perspective_compare",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Compare two perspectives on shared/unique nodes and dimension deltas",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id_a", "First perspective ID", true),
                ("perspective_id_b", "Second perspective ID", true),
                ("dimensions", "Dimensions to compare (empty = all)", false),
            ],
            returns: "Shared nodes, unique nodes per perspective, dimension deltas",
            example: r#"{"agent_id": "jimi", "perspective_id_a": "p-abc123", "perspective_id_b": "p-def456"}"#,
            next: &["perspective_inspect", "perspective_branch"],
        },
        ToolDoc {
            name: "m1nd_perspective_list",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "List all perspectives for an agent",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Active perspectives with focus nodes, route counts, depth",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["perspective_start", "perspective_close"],
        },
        ToolDoc {
            name: "m1nd_perspective_close",
            category: "Perspective",
            glyph: GLYPH_PATH,
            one_liner: "Close a perspective and release associated locks",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("perspective_id", "Perspective ID to close", true),
            ],
            returns: "Confirmation with released resource count",
            example: r#"{"agent_id": "jimi", "perspective_id": "p-abc123"}"#,
            next: &["perspective_list", "perspective_start"],
        },
        // --- Lock ---
        ToolDoc {
            name: "m1nd_lock_create",
            category: "Lock",
            glyph: GLYPH_CONNECTION,
            one_liner: "Pin a subgraph region and capture a baseline for change monitoring",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                (
                    "scope",
                    "Lock scope: node | subgraph | query_neighborhood | path",
                    true,
                ),
                ("root_nodes", "Root node IDs for the lock", true),
                ("radius", "BFS radius for subgraph scope (1-4)", false),
                ("query", "Query for query_neighborhood scope", false),
                ("path_nodes", "Ordered nodes for path scope", false),
            ],
            returns: "Lock ID, baseline snapshot, locked node count",
            example: r#"{"agent_id": "jimi", "scope": "subgraph", "root_nodes": ["file::auth.py"], "radius": 2}"#,
            next: &["lock_watch", "lock_diff", "lock_release"],
        },
        ToolDoc {
            name: "m1nd_lock_watch",
            category: "Lock",
            glyph: GLYPH_CONNECTION,
            one_liner: "Set a watcher strategy on a lock (manual, on_ingest, on_learn)",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("lock_id", "Lock to configure", true),
                (
                    "strategy",
                    "Watcher strategy: manual | on_ingest | on_learn",
                    true,
                ),
            ],
            returns: "Updated lock with active watcher strategy",
            example: r#"{"agent_id": "jimi", "lock_id": "lk-abc123", "strategy": "on_ingest"}"#,
            next: &["lock_diff", "lock_release"],
        },
        ToolDoc {
            name: "m1nd_lock_diff",
            category: "Lock",
            glyph: GLYPH_CONNECTION,
            one_liner: "Compute what changed in a locked region since baseline",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("lock_id", "Lock to diff", true),
            ],
            returns: "Added/removed/modified nodes and edges since baseline",
            example: r#"{"agent_id": "jimi", "lock_id": "lk-abc123"}"#,
            next: &["lock_rebase", "lock_release", "impact"],
        },
        ToolDoc {
            name: "m1nd_lock_rebase",
            category: "Lock",
            glyph: GLYPH_CONNECTION,
            one_liner: "Re-capture lock baseline from current graph without releasing",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("lock_id", "Lock to rebase", true),
            ],
            returns: "Updated baseline with new snapshot timestamp",
            example: r#"{"agent_id": "jimi", "lock_id": "lk-abc123"}"#,
            next: &["lock_diff", "lock_watch"],
        },
        ToolDoc {
            name: "m1nd_lock_release",
            category: "Lock",
            glyph: GLYPH_CONNECTION,
            one_liner: "Release a lock and free its resources",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("lock_id", "Lock to release", true),
            ],
            returns: "Confirmation with lock duration and change summary",
            example: r#"{"agent_id": "jimi", "lock_id": "lk-abc123"}"#,
            next: &["lock_create", "perspective_start"],
        },
        // --- Temporal Intelligence ---
        ToolDoc {
            name: "m1nd_timeline",
            category: "Temporal",
            glyph: GLYPH_DIMENSION,
            one_liner: "Git-based temporal history -- changes, co-changes, velocity, stability",
            params: &[
                (
                    "node",
                    "Node external_id (e.g. file::backend/chat_handler.py)",
                    true,
                ),
                ("agent_id", "Calling agent identifier", true),
                (
                    "depth",
                    "Time depth: 7d, 30d, 90d, all (default: 30d)",
                    false,
                ),
                (
                    "include_co_changes",
                    "Include co-changed files with coupling scores (default: true)",
                    false,
                ),
                (
                    "include_churn",
                    "Include lines added/deleted churn data (default: true)",
                    false,
                ),
                (
                    "top_k",
                    "Max co-change partners to return (default: 10)",
                    false,
                ),
            ],
            returns: "Commit history, co-change coupling matrix, churn velocity, stability score",
            example: r#"{"node": "file::backend/chat_handler.py", "agent_id": "jimi", "depth": "30d"}"#,
            next: &["diverge", "predict", "impact"],
        },
        ToolDoc {
            name: "m1nd_diverge",
            category: "Temporal",
            glyph: GLYPH_DIMENSION,
            one_liner: "Detect structural drift between a baseline and current graph state",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                (
                    "baseline",
                    "Baseline reference: ISO date, git ref, or last_session",
                    true,
                ),
                ("scope", "File path glob to limit scope", false),
                (
                    "include_coupling_changes",
                    "Include coupling matrix delta (default: true)",
                    false,
                ),
                (
                    "include_anomalies",
                    "Detect anomalies like test deficits, velocity spikes (default: true)",
                    false,
                ),
            ],
            returns: "Structural deltas, coupling drift, anomalies with severity",
            example: r#"{"agent_id": "jimi", "baseline": "last_session"}"#,
            next: &["timeline", "drift", "impact"],
        },
        // --- Investigation Memory ---
        ToolDoc {
            name: "m1nd_trail_save",
            category: "Trail",
            glyph: GLYPH_PATH,
            one_liner: "Persist current investigation state -- nodes, hypotheses, conclusions",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("label", "Human-readable label for this investigation", true),
                (
                    "hypotheses",
                    "Hypotheses formed during investigation",
                    false,
                ),
                ("conclusions", "Conclusions reached", false),
                ("open_questions", "Open questions remaining", false),
                ("tags", "Tags for organization and search", false),
                ("summary", "Summary (auto-generated if omitted)", false),
                ("visited_nodes", "Visited nodes with annotations", false),
                (
                    "activation_boosts",
                    "Map of node_id -> boost weight [0.0, 1.0]",
                    false,
                ),
            ],
            returns: "Trail ID, persisted node count, saved timestamp",
            example: r#"{"agent_id": "jimi", "label": "auth flow investigation", "tags": ["security", "auth"]}"#,
            next: &["trail_resume", "trail_list", "trail_merge"],
        },
        ToolDoc {
            name: "m1nd_trail_resume",
            category: "Trail",
            glyph: GLYPH_PATH,
            one_liner:
                "Restore a saved investigation -- re-inject activation boosts, detect staleness",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("trail_id", "Trail ID to resume", true),
                (
                    "force",
                    "Resume even if trail is stale (>50% missing nodes) (default: false)",
                    false,
                ),
            ],
            returns: "Restored state with staleness report, re-boosted nodes",
            example: r#"{"agent_id": "jimi", "trail_id": "trail-abc123"}"#,
            next: &["activate", "trail_save", "perspective_start"],
        },
        ToolDoc {
            name: "m1nd_trail_merge",
            category: "Trail",
            glyph: GLYPH_PATH,
            one_liner: "Combine two or more investigation trails -- discover cross-connections",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("trail_ids", "Two or more trail IDs to merge", true),
                (
                    "label",
                    "Label for the merged trail (auto-generated if omitted)",
                    false,
                ),
            ],
            returns: "Merged trail with cross-connections, shared nodes, combined hypotheses",
            example: r#"{"agent_id": "jimi", "trail_ids": ["trail-abc", "trail-def"]}"#,
            next: &["trail_resume", "trail_list"],
        },
        ToolDoc {
            name: "m1nd_trail_list",
            category: "Trail",
            glyph: GLYPH_PATH,
            one_liner: "List saved investigation trails with optional filters",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                (
                    "filter_agent_id",
                    "Filter to a specific agent's trails",
                    false,
                ),
                (
                    "filter_status",
                    "Filter by status: active, saved, archived, stale, merged",
                    false,
                ),
                ("filter_tags", "Filter by tags (any match)", false),
            ],
            returns: "Trails with labels, timestamps, node counts, status",
            example: r#"{"agent_id": "jimi", "filter_status": "active"}"#,
            next: &["trail_resume", "trail_save"],
        },
        // --- v0.4.0 new tools ---
        ToolDoc {
            name: "m1nd_panoramic",
            category: "Panoramic",
            glyph: GLYPH_STRUCTURE,
            one_liner: "Full graph health scan -- per-module risk from 7 signals",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                ("scope", "File path prefix filter", false),
                ("top_n", "Max modules to return (default: 50)", false),
            ],
            returns: "Per-module risk scores, critical alerts, overall health",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["impact", "antibody_scan"],
        },
        ToolDoc {
            name: "m1nd_savings",
            category: "Efficiency",
            glyph: GLYPH_DIMENSION,
            one_liner: "Token economy -- how much m1nd saved vs grep/Read",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Session + global token/cost savings",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["report", "help"],
        },
        ToolDoc {
            name: "m1nd_report",
            category: "Report",
            glyph: GLYPH_DIMENSION,
            one_liner: "Session intelligence report -- queries, savings, graph evolution",
            params: &[("agent_id", "Calling agent identifier", true)],
            returns: "Markdown report with query log, savings, graph evolution",
            example: r#"{"agent_id": "jimi"}"#,
            next: &["savings", "trail_save"],
        },
        ToolDoc {
            name: "m1nd_help",
            category: "Help",
            glyph: GLYPH_DIMENSION,
            one_liner: "Self-documenting tool reference with visual identity",
            params: &[
                ("agent_id", "Calling agent identifier", true),
                (
                    "tool_name",
                    "Specific tool name (omit for full index)",
                    false,
                ),
            ],
            returns: "Formatted help text with params, examples, and NEXT suggestions",
            example: r#"{"agent_id": "jimi", "tool_name": "activate"}"#,
            next: &["activate", "ingest"],
        },
    ]
}

/// Format the full help index.
pub fn format_help_index() -> String {
    let docs = tool_docs();
    let width = 60;
    let mut out = String::new();

    out.push_str(&gradient_top_border(width));
    out.push('\n');
    out.push_str(&format!(
        "{}{}  m1nd  {}-- neuro-symbolic code graph engine{}\n",
        ANSI_BOLD, ANSI_CYAN, ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!(
        "{}  {} SIGNAL  {} PATH  {} STRUCTURE  {} DIMENSION  {} CONNECTION{}\n",
        ANSI_DIM,
        GLYPH_SIGNAL,
        GLYPH_PATH,
        GLYPH_STRUCTURE,
        GLYPH_DIMENSION,
        GLYPH_CONNECTION,
        ANSI_RESET
    ));
    out.push_str(&gradient_bottom_border(width));
    out.push('\n');
    out.push('\n');

    // Group by category
    let categories = [
        ("Foundation", GLYPH_SIGNAL, ANSI_CYAN),
        ("Perspective", GLYPH_PATH, ANSI_MAGENTA),
        ("Lock", GLYPH_CONNECTION, ANSI_BLUE),
        ("Temporal", GLYPH_DIMENSION, ANSI_GOLD),
        ("Trail", GLYPH_PATH, ANSI_GOLD),
        ("Superpowers", GLYPH_STRUCTURE, ANSI_GOLD),
        ("Extended", GLYPH_DIMENSION, ANSI_MAGENTA),
        ("Surgical", GLYPH_CONNECTION, ANSI_GREEN),
        ("Panoramic", GLYPH_STRUCTURE, ANSI_RED),
        ("Efficiency", GLYPH_DIMENSION, ANSI_GREEN),
        ("Report", GLYPH_DIMENSION, ANSI_BLUE),
        ("Help", GLYPH_DIMENSION, ANSI_CYAN),
    ];

    for (cat_name, glyph, color) in &categories {
        let cat_tools: Vec<&ToolDoc> = docs.iter().filter(|d| d.category == *cat_name).collect();
        if cat_tools.is_empty() {
            continue;
        }

        out.push_str(&format!(
            "{}{} {} ({}):{}\n",
            color,
            glyph,
            cat_name,
            cat_tools.len(),
            ANSI_RESET
        ));
        for doc in cat_tools {
            let short_name = doc.name.strip_prefix("m1nd_").unwrap_or(doc.name);
            out.push_str(&format!(
                "  {}{}  {}{}{}\n",
                ANSI_BOLD, short_name, ANSI_DIM, doc.one_liner, ANSI_RESET
            ));
        }
        out.push('\n');
    }

    out.push_str(&format!(
        "{}use help(tool_name=\"activate\") for detailed help on any tool{}\n",
        ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!("{}tip: if you're unsure which tool to use, describe what you need — m1nd.help can suggest the right one.{}\n", ANSI_DIM, ANSI_RESET));
    out
}

/// Format detailed help for a single tool.
pub fn format_tool_help(doc: &ToolDoc) -> String {
    let width = 60;
    let mut out = String::new();

    out.push_str(&gradient_top_border(width));
    out.push('\n');

    let short_name = doc.name.strip_prefix("m1nd_").unwrap_or(doc.name);
    out.push_str(&format!(
        "{}{} {}  {}  m1nd.{}{}\n",
        ANSI_CYAN,
        doc.glyph,
        doc.category.to_uppercase(),
        ANSI_BOLD,
        short_name,
        ANSI_RESET
    ));
    out.push_str(&format!("{}{}{}\n\n", ANSI_DIM, doc.one_liner, ANSI_RESET));

    // Params section
    out.push_str(&format!("{}\u{2338} PARAMS{}\n", ANSI_GOLD, ANSI_RESET)); // ⌸
    for (i, (name, desc, required)) in doc.params.iter().enumerate() {
        let connector = if i == doc.params.len() - 1 {
            "\u{2514}\u{2500}"
        } else {
            "\u{251C}\u{2500}"
        };
        let req_mark = if *required {
            format!("{}*{}", ANSI_RED, ANSI_RESET)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  {} {}{}{} {}{}{}\n",
            connector, ANSI_BOLD, name, req_mark, ANSI_DIM, desc, ANSI_RESET
        ));
    }
    out.push('\n');

    // Returns section
    out.push_str(&format!("{}\u{234D} RETURNS{}\n", ANSI_GREEN, ANSI_RESET)); // ⍍
    out.push_str(&format!("  {}{}{}\n\n", ANSI_DIM, doc.returns, ANSI_RESET));

    // Example section
    out.push_str(&format!("{}\u{233C} EXAMPLE{}\n", ANSI_MAGENTA, ANSI_RESET)); // ⌼
    out.push_str(&format!("  {}{}{}\n\n", ANSI_DIM, doc.example, ANSI_RESET));

    // Next section
    out.push_str(&format!("{}\u{2350} NEXT{}\n", ANSI_CYAN, ANSI_RESET)); // ⍐
    for next in doc.next {
        out.push_str(&format!(
            "  {} {}{}{}\n",
            ANSI_CYAN, ANSI_BOLD, next, ANSI_RESET
        ));
    }
    out.push('\n');

    out.push_str(&gradient_bottom_border(width));
    out.push('\n');
    out
}

/// Format the "about" help -- m1nd's philosophy and identity.
pub fn format_about() -> String {
    let width = 60;
    let mut out = String::new();

    out.push_str(&gradient_top_border(width));
    out.push('\n');
    out.push_str(&format!("{}{}  m1nd{}\n", ANSI_BOLD, ANSI_CYAN, ANSI_RESET));
    out.push_str(&format!(
        "{}  neuro-symbolic code graph engine{}\n\n",
        ANSI_DIM, ANSI_RESET
    ));

    out.push_str(&format!(
        "{}  created by Max Kleinschmidt{}\n",
        ANSI_GREEN, ANSI_RESET
    ));
    out.push_str(&format!(
        "{}  cosmophonix / ROOMANIZER OS{}\n\n",
        ANSI_DIM, ANSI_RESET
    ));

    out.push_str(&format!(
        "{}  4 letters = 4 dimensions:{}\n",
        ANSI_BOLD, ANSI_RESET
    ));
    out.push_str(&format!(
        "  {}M{} = {}STRUCTURAL{} (who calls who)\n",
        ANSI_BLUE, ANSI_RESET, ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!(
        "  {}1{} = {}TEMPORAL{} (what changed together)\n",
        ANSI_GOLD, ANSI_RESET, ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!(
        "  {}N{} = {}CAUSAL{} (what broke when this changed)\n",
        ANSI_MAGENTA, ANSI_RESET, ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!(
        "  {}D{} = {}SEMANTIC{} (naming patterns)\n\n",
        ANSI_BLUE, ANSI_RESET, ANSI_DIM, ANSI_RESET
    ));

    out.push_str(&format!(
        "{}  12 disciplines from neuroscience to epidemiology.{}\n",
        ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!(
        "{}  zero tokens burned. zero API cost. all local Rust.{}\n",
        ANSI_DIM, ANSI_RESET
    ));
    out.push_str(&format!(
        "{}  every query makes the graph smarter.{}\n\n",
        ANSI_DIM, ANSI_RESET
    ));

    out.push_str(&format!(
        "{}  {} SIGNAL  {} PATH  {} STRUCTURE  {} DIMENSION  {} CONNECTION{}\n",
        ANSI_DIM,
        GLYPH_SIGNAL,
        GLYPH_PATH,
        GLYPH_STRUCTURE,
        GLYPH_DIMENSION,
        GLYPH_CONNECTION,
        ANSI_RESET
    ));

    out.push_str(&gradient_bottom_border(width));
    out.push('\n');
    out
}

/// Find the closest matching tool name for "did you mean?" suggestions.
pub fn find_similar_tools(query: &str) -> Vec<String> {
    let docs = tool_docs();
    let query_lower = query.to_lowercase();
    let query_lower = query_lower.strip_prefix("m1nd_").unwrap_or(&query_lower);

    let mut matches: Vec<(&str, usize)> = docs
        .iter()
        .map(|d| {
            let name = d.name.strip_prefix("m1nd_").unwrap_or(d.name);
            let dist = levenshtein_distance(query_lower, &name.to_lowercase());
            (d.name, dist)
        })
        .filter(|(_, dist)| *dist <= 4)
        .collect();

    matches.sort_by_key(|(_, dist)| *dist);
    matches
        .into_iter()
        .take(3)
        .map(|(name, _)| name.to_string())
        .collect()
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let a_len = a_bytes.len();
    let b_len = b_bytes.len();
    let mut dp = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in dp.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(b_len + 1) {
        *val = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[a_len][b_len]
}
