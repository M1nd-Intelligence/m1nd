// === m1nd-mcp/src/protocol/layers.rs ===
//
// Input/Output types for new MCP tools across Layers 1-7.
// From research reports L1-L7 and SYNTHESIS-7-LAYERS.
//
// Conventions (matching core.rs / perspective.rs / lock.rs):
//   - Input:  #[derive(Clone, Debug, Deserialize)]
//   - Output: #[derive(Clone, Debug, Serialize)]
//   - All inputs require `agent_id: String`
//   - Optional params with defaults use #[serde(default = "fn_name")]
//   - Optional Vec fields use #[serde(default)]
//   - Optional String fields use Option<String>
//   - Doc comments reference PRD layer + section

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// =========================================================================
// L1: Cross-File Edges — New Edge Types (no new MCP tools, ingest-only)
// =========================================================================

/// New edge relation types for cross-file edges (L1-CROSS-FILE-EDGES §9).
/// Added to the relation field of ExtractedEdge during ingest.
/// Not MCP protocol types — included here for completeness.
///
/// Values used in CSR `relations` field via StringInterner:
///   "imports"     — file A imports module from file B
///   "calls"       — function in A calls function in B
///   "registers"   — A registers B as a route/plugin (e.g. include_router)
///   "configures"  — A reads config key defined/set in B
///   "tests"       — test file A tests module B
///   "inherits"    — class in A inherits from class in B
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CrossFileEdgeType {
    Imports,
    Calls,
    Registers,
    Configures,
    Tests,
    Inherits,
}

// =========================================================================
// L2: Semantic Search — m1nd.seek + m1nd.scan
// =========================================================================

// ---------------------------------------------------------------------------
// m1nd.seek (L2-SEMANTIC-SEARCH §6.1)
// ---------------------------------------------------------------------------

/// Input for m1nd.seek — intent-aware code search.
/// Finds code by PURPOSE, not text pattern.
/// Example: seek("code that validates user credentials") returns auth modules.
#[derive(Clone, Debug, Deserialize)]
pub struct SeekInput {
    /// Natural language description of what the agent is looking for.
    pub query: String,
    pub agent_id: String,
    /// Maximum results to return. Default: 20.
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// File path prefix to limit search scope. None = entire graph.
    #[serde(default)]
    pub scope: Option<String>,
    /// Filter by node type: "function", "class", "struct", "module", "file".
    #[serde(default)]
    pub node_types: Vec<String>,
    /// Minimum combined score threshold. Default: 0.1.
    #[serde(default = "default_min_score")]
    pub min_score: f32,
    /// Whether to run graph re-ranking on embedding candidates. Default: true.
    #[serde(default = "default_true")]
    pub graph_rerank: bool,
}

/// Output for m1nd.seek.
#[derive(Clone, Debug, Serialize)]
pub struct SeekOutput {
    pub query: String,
    pub results: Vec<SeekResultEntry>,
    pub total_candidates_scanned: usize,
    /// Whether embeddings were used (false = fallback to trigram/semantic engine).
    pub embeddings_used: bool,
    pub elapsed_ms: f64,
}

/// A single seek result entry.
#[derive(Clone, Debug, Serialize)]
pub struct SeekResultEntry {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    /// Combined score: embedding * 0.5 + graph * 0.3 + temporal * 0.2.
    pub score: f32,
    pub score_breakdown: SeekScoreBreakdown,
    /// Heuristic intent summary generated during ingest.
    pub intent_summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    /// Connected nodes (callers, callees, importers).
    pub connections: Vec<SeekConnection>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SeekScoreBreakdown {
    pub embedding_similarity: f32,
    pub graph_activation: f32,
    pub temporal_recency: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SeekConnection {
    pub node_id: String,
    pub label: String,
    pub relation: String,
}

// ---------------------------------------------------------------------------
// m1nd.scan (L2-SEMANTIC-SEARCH §6.2)
// ---------------------------------------------------------------------------

/// Input for m1nd.scan — pattern-aware code analysis.
/// Detects structural issues using predefined patterns with graph-aware
/// validation across file boundaries.
#[derive(Clone, Debug, Deserialize)]
pub struct ScanInput {
    /// Pattern ID ("error_handling", "resource_cleanup", "api_surface",
    /// "state_mutation", "concurrency", "auth_boundary", "test_coverage",
    /// "dependency_injection") or a custom ast-grep pattern string.
    pub pattern: String,
    pub agent_id: String,
    /// File path prefix to limit scan scope. None = entire graph.
    #[serde(default)]
    pub scope: Option<String>,
    /// Minimum severity threshold [0.0, 1.0]. Default: 0.3.
    #[serde(default = "default_severity_min")]
    pub severity_min: f32,
    /// Whether to validate findings against graph edges (cross-file). Default: true.
    #[serde(default = "default_true")]
    pub graph_validate: bool,
    /// Maximum findings to return. Default: 50.
    #[serde(default = "default_scan_limit")]
    pub limit: usize,
}

/// Output for m1nd.scan.
#[derive(Clone, Debug, Serialize)]
pub struct ScanOutput {
    pub pattern: String,
    pub findings: Vec<ScanFinding>,
    pub files_scanned: usize,
    pub total_matches_raw: usize,
    /// Matches after graph-aware validation.
    pub total_matches_validated: usize,
    pub elapsed_ms: f64,
}

/// A single scan finding.
#[derive(Clone, Debug, Serialize)]
pub struct ScanFinding {
    pub pattern: String,
    /// "confirmed" | "mitigated" | "false_positive"
    pub status: String,
    pub severity: f32,
    pub node_id: String,
    pub label: String,
    pub file_path: String,
    pub line: u32,
    pub message: String,
    /// Related graph nodes that informed the validation decision.
    pub graph_context: Vec<ScanContextNode>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScanContextNode {
    pub node_id: String,
    pub label: String,
    pub relation: String,
}

// =========================================================================
// L3: Temporal Intelligence — m1nd.timeline + m1nd.diverge
// =========================================================================

// ---------------------------------------------------------------------------
// m1nd.timeline (L3-TEMPORAL-INTELLIGENCE §5)
// ---------------------------------------------------------------------------

/// Input for m1nd.timeline — git-based temporal history for a node.
/// Returns change history, co-change partners, velocity, and stability.
#[derive(Clone, Debug, Deserialize)]
pub struct TimelineInput {
    /// Node external_id (e.g. "file::backend/handler.py").
    pub node: String,
    pub agent_id: String,
    /// Time depth: "7d", "30d", "90d", "all". Default: "30d".
    #[serde(default = "default_depth_30d")]
    pub depth: String,
    /// Include co-changed files with coupling scores. Default: true.
    #[serde(default = "default_true")]
    pub include_co_changes: bool,
    /// Include lines added/deleted churn data. Default: true.
    #[serde(default = "default_true")]
    pub include_churn: bool,
    /// Max co-change partners to return. Default: 10.
    #[serde(default = "default_top_k_10")]
    pub top_k: usize,
}

/// Output for m1nd.timeline.
#[derive(Clone, Debug, Serialize)]
pub struct TimelineOutput {
    pub node: String,
    pub depth: String,
    pub changes: Vec<TimelineChange>,
    pub co_changed_with: Vec<CoChangePartner>,
    /// "accelerating" | "decelerating" | "stable"
    pub velocity: String,
    /// [0.0, 1.0] — 1.0 = very stable, 0.0 = very volatile.
    pub stability_score: f32,
    /// "expanding" | "shrinking" | "churning" | "dormant" | "stable"
    pub pattern: String,
    pub total_churn: ChurnSummary,
    pub commit_count_in_window: usize,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TimelineChange {
    pub date: String,
    pub commit: String,
    pub author: String,
    /// "+45/-12" format.
    pub delta: String,
    pub co_changed: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CoChangePartner {
    pub file: String,
    pub times: u32,
    /// co_changes(A,B) / max(changes(A), changes(B)). [0.0, 1.0].
    pub coupling_degree: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChurnSummary {
    pub lines_added: u32,
    pub lines_deleted: u32,
}

// ---------------------------------------------------------------------------
// m1nd.diverge (L3-TEMPORAL-INTELLIGENCE §6)
// ---------------------------------------------------------------------------

/// Input for m1nd.diverge — structural drift between two points in time.
/// Compares graph state at baseline vs current.
#[derive(Clone, Debug, Deserialize)]
pub struct DivergeInput {
    pub agent_id: String,
    /// Baseline reference: ISO date ("2026-03-01"), git ref (SHA/tag),
    /// or "last_session" to use the saved GraphFingerprint.
    pub baseline: String,
    /// File path glob to limit scope. None = all nodes.
    #[serde(default)]
    pub scope: Option<String>,
    /// Include coupling matrix delta. Default: true.
    #[serde(default = "default_true")]
    pub include_coupling_changes: bool,
    /// Detect anomalies (test deficits, velocity spikes). Default: true.
    #[serde(default = "default_true")]
    pub include_anomalies: bool,
}

/// Output for m1nd.diverge.
#[derive(Clone, Debug, Serialize)]
pub struct DivergeOutput {
    pub baseline: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// 1.0 - jaccard(baseline_nodes, current_nodes). [0.0, 1.0].
    pub structural_drift: f32,
    pub new_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub modified_nodes: Vec<DivergeModifiedNode>,
    pub coupling_changes: Vec<CouplingChange>,
    pub anomalies: Vec<DivergeAnomaly>,
    pub summary: String,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DivergeModifiedNode {
    pub file: String,
    /// "+450/-30" format.
    pub delta: String,
    pub growth_ratio: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CouplingChange {
    pub pair: [String; 2],
    pub was: f32,
    pub now: f32,
    /// "new_coupling" | "decoupled" | "strengthened" | "weakened"
    pub direction: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DivergeAnomaly {
    /// "test_deficit" | "velocity_spike" | "new_coupling" | "isolation"
    #[serde(rename = "type")]
    pub anomaly_type: String,
    pub file: String,
    pub detail: String,
    /// "critical" | "warning" | "info"
    pub severity: String,
}

// =========================================================================
// L4: Investigation Memory — m1nd.trail.*
// =========================================================================

// ---------------------------------------------------------------------------
// m1nd.trail.save (L4-INVESTIGATION-MEMORY §3, §4)
// ---------------------------------------------------------------------------

/// Input for m1nd.trail.save — persist the current investigation state.
/// Visited nodes are auto-captured from perspective + trail boosts.
#[derive(Clone, Debug, Deserialize)]
pub struct TrailSaveInput {
    pub agent_id: String,
    /// Human-readable label for this investigation.
    pub label: String,
    /// Hypotheses formed during investigation.
    #[serde(default)]
    pub hypotheses: Vec<TrailHypothesisInput>,
    /// Conclusions reached.
    #[serde(default)]
    pub conclusions: Vec<TrailConclusionInput>,
    /// Open questions remaining.
    #[serde(default)]
    pub open_questions: Vec<String>,
    /// Tags for organization and search.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional summary. Auto-generated if omitted.
    #[serde(default)]
    pub summary: Option<String>,
    /// Optional: explicitly list visited node external_ids with annotations.
    /// If omitted, captured from active perspective state.
    #[serde(default)]
    pub visited_nodes: Vec<TrailVisitedNodeInput>,
    /// Optional: activation boosts to re-inject on resume.
    /// Map of node_external_id -> boost weight [0.0, 1.0].
    #[serde(default)]
    pub activation_boosts: HashMap<String, f32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrailHypothesisInput {
    pub statement: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub supporting_nodes: Vec<String>,
    #[serde(default)]
    pub contradicting_nodes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrailConclusionInput {
    pub statement: String,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub from_hypotheses: Vec<String>,
    #[serde(default)]
    pub supporting_nodes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrailVisitedNodeInput {
    pub node_external_id: String,
    #[serde(default)]
    pub annotation: Option<String>,
    #[serde(default = "default_relevance")]
    pub relevance: f32,
}

/// Output for m1nd.trail.save.
#[derive(Clone, Debug, Serialize)]
pub struct TrailSaveOutput {
    pub trail_id: String,
    pub label: String,
    pub agent_id: String,
    pub nodes_saved: usize,
    pub hypotheses_saved: usize,
    pub conclusions_saved: usize,
    pub open_questions_saved: usize,
    pub graph_generation_at_creation: u64,
    pub created_at_ms: u64,
}

// ---------------------------------------------------------------------------
// m1nd.trail.resume (L4-INVESTIGATION-MEMORY §5)
// ---------------------------------------------------------------------------

/// Input for m1nd.trail.resume — restore a saved investigation.
/// Re-injects activation boosts, validates node existence, detects staleness.
#[derive(Clone, Debug, Deserialize)]
pub struct TrailResumeInput {
    pub agent_id: String,
    pub trail_id: String,
    /// Resume even if trail is stale (>50% missing nodes). Default: false.
    #[serde(default)]
    pub force: bool,
}

/// Output for m1nd.trail.resume.
#[derive(Clone, Debug, Serialize)]
pub struct TrailResumeOutput {
    pub trail_id: String,
    pub label: String,
    /// Whether the trail was stale (graph changed since save).
    pub stale: bool,
    /// Number of graph generations behind.
    pub generations_behind: u64,
    /// Nodes from trail that no longer exist in the graph.
    pub missing_nodes: Vec<String>,
    /// Number of nodes successfully re-activated via boost injection.
    pub nodes_reactivated: usize,
    /// Hypotheses that were downgraded due to missing supporting nodes.
    pub hypotheses_downgraded: Vec<String>,
    /// The full trail data.
    pub trail: TrailSummaryOutput,
    pub elapsed_ms: f64,
}

/// Compact trail representation in outputs.
#[derive(Clone, Debug, Serialize)]
pub struct TrailSummaryOutput {
    pub trail_id: String,
    pub agent_id: String,
    pub label: String,
    /// "active" | "saved" | "archived" | "stale" | "merged"
    pub status: String,
    pub created_at_ms: u64,
    pub last_modified_ms: u64,
    pub node_count: usize,
    pub hypothesis_count: usize,
    pub conclusion_count: usize,
    pub open_question_count: usize,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

// ---------------------------------------------------------------------------
// m1nd.trail.merge (L4-INVESTIGATION-MEMORY §6)
// ---------------------------------------------------------------------------

/// Input for m1nd.trail.merge — combine two or more investigation trails.
/// Uses confidence+recency scoring for conflict resolution.
#[derive(Clone, Debug, Deserialize)]
pub struct TrailMergeInput {
    pub agent_id: String,
    /// Two or more trail IDs to merge.
    pub trail_ids: Vec<String>,
    /// Label for the merged trail. Auto-generated if omitted.
    #[serde(default)]
    pub label: Option<String>,
}

/// Output for m1nd.trail.merge.
#[derive(Clone, Debug, Serialize)]
pub struct TrailMergeOutput {
    pub merged_trail_id: String,
    pub label: String,
    /// Source trail IDs that were merged (now status = "merged").
    pub source_trails: Vec<String>,
    pub nodes_merged: usize,
    pub hypotheses_merged: usize,
    /// Hypothesis conflicts detected during merge.
    pub conflicts: Vec<TrailMergeConflict>,
    /// Connections discovered between the two independently explored areas.
    pub connections_discovered: Vec<TrailConnection>,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrailMergeConflict {
    pub hypothesis_a: String,
    pub hypothesis_b: String,
    /// "resolved" (one won) or "unresolved" (flagged for human review).
    pub resolution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<String>,
    pub score_delta: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TrailConnection {
    /// "shared_node" | "bridge_edge" | "cross_support"
    #[serde(rename = "type")]
    pub connection_type: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
}

// ---------------------------------------------------------------------------
// m1nd.trail.list (L4-INVESTIGATION-MEMORY §8.2)
// ---------------------------------------------------------------------------

/// Input for m1nd.trail.list — list trails matching filters.
#[derive(Clone, Debug, Deserialize)]
pub struct TrailListInput {
    pub agent_id: String,
    /// Filter to a specific agent's trails. None = all agents.
    #[serde(default)]
    pub filter_agent_id: Option<String>,
    /// Filter by status: "active", "saved", "archived", "stale", "merged".
    #[serde(default)]
    pub filter_status: Option<String>,
    /// Filter by tags (any match).
    #[serde(default)]
    pub filter_tags: Vec<String>,
}

/// Output for m1nd.trail.list.
#[derive(Clone, Debug, Serialize)]
pub struct TrailListOutput {
    pub trails: Vec<TrailSummaryOutput>,
    pub total_count: usize,
}

// =========================================================================
// L5: Hypothesis Engine — m1nd.hypothesize + m1nd.differential
// =========================================================================

// ---------------------------------------------------------------------------
// m1nd.hypothesize (L5-HYPOTHESIS-ENGINE §2, §3, §4)
// ---------------------------------------------------------------------------

/// Input for m1nd.hypothesize — test a structural claim about the codebase.
/// Encodes the claim as a graph query and returns evidence for/against.
///
/// Supported claim patterns (auto-detected from natural language):
///   NEVER_CALLS, ALWAYS_BEFORE, DEPENDS_ON, NO_DEPENDENCY,
///   COUPLING, ISOLATED, GATEWAY, CIRCULAR
#[derive(Clone, Debug, Deserialize)]
pub struct HypothesizeInput {
    /// Natural language claim about the codebase.
    /// Examples:
    ///   "handler never validates session tokens"
    ///   "all external calls go through the router"
    ///   "auditor is independent of messaging"
    pub claim: String,
    pub agent_id: String,
    /// Max BFS hops for evidence search. Default: 5.
    #[serde(default = "default_max_hops")]
    pub max_hops: u8,
    /// Whether to include ghost edges as weak evidence. Default: true.
    #[serde(default = "default_true")]
    pub include_ghost_edges: bool,
    /// Whether to include partial flow when full path not found. Default: true.
    #[serde(default = "default_true")]
    pub include_partial_flow: bool,
    /// Budget cap for all-paths enumeration. Default: 1000.
    #[serde(default = "default_path_budget")]
    pub path_budget: usize,
}

/// Output for m1nd.hypothesize.
#[derive(Clone, Debug, Serialize)]
pub struct HypothesizeOutput {
    pub claim: String,
    /// Parsed claim type: "never_calls", "always_before", "depends_on",
    /// "no_dependency", "coupling", "isolated", "gateway", "circular".
    pub claim_type: String,
    /// Resolved subject node(s).
    pub subject_nodes: Vec<String>,
    /// Resolved object/target node(s).
    pub object_nodes: Vec<String>,
    /// "likely_true" (>0.8), "likely_false" (<0.2), or "inconclusive".
    pub verdict: String,
    /// Bayesian posterior confidence [0.01, 0.99].
    pub confidence: f32,
    pub supporting_evidence: Vec<HypothesisEvidence>,
    pub contradicting_evidence: Vec<HypothesisEvidence>,
    /// Partial flow: how far the search reached before stopping.
    /// Only populated when full path was not found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_reach: Option<Vec<PartialReachEntry>>,
    pub paths_explored: usize,
    pub elapsed_ms: f64,
}

/// A single piece of evidence for or against a hypothesis.
#[derive(Clone, Debug, Serialize)]
pub struct HypothesisEvidence {
    /// "path_found" | "no_path" | "ghost_edge" | "community_membership" |
    /// "causal_chain" | "counterfactual_impact" | "activation_reach"
    #[serde(rename = "type")]
    pub evidence_type: String,
    pub description: String,
    /// Likelihood factor contributed by this evidence.
    pub likelihood_factor: f32,
    /// Node IDs involved in this evidence.
    pub nodes: Vec<String>,
    /// Edge relations along the evidence path (if path-based).
    #[serde(default)]
    pub relations: Vec<String>,
    /// Total edge weight along the path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_weight: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PartialReachEntry {
    pub node_id: String,
    pub label: String,
    pub hops_from_source: u8,
    pub activation_at_stop: f32,
}

// ---------------------------------------------------------------------------
// m1nd.differential (L5-HYPOTHESIS-ENGINE §5)
// ---------------------------------------------------------------------------

/// Input for m1nd.differential — focused structural diff between two
/// graph snapshots.
#[derive(Clone, Debug, Deserialize)]
pub struct DifferentialInput {
    pub agent_id: String,
    /// Path to snapshot A, or "current" for the in-memory graph.
    pub snapshot_a: String,
    /// Path to snapshot B, or "current" for the in-memory graph.
    pub snapshot_b: String,
    /// Focus filter question. Narrows the diff output.
    /// Examples: "what new coupling was introduced?",
    ///           "what modules became isolated?"
    #[serde(default)]
    pub question: Option<String>,
    /// Optional: limit diff to neighborhood of specific nodes.
    #[serde(default)]
    pub focus_nodes: Vec<String>,
}

/// Output for m1nd.differential.
#[derive(Clone, Debug, Serialize)]
pub struct DifferentialOutput {
    pub snapshot_a: String,
    pub snapshot_b: String,
    pub new_edges: Vec<DiffEdgeDelta>,
    pub removed_edges: Vec<DiffEdgeDelta>,
    pub weight_changes: Vec<DiffWeightDelta>,
    pub new_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub coupling_deltas: Vec<DiffCouplingDelta>,
    pub summary: String,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiffEdgeDelta {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiffWeightDelta {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub old_weight: f32,
    pub new_weight: f32,
    pub delta: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DiffCouplingDelta {
    pub community_a: String,
    pub community_b: String,
    pub old_coupling: f32,
    pub new_coupling: f32,
    pub delta: f32,
}

// =========================================================================
// L6: Execution Feedback — m1nd.trace + m1nd.validate_plan
// =========================================================================

// ---------------------------------------------------------------------------
// m1nd.trace (L6-EXECUTION-FEEDBACK §4)
// ---------------------------------------------------------------------------

/// Input for m1nd.trace — map runtime errors to structural root causes.
/// Parses stacktraces, maps frames to graph nodes, scores suspiciousness.
#[derive(Clone, Debug, Deserialize)]
pub struct TraceInput {
    /// Full error output (stacktrace + error message).
    pub error_text: String,
    pub agent_id: String,
    /// Language hint: "python", "rust", "typescript", "javascript", "go".
    /// Auto-detected if omitted.
    #[serde(default)]
    pub language: Option<String>,
    /// Temporal window (hours) for co-change suspect scan. Default: 24.0.
    #[serde(default = "default_window_hours")]
    pub window_hours: f32,
    /// Max suspects to return. Default: 10.
    #[serde(default = "default_top_k_10")]
    pub top_k: usize,
}

/// Output for m1nd.trace.
#[derive(Clone, Debug, Serialize)]
pub struct TraceOutput {
    pub language_detected: String,
    pub error_type: String,
    pub error_message: String,
    pub frames_parsed: usize,
    /// How many frames matched graph nodes.
    pub frames_mapped: usize,
    /// Ranked suspects: most likely root cause first.
    pub suspects: Vec<TraceSuspect>,
    /// Files modified in the same temporal window as the top suspect.
    pub co_change_suspects: Vec<TraceCoChangeSuspect>,
    /// Causal chain from suspected root cause to error site.
    pub causal_chain: Vec<String>,
    pub fix_scope: TraceFixScope,
    /// Frames that could not be mapped to graph nodes.
    pub unmapped_frames: Vec<TraceUnmappedFrame>,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceSuspect {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    /// Composite suspiciousness [0.0, 1.0].
    pub suspiciousness: f32,
    pub signals: TraceSuspiciousnessSignals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    /// Who calls this suspect.
    pub related_callers: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceSuspiciousnessSignals {
    /// 1.0 = deepest frame; decays linearly.
    pub trace_depth_score: f32,
    /// Exponential decay from last modification time.
    pub recency_score: f32,
    /// Normalized PageRank centrality.
    pub centrality_score: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceCoChangeSuspect {
    pub node_id: String,
    pub label: String,
    /// Unix timestamp of last modification.
    pub modified_at: f64,
    /// "Modified within Nh of top suspect".
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceFixScope {
    pub files_to_inspect: Vec<String>,
    pub estimated_blast_radius: usize,
    /// "low" | "medium" | "high" | "critical"
    pub risk_level: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceUnmappedFrame {
    pub file: String,
    pub line: u32,
    pub function: String,
    /// "file not in graph" | "line outside any node range" | "stdlib/third-party"
    pub reason: String,
}

// ---------------------------------------------------------------------------
// m1nd.validate_plan (L6-EXECUTION-FEEDBACK §5)
// ---------------------------------------------------------------------------

/// Input for m1nd.validate_plan — validate a proposed modification plan
/// against the code graph. Detects gaps, risk, and missing test coverage.
#[derive(Clone, Debug, Deserialize)]
pub struct ValidatePlanInput {
    pub agent_id: String,
    /// Ordered list of planned actions.
    pub actions: Vec<PlannedAction>,
    /// Whether to analyze test coverage for modified files. Default: true.
    #[serde(default = "default_true")]
    pub include_test_impact: bool,
    /// Whether to compute composite risk score. Default: true.
    #[serde(default = "default_true")]
    pub include_risk_score: bool,
}

/// A single action in a modification plan.
#[derive(Clone, Debug, Deserialize)]
pub struct PlannedAction {
    /// "modify" | "create" | "delete" | "rename" | "test"
    pub action_type: String,
    /// Relative file path.
    pub file_path: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Other file_paths this action depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Output for m1nd.validate_plan.
#[derive(Clone, Debug, Serialize)]
pub struct ValidatePlanOutput {
    pub actions_analyzed: usize,
    /// Matched to graph nodes.
    pub actions_resolved: usize,
    /// New files not yet in graph.
    pub actions_unresolved: usize,
    /// Files affected but not in the plan.
    pub gaps: Vec<PlanGap>,
    /// Composite risk [0.0, 1.0].
    pub risk_score: f32,
    /// "low" (<0.3) | "medium" (<0.6) | "high" (<0.8) | "critical" (>=0.8)
    pub risk_level: String,
    pub test_coverage: PlanTestCoverage,
    /// Suggested additions to the plan.
    pub suggested_additions: Vec<PlanSuggestedAction>,
    pub blast_radius_total: usize,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlanGap {
    pub file_path: String,
    pub node_id: String,
    /// "imported by modified file X" | "in blast radius of Y"
    pub reason: String,
    /// "critical" | "warning" | "info"
    pub severity: String,
    pub signal_strength: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlanTestCoverage {
    pub modified_files: usize,
    pub tested_files: usize,
    pub untested_files: Vec<String>,
    pub coverage_ratio: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlanSuggestedAction {
    /// "modify" | "test"
    pub action_type: String,
    pub file_path: String,
    pub reason: String,
}

// =========================================================================
// L7: Multi-Repository Federation — m1nd.federate
// =========================================================================

// ---------------------------------------------------------------------------
// m1nd.federate (L7-MULTI-REPO-FEDERATION §6.3)
// ---------------------------------------------------------------------------

/// Input for m1nd.federate — ingest multiple repositories into a unified
/// federated graph with cross-repo edge detection.
///
/// Node IDs in the federated graph use `{repo_name}::file::path` format.
/// All existing query tools (activate, impact, why, etc.) traverse
/// cross-repo edges automatically.
#[derive(Clone, Debug, Deserialize)]
pub struct FederateInput {
    pub agent_id: String,
    /// List of repositories to federate.
    pub repos: Vec<FederateRepo>,
    /// Auto-detect cross-repo edges (config, API, import, type, deployment).
    /// Default: true.
    #[serde(default = "default_true")]
    pub detect_cross_repo_edges: bool,
    /// Only re-ingest repos that changed since last federation. Default: false.
    #[serde(default)]
    pub incremental: bool,
}

/// A single repository in a federation request.
#[derive(Clone, Debug, Deserialize)]
pub struct FederateRepo {
    /// Repository name (used as namespace prefix in external_ids).
    pub name: String,
    /// Absolute path to repository root.
    pub path: String,
    /// Ingest adapter override. Default: "code".
    #[serde(default = "default_adapter")]
    pub adapter: String,
}

/// Output for m1nd.federate.
#[derive(Clone, Debug, Serialize)]
pub struct FederateOutput {
    pub repos_ingested: Vec<FederateRepoResult>,
    pub total_nodes: u32,
    pub total_edges: u64,
    pub cross_repo_edges: Vec<FederateCrossRepoEdge>,
    pub cross_repo_edge_count: usize,
    /// Whether incremental mode was used.
    pub incremental: bool,
    /// Repos that were skipped (unchanged) in incremental mode.
    pub skipped_repos: Vec<String>,
    pub elapsed_ms: f64,
}

/// Per-repo ingestion result in federation.
#[derive(Clone, Debug, Serialize)]
pub struct FederateRepoResult {
    pub name: String,
    pub path: String,
    pub node_count: u32,
    pub edge_count: u32,
    /// Whether this repo was freshly ingested or loaded from cache.
    pub from_cache: bool,
    pub ingest_ms: f64,
}

/// A detected cross-repo edge.
#[derive(Clone, Debug, Serialize)]
pub struct FederateCrossRepoEdge {
    pub source_repo: String,
    pub target_repo: String,
    pub source_node: String,
    pub target_node: String,
    /// "shared_config" | "api_contract" | "package_dep" | "shared_type" |
    /// "deployment_dep" | "mcp_contract"
    pub edge_type: String,
    pub relation: String,
    pub weight: f32,
    pub causal_strength: f32,
}

// =========================================================================
// Default value helpers
// =========================================================================

fn default_top_k() -> usize { 20 }
fn default_top_k_10() -> usize { 10 }
fn default_true() -> bool { true }
fn default_max_hops() -> u8 { 5 }
fn default_min_score() -> f32 { 0.1 }
fn default_severity_min() -> f32 { 0.3 }
fn default_scan_limit() -> usize { 50 }
fn default_depth_30d() -> String { "30d".into() }
fn default_confidence() -> f32 { 0.5 }
fn default_relevance() -> f32 { 0.5 }
fn default_path_budget() -> usize { 1000 }
fn default_window_hours() -> f32 { 24.0 }
fn default_adapter() -> String { "code".into() }

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- L2 ---

    #[test]
    fn seek_input_deserializes_minimal() {
        let json = r#"{"query": "find auth code", "agent_id": "jimi"}"#;
        let input: SeekInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "find auth code");
        assert_eq!(input.agent_id, "jimi");
        assert_eq!(input.top_k, 20);
        assert!(input.scope.is_none());
        assert!(input.node_types.is_empty());
        assert!((input.min_score - 0.1).abs() < f32::EPSILON);
        assert!(input.graph_rerank);
    }

    #[test]
    fn scan_input_defaults() {
        let json = r#"{"pattern": "error_handling", "agent_id": "jimi"}"#;
        let input: ScanInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.pattern, "error_handling");
        assert!((input.severity_min - 0.3).abs() < f32::EPSILON);
        assert!(input.graph_validate);
        assert_eq!(input.limit, 50);
    }

    // --- L3 ---

    #[test]
    fn timeline_input_deserializes_minimal() {
        let json = r#"{"node": "file::backend/config.py", "agent_id": "jimi"}"#;
        let input: TimelineInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.node, "file::backend/config.py");
        assert_eq!(input.depth, "30d");
        assert!(input.include_co_changes);
        assert!(input.include_churn);
        assert_eq!(input.top_k, 10);
    }

    #[test]
    fn diverge_input_with_scope() {
        let json = r#"{
            "agent_id": "jimi",
            "baseline": "2026-03-01",
            "scope": "backend/stormender*"
        }"#;
        let input: DivergeInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.baseline, "2026-03-01");
        assert_eq!(input.scope.as_deref(), Some("backend/stormender*"));
        assert!(input.include_coupling_changes);
        assert!(input.include_anomalies);
    }

    // --- L4 ---

    #[test]
    fn trail_save_input_minimal() {
        let json = r#"{"agent_id": "jimi", "label": "race condition investigation"}"#;
        let input: TrailSaveInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.label, "race condition investigation");
        assert!(input.hypotheses.is_empty());
        assert!(input.conclusions.is_empty());
        assert!(input.open_questions.is_empty());
        assert!(input.tags.is_empty());
    }

    #[test]
    fn trail_resume_input_defaults() {
        let json = r#"{"agent_id": "jimi", "trail_id": "trail_jimi_001_abc"}"#;
        let input: TrailResumeInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.trail_id, "trail_jimi_001_abc");
        assert!(!input.force);
    }

    #[test]
    fn trail_merge_input_two_trails() {
        let json = r#"{
            "agent_id": "jimi",
            "trail_ids": ["trail_a", "trail_b"]
        }"#;
        let input: TrailMergeInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.trail_ids.len(), 2);
        assert!(input.label.is_none());
    }

    #[test]
    fn trail_list_input_with_filters() {
        let json = r#"{
            "agent_id": "jimi",
            "filter_status": "saved",
            "filter_tags": ["stormender", "concurrency"]
        }"#;
        let input: TrailListInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.filter_status.as_deref(), Some("saved"));
        assert_eq!(input.filter_tags.len(), 2);
    }

    // --- L5 ---

    #[test]
    fn hypothesize_input_minimal() {
        let json = r#"{
            "claim": "handler never validates session tokens",
            "agent_id": "jimi"
        }"#;
        let input: HypothesizeInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.claim, "handler never validates session tokens");
        assert_eq!(input.max_hops, 5);
        assert!(input.include_ghost_edges);
        assert!(input.include_partial_flow);
        assert_eq!(input.path_budget, 1000);
    }

    #[test]
    fn differential_input_minimal() {
        let json = r#"{
            "agent_id": "jimi",
            "snapshot_a": "before.json",
            "snapshot_b": "current"
        }"#;
        let input: DifferentialInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.snapshot_a, "before.json");
        assert_eq!(input.snapshot_b, "current");
        assert!(input.question.is_none());
        assert!(input.focus_nodes.is_empty());
    }

    // --- L6 ---

    #[test]
    fn trace_input_minimal() {
        let json = r#"{
            "error_text": "Traceback (most recent call last):\n  File \"test.py\", line 1\nTypeError: bad",
            "agent_id": "jimi"
        }"#;
        let input: TraceInput = serde_json::from_str(json).unwrap();
        assert!(input.language.is_none());
        assert!((input.window_hours - 24.0).abs() < f32::EPSILON);
        assert_eq!(input.top_k, 10);
    }

    #[test]
    fn validate_plan_input_with_actions() {
        let json = r#"{
            "agent_id": "jimi",
            "actions": [
                {"action_type": "modify", "file_path": "backend/config.py"},
                {"action_type": "test", "file_path": "backend/tests/test_config.py"}
            ]
        }"#;
        let input: ValidatePlanInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.actions.len(), 2);
        assert!(input.include_test_impact);
        assert!(input.include_risk_score);
        assert_eq!(input.actions[0].action_type, "modify");
        assert!(input.actions[0].depends_on.is_empty());
    }

    // --- L7 ---

    #[test]
    fn federate_input_minimal() {
        let json = r#"{
            "agent_id": "jimi",
            "repos": [
                {"name": "my-project", "path": "/tmp/my-project"},
                {"name": "my-library", "path": "/tmp/my-library"}
            ]
        }"#;
        let input: FederateInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.repos.len(), 2);
        assert!(input.detect_cross_repo_edges);
        assert!(!input.incremental);
        assert_eq!(input.repos[0].name, "my-project");
        assert_eq!(input.repos[1].adapter, "code");
    }
}
