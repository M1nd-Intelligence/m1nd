// === m1nd-mcp/src/protocol/surgical.rs ===
//
// Input/Output types for m1nd.surgical_context and m1nd.apply.
//
// Conventions (matching core.rs / layers.rs / perspective.rs):
//   - Input:  #[derive(Clone, Debug, Deserialize)]
//   - Output: #[derive(Clone, Debug, Serialize)]
//   - All inputs require `agent_id: String`
//   - Optional params use Option<T> or serde default helpers

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// m1nd.surgical_context
// ---------------------------------------------------------------------------

/// Input for m1nd.surgical_context.
///
/// Returns everything needed to surgically edit a single file:
/// file contents + graph neighbourhood + provenance.
#[derive(Clone, Debug, Deserialize)]
pub struct SurgicalContextInput {
    /// Absolute or workspace-relative path to the file being edited.
    pub file_path: String,
    /// Calling agent identifier (required by all m1nd tools).
    pub agent_id: String,
    /// Optional: narrow context to a specific symbol (function / struct / class name).
    /// When provided, only the symbol's line range + its direct neighbours are returned.
    #[serde(default)]
    pub symbol: Option<String>,
    /// BFS radius for graph neighbourhood. Default: 1.
    #[serde(default = "default_radius")]
    pub radius: u32,
    /// Include test files in the neighbourhood. Default: true.
    #[serde(default = "default_true")]
    pub include_tests: bool,
}

fn default_radius() -> u32 { 1 }
fn default_true() -> bool { true }

/// Output for m1nd.surgical_context.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalContextOutput {
    /// Absolute path of the file (resolved).
    pub file_path: String,
    /// Full contents of the file as a UTF-8 string.
    pub file_contents: String,
    /// Total number of lines in the file.
    pub line_count: u32,
    /// Graph node ID for this file (empty string if not yet ingested).
    pub node_id: String,
    /// Symbols defined in this file with their line ranges.
    pub symbols: Vec<SurgicalSymbol>,
    /// Focused symbol details (populated when `symbol` input is given).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_symbol: Option<SurgicalSymbol>,
    /// Neighbourhood: files / modules that call into this file.
    pub callers: Vec<SurgicalNeighbour>,
    /// Neighbourhood: files / modules this file calls into.
    pub callees: Vec<SurgicalNeighbour>,
    /// Neighbourhood: test files that cover this file.
    pub tests: Vec<SurgicalNeighbour>,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}

/// A symbol (function, struct, class, etc.) within the file.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalSymbol {
    pub name: String,
    #[serde(rename = "type")]
    pub symbol_type: String,
    pub line_start: u32,
    pub line_end: u32,
    /// Excerpt of the symbol's source (first 20 lines max).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

/// A neighbouring node in the graph.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalNeighbour {
    pub node_id: String,
    pub label: String,
    pub file_path: String,
    pub relation: String,
    pub edge_weight: f32,
}

// ---------------------------------------------------------------------------
// m1nd.apply
// ---------------------------------------------------------------------------

/// Input for m1nd.apply.
///
/// Writes new file contents to disk and triggers an incremental re-ingest
/// so the graph stays coherent with the updated source.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplyInput {
    /// Absolute or workspace-relative path of the file to overwrite.
    pub file_path: String,
    /// Calling agent identifier.
    pub agent_id: String,
    /// New file contents (full replacement, UTF-8).
    pub new_content: String,
    /// Human-readable description of the edit (used in the apply log).
    #[serde(default)]
    pub description: Option<String>,
    /// Re-ingest after writing. Default: true.
    #[serde(default = "default_true")]
    pub reingest: bool,
}

/// Output for m1nd.apply.
#[derive(Clone, Debug, Serialize)]
pub struct ApplyOutput {
    /// Absolute path that was written.
    pub file_path: String,
    /// Number of bytes written.
    pub bytes_written: usize,
    /// Lines added (unified diff summary).
    pub lines_added: i32,
    /// Lines removed (unified diff summary).
    pub lines_removed: i32,
    /// Whether an incremental re-ingest was triggered.
    pub reingested: bool,
    /// Node IDs that were updated or added during re-ingest.
    pub updated_node_ids: Vec<String>,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}
