// === m1nd-mcp/src/perspective/state.rs ===
// Themes 1, 2, 5, 11, 13, 14, 15, 16 from 12-PERSPECTIVE-SYNTHESIS.
// All perspective and lock state types. All serializable (u64 timestamps, no Instant).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Perspective navigation mode (Theme 11).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerspectiveMode {
    Anchored,
    Local,
}

/// Route family classification (Theme 4, used in keys.rs).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteFamily {
    Structural,
    Semantic,
    Temporal,
    Causal,
    Ghost,
    Hole,
    Resonant,
}

impl RouteFamily {
    /// Enum ordinal for deterministic tie-breaking (Theme 4).
    pub fn ordinal(&self) -> u8 {
        match self {
            Self::Structural => 0,
            Self::Semantic => 1,
            Self::Temporal => 2,
            Self::Causal => 3,
            Self::Ghost => 4,
            Self::Hole => 5,
            Self::Resonant => 6,
        }
    }
}

/// Watch trigger events (Theme 10).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchTrigger {
    Ingest,
    Learn,
}

/// Watch strategy for lock watchers (Theme 10).
/// V1: Manual, OnIngest, OnLearn only. Periodic returns WatchStrategyNotSupported.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchStrategy {
    Manual,
    OnIngest,
    OnLearn,
    Periodic, // V1: rejected at validation
}

/// Lock scope type (Theme 14).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockScope {
    Node,
    Subgraph,
    QueryNeighborhood,
    Path,
}

// ---------------------------------------------------------------------------
// Perspective Lens (Theme 9)
// ---------------------------------------------------------------------------

/// Lens configuration controlling what the agent sees.
/// Empty arrays = no filter = all values (consistent across all tools).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerspectiveLens {
    /// Activation dimensions. Default: all 4. Empty = all.
    #[serde(default = "default_dimensions")]
    pub dimensions: Vec<String>,

    /// Route family filter. Empty = all families.
    #[serde(default)]
    pub route_families: Vec<RouteFamily>,

    /// Enable XLR noise cancellation.
    #[serde(default = "default_true")]
    pub xlr: bool,

    /// Include ghost edge detection in route synthesis.
    #[serde(default = "default_true")]
    pub include_ghost_edges: bool,

    /// Include structural holes in route synthesis.
    #[serde(default = "default_true")]
    pub include_structural_holes: bool,

    /// Number of top results for route synthesis. Default: 8 (perspective-specific, vs activate's 20).
    #[serde(default = "default_perspective_top_k")]
    pub top_k: u32,

    /// Namespace filter. Empty = all namespaces.
    #[serde(default)]
    pub namespaces: Vec<String>,

    /// Tag filter. Empty = all tags.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Node type filter. Empty = all node types.
    #[serde(default)]
    pub node_types: Vec<String>,

    /// Optional custom ranking weights. None = use defaults.
    #[serde(default)]
    pub ranking_weights: Option<RankingWeights>,
}

impl Default for PerspectiveLens {
    fn default() -> Self {
        Self {
            dimensions: default_dimensions(),
            route_families: Vec::new(),
            xlr: true,
            include_ghost_edges: true,
            include_structural_holes: true,
            top_k: 8,
            namespaces: Vec::new(),
            tags: Vec::new(),
            node_types: Vec::new(),
            ranking_weights: None,
        }
    }
}

/// Custom ranking weights for route scoring (Theme 11).
/// All weights should sum to 1.0. Validated at lens validation time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RankingWeights {
    pub local_activation: f32,
    pub path_coherence: f32,
    pub novelty: f32,
    pub anchor_relevance: f32,
    pub continuity: f32,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            local_activation: 0.35,
            path_coherence: 0.25,
            novelty: 0.15,
            anchor_relevance: 0.15,
            continuity: 0.10,
        }
    }
}

// ---------------------------------------------------------------------------
// Mode Context (Theme 11)
// ---------------------------------------------------------------------------

/// Mode context passed to every scoring function.
/// Behavioral contract:
/// - Anchored: anchor_relevance weight = 0.15, floor at 0.05.
/// - Local: anchor_relevance = 0.0, redistributed +0.10 to local_activation, +0.05 to novelty.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModeContext {
    pub mode: PerspectiveMode,
    pub anchor_node: Option<String>,
    pub anchor_query: Option<String>,
}

// ---------------------------------------------------------------------------
// Navigation (Theme 5)
// ---------------------------------------------------------------------------

/// A single navigation event in the perspective history.
/// Cap: 1000 per perspective. Compression: keep first 10, last 200, every 10th in between.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NavigationEvent {
    pub action: String, // "start", "follow", "back", "branch", "lens_change"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>, // node external_id for follow, branch name for branch
    pub timestamp_ms: u64,
    pub route_set_version: u64,
}

/// A saved checkpoint for back/restore operations (Theme 11).
/// Mode is stored so back() restores mode from checkpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerspectiveCheckpoint {
    pub focus_node: Option<String>,
    pub lens: PerspectiveLens,
    pub mode: PerspectiveMode,
    pub route_set_version: u64,
    pub timestamp_ms: u64,
}

// ---------------------------------------------------------------------------
// Route types (Theme 4, 12, 13)
// ---------------------------------------------------------------------------

/// A single route in the perspective route set.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    /// Stable content-addressed ID: R_{hash[:6]}. Survives graph rebuild.
    pub route_id: String,
    /// 1-based page-local position.
    pub route_index: u32,
    /// Route family classification.
    pub family: RouteFamily,
    /// Target node external ID.
    pub target_node: String,
    /// Target node label (human-readable).
    pub target_label: String,
    /// One-line reason for this route.
    pub reason: String,
    /// Combined score [0.0, 1.0].
    pub score: f32,
    /// Whether peek is available for this route.
    #[serde(default)]
    pub peek_available: bool,
    /// Provenance information (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<RouteProvenance>,
}

/// Provenance for a route target (relative paths only per Theme 6).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
}

/// Cached route set with version for staleness detection (Theme 1).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedRouteSet {
    pub routes: Vec<Route>,
    pub total_routes: usize,
    pub page_size: u32,
    pub version: u64, // timestamp-based for monotonicity across restarts (Theme 15)
    pub synthesis_elapsed_ms: f64,
    pub captured_cache_generation: u64,
}

// ---------------------------------------------------------------------------
// Diagnostic (Theme 12)
// ---------------------------------------------------------------------------

/// Diagnostic object included when routes/suggest/affinity return empty results.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub sources_checked: Vec<String>,
    pub sources_with_results: Vec<String>,
    pub sources_failed: Vec<String>,
    pub reason: String, // "well_connected" | "under_indexed" | "narrow_lens" | "graph_empty" | "query_mismatch" | "dead_end" | "all_visited"
    pub suggestion: String,
    pub graph_stats: DiagnosticGraphStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagnosticGraphStats {
    pub node_count: u32,
    pub edge_count: u64,
}

// ---------------------------------------------------------------------------
// Affinity candidate (Theme 13)
// ---------------------------------------------------------------------------

/// A hypothesized connection candidate. NEVER phrase as certainty.
/// Max confidence capped at 0.85. Minimum threshold: 0.15.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AffinityCandidate {
    /// Candidate node external ID.
    pub candidate_node: String,
    /// Candidate node label.
    pub candidate_label: String,
    /// Kind of hypothesized connection.
    pub kind: AffinityCandidateKind,
    /// Combined confidence [0.15, 0.85]. Geometric mean of normalized sources.
    pub confidence: f32,
    /// Top-level epistemic guard: always true for affinity candidates.
    pub is_hypothetical: bool,
    /// V1: null. Only inferred from neighbor edge patterns in V2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposed_relation: Option<String>,
    /// Per-source confidence scores.
    pub confidence_breakdown: ConfidenceBreakdown,
}

/// Affinity candidate kind. Uses "hypothesized_" prefix per Theme 13.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffinityCandidateKind {
    HypothesizedLatentEdge,
    MissingBridge,
    ResonantNeighbor,
}

/// Per-source confidence breakdown (Theme 13).
/// All normalized to [0.0, 1.0].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfidenceBreakdown {
    /// Ghost edge: sqrt(raw) to spread biased-low distribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_edge_strength: Option<f32>,
    /// Structural hole: (avg - min_sibling) / (1.0 - min_sibling).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structural_hole_pressure: Option<f32>,
    /// Resonant amplitude: divide by max amplitude in report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resonant_amplitude: Option<f32>,
    /// Semantic overlap: cosine similarity, already [0, 1].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_overlap: Option<f32>,
    /// Provenance: 1.0 same file within 50 lines, 0.5 same file, 0.0 otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_overlap: Option<f32>,
    /// Route-path neighborhood: 1.0 / (1.0 + hop_distance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_path_neighborhood: Option<f32>,
}

// ---------------------------------------------------------------------------
// Suggest types (Theme 12)
// ---------------------------------------------------------------------------

/// Suggestion output from the "next best move" advisor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestResult {
    /// Recommended action (e.g., "follow R_abc123", "inspect R_def456").
    pub recommended_action: String,
    /// Confidence [0.0, 1.0].
    pub confidence: f32,
    /// Why this suggestion.
    pub why: String,
    /// What the suggestion is based on.
    pub based_on: String, // "navigation_history" | "initial_ranking" | "exhaustion_recovery"
    /// Up to 3 alternatives (Theme 5 cap).
    pub alternatives: Vec<SuggestAlternative>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestAlternative {
    pub action: String,
    pub confidence: f32,
    pub why: String,
}

// ---------------------------------------------------------------------------
// PerspectiveState (Theme 2 — the core stateful struct)
// ---------------------------------------------------------------------------

/// Full perspective state for a single agent's navigation session.
/// Storage: `HashMap<(String, String), PerspectiveState>` keyed by (agent_id, perspective_id).
/// Limits: max 5 per agent, max 50MB combined memory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerspectiveState {
    /// Unique perspective ID: `persp_{agent_id_short}_{counter}` (max 20 chars).
    pub perspective_id: String,
    /// Owning agent ID.
    pub agent_id: String,
    /// Current navigation mode.
    pub mode: PerspectiveMode,
    /// Anchor node external ID (None if local mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_node: Option<String>,
    /// Original start query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_query: Option<String>,
    /// Current focus node external ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_node: Option<String>,
    /// Current lens configuration.
    pub lens: PerspectiveLens,
    /// Navigation path (ordered list of visited node external_ids).
    pub entry_path: Vec<String>,
    /// Navigation history (capped at 1000 events).
    pub navigation_history: Vec<NavigationEvent>,
    /// Saved checkpoints for back/restore (capped at 200, LRU eviction).
    pub checkpoints: Vec<PerspectiveCheckpoint>,
    /// Set of visited node external_ids (survives checkpoint compression, Theme 13).
    pub visited_nodes: HashSet<String>,
    /// Cached route set (None until first routes call).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_cache: Option<CachedRouteSet>,
    /// Route set version (timestamp-based for monotonicity, Theme 15).
    pub route_set_version: u64,
    /// Cache generation at time of last route synthesis (Theme 1).
    pub captured_cache_generation: u64,
    /// Whether this perspective is stale (invalidated by rebuild_engines, Theme 16).
    pub stale: bool,
    /// Creation timestamp (u64 epoch-ms, Theme 15).
    pub created_at_ms: u64,
    /// Last access timestamp (u64 epoch-ms).
    pub last_accessed_ms: u64,
    /// Branch names (capped at 10 per agent, Theme 5).
    pub branches: Vec<String>,
}

// ---------------------------------------------------------------------------
// Lock types (Theme 2, 14)
// ---------------------------------------------------------------------------

/// Lock scope configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockScopeConfig {
    pub scope_type: LockScope,
    /// Root nodes for the scope. Non-empty for Subgraph, Node. Resolved for QueryNeighborhood.
    pub root_nodes: Vec<String>,
    /// BFS radius for Subgraph scope. Min 1, max 4. Exactly 0 for Node scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<u32>,
    /// Query string for QueryNeighborhood scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Ordered list of node external_ids for Path scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_nodes: Option<Vec<String>>,
}

/// Snapshot of a subgraph at lock creation time (Theme 4, 14).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockSnapshot {
    /// Set of node external_ids in scope.
    pub nodes: HashSet<String>,
    /// Edges by content-addressable key (Theme 4).
    pub edges: HashMap<String, EdgeSnapshotEntry>,
    /// Graph generation at capture time.
    pub graph_generation: u64,
    /// Capture timestamp (u64 epoch-ms).
    pub captured_at_ms: u64,
    /// Key format version for future migration.
    pub key_format: String, // "v1_content_addr"
}

/// A single edge in a lock baseline snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeSnapshotEntry {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f32,
}

/// Watcher configuration for a lock (Theme 10).
/// One watcher per lock in V1. lock.watch replaces existing strategy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatchConfig {
    pub strategy: WatchStrategy,
    /// Initialized to max(lock.created_at_ms, lock.last_diff_ms).
    pub last_scan_ms: u64,
}

/// Full lock state (Theme 2, 14).
/// Storage: `HashMap<String, LockState>` keyed by lock_id.
/// Limits: max 10 per agent, max 2000 baseline nodes, max 10000 baseline edges.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockState {
    /// Unique lock ID: `lock_{agent_id_short}_{counter}` (max 20 chars).
    pub lock_id: String,
    /// Owning agent ID. Checked on every operation.
    pub agent_id: String,
    /// Scope configuration.
    pub scope: LockScopeConfig,
    /// Baseline snapshot captured at lock creation.
    pub baseline: LockSnapshot,
    /// Optional watcher configuration (Theme 10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watcher: Option<WatchConfig>,
    /// Whether baseline is stale (set by rebuild_engines invalidation, Theme 16).
    pub baseline_stale: bool,
    /// Creation timestamp (u64 epoch-ms).
    pub created_at_ms: u64,
    /// Last diff computation timestamp.
    pub last_diff_ms: u64,
}

/// Pending watcher event (Theme 10).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WatcherEvent {
    pub lock_id: String,
    pub trigger: WatchTrigger,
    pub timestamp_ms: u64,
}

// ---------------------------------------------------------------------------
// Limits (Theme 5)
// ---------------------------------------------------------------------------

/// Hard caps enforced at creation/mutation time. All configurable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerspectiveLimits {
    pub max_perspectives_per_agent: usize,
    pub max_locks_per_agent: usize,
    pub max_branches_per_agent: usize,
    pub max_nav_events_per_perspective: usize,
    pub max_checkpoints_per_perspective: usize,
    pub max_route_set_snapshots: usize,
    pub max_lock_baseline_nodes: usize,
    pub max_lock_baseline_edges: usize,
    pub max_lock_subgraph_radius: u32,
    pub max_affinity_candidates: usize,
    pub max_inspect_chars: usize,
    pub max_suggest_alternatives: usize,
    pub max_compare_chars: usize,
    pub max_lock_diff_new_nodes: usize,
    pub max_lock_diff_new_edges: usize,
    pub max_total_memory_bytes: usize,
}

impl Default for PerspectiveLimits {
    fn default() -> Self {
        Self {
            max_perspectives_per_agent: 5,
            max_locks_per_agent: 10,
            max_branches_per_agent: 10,
            max_nav_events_per_perspective: 1000,
            max_checkpoints_per_perspective: 200,
            max_route_set_snapshots: 10,
            max_lock_baseline_nodes: 2000,
            max_lock_baseline_edges: 10000,
            max_lock_subgraph_radius: 4,
            max_affinity_candidates: 8,
            max_inspect_chars: 1500,
            max_suggest_alternatives: 3,
            max_compare_chars: 3000,
            max_lock_diff_new_nodes: 50,
            max_lock_diff_new_edges: 100,
            max_total_memory_bytes: 50 * 1024 * 1024, // 50MB
        }
    }
}

// ---------------------------------------------------------------------------
// Peek types (Theme 6)
// ---------------------------------------------------------------------------

/// Security-checked peek result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeekContent {
    /// The extracted content (truncated on char boundaries, max 2000 chars).
    pub content: String,
    /// Whether content was truncated.
    pub truncated: bool,
    /// Whether provenance is stale (file newer than graph's last ingest).
    pub provenance_stale: bool,
    /// Whether lossy encoding was applied.
    pub encoding_lossy: bool,
    /// Relative path (ingest root stripped, Theme 6).
    pub relative_path: String,
    /// Line range of the extracted content.
    pub line_start: u32,
    pub line_end: u32,
}

/// Peek security configuration (Theme 6).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeekSecurityConfig {
    /// Allow-list of ingest root paths.
    pub allow_roots: Vec<String>,
    /// Max file size to read (bytes). Default: 10MB.
    pub max_file_size: u64,
    /// Max lines before provenance line.
    pub max_lines_before: u32,
    /// Max lines after provenance line.
    pub max_lines_after: u32,
    /// Max chars in output.
    pub max_chars: usize,
}

impl Default for PeekSecurityConfig {
    fn default() -> Self {
        Self {
            allow_roots: Vec::new(),
            max_file_size: 10 * 1024 * 1024,
            max_lines_before: 20,
            max_lines_after: 30,
            max_chars: 2000,
        }
    }
}

// ---------------------------------------------------------------------------
// Lock diff output (Theme 14)
// ---------------------------------------------------------------------------

/// Result of diffing a lock against current graph state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockDiffResult {
    pub lock_id: String,
    /// If graph_generation unchanged since last diff, this is true and other fields are empty.
    pub no_changes: bool,
    pub new_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub new_edges: Vec<String>, // content-addressable edge keys
    pub removed_edges: Vec<String>,
    /// Edges where one endpoint is in scope and the other is not.
    pub boundary_edges_added: Vec<String>,
    pub boundary_edges_removed: Vec<String>,
    /// Weight changes for existing edges.
    pub weight_changes: Vec<EdgeWeightChange>,
    /// Whether baseline is stale and needs rebase.
    pub baseline_stale: bool,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeWeightChange {
    pub edge_key: String,
    pub old_weight: f32,
    pub new_weight: f32,
}

// ---------------------------------------------------------------------------
// Default helpers
// ---------------------------------------------------------------------------

fn default_dimensions() -> Vec<String> {
    vec![
        "structural".into(),
        "semantic".into(),
        "temporal".into(),
        "causal".into(),
    ]
}

fn default_true() -> bool {
    true
}

fn default_perspective_top_k() -> u32 {
    8
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perspective_lens_defaults_match_spec() {
        let lens = PerspectiveLens::default();
        assert_eq!(lens.dimensions.len(), 4);
        assert!(lens.xlr);
        assert!(lens.include_ghost_edges);
        assert!(lens.include_structural_holes);
        assert_eq!(lens.top_k, 8); // perspective-specific, not activate's 20
        assert!(lens.route_families.is_empty()); // empty = all
    }

    #[test]
    fn ranking_weights_default_sum_to_one() {
        let w = RankingWeights::default();
        let sum = w.local_activation + w.path_coherence + w.novelty + w.anchor_relevance + w.continuity;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn limits_defaults_match_synthesis() {
        let l = PerspectiveLimits::default();
        assert_eq!(l.max_perspectives_per_agent, 5);
        assert_eq!(l.max_locks_per_agent, 10);
        assert_eq!(l.max_lock_baseline_nodes, 2000);
        assert_eq!(l.max_lock_subgraph_radius, 4);
        assert_eq!(l.max_affinity_candidates, 8);
        assert_eq!(l.max_total_memory_bytes, 50 * 1024 * 1024);
    }

    #[test]
    fn perspective_state_serializes_round_trip() {
        let state = PerspectiveState {
            perspective_id: "persp_jimi_001".into(),
            agent_id: "jimi".into(),
            mode: PerspectiveMode::Anchored,
            anchor_node: Some("session.rs".into()),
            anchor_query: Some("session management".into()),
            focus_node: Some("session.rs".into()),
            lens: PerspectiveLens::default(),
            entry_path: vec!["session.rs".into()],
            navigation_history: Vec::new(),
            checkpoints: Vec::new(),
            visited_nodes: HashSet::new(),
            route_cache: None,
            route_set_version: 1710000000000,
            captured_cache_generation: 0,
            stale: false,
            created_at_ms: 1710000000000,
            last_accessed_ms: 1710000000000,
            branches: Vec::new(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PerspectiveState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.perspective_id, "persp_jimi_001");
        assert_eq!(deserialized.mode, PerspectiveMode::Anchored);
    }

    #[test]
    fn lock_state_serializes_round_trip() {
        let lock = LockState {
            lock_id: "lock_jimi_001".into(),
            agent_id: "jimi".into(),
            scope: LockScopeConfig {
                scope_type: LockScope::Node,
                root_nodes: vec!["session.rs".into()],
                radius: None,
                query: None,
                path_nodes: None,
            },
            baseline: LockSnapshot {
                nodes: HashSet::new(),
                edges: HashMap::new(),
                graph_generation: 0,
                captured_at_ms: 1710000000000,
                key_format: "v1_content_addr".into(),
            },
            watcher: None,
            baseline_stale: false,
            created_at_ms: 1710000000000,
            last_diff_ms: 1710000000000,
        };
        let json = serde_json::to_string(&lock).unwrap();
        let deserialized: LockState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.lock_id, "lock_jimi_001");
        assert_eq!(deserialized.scope.scope_type, LockScope::Node);
    }

    #[test]
    fn route_family_ordinal_is_stable() {
        assert_eq!(RouteFamily::Structural.ordinal(), 0);
        assert_eq!(RouteFamily::Resonant.ordinal(), 6);
    }

    #[test]
    fn affinity_candidate_epistemic_guard() {
        let c = AffinityCandidate {
            candidate_node: "foo".into(),
            candidate_label: "Foo".into(),
            kind: AffinityCandidateKind::HypothesizedLatentEdge,
            confidence: 0.42,
            is_hypothetical: true,
            proposed_relation: None,
            confidence_breakdown: ConfidenceBreakdown {
                ghost_edge_strength: Some(0.6),
                structural_hole_pressure: None,
                resonant_amplitude: None,
                semantic_overlap: Some(0.3),
                provenance_overlap: None,
                route_path_neighborhood: None,
            },
        };
        assert!(c.is_hypothetical);
        assert!(c.confidence <= 0.85);
        assert!(c.proposed_relation.is_none()); // V1: always None
    }
}
