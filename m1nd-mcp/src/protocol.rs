// === crates/m1nd-mcp/src/protocol.rs ===

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// JSON-RPC transport types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// MCP tool input types (03-MCP Section 2 tool schemas)
// ---------------------------------------------------------------------------

/// Input for m1nd.activate (03-MCP Section 2.1).
#[derive(Clone, Debug, Deserialize)]
pub struct ActivateInput {
    pub query: String,
    pub agent_id: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_dimensions")]
    pub dimensions: Vec<String>,
    #[serde(default = "default_true")]
    pub xlr: bool,
    #[serde(default = "default_true")]
    pub include_ghost_edges: bool,
    #[serde(default)]
    pub include_structural_holes: bool,
}

/// Input for m1nd.impact (03-MCP Section 2.2).
#[derive(Clone, Debug, Deserialize)]
pub struct ImpactInput {
    pub node_id: String,
    pub agent_id: String,
    #[serde(default = "default_forward")]
    pub direction: String,
    #[serde(default = "default_true")]
    pub include_causal_chains: bool,
}

/// Input for m1nd.missing (03-MCP Section 2.3).
#[derive(Clone, Debug, Deserialize)]
pub struct MissingInput {
    pub query: String,
    pub agent_id: String,
    #[serde(default = "default_min_sibling")]
    pub min_sibling_activation: f32,
}

/// Input for m1nd.why (03-MCP Section 2.4).
#[derive(Clone, Debug, Deserialize)]
pub struct WhyInput {
    pub source: String,
    pub target: String,
    pub agent_id: String,
    #[serde(default = "default_max_hops")]
    pub max_hops: u8,
}

/// Input for m1nd.warmup (03-MCP Section 2.5).
#[derive(Clone, Debug, Deserialize)]
pub struct WarmupInput {
    pub task_description: String,
    pub agent_id: String,
    #[serde(default = "default_boost")]
    pub boost_strength: f32,
}

/// Input for m1nd.counterfactual (03-MCP Section 2.6).
#[derive(Clone, Debug, Deserialize)]
pub struct CounterfactualInput {
    pub node_ids: Vec<String>,
    pub agent_id: String,
    #[serde(default = "default_true")]
    pub include_cascade: bool,
}

/// Input for m1nd.predict (03-MCP Section 2.7).
#[derive(Clone, Debug, Deserialize)]
pub struct PredictInput {
    pub changed_node: String,
    pub agent_id: String,
    #[serde(default = "default_top_k_10")]
    pub top_k: usize,
    #[serde(default = "default_true")]
    pub include_velocity: bool,
}

/// Input for m1nd.fingerprint (03-MCP Section 2.8).
#[derive(Clone, Debug, Deserialize)]
pub struct FingerprintInput {
    pub target_node: Option<String>,
    pub agent_id: String,
    #[serde(default = "default_similarity")]
    pub similarity_threshold: f32,
    pub probe_queries: Option<Vec<String>>,
}

/// Input for m1nd.drift (03-MCP Section 2.9).
#[derive(Clone, Debug, Deserialize)]
pub struct DriftInput {
    pub agent_id: String,
    #[serde(default = "default_last_session")]
    pub since: String,
    #[serde(default = "default_true")]
    pub include_weight_drift: bool,
}

/// Input for m1nd.learn (03-MCP Section 2.10).
#[derive(Clone, Debug, Deserialize)]
pub struct LearnInput {
    pub query: String,
    pub agent_id: String,
    pub feedback: String,  // "correct", "wrong", or "partial"
    pub node_ids: Vec<String>,
    #[serde(default = "default_feedback_strength")]
    pub strength: f32,
}

/// Input for m1nd.ingest (03-MCP Section 2.11).
#[derive(Clone, Debug, Deserialize)]
pub struct IngestInput {
    pub path: String,
    pub agent_id: String,
    #[serde(default)]
    pub incremental: bool,
    /// Adapter to use: "code" (default), "json", or future adapters.
    #[serde(default = "default_adapter")]
    pub adapter: String,
}

/// Input for m1nd.resonate (resonance analysis).
#[derive(Clone, Debug, Deserialize)]
pub struct ResonateInput {
    pub query: Option<String>,
    pub node_id: Option<String>,
    pub agent_id: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

/// Input for m1nd.health (03-MCP Section 2.12).
#[derive(Clone, Debug, Deserialize)]
pub struct HealthInput {
    pub agent_id: String,
}

// ---------------------------------------------------------------------------
// Default value helpers
// ---------------------------------------------------------------------------

fn default_top_k() -> usize { 20 }
fn default_top_k_10() -> usize { 10 }
fn default_dimensions() -> Vec<String> {
    vec!["structural".into(), "semantic".into(), "temporal".into(), "causal".into()]
}
fn default_true() -> bool { true }
fn default_forward() -> String { "forward".into() }
fn default_min_sibling() -> f32 { 0.3 }
fn default_max_hops() -> u8 { 6 }
fn default_boost() -> f32 { 0.15 }
fn default_similarity() -> f32 { 0.85 }
fn default_last_session() -> String { "last_session".into() }
fn default_feedback_strength() -> f32 { 0.2 }
fn default_adapter() -> String { "code".into() }

// ---------------------------------------------------------------------------
// MCP tool output types (03-MCP Section 2 output schemas)
// All output types are Serialize for JSON-RPC response.
// ---------------------------------------------------------------------------

/// Output for m1nd.activate.
#[derive(Clone, Debug, Serialize)]
pub struct ActivateOutput {
    pub query: String,
    pub seeds: Vec<SeedOutput>,
    pub activated: Vec<ActivatedNodeOutput>,
    pub ghost_edges: Vec<GhostEdgeOutput>,
    pub structural_holes: Vec<StructuralHoleOutput>,
    pub plasticity: PlasticityOutput,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SeedOutput {
    pub node_id: String,
    pub label: String,
    pub relevance: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActivatedNodeOutput {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub activation: f32,
    pub dimensions: DimensionsOutput,
    pub pagerank: f32,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DimensionsOutput {
    pub structural: f32,
    pub semantic: f32,
    pub temporal: f32,
    pub causal: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct GhostEdgeOutput {
    pub source: String,
    pub target: String,
    pub shared_dimensions: Vec<String>,
    pub strength: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct StructuralHoleOutput {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlasticityOutput {
    pub edges_strengthened: u32,
    pub edges_decayed: u32,
    pub ltp_events: u32,
    pub priming_nodes: u32,
}

/// Output for m1nd.impact.
#[derive(Clone, Debug, Serialize)]
pub struct ImpactOutput {
    pub source: String,
    pub source_label: String,
    pub direction: String,
    pub blast_radius: Vec<BlastRadiusEntry>,
    pub total_energy: f32,
    pub max_hops_reached: u8,
    pub causal_chains: Vec<CausalChainOutput>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlastRadiusEntry {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub signal_strength: f32,
    pub hop_distance: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct CausalChainOutput {
    pub path: Vec<String>,
    pub relations: Vec<String>,
    pub cumulative_strength: f32,
}

/// Output for m1nd.health.
#[derive(Clone, Debug, Serialize)]
pub struct HealthOutput {
    pub status: String,
    pub node_count: u32,
    pub edge_count: u64,
    pub queries_processed: u64,
    pub uptime_seconds: f64,
    pub memory_usage_bytes: u64,
    pub plasticity_state: String,
    pub last_persist_time: Option<String>,
    pub active_sessions: Vec<serde_json::Value>,
}
