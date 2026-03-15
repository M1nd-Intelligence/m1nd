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

fn default_radius() -> u32 {
    1
}
fn default_true() -> bool {
    true
}

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

// ---------------------------------------------------------------------------
// m1nd.surgical_context_v2
// ---------------------------------------------------------------------------

/// Input for m1nd.surgical_context_v2.
///
/// Extended version that also fetches source code for each connected file
/// (callers, callees, tests), respects per-file line caps, and returns
/// total_lines for context budget management.
#[derive(Clone, Debug, Deserialize)]
pub struct SurgicalContextV2Input {
    /// Absolute or workspace-relative path to the target file.
    pub file_path: String,
    /// Calling agent identifier.
    pub agent_id: String,
    /// Optional: narrow to a specific symbol within the file.
    #[serde(default)]
    pub symbol: Option<String>,
    /// BFS radius for graph neighbourhood. Default: 1.
    #[serde(default = "default_radius")]
    pub radius: u32,
    /// Include test files in the neighbourhood. Default: true.
    #[serde(default = "default_true")]
    pub include_tests: bool,
    /// Maximum number of connected files to include source for. Default: 5.
    #[serde(default = "default_max_connected_files")]
    pub max_connected_files: usize,
    /// Maximum lines to return per connected file. Default: 60.
    #[serde(default = "default_max_lines_per_file")]
    pub max_lines_per_file: usize,
}

fn default_max_connected_files() -> usize {
    5
}
fn default_max_lines_per_file() -> usize {
    60
}

/// Source excerpt for a connected file in v2 context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectedFileSource {
    /// Graph node ID for this connected file.
    pub node_id: String,
    /// Human-readable label.
    pub label: String,
    /// Absolute path to the file.
    pub file_path: String,
    /// How this file relates to the target: "caller", "callee", or "test".
    pub relation_type: String,
    /// Edge weight from the graph.
    pub edge_weight: f32,
    /// Source excerpt (up to max_lines_per_file lines).
    pub source_excerpt: String,
    /// Number of lines in the excerpt.
    pub excerpt_lines: usize,
    /// True when the file had more lines than max_lines_per_file.
    pub truncated: bool,
}

/// Output for m1nd.surgical_context_v2.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalContextV2Output {
    /// Absolute path of the target file (resolved).
    pub file_path: String,
    /// Full contents of the target file.
    pub file_contents: String,
    /// Total lines in the target file.
    pub line_count: u32,
    /// Graph node ID for the target file.
    pub node_id: String,
    /// Symbols defined in the target file.
    pub symbols: Vec<SurgicalSymbol>,
    /// Focused symbol (when `symbol` input provided).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_symbol: Option<SurgicalSymbol>,
    /// Connected files with source excerpts (callers + callees + tests combined,
    /// capped at max_connected_files, ordered by edge_weight descending).
    pub connected_files: Vec<ConnectedFileSource>,
    /// Sum of all lines returned: line_count + sum(excerpt_lines).
    pub total_lines: usize,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}

// ---------------------------------------------------------------------------
// m1nd.apply_batch
// ---------------------------------------------------------------------------

/// A single file edit within an apply_batch request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchEditItem {
    /// Absolute or workspace-relative path of the file to write.
    pub file_path: String,
    /// New full contents for the file (UTF-8).
    pub new_content: String,
    /// Optional description for the apply log.
    #[serde(default)]
    pub description: Option<String>,
}

/// Per-file result within an apply_batch response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchEditResult {
    /// Absolute path that was written (or attempted).
    pub file_path: String,
    /// True when this specific file was written successfully.
    pub success: bool,
    /// Unified diff for this file.
    pub diff: String,
    /// Lines added in this file.
    pub lines_added: i32,
    /// Lines removed in this file.
    pub lines_removed: i32,
    /// Failure reason when success=false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Input for m1nd.apply_batch.
///
/// Writes multiple files atomically: either ALL succeed or NONE are written
/// (rollback on partial failure when atomic=true).
/// A single incremental re-ingest covers all modified files.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplyBatchInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Files to write. Empty list is a no-op (returns success immediately).
    pub edits: Vec<BatchEditItem>,
    /// When true (default), abort and rollback all writes if any single file fails.
    #[serde(default = "default_true")]
    pub atomic: bool,
    /// Re-ingest all modified files after writing. Default: true.
    #[serde(default = "default_true")]
    pub reingest: bool,
}

/// Output for m1nd.apply_batch.
#[derive(Clone, Debug, Serialize)]
pub struct ApplyBatchOutput {
    /// True when all files were written successfully.
    pub all_succeeded: bool,
    /// Number of files successfully written.
    pub files_written: usize,
    /// Total files attempted.
    pub files_total: usize,
    /// Per-file results (one entry per input edit, in input order).
    pub results: Vec<BatchEditResult>,
    /// Whether a re-ingest was triggered (single pass covering all files).
    pub reingested: bool,
    /// Total bytes written across all files.
    pub total_bytes_written: usize,
    /// Elapsed milliseconds.
    pub elapsed_ms: f64,
}
