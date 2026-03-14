// === m1nd-mcp/src/protocol/perspective.rs ===
// Input/Output types for the 12 perspective MCP tools.
// From 12-PERSPECTIVE-SYNTHESIS, PRD sections 5-14.

use serde::{Deserialize, Serialize};

use crate::perspective::state::{
    AffinityCandidate, Diagnostic, PeekContent, PerspectiveLens, PerspectiveMode, Route,
    RouteFamily, SuggestResult,
};

// ---------------------------------------------------------------------------
// perspective.start (PRD §6)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveStartInput {
    pub agent_id: String,
    /// Seed query for route synthesis.
    pub query: String,
    /// Optional: anchor to a specific node (activates anchored mode).
    #[serde(default)]
    pub anchor_node: Option<String>,
    /// Optional: starting lens configuration.
    #[serde(default)]
    pub lens: Option<PerspectiveLens>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveStartOutput {
    pub perspective_id: String,
    pub mode: PerspectiveMode,
    pub anchor_node: Option<String>,
    pub focus_node: Option<String>,
    /// Initial route set (first page).
    pub routes: Vec<Route>,
    pub total_routes: usize,
    pub page: u32,
    pub total_pages: u32,
    pub route_set_version: u64,
    pub cache_generation: u64,
    pub suggested: Option<String>, // e.g. "inspect R03"
}

// ---------------------------------------------------------------------------
// perspective.routes (PRD §7)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveRoutesInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Page number (1-based). Default: 1. Must be >= 1.
    #[serde(default = "default_page")]
    pub page: u32,
    /// Page size. Clamped to [1, 10]. Default: 6.
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Required: route_set_version from previous response. Staleness check.
    #[serde(default)]
    pub route_set_version: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveRoutesOutput {
    pub perspective_id: String,
    pub mode: PerspectiveMode,
    pub mode_effective: String, // "anchored" or "local" (degraded if >8 hops from anchor)
    pub anchor: Option<String>,
    pub focus: Option<String>,
    pub lens_summary: String, // compact one-line lens description
    pub page: u32,
    pub total_pages: u32,
    pub total_routes: usize,
    pub route_set_version: u64,
    pub cache_generation: u64,
    pub routes: Vec<Route>,
    pub suggested: Option<String>,
    /// Diagnostic if routes are empty (Theme 12).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<Diagnostic>,
    /// Warning if all routes are from one family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_diversity_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dominant_family: Option<RouteFamily>,
    /// Whether page_size was clamped.
    #[serde(default)]
    pub page_size_clamped: bool,
}

// ---------------------------------------------------------------------------
// perspective.inspect (PRD §9)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveInspectInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Exactly one of route_id or route_index must be provided.
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub route_index: Option<u32>,
    /// Route set version for staleness check.
    pub route_set_version: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveInspectOutput {
    pub route_id: String,
    pub route_index: u32,
    pub family: RouteFamily,
    pub target_node: String,
    pub target_label: String,
    pub target_type: String,
    /// Fuller path preview.
    pub path_preview: Vec<String>,
    /// Route family explanation.
    pub family_explanation: String,
    /// Stronger metrics than the route list.
    pub score: f32,
    pub score_breakdown: InspectScoreBreakdown,
    /// Provenance summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<InspectProvenance>,
    /// Whether peek is available.
    pub peek_available: bool,
    /// Affinity candidates for this route (Theme 12).
    pub affinity_candidates: Vec<AffinityCandidate>,
    /// Total chars in this response (for Theme 5 cap enforcement).
    pub response_chars: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct InspectScoreBreakdown {
    pub local_activation: f32,
    pub path_coherence: f32,
    pub novelty: f32,
    pub anchor_relevance: Option<f32>, // None in local mode
    pub continuity: Option<f32>,       // None in local mode
}

#[derive(Clone, Debug, Serialize)]
pub struct InspectProvenance {
    pub source_path: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub namespace: Option<String>,
    pub provenance_stale: bool,
}

// ---------------------------------------------------------------------------
// perspective.peek (PRD §10)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectivePeekInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Exactly one of route_id or route_index must be provided.
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub route_index: Option<u32>,
    /// Route set version for staleness check.
    pub route_set_version: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectivePeekOutput {
    pub route_id: String,
    pub route_index: u32,
    pub target_node: String,
    /// Security-checked peek content.
    pub content: PeekContent,
}

// ---------------------------------------------------------------------------
// perspective.follow (PRD §8 implied)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveFollowInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Exactly one of route_id or route_index must be provided.
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub route_index: Option<u32>,
    /// Route set version for staleness check.
    pub route_set_version: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveFollowOutput {
    pub perspective_id: String,
    pub previous_focus: Option<String>,
    pub new_focus: String,
    pub mode: PerspectiveMode,
    pub mode_effective: String,
    /// New routes from the new focus.
    pub routes: Vec<Route>,
    pub total_routes: usize,
    pub page: u32,
    pub total_pages: u32,
    pub route_set_version: u64,
    pub cache_generation: u64,
    pub suggested: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<Diagnostic>,
}

// ---------------------------------------------------------------------------
// perspective.suggest (PRD §11)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveSuggestInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Route set version for staleness check.
    pub route_set_version: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveSuggestOutput {
    pub perspective_id: String,
    pub suggestion: SuggestResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<Diagnostic>,
}

// ---------------------------------------------------------------------------
// perspective.affinity (PRD §12)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveAffinityInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Exactly one of route_id or route_index must be provided.
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub route_index: Option<u32>,
    /// Route set version for staleness check.
    pub route_set_version: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveAffinityOutput {
    pub route_id: String,
    pub target_node: String,
    /// Epistemic notice (Theme 13).
    pub notice: String, // "Probable connections, not verified edges."
    /// Up to 8 candidates (Theme 5 cap). Min threshold: 0.15.
    pub candidates: Vec<AffinityCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<Diagnostic>,
}

// ---------------------------------------------------------------------------
// perspective.branch (PRD implied)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveBranchInput {
    pub agent_id: String,
    pub perspective_id: String,
    /// Optional branch name. Auto-generated if not provided.
    #[serde(default)]
    pub branch_name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveBranchOutput {
    pub perspective_id: String,
    /// The new branch's perspective_id.
    pub branch_perspective_id: String,
    pub branch_name: String,
    pub branched_from_focus: Option<String>,
}

// ---------------------------------------------------------------------------
// perspective.back (PRD §14.2)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveBackInput {
    pub agent_id: String,
    pub perspective_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveBackOutput {
    pub perspective_id: String,
    pub restored_focus: Option<String>,
    pub restored_mode: PerspectiveMode,
    /// New routes from the restored focus.
    pub routes: Vec<Route>,
    pub total_routes: usize,
    pub page: u32,
    pub total_pages: u32,
    pub route_set_version: u64,
    pub cache_generation: u64,
}

// ---------------------------------------------------------------------------
// perspective.compare (PRD §9 implied)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveCompareInput {
    pub agent_id: String,
    /// Two perspective IDs to compare. Must be same agent (V1 restriction).
    pub perspective_id_a: String,
    pub perspective_id_b: String,
    /// Dimensions to compare on. Empty = all.
    #[serde(default)]
    pub dimensions: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveCompareOutput {
    pub perspective_id_a: String,
    pub perspective_id_b: String,
    pub shared_nodes: Vec<String>,
    pub unique_to_a: Vec<String>,
    pub unique_to_b: Vec<String>,
    pub dimension_deltas: Vec<DimensionDelta>,
    /// Total chars (for Theme 5 cap of 3000).
    pub response_chars: usize,
    /// Warning if comparing across different generations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_mismatch_warning: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DimensionDelta {
    pub dimension: String,
    pub score_a: f32,
    pub score_b: f32,
    pub delta: f32,
}

// ---------------------------------------------------------------------------
// perspective.list (Theme 2 — management tool)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveListInput {
    pub agent_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveListOutput {
    pub agent_id: String,
    pub perspectives: Vec<PerspectiveSummary>,
    pub total_memory_bytes: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveSummary {
    pub perspective_id: String,
    pub mode: PerspectiveMode,
    pub focus_node: Option<String>,
    pub route_count: usize,
    pub nav_event_count: usize,
    pub stale: bool,
    pub created_at_ms: u64,
    pub last_accessed_ms: u64,
}

// ---------------------------------------------------------------------------
// perspective.close (Theme 2 — management tool)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct PerspectiveCloseInput {
    pub agent_id: String,
    pub perspective_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PerspectiveCloseOutput {
    pub perspective_id: String,
    pub closed: bool,
    /// Locks that were cascade-released (Theme 5).
    pub locks_released: Vec<String>,
}

// ---------------------------------------------------------------------------
// Default helpers
// ---------------------------------------------------------------------------

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    6
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_input_deserializes_minimal() {
        let json = r#"{"agent_id": "jimi", "query": "session management"}"#;
        let input: PerspectiveStartInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.agent_id, "jimi");
        assert_eq!(input.query, "session management");
        assert!(input.anchor_node.is_none());
        assert!(input.lens.is_none());
    }

    #[test]
    fn routes_input_defaults() {
        let json = r#"{"agent_id": "jimi", "perspective_id": "persp_jimi_001"}"#;
        let input: PerspectiveRoutesInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.page, 1);
        assert_eq!(input.page_size, 6);
    }

    #[test]
    fn inspect_input_requires_route_ref() {
        // Both route_id and route_index are optional at serde level;
        // validation layer rejects if neither is provided.
        let json = r#"{"agent_id": "jimi", "perspective_id": "p1", "route_set_version": 100}"#;
        let input: PerspectiveInspectInput = serde_json::from_str(json).unwrap();
        assert!(input.route_id.is_none());
        assert!(input.route_index.is_none());
    }

    #[test]
    fn follow_input_accepts_route_id() {
        let json = r#"{"agent_id": "jimi", "perspective_id": "p1", "route_id": "R_abc123", "route_set_version": 100}"#;
        let input: PerspectiveFollowInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.route_id.as_deref(), Some("R_abc123"));
    }

    #[test]
    fn compare_input_deserializes() {
        let json = r#"{"agent_id": "jimi", "perspective_id_a": "p1", "perspective_id_b": "p2"}"#;
        let input: PerspectiveCompareInput = serde_json::from_str(json).unwrap();
        assert!(input.dimensions.is_empty()); // empty = all
    }
}
