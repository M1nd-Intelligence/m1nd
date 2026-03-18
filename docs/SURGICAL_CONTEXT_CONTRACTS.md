# SURGICAL_CONTEXT + APPLY — Contracts (Step 6)

> TEMPESTA Step 6 output. Compilable Rust structs, JSON schemas, error types, dispatch integration.

---

## Step 5 Decisions (recorded)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **File location** | New file `m1nd-mcp/src/surgical_handlers.rs` | Follows `perspective_handlers.rs`, `lock_handlers.rs`, `layer_handlers.rs` pattern. One handler file per tool family. |
| **Protocol types** | New file `m1nd-mcp/src/protocol/surgical.rs` | Follows `protocol/perspective.rs`, `protocol/lock.rs`, `protocol/layers.rs` pattern. Add `pub mod surgical;` to `protocol/mod.rs`. |
| **Dispatch routing** | New prefix match `"m1nd.surgical."` in `dispatch_tool()` (server.rs) | Same pattern as `"m1nd.perspective."` and `"m1nd.lock."`. Delegates to `dispatch_surgical_tool()`. |
| **Apply write strategy** | `std::fs::read_to_string` + split lines + replace range + `std::fs::write` | Simple, atomic at filesystem level. No partial writes. |
| **Apply re-ingest** | Call `tools::handle_ingest()` internally on single file after write | Reuse existing ingest pipeline. `incremental: false`, `mode: "merge"`, `path: <file>`. |
| **Path validation** | Reuse `peek_security.rs` allow-list (canonicalize + `validate_allow_list`) | Apply is a superset of peek permissions -- if you can peek it, you can apply to it. Reject symlink escapes, binaries, non-existent parents. |
| **Parallel vs sequential in surgical_context** | Sequential calls, no `tokio::join!` | Server is synchronous (`&mut SessionState`). Impact/predict/antibody/trust are all CPU-bound on the same graph. No benefit from parallelism. Sequential also avoids borrow checker issues with `&mut`. |

---

## Protocol Types — `m1nd-mcp/src/protocol/surgical.rs`

```rust
// === m1nd-mcp/src/protocol/surgical.rs ===
//
// Input/Output types for m1nd.surgical.context and m1nd.surgical.apply.
// Conventions match protocol/core.rs, protocol/layers.rs.

use serde::{Deserialize, Serialize};

// =========================================================================
// m1nd.surgical.context
// =========================================================================

/// Input for m1nd.surgical.context.
/// Gathers everything an agent needs to surgically modify a single node:
/// source code, callers, callees, tests, antibodies, trust, blast radius.
#[derive(Clone, Debug, Deserialize)]
pub struct SurgicalContextInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Node external_id to get context for (e.g. "function::backend/chat_handler.py::handle_message").
    pub node_id: String,
    /// Include related test files/functions. Default: true.
    #[serde(default = "default_true")]
    pub include_tests: bool,
    /// Include antibody pattern matches against this node. Default: true.
    #[serde(default = "default_true")]
    pub include_antibodies: bool,
    /// Maximum lines of source code to extract around the node. Default: 200.
    #[serde(default = "default_max_peek_lines")]
    pub max_peek_lines: usize,
    /// Include trust score from defect history. Default: true.
    #[serde(default = "default_true")]
    pub include_trust: bool,
    /// Include ghost edge detection. Default: true.
    #[serde(default = "default_true")]
    pub include_ghost_edges: bool,
    /// Include structural hole analysis. Default: false.
    #[serde(default)]
    pub include_structural_holes: bool,
    /// Maximum callers/callees to return. Default: 20.
    #[serde(default = "default_top_k")]
    pub max_connections: usize,
}

/// Output for m1nd.surgical.context.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalContextOutput {
    /// The resolved node metadata.
    pub node: SurgicalNodeInfo,
    /// Source code content (line-bounded, security-filtered via peek pipeline).
    pub source_code: String,
    /// Absolute file path on disk.
    pub file_path: String,
    /// Line range of the extracted source code (1-indexed, inclusive).
    pub line_range: LineRange,
    /// Whether source code is stale relative to last ingest.
    pub provenance_stale: bool,
    /// Nodes that call/reference this node (incoming edges).
    pub callers: Vec<SurgicalNodeInfo>,
    /// Nodes that this node calls/references (outgoing edges).
    pub callees: Vec<SurgicalNodeInfo>,
    /// Import statements in the containing file.
    pub imports: Vec<String>,
    /// Related test files or test functions.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related_tests: Vec<SurgicalNodeInfo>,
    /// Antibody pattern matches against this node.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub antibody_matches: Vec<AntibodyMatchEntry>,
    /// Trust score from defect history (0.0 = untrusted, 1.0 = fully trusted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_score: Option<f64>,
    /// Ghost edges (statistically likely but not structurally present connections).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ghost_edges: Vec<SurgicalGhostEdge>,
    /// Structural holes (expected connections that are missing).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub structural_holes: Vec<SurgicalStructuralHole>,
    /// Blast radius count (number of transitively affected nodes).
    pub blast_radius: usize,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: f64,
}

/// Line range (1-indexed, inclusive on both ends).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

/// Compact node info returned in surgical context connections.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalNodeInfo {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    /// File path (if available from provenance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Line range in source file (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<LineRange>,
    /// Edge relation to the focal node (e.g. "calls", "imports", "tests").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
    /// Edge weight / signal strength to focal node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<f32>,
}

/// A single antibody match within surgical context.
#[derive(Clone, Debug, Serialize)]
pub struct AntibodyMatchEntry {
    pub antibody_id: String,
    pub antibody_name: String,
    pub severity: String,
    pub description: String,
    /// How the node matched (which pattern component).
    pub match_reason: String,
}

/// Ghost edge in surgical context.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalGhostEdge {
    pub source: String,
    pub target: String,
    pub shared_dimensions: Vec<String>,
    pub strength: f32,
}

/// Structural hole in surgical context.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalStructuralHole {
    pub node_id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub reason: String,
}

// =========================================================================
// m1nd.surgical.apply
// =========================================================================

/// Input for m1nd.surgical.apply.
/// Writes new content to a node's source file, replacing the exact line range.
/// Optionally re-ingests the file to update the graph.
#[derive(Clone, Debug, Deserialize)]
pub struct SurgicalApplyInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// Node external_id that was obtained from surgical.context.
    pub node_id: String,
    /// New source code content to replace the node's line range.
    pub new_content: String,
    /// Expected line range (from surgical.context output). Used as safety guard:
    /// apply will fail if the file's current content at this range doesn't match
    /// what was seen during context extraction.
    pub expected_line_range: LineRange,
    /// Run basic syntax validation before writing. Default: true.
    /// For Rust: `syn::parse_file`. For Python: `ast.parse`. For TS: noop.
    #[serde(default = "default_true")]
    pub verify_syntax: bool,
    /// Re-ingest the modified file into the graph after writing. Default: true.
    #[serde(default = "default_true")]
    pub re_ingest: bool,
    /// Optional: expected hash (SHA-256 of the original line range content).
    /// If provided, apply verifies the file hasn't changed since context was read.
    #[serde(default)]
    pub expected_content_hash: Option<String>,
}

/// Output for m1nd.surgical.apply.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalApplyOutput {
    /// Whether the write succeeded.
    pub success: bool,
    /// Absolute path of the modified file.
    pub file_path: String,
    /// Actual line range that was replaced.
    pub lines_changed: LineRange,
    /// Unified diff of the change (for verification / cortex logging).
    pub diff: String,
    /// Predict results: other nodes likely needing changes after this modification.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub predictions: Vec<SurgicalNodeInfo>,
    /// New node_id if the node's identity changed after re-ingest (e.g. function renamed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_node_id: Option<String>,
    /// Whether re-ingest was performed.
    pub re_ingested: bool,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: f64,
}

// =========================================================================
// Default helpers
// =========================================================================

fn default_true() -> bool { true }
fn default_top_k() -> usize { 20 }
fn default_max_peek_lines() -> usize { 200 }
```

---

## Error Types — additions to `m1nd-core/src/error.rs`

```rust
// Add these variants to the M1ndError enum in m1nd-core/src/error.rs:

    // --- Surgical ---

    /// Node not found in graph by external_id.
    #[error("surgical: node not found: {node_id}")]
    SurgicalNodeNotFound { node_id: String },

    /// Node has no provenance (no source_path in graph). Cannot peek or apply.
    #[error("surgical: node has no provenance (no source file): {node_id}")]
    SurgicalNoProvenance { node_id: String },

    /// Apply: line range mismatch. File was modified between context and apply.
    #[error("surgical apply: line range mismatch for {file_path}: expected {expected_start}-{expected_end}, file has {actual_lines} lines")]
    SurgicalLineRangeMismatch {
        file_path: String,
        expected_start: u32,
        expected_end: u32,
        actual_lines: u32,
    },

    /// Apply: content hash mismatch. File was modified since context was read.
    #[error("surgical apply: content hash mismatch for {file_path}: expected {expected}, got {actual}")]
    SurgicalContentHashMismatch {
        file_path: String,
        expected: String,
        actual: String,
    },

    /// Apply: syntax verification failed.
    #[error("surgical apply: syntax error in new content: {detail}")]
    SurgicalSyntaxError { detail: String },

    /// Apply: path validation failed (outside allow roots, symlink escape, etc).
    #[error("surgical apply: path rejected: {detail}")]
    SurgicalPathRejected { detail: String },

    /// Apply: parent directory does not exist.
    #[error("surgical apply: parent directory does not exist for {file_path}")]
    SurgicalParentMissing { file_path: String },
```

---

## JSON Schemas for MCP Tool Registration — additions to `tool_schemas()` in `server.rs`

```json
{
    "name": "m1nd.surgical.context",
    "description": "Get complete surgical context for a node: source code, callers, callees, tests, antibodies, trust score, ghost edges, structural holes, and blast radius. Everything an agent needs to safely modify a single code element.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" },
            "node_id": { "type": "string", "description": "Node external_id (e.g. function::backend/chat_handler.py::handle_message)" },
            "include_tests": { "type": "boolean", "default": true, "description": "Include related test files/functions" },
            "include_antibodies": { "type": "boolean", "default": true, "description": "Include antibody pattern matches" },
            "max_peek_lines": { "type": "integer", "default": 200, "description": "Maximum source lines to extract" },
            "include_trust": { "type": "boolean", "default": true, "description": "Include trust score from defect history" },
            "include_ghost_edges": { "type": "boolean", "default": true, "description": "Include ghost edge detection" },
            "include_structural_holes": { "type": "boolean", "default": false, "description": "Include structural hole analysis" },
            "max_connections": { "type": "integer", "default": 20, "description": "Maximum callers/callees to return" }
        },
        "required": ["agent_id", "node_id"]
    }
},
{
    "name": "m1nd.surgical.apply",
    "description": "Apply a surgical modification to a node's source file. Replaces the exact line range from surgical.context, validates content hash, optionally verifies syntax, generates diff, re-ingests the file, and predicts co-change candidates.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": { "type": "string", "description": "Calling agent identifier" },
            "node_id": { "type": "string", "description": "Node external_id from surgical.context" },
            "new_content": { "type": "string", "description": "New source code content to replace the line range" },
            "expected_line_range": {
                "type": "object",
                "properties": {
                    "start": { "type": "integer", "description": "Start line (1-indexed, inclusive)" },
                    "end": { "type": "integer", "description": "End line (1-indexed, inclusive)" }
                },
                "required": ["start", "end"],
                "description": "Expected line range from surgical.context output"
            },
            "verify_syntax": { "type": "boolean", "default": true, "description": "Run syntax check before writing" },
            "re_ingest": { "type": "boolean", "default": true, "description": "Re-ingest the file into graph after writing" },
            "expected_content_hash": { "type": "string", "description": "SHA-256 of original content at line range (optimistic concurrency guard)" }
        },
        "required": ["agent_id", "node_id", "new_content", "expected_line_range"]
    }
}
```

---

## Dispatch Integration — additions to `server.rs`

### 1. Top-level dispatch: add surgical prefix match

```rust
// In dispatch_tool(), add BEFORE the fallback `_ =>` arm:
// File: m1nd-mcp/src/server.rs, function dispatch_tool()

pub fn dispatch_tool(
    state: &mut SessionState,
    tool_name: &str,
    params: &serde_json::Value,
) -> M1ndResult<serde_json::Value> {
    let normalized = tool_name.replace('_', ".");
    match normalized.as_str() {
        name if name.starts_with("m1nd.perspective.") => {
            dispatch_perspective_tool(state, name, params)
        }
        name if name.starts_with("m1nd.lock.") => {
            dispatch_lock_tool(state, name, params)
        }
        // --- NEW: Surgical tools ---
        name if name.starts_with("m1nd.surgical.") => {
            dispatch_surgical_tool(state, name, params)
        }
        _ => dispatch_core_tool(state, &normalized, params),
    }
}
```

### 2. New dispatch function

```rust
// Add to server.rs after dispatch_lock_tool():

/// Dispatch surgical tools (2 tools).
fn dispatch_surgical_tool(
    state: &mut SessionState,
    tool_name: &str,
    params: &serde_json::Value,
) -> M1ndResult<serde_json::Value> {
    use crate::protocol::surgical::*;
    use crate::surgical_handlers;

    match tool_name {
        "m1nd.surgical.context" => {
            let input: SurgicalContextInput = serde_json::from_value(params.clone())
                .map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd.surgical.context".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_surgical_context(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd.surgical.apply" => {
            let input: SurgicalApplyInput = serde_json::from_value(params.clone())
                .map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd.surgical.apply".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_surgical_apply(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        _ => Err(M1ndError::UnknownTool { name: tool_name.to_string() }),
    }
}
```

### 3. Module registration

```rust
// Add to m1nd-mcp/src/lib.rs:
pub mod surgical_handlers;

// Add to m1nd-mcp/src/protocol/mod.rs:
pub mod surgical;
```

---

## Handler Signatures — `m1nd-mcp/src/surgical_handlers.rs`

```rust
// === m1nd-mcp/src/surgical_handlers.rs ===
//
// Handlers for m1nd.surgical.context and m1nd.surgical.apply.
// Split from server.rs dispatch (same pattern as perspective_handlers.rs).

use m1nd_core::error::{M1ndError, M1ndResult};
use crate::session::SessionState;
use crate::protocol::surgical::*;
use std::time::Instant;

// ---------------------------------------------------------------------------
// m1nd.surgical.context handler
// ---------------------------------------------------------------------------

/// Gather complete surgical context for a single graph node.
///
/// Execution order (sequential, all on &mut SessionState):
/// 1. Resolve node_id -> NodeIdx via graph lookup
/// 2. Extract provenance (source_path, line_start, line_end)
/// 3. Peek source code via peek_security pipeline
/// 4. Walk outgoing edges -> callees
/// 5. Walk incoming edges -> callers (reverse lookup)
/// 6. Filter edges with relation "tests" -> related_tests
/// 7. Extract import lines from file content
/// 8. Run impact analysis -> blast_radius count
/// 9. (optional) Run antibody_scan scoped to this node
/// 10. (optional) Query trust_ledger for this node
/// 11. (optional) Detect ghost edges around this node
/// 12. (optional) Detect structural holes around this node
/// 13. Assemble SurgicalContextOutput
pub fn handle_surgical_context(
    state: &mut SessionState,
    input: SurgicalContextInput,
) -> M1ndResult<SurgicalContextOutput> {
    let start = Instant::now();
    state.track_agent(&input.agent_id);

    // Step 1: Resolve node
    let graph = state.graph.read();
    let node_idx = graph.find_node_by_external_id(&input.node_id)
        .ok_or_else(|| M1ndError::SurgicalNodeNotFound {
            node_id: input.node_id.clone(),
        })?;
    let node = &graph.nodes[node_idx.0];

    // Step 2: Extract provenance
    let provenance = node.provenance.as_ref()
        .ok_or_else(|| M1ndError::SurgicalNoProvenance {
            node_id: input.node_id.clone(),
        })?;
    let source_path = provenance.source_path.as_ref()
        .ok_or_else(|| M1ndError::SurgicalNoProvenance {
            node_id: input.node_id.clone(),
        })?;
    let line_start = provenance.line_start.unwrap_or(1);
    let line_end = provenance.line_end.unwrap_or(line_start + input.max_peek_lines as u32);

    // Step 3: Peek source code (reuse peek_security pipeline)
    // Uses the same allow-list, binary detection, staleness checks as perspective.peek.
    let peek_config = &state.peek_security;
    let last_ingest_ms = None; // TODO: track per-file ingest timestamps
    drop(graph); // Release read lock before mutable operations

    let peek_result = crate::perspective::peek_security::secure_peek(
        source_path,
        peek_config,
        Some(line_start),
        last_ingest_ms,
    )?;

    // Steps 4-12: Walk graph edges, run analyses
    // (Full implementation in FORGE-BUILD phase)

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    state.queries_processed += 1;

    // Assemble output (skeleton — FORGE-BUILD fills in graph walks)
    Ok(SurgicalContextOutput {
        node: SurgicalNodeInfo {
            node_id: input.node_id.clone(),
            label: String::new(), // filled by graph lookup
            node_type: String::new(),
            file_path: Some(source_path.to_string()),
            line_range: Some(LineRange { start: line_start, end: line_end }),
            relation: None,
            strength: None,
        },
        source_code: peek_result.content,
        file_path: source_path.to_string(),
        line_range: LineRange {
            start: peek_result.line_start,
            end: peek_result.line_end,
        },
        provenance_stale: peek_result.provenance_stale,
        callers: Vec::new(),
        callees: Vec::new(),
        imports: Vec::new(),
        related_tests: Vec::new(),
        antibody_matches: Vec::new(),
        trust_score: None,
        ghost_edges: Vec::new(),
        structural_holes: Vec::new(),
        blast_radius: 0,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// m1nd.surgical.apply handler
// ---------------------------------------------------------------------------

/// Apply a surgical modification to a node's source file.
///
/// Execution order:
/// 1. Resolve node_id -> provenance (source_path, line range)
/// 2. Validate path via peek_security allow-list
/// 3. Read current file content
/// 4. Verify expected_line_range is within bounds
/// 5. (optional) Verify expected_content_hash matches current content at range
/// 6. (optional) Verify syntax of new_content
/// 7. Replace lines [start..=end] with new_content
/// 8. Write file atomically (write to .tmp, then rename)
/// 9. Generate unified diff
/// 10. (optional) Re-ingest the modified file
/// 11. Run predict on the modified node for co-change candidates
/// 12. Assemble SurgicalApplyOutput
pub fn handle_surgical_apply(
    state: &mut SessionState,
    input: SurgicalApplyInput,
) -> M1ndResult<SurgicalApplyOutput> {
    let start = Instant::now();
    state.track_agent(&input.agent_id);

    // Step 1: Resolve node -> file path
    let graph = state.graph.read();
    let node_idx = graph.find_node_by_external_id(&input.node_id)
        .ok_or_else(|| M1ndError::SurgicalNodeNotFound {
            node_id: input.node_id.clone(),
        })?;
    let node = &graph.nodes[node_idx.0];
    let provenance = node.provenance.as_ref()
        .ok_or_else(|| M1ndError::SurgicalNoProvenance {
            node_id: input.node_id.clone(),
        })?;
    let source_path = provenance.source_path.as_ref()
        .ok_or_else(|| M1ndError::SurgicalNoProvenance {
            node_id: input.node_id.clone(),
        })?
        .to_string();
    drop(graph);

    // Step 2: Validate path
    let canonical = std::fs::canonicalize(&source_path)
        .map_err(|e| M1ndError::SurgicalPathRejected {
            detail: format!("canonicalize failed for '{}': {}", source_path, e),
        })?;
    crate::perspective::peek_security::validate_allow_list_pub(
        &canonical,
        &state.peek_security.allow_roots,
    ).map_err(|_| M1ndError::SurgicalPathRejected {
        detail: format!("path '{}' outside allowed roots", source_path),
    })?;

    // Step 3: Read current file
    let current_content = std::fs::read_to_string(&canonical)
        .map_err(|e| M1ndError::Io(e))?;
    let lines: Vec<&str> = current_content.lines().collect();
    let total_lines = lines.len() as u32;

    // Step 4: Validate line range
    let range_start = input.expected_line_range.start;
    let range_end = input.expected_line_range.end;
    if range_start == 0 || range_end == 0 || range_start > range_end || range_end > total_lines {
        return Err(M1ndError::SurgicalLineRangeMismatch {
            file_path: source_path.clone(),
            expected_start: range_start,
            expected_end: range_end,
            actual_lines: total_lines,
        });
    }

    // Step 5: Verify content hash (optimistic concurrency)
    if let Some(ref expected_hash) = input.expected_content_hash {
        let original_slice: String = lines[(range_start as usize - 1)..=(range_end as usize - 1)]
            .join("\n");
        let actual_hash = sha256_hex(&original_slice);
        if &actual_hash != expected_hash {
            return Err(M1ndError::SurgicalContentHashMismatch {
                file_path: source_path.clone(),
                expected: expected_hash.clone(),
                actual: actual_hash,
            });
        }
    }

    // Step 6: Syntax verification (optional)
    // Deferred to FORGE-BUILD: requires language detection + parser dispatch.

    // Step 7: Replace lines
    let mut new_lines: Vec<String> = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let line_num = (i + 1) as u32;
        if line_num < range_start || line_num > range_end {
            new_lines.push(line.to_string());
        } else if line_num == range_start {
            // Insert new content at the start of the range
            new_lines.push(input.new_content.clone());
        }
        // Lines within range (after start) are dropped — replaced by new_content
    }
    let new_file_content = new_lines.join("\n");

    // Step 8: Atomic write (write .tmp, then rename)
    let tmp_path = canonical.with_extension("m1nd-surgical-tmp");
    std::fs::write(&tmp_path, &new_file_content)
        .map_err(|e| M1ndError::Io(e))?;
    std::fs::rename(&tmp_path, &canonical)
        .map_err(|e| M1ndError::Io(e))?;

    // Step 9: Generate diff
    let original_slice: String = lines[(range_start as usize - 1)..=(range_end as usize - 1)]
        .join("\n");
    let diff = generate_unified_diff(&source_path, &original_slice, &input.new_content, range_start);

    // Step 10: Re-ingest (optional)
    let re_ingested = if input.re_ingest {
        let ingest_input = crate::protocol::IngestInput {
            path: source_path.clone(),
            agent_id: input.agent_id.clone(),
            incremental: false,
            adapter: "code".to_string(),
            mode: "merge".to_string(),
            namespace: None,
        };
        // Best-effort re-ingest. Log errors but don't fail the apply.
        match crate::tools::handle_ingest(state, ingest_input) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("[m1nd] WARNING: surgical.apply re-ingest failed: {}", e);
                false
            }
        }
    } else {
        false
    };

    // Step 11: Predict co-changes (best-effort)
    let predictions = Vec::new(); // Filled in FORGE-BUILD via handle_predict

    let new_content_lines: usize = input.new_content.lines().count();
    let new_end = range_start + new_content_lines.saturating_sub(1) as u32;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    state.queries_processed += 1;

    Ok(SurgicalApplyOutput {
        success: true,
        file_path: source_path,
        lines_changed: LineRange { start: range_start, end: new_end },
        diff,
        predictions,
        new_node_id: None, // Determined after re-ingest by comparing graph state
        re_ingested,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// SHA-256 hex digest of a string.
fn sha256_hex(input: &str) -> String {
    use std::io::Write;
    // Use a simple implementation — no external dependency.
    // In production: use `sha2` crate. For now: shell out or inline.
    // Placeholder: FORGE-BUILD will add sha2 dependency.
    format!("{:x}", md5_like_hash(input)) // PLACEHOLDER — replace with sha2::Sha256
}

/// Placeholder hash function. Replace with sha2 crate in BUILD phase.
fn md5_like_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Generate a minimal unified diff for cortex logging.
fn generate_unified_diff(
    file_path: &str,
    old_content: &str,
    new_content: &str,
    start_line: u32,
) -> String {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let mut diff = String::new();
    diff.push_str(&format!("--- a/{}\n", file_path));
    diff.push_str(&format!("+++ b/{}\n", file_path));
    diff.push_str(&format!(
        "@@ -{},{} +{},{} @@\n",
        start_line,
        old_lines.len(),
        start_line,
        new_lines.len()
    ));

    for line in &old_lines {
        diff.push_str(&format!("-{}\n", line));
    }
    for line in &new_lines {
        diff.push_str(&format!("+{}\n", line));
    }

    diff
}
```

---

## Public API Surface for `peek_security.rs`

The `validate_allow_list` function is currently private. `surgical.apply` needs it.
Add a public wrapper:

```rust
// Add to m1nd-mcp/src/perspective/peek_security.rs:

/// Public wrapper for allow-list validation. Used by surgical_handlers.
pub fn validate_allow_list_pub(canonical: &std::path::Path, allow_roots: &[String]) -> M1ndResult<()> {
    validate_allow_list(canonical, allow_roots)
}
```

---

## Dependencies to Add (Cargo.toml)

```toml
# In m1nd-mcp/Cargo.toml, under [dependencies]:
sha2 = "0.10"    # For content hash verification in surgical.apply
hex = "0.4"      # For hex encoding of SHA-256 digest
```

Once `sha2` is added, replace the placeholder `sha256_hex`:

```rust
fn sha256_hex(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
```

---

## Checklist for FORGE-SCAFFOLD / FORGE-BUILD

- [ ] Create `m1nd-mcp/src/protocol/surgical.rs` with types above
- [ ] Add `pub mod surgical;` to `m1nd-mcp/src/protocol/mod.rs`
- [ ] Create `m1nd-mcp/src/surgical_handlers.rs` with handler stubs
- [ ] Add `pub mod surgical_handlers;` to `m1nd-mcp/src/lib.rs`
- [ ] Add error variants to `m1nd-core/src/error.rs`
- [ ] Add `validate_allow_list_pub` to `peek_security.rs`
- [ ] Add JSON schemas to `tool_schemas()` in `server.rs`
- [ ] Add `m1nd.surgical.*` prefix match to `dispatch_tool()` in `server.rs`
- [ ] Add `dispatch_surgical_tool()` function to `server.rs`
- [ ] Add `sha2` + `hex` to `m1nd-mcp/Cargo.toml`
- [ ] Fill in graph walk logic in `handle_surgical_context` (callers, callees, imports, tests)
- [ ] Fill in predict call in `handle_surgical_apply`
- [ ] Add `find_node_by_external_id` to graph if not present
- [ ] Tests: unit tests for line replacement, hash verification, diff generation
- [ ] Tests: integration test via MCP stdio (round-trip context -> apply)
