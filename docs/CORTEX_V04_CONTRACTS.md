# m1nd v0.4.0 Contracts -- FORGE-CONTRACTS

## Status: COMPLETE
Agent: `contracts-v04` | Date: 2026-03-15

---

## Overview

6 new tools for v0.4.0: `search`, `help`, `report`, `panoramic`, `savings`, plus `perspective.routes` fix.
All contracts follow existing conventions from `protocol/core.rs`, `protocol/surgical.rs`, `protocol/layers.rs`:

- Input: `#[derive(Clone, Debug, Deserialize)]`
- Output: `#[derive(Clone, Debug, Serialize)]`
- All inputs require `agent_id: String`
- Optional params use `Option<T>` or `#[serde(default = "fn_name")]`
- Doc comments reference PRD section

---

## 1. m1nd.search

Full-text + graph-aware code search. Unlike `seek` (intent-based semantic search), `search` is a
lower-level tool that supports literal, regex, and semantic search modes with context lines around
matches -- closer to grep but enriched with graph node references.

### Rust Structs

```rust
// === protocol/v04.rs (or add to protocol/layers.rs) ===

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// m1nd.search
// ---------------------------------------------------------------------------

/// Search mode: how the query string is interpreted.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// Exact substring match (case-insensitive by default).
    Literal,
    /// Regular expression (Rust regex crate syntax).
    Regex,
    /// Semantic: tokenize query, score nodes by activation + trigram similarity.
    /// Falls back to Literal if graph is empty.
    Semantic,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Literal
    }
}

/// Input for m1nd.search.
///
/// Low-level search across ingested codebase. Returns file-level matches
/// with optional context lines and graph node cross-references.
#[derive(Clone, Debug, Deserialize)]
pub struct SearchInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Search query string. Interpretation depends on `mode`.
    pub query: String,
    /// How to interpret the query. Default: "literal".
    #[serde(default)]
    pub mode: SearchMode,
    /// Optional file path prefix filter (e.g. "backend/core/" to scope search).
    #[serde(default)]
    pub scope: Option<String>,
    /// Maximum results to return. Default: 50. Clamped to 1..=500.
    #[serde(default = "default_search_top_k")]
    pub top_k: usize,
    /// Lines of context before and after each match. Default: 2. Clamped to 0..=10.
    #[serde(default = "default_context_lines")]
    pub context_lines: u32,
    /// Case-sensitive matching (literal/regex modes only). Default: false.
    #[serde(default)]
    pub case_sensitive: bool,
}

fn default_search_top_k() -> usize { 50 }
fn default_context_lines() -> u32 { 2 }

/// A single search match result.
#[derive(Clone, Debug, Serialize)]
pub struct SearchMatch {
    /// Absolute or workspace-relative file path where the match was found.
    pub file_path: String,
    /// 1-based line number of the match.
    pub line_number: u32,
    /// Content of the matching line.
    pub line_content: String,
    /// Lines before the match (up to `context_lines`).
    pub context_before: Vec<String>,
    /// Lines after the match (up to `context_lines`).
    pub context_after: Vec<String>,
    /// Graph node ID if this file/symbol is in the graph. None if not ingested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Relevance score: 1.0 for exact matches, 0.0-1.0 for semantic.
    pub match_score: f64,
}

/// Output for m1nd.search.
#[derive(Clone, Debug, Serialize)]
pub struct SearchOutput {
    /// Original query echoed back.
    pub query: String,
    /// Search mode used.
    pub mode: String,
    /// Matching results, ordered by match_score descending.
    pub matches: Vec<SearchMatch>,
    /// Total matches found (may exceed top_k).
    pub total_matches: usize,
    /// True when total_matches > top_k (results were truncated).
    pub truncated: bool,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}
```

### JSON Schema (MCP Registration)

```json
{
    "name": "m1nd.search",
    "description": "Low-level code search: literal, regex, or semantic. Returns file matches with context lines and graph node cross-references.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" },
            "query": { "type": "string", "description": "Search query string" },
            "mode": {
                "type": "string",
                "enum": ["literal", "regex", "semantic"],
                "default": "literal",
                "description": "Search mode: literal (substring), regex (pattern), semantic (graph-aware)"
            },
            "scope": { "type": "string", "description": "File path prefix filter" },
            "top_k": { "type": "integer", "default": 50, "description": "Max results (1-500)" },
            "context_lines": { "type": "integer", "default": 2, "description": "Lines of context before/after match (0-10)" },
            "case_sensitive": { "type": "boolean", "default": false, "description": "Case-sensitive matching" }
        },
        "required": ["agent_id", "query"]
    }
}
```

### Dispatch Match Arm

```rust
"m1nd.search" => {
    let input: v04::SearchInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_search(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
```

### Error Variants

No new error variants needed. Uses existing:
- `M1ndError::InvalidParams` -- invalid regex pattern, top_k=0, etc.
- `M1ndError::EmptyGraph` -- semantic mode with no graph (falls back to literal)
- `M1ndError::Serde` -- malformed input

---

## 2. m1nd.help

Returns a formatted help string. No JSON struct output -- just a String.

### Rust Structs

```rust
// ---------------------------------------------------------------------------
// m1nd.help
// ---------------------------------------------------------------------------

/// Input for m1nd.help.
///
/// Returns formatted help text for m1nd tools. When `tool` is specified,
/// returns detailed help for that tool. Otherwise returns overview.
#[derive(Clone, Debug, Deserialize)]
pub struct HelpInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Optional: specific tool name to get detailed help for (e.g. "activate", "search").
    /// Omit for overview of all tools.
    #[serde(default)]
    pub tool: Option<String>,
    /// Output format. Default: "text". Options: "text", "markdown", "json".
    #[serde(default = "default_help_format")]
    pub format: String,
}

fn default_help_format() -> String { "text".into() }
```

Output is `serde_json::Value::String(...)` -- no output struct needed.
The handler builds a formatted string from the tool registry.

### JSON Schema

```json
{
    "name": "m1nd.help",
    "description": "Get help text for m1nd tools. Returns overview or detailed help for a specific tool.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" },
            "tool": { "type": "string", "description": "Specific tool name for detailed help (omit for overview)" },
            "format": {
                "type": "string",
                "enum": ["text", "markdown", "json"],
                "default": "text",
                "description": "Output format"
            }
        },
        "required": ["agent_id"]
    }
}
```

### Dispatch Match Arm

```rust
"m1nd.help" => {
    let input: v04::HelpInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let text = layer_handlers::handle_help(state, input)?;
    Ok(serde_json::Value::String(text))
}
```

---

## 3. m1nd.report

Session intelligence report: what was queried, what was found, graph evolution, and savings.

### Rust Structs

```rust
// ---------------------------------------------------------------------------
// m1nd.report
// ---------------------------------------------------------------------------

/// Input for m1nd.report.
///
/// Generates a session intelligence report covering queries made,
/// bugs found, graph evolution, and estimated savings.
#[derive(Clone, Debug, Deserialize)]
pub struct ReportInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Output format. Default: "markdown".
    #[serde(default = "default_report_format")]
    pub format: Option<String>,
}

fn default_report_format() -> Option<String> { Some("markdown".into()) }

/// Top-level output for m1nd.report.
#[derive(Clone, Debug, Serialize)]
pub struct ReportOutput {
    /// Session summary: uptime, queries, graph size.
    pub session_summary: SessionSummary,
    /// Record of queries made during this session.
    pub queries_made: Vec<QueryRecord>,
    /// Bugs or anomalies detected by antibody/tremor systems.
    pub bugs_found: Vec<BugRecord>,
    /// Graph evolution metrics since session start.
    pub graph_evolution: GraphEvolution,
    /// Estimated token/cost savings from using m1nd.
    pub savings: SavingsEstimate,
    /// Formatted report text (in requested format).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}

/// Session summary section of the report.
#[derive(Clone, Debug, Serialize)]
pub struct SessionSummary {
    /// Server uptime in seconds.
    pub uptime_seconds: f64,
    /// Total queries processed this session.
    pub total_queries: u64,
    /// Number of unique agents seen.
    pub unique_agents: usize,
    /// Current graph node count.
    pub node_count: u32,
    /// Current graph edge count.
    pub edge_count: u64,
    /// Number of active perspectives.
    pub active_perspectives: usize,
    /// Number of active locks.
    pub active_locks: usize,
}

/// A recorded query for the report.
#[derive(Clone, Debug, Serialize)]
pub struct QueryRecord {
    /// Tool name that was called.
    pub tool: String,
    /// Agent that made the call.
    pub agent_id: String,
    /// Unix timestamp (ms) of the call.
    pub timestamp_ms: u64,
    /// Elapsed time for this query in ms.
    pub elapsed_ms: f64,
    /// Number of results returned.
    pub result_count: usize,
    /// Brief summary of the query (first 100 chars of query/input).
    pub summary: String,
}

/// A bug/anomaly record for the report.
#[derive(Clone, Debug, Serialize)]
pub struct BugRecord {
    /// Bug category: "antibody_match", "tremor_spike", "layer_violation", "trust_deficit".
    pub category: String,
    /// Affected node ID.
    pub node_id: String,
    /// Human-readable description.
    pub description: String,
    /// Severity: 0.0 (info) to 1.0 (critical).
    pub severity: f64,
    /// Unix timestamp (ms) when detected.
    pub detected_at_ms: u64,
}

/// Graph evolution metrics.
#[derive(Clone, Debug, Serialize)]
pub struct GraphEvolution {
    /// Nodes added during this session.
    pub nodes_added: u32,
    /// Nodes removed during this session.
    pub nodes_removed: u32,
    /// Edges added during this session.
    pub edges_added: u64,
    /// Edges removed during this session.
    pub edges_removed: u64,
    /// Number of ingestions performed.
    pub ingestions: u32,
    /// Number of learn/feedback events.
    pub learn_events: u32,
    /// Number of apply/write operations.
    pub apply_operations: u32,
    /// Graph generation at session start.
    pub generation_start: u64,
    /// Graph generation now.
    pub generation_current: u64,
}

/// Estimated savings from using m1nd vs. manual grep/read.
#[derive(Clone, Debug, Serialize)]
pub struct SavingsEstimate {
    /// Estimated tokens saved by using m1nd instead of reading full files.
    pub tokens_saved: u64,
    /// Estimated cost saved in USD (at ~$0.01/1K input tokens for Opus).
    pub cost_saved_usd: f64,
    /// Number of file reads avoided via graph queries.
    pub file_reads_avoided: u64,
    /// Total lines that would have been read without m1nd.
    pub lines_avoided: u64,
}
```

### JSON Schema

```json
{
    "name": "m1nd.report",
    "description": "Session intelligence report: queries, bugs, graph evolution, and estimated savings.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" },
            "format": {
                "type": "string",
                "enum": ["text", "json", "markdown"],
                "default": "markdown",
                "description": "Output format for the formatted report"
            }
        },
        "required": ["agent_id"]
    }
}
```

### Dispatch Match Arm

```rust
"m1nd.report" => {
    let input: v04::ReportInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_report(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
```

### Implementation Notes

The handler needs access to `SessionState` fields:
- `sessions` -- for agent count and query history
- `queries_processed` -- total count
- `start_time` -- uptime calculation
- `graph` -- node/edge counts
- `perspectives`, `locks` -- active counts
- New: `query_log: Vec<QueryRecord>` field on `SessionState` (capped at 1000 entries)
- New: `session_start_generation: u64` field on `SessionState` (set at boot)

---

## 4. m1nd.panoramic

Full-graph health overview: per-module risk scores combining trust, tremor, layer violations,
antibody matches, and turbulence.

### Rust Structs

```rust
// ---------------------------------------------------------------------------
// m1nd.panoramic
// ---------------------------------------------------------------------------

/// Input for m1nd.panoramic.
///
/// Generates a panoramic health overview of the entire graph or a scoped
/// subset. Combines trust, tremor, layer, antibody, and turbulence signals
/// into a per-module risk score.
#[derive(Clone, Debug, Deserialize)]
pub struct PanoramicInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Optional file path prefix to scope the analysis.
    #[serde(default)]
    pub scope: Option<String>,
    /// Include recommendation text per module. Default: true.
    #[serde(default = "default_true")]
    pub include_recommendations: bool,
    /// Minimum combined risk score to include a module. Default: 0.0 (include all).
    #[serde(default)]
    pub min_risk: f64,
    /// Maximum modules to return. Default: 100. Clamped to 1..=1000.
    #[serde(default = "default_panoramic_limit")]
    pub limit: usize,
}

fn default_true() -> bool { true }
fn default_panoramic_limit() -> usize { 100 }

/// Per-module risk assessment.
#[derive(Clone, Debug, Serialize)]
pub struct ModuleRisk {
    /// Graph node ID for this module.
    pub node_id: String,
    /// Human-readable label.
    pub label: String,
    /// Node type (File, Function, Class, Module).
    #[serde(rename = "type")]
    pub node_type: String,
    /// Trust score from TrustLedger (0.0 = untrusted, 1.0 = fully trusted).
    pub trust_score: f64,
    /// Tremor magnitude from TremorRegistry (0.0 = stable, higher = more volatile).
    pub tremor_magnitude: f64,
    /// Number of layer violations detected for this node.
    pub layer_violations: u32,
    /// Turbulence: rate of change in activation patterns (high = unstable).
    pub turbulence: f64,
    /// Number of antibody pattern matches (potential bugs).
    pub antibody_matches: u32,
    /// Combined risk score: weighted combination of all signals.
    /// Formula: (1-trust)*0.3 + tremor*0.25 + violations*0.2 + turbulence*0.15 + antibodies*0.1
    pub combined_risk: f64,
    /// Optional recommendation text (populated when include_recommendations=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
    /// Provenance: source file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

/// Alert: a critical-severity issue detected during panoramic scan.
#[derive(Clone, Debug, Serialize)]
pub struct PanoramicAlert {
    /// Alert category: "high_risk", "trust_deficit", "tremor_spike",
    /// "layer_violation_cluster", "antibody_critical".
    pub category: String,
    /// Affected node IDs.
    pub affected_nodes: Vec<String>,
    /// Human-readable description.
    pub description: String,
    /// Severity: 0.0-1.0.
    pub severity: f64,
}

/// Output for m1nd.panoramic.
#[derive(Clone, Debug, Serialize)]
pub struct PanoramicOutput {
    /// Per-module risk assessments, ordered by combined_risk descending.
    pub modules: Vec<ModuleRisk>,
    /// Overall graph health score (0.0 = critical, 1.0 = perfect).
    pub overall_health: f64,
    /// Critical alerts that need immediate attention.
    pub critical_alerts: Vec<PanoramicAlert>,
    /// Total modules scanned (before filtering by min_risk/limit).
    pub total_scanned: usize,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}
```

### JSON Schema

```json
{
    "name": "m1nd.panoramic",
    "description": "Panoramic graph health overview: per-module risk scores combining trust, tremor, layer violations, antibodies, and turbulence.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" },
            "scope": { "type": "string", "description": "File path prefix filter" },
            "include_recommendations": { "type": "boolean", "default": true, "description": "Include recommendation text per module" },
            "min_risk": { "type": "number", "default": 0.0, "description": "Minimum combined risk to include (0.0-1.0)" },
            "limit": { "type": "integer", "default": 100, "description": "Max modules to return (1-1000)" }
        },
        "required": ["agent_id"]
    }
}
```

### Dispatch Match Arm

```rust
"m1nd.panoramic" => {
    let input: v04::PanoramicInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_panoramic(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
```

### Implementation Notes

Reads from existing engines (no new state):
- `state.trust` (TrustLedger) -- `trust_score` per node
- `state.tremor_registry` (TremorRegistry) -- tremor magnitude
- `state.graph` -- layer detection via `state.topology`
- Antibody scan via `state.graph` antibody store

Combined risk formula:
```
combined_risk = (1.0 - trust_score) * 0.30
              + clamp(tremor_magnitude, 0.0, 1.0) * 0.25
              + clamp(layer_violations as f64 / 5.0, 0.0, 1.0) * 0.20
              + clamp(turbulence, 0.0, 1.0) * 0.15
              + clamp(antibody_matches as f64 / 3.0, 0.0, 1.0) * 0.10
```

Overall health: `1.0 - (mean of top 10% combined_risk scores)`

---

## 5. m1nd.savings

Token/cost savings estimator. Standalone tool (subset of report's savings).

### Rust Structs

```rust
// ---------------------------------------------------------------------------
// m1nd.savings
// ---------------------------------------------------------------------------

/// Input for m1nd.savings.
///
/// Returns estimated token and cost savings from using m1nd during this session.
#[derive(Clone, Debug, Deserialize)]
pub struct SavingsInput {
    /// Calling agent identifier.
    pub agent_id: String,
}

/// Per-session savings breakdown.
#[derive(Clone, Debug, Serialize)]
pub struct SessionSavings {
    /// Number of m1nd queries made this session.
    pub queries_made: u64,
    /// Estimated tokens saved by using m1nd instead of reading full files.
    pub tokens_saved: u64,
    /// Estimated cost saved in USD.
    pub cost_saved_usd: f64,
    /// Number of file reads avoided.
    pub file_reads_avoided: u64,
    /// Total lines that would have been read without m1nd.
    pub lines_avoided: u64,
    /// Session uptime in seconds.
    pub session_uptime_seconds: f64,
    /// Average tokens saved per query.
    pub avg_tokens_per_query: f64,
}

/// Global (all-time) savings estimate.
#[derive(Clone, Debug, Serialize)]
pub struct GlobalSavings {
    /// Total sessions tracked.
    pub total_sessions: u64,
    /// Total queries across all sessions.
    pub total_queries: u64,
    /// Total tokens saved across all sessions.
    pub total_tokens_saved: u64,
    /// Total cost saved in USD across all sessions.
    pub total_cost_saved_usd: f64,
    /// Total file reads avoided across all sessions.
    pub total_file_reads_avoided: u64,
}

/// Output for m1nd.savings.
#[derive(Clone, Debug, Serialize)]
pub struct SavingsOutput {
    /// Current session savings.
    pub current_session: SessionSavings,
    /// Global (all-time) savings. May be empty if no persistence.
    pub global: GlobalSavings,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}
```

### JSON Schema

```json
{
    "name": "m1nd.savings",
    "description": "Estimated token and cost savings from using m1nd. Shows current session and global totals.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" }
        },
        "required": ["agent_id"]
    }
}
```

### Dispatch Match Arm

```rust
"m1nd.savings" => {
    let input: v04::SavingsInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_savings(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
```

### Savings Estimation Algorithm

Per query, estimate tokens saved:
1. `activate` query: ~2000 tokens saved (avoids reading 5+ files)
2. `impact/predict/counterfactual`: ~3000 tokens (avoids reading blast radius)
3. `surgical_context`: ~1500 tokens (targeted vs full file reads)
4. `seek/scan`: ~1000 tokens (vs grep + read)
5. Other tools: ~500 tokens average

Cost: tokens_saved * $0.00001 (Opus input token price ~$10/1M)

### Implementation Notes

New `SessionState` fields needed:
- `savings_tracker: SavingsTracker` -- accumulates per-query savings
- Global savings: persisted to `~/.m1nd/savings.json` alongside graph snapshot

```rust
/// Savings tracker (lives on SessionState).
pub struct SavingsTracker {
    pub queries_by_tool: HashMap<String, u64>,
    pub tokens_saved: u64,
    pub file_reads_avoided: u64,
    pub lines_avoided: u64,
}

impl SavingsTracker {
    pub fn new() -> Self {
        Self {
            queries_by_tool: HashMap::new(),
            tokens_saved: 0,
            file_reads_avoided: 0,
            lines_avoided: 0,
        }
    }

    /// Call after every successful tool dispatch.
    pub fn record(&mut self, tool: &str, result_nodes: usize) {
        *self.queries_by_tool.entry(tool.to_string()).or_insert(0) += 1;
        let (tokens, files, lines) = match tool {
            "m1nd.activate" => (2000, 5, 500),
            "m1nd.impact" | "m1nd.predict" | "m1nd.counterfactual" => (3000, 8, 800),
            "m1nd.surgical.context" | "m1nd.surgical.context.v2" => (1500, 3, 300),
            "m1nd.seek" | "m1nd.scan" | "m1nd.search" => (1000, 4, 400),
            _ => (500, 2, 200),
        };
        self.tokens_saved += tokens;
        self.file_reads_avoided += files;
        self.lines_avoided += lines;
    }
}
```

---

## 6. perspective.routes Fix

The `perspective.routes` handler currently works but has a known issue: it does not
include `route_set_version` validation feedback when the client sends a stale version.
The fix is in the handler, not in the protocol types.

### Current Behavior (perspective_handlers.rs:392)

The handler calls `validate_route_set_version()` which returns `Err(RouteSetStale)` when
version mismatches. This is correct but the error message is generic.

### Fix: Enhanced Staleness Response

Instead of returning an error, return a success response with `stale: true` and the
current version, so the agent can retry without an error path.

```rust
// Add to PerspectiveRoutesOutput (in protocol/perspective.rs):

/// Output for m1nd.perspective.routes (enhanced).
#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveRoutesOutput {
    // ... existing fields ...

    /// True when the requested route_set_version was stale.
    /// When stale=true, routes are re-synthesized from current state.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stale: bool,

    /// The previous version that was requested (when stale=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_version: Option<u64>,
}
```

### Handler Fix (perspective_handlers.rs)

```rust
pub fn handle_perspective_routes(
    state: &mut SessionState,
    input: PerspectiveRoutesInput,
) -> M1ndResult<PerspectiveRoutesOutput> {
    let persp = require_perspective(state, &input.agent_id, &input.perspective_id)?;

    let mut stale = false;
    let mut requested_version: Option<u64> = None;

    // Instead of erroring on stale version, mark as stale and continue
    if let Some(input_version) = input.route_set_version {
        if let Some(cached) = &persp.route_cache {
            if input_version != cached.route_set_version {
                stale = true;
                requested_version = Some(input_version);
                // Continue with current version instead of erroring
            }
        }
    }

    // ... rest of handler unchanged, but include stale + requested_version in output ...
}
```

---

## New Error Variants

No new error variants needed for v0.4.0 tools. All errors map to existing variants:

| Error | Used By |
|-------|---------|
| `InvalidParams { tool, detail }` | search (bad regex), panoramic (invalid limit) |
| `EmptyGraph` | search (semantic fallback), panoramic, report |
| `Serde(e)` | All tools (malformed input) |

---

## New SessionState Fields

```rust
// Add to SessionState in session.rs:

/// Query log for report tool (ring buffer, capped at 1000).
pub query_log: Vec<v04::QueryRecord>,

/// Graph generation at session start (for report's graph_evolution).
pub session_start_generation: u64,
/// Node count at session start.
pub session_start_node_count: u32,
/// Edge count at session start.
pub session_start_edge_count: u64,

/// Savings tracker.
pub savings_tracker: v04::SavingsTracker,
```

---

## Protocol Module Update

```rust
// === protocol/mod.rs ===
pub mod core;
pub mod perspective;
pub mod lock;
pub mod layers;
pub mod surgical;
pub mod v04;  // NEW

pub use self::core::*;
```

---

## Summary of Dispatch Arms (server.rs)

Add to `dispatch_core_tool()` match:

```rust
"m1nd.search" => {
    let input: v04::SearchInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_search(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
"m1nd.help" => {
    let input: v04::HelpInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let text = layer_handlers::handle_help(state, input)?;
    Ok(serde_json::Value::String(text))
}
"m1nd.report" => {
    let input: v04::ReportInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_report(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
"m1nd.panoramic" => {
    let input: v04::PanoramicInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_panoramic(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
"m1nd.savings" => {
    let input: v04::SavingsInput = serde_json::from_value(params.clone())
        .map_err(M1ndError::Serde)?;
    let output = layer_handlers::handle_savings(state, input)?;
    serde_json::to_value(output).map_err(M1ndError::Serde)
}
```

---

## Handler Signatures (layer_handlers.rs)

```rust
pub fn handle_search(state: &mut SessionState, input: v04::SearchInput) -> M1ndResult<v04::SearchOutput>;
pub fn handle_help(state: &mut SessionState, input: v04::HelpInput) -> M1ndResult<String>;
pub fn handle_report(state: &mut SessionState, input: v04::ReportInput) -> M1ndResult<v04::ReportOutput>;
pub fn handle_panoramic(state: &mut SessionState, input: v04::PanoramicInput) -> M1ndResult<v04::PanoramicOutput>;
pub fn handle_savings(state: &mut SessionState, input: v04::SavingsInput) -> M1ndResult<v04::SavingsOutput>;
```

---

## Verification Checklist

- [ ] All structs have `#[derive(Clone, Debug, Serialize)]` (output) or `#[derive(Clone, Debug, Deserialize)]` (input)
- [ ] All inputs have `agent_id: String`
- [ ] All outputs have `elapsed_ms: f64` where applicable
- [ ] Optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`
- [ ] Default helpers match existing naming convention
- [ ] JSON schemas match Rust struct fields exactly
- [ ] Dispatch arms follow `parse -> handle -> serialize` pattern
- [ ] No new dependencies required (uses existing engines)
- [ ] perspective.routes fix is backward-compatible (new fields are skip_serializing_if)
