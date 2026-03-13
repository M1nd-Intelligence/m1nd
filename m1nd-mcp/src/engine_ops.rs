// === m1nd-mcp/src/engine_ops.rs ===
// Theme 7: Route Synthesis Engine API Layer.
// Read-only wrappers around engine operations for perspective synthesis.
// These take &SessionState (immutable), do NOT increment queries_processed,
// do NOT trigger plasticity side effects, hold a single graph read lock.

use m1nd_core::error::M1ndResult;
use crate::session::SessionState;

// ---------------------------------------------------------------------------
// Config types
// ---------------------------------------------------------------------------

/// Configuration for read-only activate.
#[derive(Clone, Debug)]
pub struct ActivateConfig {
    pub top_k: usize,
    pub dimensions: Vec<String>,
    pub xlr: bool,
    pub include_ghost_edges: bool,
    pub include_structural_holes: bool,
}

impl Default for ActivateConfig {
    fn default() -> Self {
        Self {
            top_k: 8, // perspective default, not activate's 20
            dimensions: vec![
                "structural".into(),
                "semantic".into(),
                "temporal".into(),
                "causal".into(),
            ],
            xlr: true,
            include_ghost_edges: true,
            include_structural_holes: true,
        }
    }
}

/// Direction for impact analysis.
#[derive(Clone, Debug)]
pub enum ImpactDirection {
    Forward,
    Reverse,
    Both,
}

// ---------------------------------------------------------------------------
// Result types (lightweight, perspective-internal)
// ---------------------------------------------------------------------------

/// Activated node result for perspective synthesis.
#[derive(Clone, Debug)]
pub struct ActivatedNode {
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub activation: f32,
    pub pagerank: f32,
    pub source_path: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

/// Read-only activate result.
#[derive(Clone, Debug)]
pub struct ActivateResult {
    pub nodes: Vec<ActivatedNode>,
    pub ghost_edges: Vec<GhostEdge>,
    pub structural_holes: Vec<StructuralHole>,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug)]
pub struct GhostEdge {
    pub source: String,
    pub target: String,
    pub strength: f32,
}

#[derive(Clone, Debug)]
pub struct StructuralHole {
    pub node_id: String,
    pub label: String,
    pub node_type: String,
    pub reason: String,
}

/// Read-only impact result.
#[derive(Clone, Debug)]
pub struct ImpactResult {
    pub blast_radius: Vec<ImpactEntry>,
    pub total_energy: f32,
}

#[derive(Clone, Debug)]
pub struct ImpactEntry {
    pub node_id: String,
    pub label: String,
    pub signal_strength: f32,
    pub hop_distance: u8,
}

/// Read-only missing result.
#[derive(Clone, Debug)]
pub struct MissingResult {
    pub holes: Vec<StructuralHole>,
}

/// Read-only why result.
#[derive(Clone, Debug)]
pub struct WhyResult {
    pub paths: Vec<WhyPath>,
}

#[derive(Clone, Debug)]
pub struct WhyPath {
    pub nodes: Vec<String>,
    pub relations: Vec<String>,
    pub cumulative_strength: f32,
}

/// Read-only resonate result.
#[derive(Clone, Debug)]
pub struct ResonateResult {
    pub harmonics: Vec<HarmonicEntry>,
}

#[derive(Clone, Debug)]
pub struct HarmonicEntry {
    pub node_id: String,
    pub label: String,
    pub amplitude: f32,
}

// ---------------------------------------------------------------------------
// Read-only engine operations (Theme 7)
// ---------------------------------------------------------------------------

/// Read-only activate. No query counting, no plasticity side effects.
/// Holds a single graph read lock for the entire call.
///
/// Performance budget for route synthesis: max 8 underlying calls total.
/// This counts as 1 call.
pub fn activate_readonly(
    state: &SessionState,
    query: &str,
    config: ActivateConfig,
) -> M1ndResult<ActivateResult> {
    // TODO: Extract computation logic from handle_activate in tools.rs
    // into shared functions. Call the shared function here with &graph (read lock).
    // Do NOT call handle_activate directly — it has side effects.
    todo!("activate_readonly: extract from tools::handle_activate")
}

/// Read-only impact. No side effects.
pub fn impact_readonly(
    state: &SessionState,
    node: &str,
    direction: ImpactDirection,
) -> M1ndResult<ImpactResult> {
    todo!("impact_readonly: extract from tools::handle_impact")
}

/// Read-only missing. No side effects.
pub fn missing_readonly(
    state: &SessionState,
    query: &str,
) -> M1ndResult<MissingResult> {
    todo!("missing_readonly: extract from tools::handle_missing")
}

/// Read-only why. No side effects.
pub fn why_readonly(
    state: &SessionState,
    from: &str,
    to: &str,
) -> M1ndResult<WhyResult> {
    todo!("why_readonly: extract from tools::handle_why")
}

/// Read-only resonate. Only called if remaining budget allows after first 8 calls.
pub fn resonate_readonly(
    state: &SessionState,
    query: &str,
) -> M1ndResult<ResonateResult> {
    todo!("resonate_readonly: extract from tools::handle_resonate")
}

// ---------------------------------------------------------------------------
// Budget tracking for route synthesis
// ---------------------------------------------------------------------------

/// Track call budget during route synthesis.
/// Max 8 calls (1 activate + 3 impact + 3 why + 1 missing).
/// Wall-clock timeout: 500ms.
pub struct SynthesisBudget {
    pub max_calls: u32,
    pub calls_used: u32,
    pub timeout_ms: f64,
    pub start_time: std::time::Instant,
}

impl SynthesisBudget {
    pub fn new() -> Self {
        Self {
            max_calls: 8,
            calls_used: 0,
            timeout_ms: 500.0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Check if another call is within budget.
    pub fn can_call(&self) -> bool {
        self.calls_used < self.max_calls && self.elapsed_ms() < self.timeout_ms
    }

    /// Record a call.
    pub fn record_call(&mut self) {
        self.calls_used += 1;
    }

    /// Elapsed time in ms.
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64() * 1000.0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_starts_with_capacity() {
        let budget = SynthesisBudget::new();
        assert!(budget.can_call());
        assert_eq!(budget.calls_used, 0);
        assert_eq!(budget.max_calls, 8);
    }

    #[test]
    fn budget_exhausts() {
        let mut budget = SynthesisBudget::new();
        for _ in 0..8 {
            assert!(budget.can_call());
            budget.record_call();
        }
        assert!(!budget.can_call());
    }

    #[test]
    fn activate_config_defaults() {
        let config = ActivateConfig::default();
        assert_eq!(config.top_k, 8); // perspective-specific
        assert_eq!(config.dimensions.len(), 4);
    }
}
