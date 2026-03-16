// === crates/m1nd-core/src/error.rs ===

use crate::types::{EdgeIdx, Generation, NodeId};

/// Central error type covering all failure modes from 05-HARDENING-SYNTHESIS.
/// Each variant references its FM-ID for traceability.
#[derive(Debug, thiserror::Error)]
pub enum M1ndError {
    // --- Graph integrity ---
    /// FM-ACT-011: Edge references a node index that does not exist.
    #[error("dangling edge: edge {edge:?} references non-existent node {node:?}")]
    DanglingEdge { edge: EdgeIdx, node: NodeId },

    /// FM-PL-006: Graph structure changed since engine was initialised.
    #[error("graph generation mismatch: expected {expected:?}, actual {actual:?}")]
    GraphGenerationMismatch {
        expected: Generation,
        actual: Generation,
    },

    /// FM-ACT-016: Attempted to add a node whose interned ID already exists.
    #[error("duplicate node: interned ID {0:?}")]
    DuplicateNode(NodeId),

    /// Graph not finalised — CSR not built yet.
    #[error("graph not finalised: call Graph::finalize() before queries")]
    GraphNotFinalized,

    /// Graph is empty (zero nodes).
    #[error("graph is empty")]
    EmptyGraph,

    // --- Numerical safety ---
    /// FM-PL-001: Non-finite value detected at a NaN firewall boundary.
    #[error("non-finite value at firewall: node={node:?}, value={value}")]
    NonFiniteActivation { node: NodeId, value: f32 },

    /// FM-ACT-012: A tuneable parameter is outside its valid range.
    #[error("parameter out of range: {name} = {value} (expected {range})")]
    ParameterOutOfRange {
        name: &'static str,
        value: f64,
        range: &'static str,
    },

    /// FM-RES-001: Zero or negative wavelength/frequency supplied.
    #[error("non-positive resonance parameter: {name} = {value}")]
    NonPositiveResonanceParam { name: &'static str, value: f32 },

    // --- Resource exhaustion ---
    /// FM-RES-004: Pulse propagation exceeded budget.
    #[error("pulse budget exhausted: {budget} pulses processed")]
    PulseBudgetExhausted { budget: u64 },

    /// FM-TMP-005: Causal chain DFS exceeded budget.
    #[error("chain budget exhausted: {budget} chains generated")]
    ChainBudgetExhausted { budget: u64 },

    /// FM-TMP-001: Co-change sparse matrix exceeded entry budget.
    #[error("matrix entry budget exhausted: {budget} entries")]
    MatrixBudgetExhausted { budget: u64 },

    /// FM-ING-002: Ingestion exceeded timeout.
    #[error("ingestion timeout after {elapsed_s:.1}s")]
    IngestionTimeout { elapsed_s: f64 },

    /// FM-ING-002: Ingestion exceeded node count budget.
    #[error("ingestion node budget exhausted: {budget} nodes")]
    IngestionNodeBudget { budget: u64 },

    /// FM-TOP-014: Fingerprint pair budget exceeded.
    #[error("fingerprint pair budget exhausted: {budget} pairs")]
    FingerprintPairBudget { budget: u64 },

    // --- Analysis quality ---
    /// FM-XLR-010: XLR cancelled all signal — fallback to hot-only.
    #[error("XLR over-cancellation: all signal cancelled")]
    XlrOverCancellation,

    /// FM-TOP-003: Louvain community detection did not converge.
    #[error("Louvain non-convergence after {passes} passes")]
    LouvainNonConvergence { passes: u32 },

    /// FM-TOP-010: Power iteration may have diverged.
    #[error("spectral analysis: power iteration divergence suspected")]
    SpectralDivergence,

    /// FM-RES-020: Division by zero in normalization (max_amp == 0).
    #[error("resonance normalization: max amplitude is zero")]
    ResonanceZeroAmplitude,

    /// FM-ACT-019: Atomic CAS retry limit exceeded during concurrent weight update.
    #[error("CAS retry limit ({limit}) exceeded at edge {edge:?}")]
    CasRetryExhausted { edge: EdgeIdx, limit: u32 },

    // --- Ingestion ---
    /// FM-ING-003: File encoding could not be determined.
    #[error("encoding detection failed for {path} (confidence={confidence:.2})")]
    EncodingDetectionFailed { path: String, confidence: f32 },

    /// FM-ING-004: Binary file detected and skipped.
    #[error("binary file skipped: {path}")]
    BinaryFileSkipped { path: String },

    /// FM-ING-008: Label collision — multiple nodes share a label.
    #[error("label collision: {label} maps to {count} nodes")]
    LabelCollision { label: String, count: usize },

    // --- Persistence ---
    /// FM-PL-007: Corrupt state file on load.
    #[error("corrupt persistence state: {reason}")]
    CorruptState { reason: String },

    /// FM-PL-009: Schema drift — edge identity mismatch on import.
    #[error("schema drift on import: {reason}")]
    SchemaDrift { reason: String },

    // --- Counterfactual ---
    /// FM-CF-001: Seed node was in the removal set.
    #[error("counterfactual seed overlap: seed {node:?} is in the removal set")]
    CounterfactualSeedOverlap { node: NodeId },

    // --- Perspective / Lock / Navigation (12-PERSPECTIVE-SYNTHESIS Theme 3) ---
    /// Theme 3: Unknown tool name in dispatch.
    #[error("unknown tool: {name}")]
    UnknownTool { name: String },

    /// Theme 3: Invalid parameters for a tool call.
    #[error("invalid params for {tool}: {detail}")]
    InvalidParams { tool: String, detail: String },

    /// Theme 3: Perspective does not exist for agent.
    #[error("perspective not found: {perspective_id} for agent {agent_id}")]
    PerspectiveNotFound {
        perspective_id: String,
        agent_id: String,
    },

    /// Theme 3: Perspective route set is stale (generation mismatch).
    #[error(
        "perspective stale: {perspective_id} expected gen {expected_gen}, actual {actual_gen}"
    )]
    PerspectiveStale {
        perspective_id: String,
        expected_gen: u64,
        actual_gen: u64,
    },

    /// Theme 3: Agent exceeded max perspective count.
    #[error("perspective limit exceeded for agent {agent_id}: {current}/{limit}")]
    PerspectiveLimitExceeded {
        agent_id: String,
        current: usize,
        limit: usize,
    },

    /// Theme 3: Route set version mismatch (stale cached routes).
    #[error("route set stale: version {route_set_version}, current {current_version}")]
    RouteSetStale {
        route_set_version: u64,
        current_version: u64,
    },

    /// Theme 3: Route not found in perspective.
    #[error("route not found: {route_id} in perspective {perspective_id}")]
    RouteNotFound {
        route_id: String,
        perspective_id: String,
    },

    /// Theme 3: Cannot navigate back — already at root.
    #[error("navigation at root: perspective {perspective_id}")]
    NavigationAtRoot { perspective_id: String },

    /// Theme 3: Branch depth limit exceeded.
    #[error("branch depth exceeded in {perspective_id}: depth {depth}/{limit}")]
    BranchDepthExceeded {
        perspective_id: String,
        depth: usize,
        limit: usize,
    },

    /// Theme 3: Lock not found.
    #[error("lock not found: {lock_id}")]
    LockNotFound { lock_id: String },

    /// Theme 3: Lock ownership violation.
    #[error("lock ownership violation: {lock_id} owned by {owner}, called by {caller}")]
    LockOwnership {
        lock_id: String,
        owner: String,
        caller: String,
    },

    /// Theme 3: Lock scope too large (BFS budget exceeded).
    #[error("lock scope too large: {node_count} nodes exceeds cap of {cap}")]
    LockScopeTooLarge { node_count: usize, cap: usize },

    /// Theme 3: Agent exceeded max lock count.
    #[error("lock limit exceeded for agent {agent_id}: {current}/{limit}")]
    LockLimitExceeded {
        agent_id: String,
        current: usize,
        limit: usize,
    },

    /// Theme 3: Watcher strategy not supported (e.g. Periodic in V1).
    #[error("watch strategy not supported: {strategy}")]
    WatchStrategyNotSupported { strategy: String },

    /// Theme 3: Affinity computation exceeded time budget.
    #[error("affinity timeout: {elapsed_ms:.1}ms exceeded budget of {budget_ms:.1}ms")]
    AffinityTimeout { elapsed_ms: f64, budget_ms: f64 },

    // --- Antibody ---
    /// FM-AB-001: Antibody pattern specificity below minimum threshold.
    #[error("pattern too broad: specificity {specificity:.2} below minimum {minimum:.2}")]
    PatternTooBroad { specificity: f32, minimum: f32 },

    /// Antibody not found by ID.
    #[error("antibody not found: {id}")]
    AntibodyNotFound { id: String },

    /// Antibody storage limit exceeded.
    #[error("antibody limit exceeded: {current}/{limit}")]
    AntibodyLimitExceeded { current: usize, limit: usize },

    // --- Epidemic ---
    /// Epidemic burnout: too many nodes infected too fast.
    #[error("epidemic burnout: {infected_pct:.1}% infected in {iteration} iterations")]
    EpidemicBurnout { infected_pct: f32, iteration: u32 },

    /// No valid infected nodes provided for epidemic simulation.
    #[error("no valid infected nodes")]
    NoValidInfectedNodes,

    // --- Flow ---
    /// No entry points found for flow simulation.
    #[error("no entry points found for flow simulation")]
    NoEntryPoints,

    // --- Layers ---
    /// Layer level not found in detection result.
    #[error("layer not found: level {level}")]
    LayerNotFound { level: u8 },

    // --- Ingestion (runtime) ---
    /// Tree-sitter or extractor runtime error.
    #[error("ingest error: {0}")]
    IngestError(String),

    // --- I/O ---
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("persistence failed: {0}")]
    PersistenceFailed(String),
}

/// Convenience alias used throughout the crate.
pub type M1ndResult<T> = Result<T, M1ndError>;
