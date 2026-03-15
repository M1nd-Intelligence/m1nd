// === m1nd-mcp HTTP request/response types ===
//
// Shared types for the axum HTTP server. Feature-gated behind "serve".

#![cfg(feature = "serve")]

use serde::{Deserialize, Serialize};

/// Health response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_secs: f64,
    pub node_count: usize,
    pub edge_count: usize,
    pub queries_processed: u64,
    pub agent_sessions: Vec<serde_json::Value>,
    pub domain: String,
    pub graph_generation: u64,
    pub plasticity_generation: u64,
}

/// Graph stats response.
#[derive(Serialize)]
pub struct GraphStatsResponse {
    pub node_count: usize,
    pub edge_count: usize,
    pub domain: String,
    pub namespaces: Vec<String>,
    pub memory_estimate_bytes: usize,
}

/// Subgraph node for React Flow.
#[derive(Serialize)]
pub struct SubgraphNode {
    pub id: String,
    pub label: String,
    pub node_type: u8,
    pub activation: f32,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagerank: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<f32>,
}

/// Subgraph edge for React Flow.
#[derive(Serialize)]
pub struct SubgraphEdge {
    pub source: String,
    pub target: String,
    pub weight: f32,
    pub relation: String,
}

/// Subgraph response.
#[derive(Serialize)]
pub struct SubgraphResponse {
    pub nodes: Vec<SubgraphNode>,
    pub edges: Vec<SubgraphEdge>,
    pub meta: SubgraphMeta,
}

/// Subgraph metadata.
#[derive(Serialize)]
pub struct SubgraphMeta {
    pub total_nodes: usize,
    pub rendered_nodes: usize,
    pub query: String,
    pub elapsed_ms: u64,
}

/// Error response for HTTP API.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub detail: String,
}

/// Tool list response.
#[derive(Serialize)]
pub struct ToolListResponse {
    pub tools: Vec<serde_json::Value>,
}

/// Query params for subgraph endpoint.
#[derive(Deserialize)]
pub struct SubgraphQuery {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_depth")]
    pub depth: usize,
}

fn default_top_k() -> usize {
    30
}
fn default_depth() -> usize {
    2
}

impl SubgraphQuery {
    /// Cap top_k to maximum allowed value (100).
    pub fn clamped_top_k(&self) -> usize {
        self.top_k.min(100)
    }
}
