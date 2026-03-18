# SURGICAL V2 CONTRACTS — surgical_context_v2 + apply_batch (ONDA 1)

> Compilable Rust structs, JSON schemas, dispatch code, error types.
> Extends the existing surgical tooling with connected-source expansion and batch apply.

---

## Design Rationale

### surgical_context_v2

V1 `surgical_context` returns the target file's contents and its graph neighbourhood
(callers, callees, tests) as `SurgicalNeighbour` metadata — but NOT their source code.
An agent wanting to edit a function and its callers must issue N separate reads.

V2 adds `include_connected_sources: bool` (default true). When enabled, the response
includes the actual source code of connected files, ranked by relevance. This gives
the agent a single-call "workspace snapshot" — the target file plus everything it
touches.

The V1 output is embedded verbatim as the `primary` field so V2 is a strict superset.

### apply_batch

V1 `apply` writes one file at a time. A typical surgical edit modifies 2-5 files
(target + callers + tests). Issuing 5 sequential applies is slow and non-atomic:
a crash between apply #3 and #4 leaves the codebase in an inconsistent state.

V2 `apply_batch` accepts a `Vec<SingleEdit>` and writes all files atomically
(all-or-nothing when `atomic: true`). After the batch, a single re-ingest pass
updates the graph once instead of N times.

---

## Step 5 Decisions (recorded)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **V2 output embedding** | `primary: SurgicalContextOutput` (reuse existing V1 struct) | Zero duplication, V2 is a strict superset |
| **Connected file cap** | `max_connected_files: usize` default 10 | Prevents context blowup while covering typical edit radius |
| **Lines cap per file** | `max_lines_per_file: u32` default 500 | Matches V1 cap philosophy, prevents multi-MB responses |
| **BFS radius** | `radius: u32` default 1 | V1 already has this; V2 reuses it for connected-source BFS |
| **Relevance scoring** | Edge weight × recency × co-change frequency | Combines structural + temporal + statistical signals |
| **Batch atomicity** | `atomic: bool` default true, write-to-tmp-then-rename all | Crash-safe: either all files are updated or none |
| **Batch re-ingest** | Single bulk re-ingest after all writes | Avoids N separate finalize passes |
| **V2 dispatch routing** | Same `dispatch_core_tool` match, new names `m1nd.surgical.context.v2` and `m1nd.apply.batch` | Follows existing convention, no new prefix router needed |

---

## Protocol Types — `m1nd-mcp/src/protocol/surgical.rs` (additions)

```rust
// =========================================================================
// m1nd.surgical.context.v2 — connected-source expansion
// =========================================================================

/// Input for m1nd.surgical.context.v2.
///
/// Extends V1 surgical_context with multi-file source fetching.
/// Returns the target file's full context (V1 output) PLUS source code
/// of connected files (callers, callees, tests) so the agent has a
/// complete workspace snapshot in one call.
#[derive(Clone, Debug, Deserialize)]
pub struct SurgicalContextV2Input {
    /// Calling agent identifier (required by all m1nd tools).
    pub agent_id: String,
    /// Absolute or workspace-relative path to the primary file being edited.
    pub file_path: String,
    /// Optional: narrow context to a specific symbol (function / struct / class name).
    /// When provided, only the symbol's neighbourhood drives connected-source expansion.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Fetch source code of connected files (callers, callees, tests).
    /// When false, behaves identically to V1 surgical_context. Default: true.
    #[serde(default = "default_true")]
    pub include_connected_sources: bool,
    /// Maximum number of connected files to include source for. Default: 10.
    /// Files beyond this cap are still listed in the V1 callers/callees/tests
    /// but without source code.
    #[serde(default = "default_max_connected_files")]
    pub max_connected_files: usize,
    /// Maximum lines of source to extract per connected file. Default: 500.
    /// The primary file is unbounded (full content, same as V1).
    #[serde(default = "default_max_lines_per_file")]
    pub max_lines_per_file: u32,
    /// BFS radius for graph neighbourhood expansion. Default: 1.
    /// radius=2 follows callers-of-callers, useful for broad refactors.
    #[serde(default = "default_radius")]
    pub radius: u32,
    /// Include test files in the neighbourhood. Default: true.
    #[serde(default = "default_true")]
    pub include_tests: bool,
}

fn default_max_connected_files() -> usize { 10 }
fn default_max_lines_per_file() -> u32 { 500 }

/// Output for m1nd.surgical.context.v2.
///
/// Strict superset of V1: `primary` is the exact `SurgicalContextOutput`
/// from V1. `connected_files` adds source code for the neighbourhood.
#[derive(Clone, Debug, Serialize)]
pub struct SurgicalContextV2Output {
    /// The full V1 surgical context for the primary file.
    /// Contains: file_path, file_contents, line_count, node_id, symbols,
    /// focused_symbol, callers, callees, tests, elapsed_ms.
    pub primary: SurgicalContextOutput,
    /// Source code of connected files (callers, callees, test files).
    /// Sorted by relevance_score descending. Capped at max_connected_files.
    pub connected_files: Vec<ConnectedFileContext>,
    /// Total lines across primary + all connected files.
    pub total_lines: u32,
    /// Total files returned (1 primary + connected_files.len()).
    pub total_files: u32,
    /// Total elapsed time in milliseconds (includes all file reads + graph walks).
    pub elapsed_ms: f64,
}

/// Context for a single connected file in the V2 response.
#[derive(Clone, Debug, Serialize)]
pub struct ConnectedFileContext {
    /// Absolute path of the connected file.
    pub file_path: String,
    /// Source code contents (may be truncated to max_lines_per_file).
    pub source_code: String,
    /// Relation to the primary file: "caller", "callee", "import", "test", "imported_by".
    pub relation: String,
    /// Relevance score [0.0, 1.0] — higher means more likely to need co-editing.
    /// Computed from: edge_weight × recency_factor × co_change_frequency.
    pub relevance_score: f64,
    /// Number of lines in source_code.
    pub line_count: u32,
    /// Whether the source was truncated due to max_lines_per_file.
    pub truncated: bool,
    /// Graph node ID for this file (empty string if not ingested).
    pub node_id: String,
}

// =========================================================================
// m1nd.apply.batch — atomic multi-file apply
// =========================================================================

/// Input for m1nd.apply.batch.
///
/// Writes multiple files atomically and triggers a single bulk re-ingest.
/// If `atomic` is true (default), all files must be writable or none are written.
#[derive(Clone, Debug, Deserialize)]
pub struct ApplyBatchInput {
    /// Calling agent identifier.
    pub agent_id: String,
    /// List of edits to apply. Each edit replaces an entire file's contents.
    pub edits: Vec<SingleEdit>,
    /// Atomic mode: all-or-nothing. Default: true.
    /// When true: writes to temp files first, then renames all atomically.
    /// When false: writes each file independently (partial success possible).
    #[serde(default = "default_true")]
    pub atomic: bool,
    /// Re-ingest all modified files into the graph after writing. Default: true.
    #[serde(default = "default_true")]
    pub reingest: bool,
    /// Human-readable description of the batch edit (used in apply log).
    #[serde(default)]
    pub description: Option<String>,
}

/// A single file edit within a batch.
#[derive(Clone, Debug, Deserialize)]
pub struct SingleEdit {
    /// Absolute or workspace-relative path of the file to write.
    pub file_path: String,
    /// New file contents (full replacement, UTF-8).
    pub new_content: String,
    /// Optional human-readable label for this specific edit (e.g. "update caller").
    #[serde(default)]
    pub label: Option<String>,
}

/// Output for m1nd.apply.batch.
#[derive(Clone, Debug, Serialize)]
pub struct ApplyBatchOutput {
    /// Per-file results, in the same order as input `edits`.
    pub results: Vec<SingleEditResult>,
    /// Total number of files in the batch.
    pub total_files: usize,
    /// Sum of lines added across all files.
    pub total_lines_added: u32,
    /// Sum of lines removed across all files.
    pub total_lines_removed: u32,
    /// Whether a bulk re-ingest was performed after the batch.
    pub reingested: bool,
    /// Node IDs that were updated or created during re-ingest.
    pub updated_node_ids: Vec<String>,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: f64,
}

/// Result for a single file within apply_batch.
#[derive(Clone, Debug, Serialize)]
pub struct SingleEditResult {
    /// Absolute path of the file.
    pub file_path: String,
    /// Whether this individual file was written successfully.
    pub success: bool,
    /// Number of bytes written.
    pub bytes_written: usize,
    /// Lines added in this file.
    pub lines_added: i32,
    /// Lines removed in this file.
    pub lines_removed: i32,
    /// Error message if this file failed (only populated when success=false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

---

## Error Types — additions to `m1nd-core/src/error.rs`

```rust
// Add these variants to the M1ndError enum:

    // --- Surgical V2 / Batch ---

    /// apply_batch: atomic batch failed — one or more files could not be written.
    /// All files are rolled back (temp files deleted, originals untouched).
    #[error("apply_batch: atomic batch failed: {failed_count}/{total_count} files failed. First error: {first_error}")]
    ApplyBatchAtomicFailure {
        failed_count: usize,
        total_count: usize,
        first_error: String,
    },

    /// apply_batch: empty edits list.
    #[error("apply_batch: edits list is empty")]
    ApplyBatchEmptyEdits,

    /// surgical_context_v2: connected file read failed (non-fatal, logged in output).
    #[error("surgical_context_v2: cannot read connected file {file_path}: {detail}")]
    ConnectedFileReadFailed {
        file_path: String,
        detail: String,
    },
```

---

## JSON Schemas for MCP Tool Registration

```json
{
    "name": "m1nd.surgical_context_v2",
    "description": "Get full surgical context for a file PLUS source code of connected files (callers, callees, tests). Returns a complete workspace snapshot in one call. Superset of m1nd.surgical_context — use this when you need to see related code, not just the target file.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "description": "Calling agent identifier"
            },
            "file_path": {
                "type": "string",
                "description": "Absolute or workspace-relative path to the primary file"
            },
            "symbol": {
                "type": "string",
                "description": "Optional: narrow context to a specific symbol (function/struct/class name)"
            },
            "include_connected_sources": {
                "type": "boolean",
                "default": true,
                "description": "Fetch source code of connected files. When false, behaves like V1."
            },
            "max_connected_files": {
                "type": "integer",
                "default": 10,
                "description": "Maximum number of connected files to include source for"
            },
            "max_lines_per_file": {
                "type": "integer",
                "default": 500,
                "description": "Maximum lines per connected file (primary file is unbounded)"
            },
            "radius": {
                "type": "integer",
                "default": 1,
                "description": "BFS radius for graph neighbourhood (1 or 2)"
            },
            "include_tests": {
                "type": "boolean",
                "default": true,
                "description": "Include test files in the neighbourhood"
            }
        },
        "required": ["agent_id", "file_path"]
    }
}
```

```json
{
    "name": "m1nd.apply_batch",
    "description": "Atomically write multiple files and trigger a single bulk re-ingest. Use after m1nd.surgical_context_v2 when editing a file and its callers/tests together. All-or-nothing by default.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "agent_id": {
                "type": "string",
                "description": "Calling agent identifier"
            },
            "edits": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Absolute or workspace-relative path of the file to write"
                        },
                        "new_content": {
                            "type": "string",
                            "description": "New file contents (full replacement, UTF-8)"
                        },
                        "label": {
                            "type": "string",
                            "description": "Optional human-readable label for this edit"
                        }
                    },
                    "required": ["file_path", "new_content"]
                },
                "description": "List of file edits to apply"
            },
            "atomic": {
                "type": "boolean",
                "default": true,
                "description": "All-or-nothing: if any file fails, none are written"
            },
            "reingest": {
                "type": "boolean",
                "default": true,
                "description": "Re-ingest all modified files after writing"
            },
            "description": {
                "type": "string",
                "description": "Human-readable description of the batch edit"
            }
        },
        "required": ["agent_id", "edits"]
    }
}
```

---

## Dispatch Integration — additions to `server.rs`

### 1. New match arms in `dispatch_core_tool()`

```rust
// Add to dispatch_core_tool(), in the match block, AFTER the existing
// "m1nd.surgical.context" and "m1nd.apply" arms:

        // -----------------------------------------------------------------
        // Surgical V2: context_v2 + apply_batch
        // -----------------------------------------------------------------
        "m1nd.surgical.context.v2" => {
            let input: crate::protocol::surgical::SurgicalContextV2Input =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd.surgical_context_v2".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_surgical_context_v2(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd.apply.batch" => {
            let input: crate::protocol::surgical::ApplyBatchInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd.apply_batch".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_apply_batch(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
```

### 2. Tool schema registration

Add the two JSON schema objects above to the `tool_schemas()` array in `server.rs`,
after the existing `m1nd.apply` schema entry.

---

## Handler Signatures — additions to `m1nd-mcp/src/surgical_handlers.rs`

```rust
// ---------------------------------------------------------------------------
// m1nd.surgical_context_v2 handler
// ---------------------------------------------------------------------------

/// Handle m1nd.surgical_context_v2.
///
/// Returns V1 surgical context for the primary file PLUS source code
/// of connected files (callers, callees, tests).
///
/// Steps:
///   1. Delegate to handle_surgical_context() for the primary file (V1 output).
///   2. If include_connected_sources is false, return V2 wrapper with empty connected_files.
///   3. Collect unique file paths from primary.callers + primary.callees + primary.tests.
///   4. Deduplicate and exclude the primary file itself.
///   5. Score each connected file by relevance (edge_weight * recency * co_change).
///   6. Sort by relevance descending, take top max_connected_files.
///   7. Read each connected file's source (truncate to max_lines_per_file).
///   8. Assemble SurgicalContextV2Output.
pub fn handle_surgical_context_v2(
    state: &mut SessionState,
    input: surgical::SurgicalContextV2Input,
) -> M1ndResult<surgical::SurgicalContextV2Output> {
    let start = Instant::now();

    // Step 1: Get V1 context for the primary file
    let v1_input = surgical::SurgicalContextInput {
        file_path: input.file_path.clone(),
        agent_id: input.agent_id.clone(),
        symbol: input.symbol.clone(),
        radius: input.radius,
        include_tests: input.include_tests,
    };
    let primary = handle_surgical_context(state, v1_input)?;

    // Step 2: Early return if connected sources not requested
    if !input.include_connected_sources {
        let total_lines = primary.line_count;
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        return Ok(surgical::SurgicalContextV2Output {
            primary,
            connected_files: Vec::new(),
            total_lines,
            total_files: 1,
            elapsed_ms,
        });
    }

    // Step 3: Collect unique file paths from neighbourhood
    let primary_path = primary.file_path.clone();
    let mut candidate_files: Vec<(String, String, f32)> = Vec::new(); // (path, relation, weight)

    for caller in &primary.callers {
        if !caller.file_path.is_empty() && caller.file_path != primary_path {
            candidate_files.push((
                caller.file_path.clone(),
                "caller".to_string(),
                caller.edge_weight,
            ));
        }
    }
    for callee in &primary.callees {
        if !callee.file_path.is_empty() && callee.file_path != primary_path {
            candidate_files.push((
                callee.file_path.clone(),
                "callee".to_string(),
                callee.edge_weight,
            ));
        }
    }
    for test in &primary.tests {
        if !test.file_path.is_empty() && test.file_path != primary_path {
            candidate_files.push((
                test.file_path.clone(),
                "test".to_string(),
                test.edge_weight,
            ));
        }
    }

    // Step 4: Deduplicate by file_path (keep highest weight per path)
    let mut seen: std::collections::HashMap<String, (String, f32)> =
        std::collections::HashMap::new();
    for (path, relation, weight) in &candidate_files {
        let entry = seen.entry(path.clone()).or_insert((relation.clone(), *weight));
        if *weight > entry.1 {
            *entry = (relation.clone(), *weight);
        }
    }

    // Step 5+6: Score by relevance (edge weight as primary signal), sort, cap
    let mut scored: Vec<(String, String, f64)> = seen
        .into_iter()
        .map(|(path, (relation, weight))| (path, relation, weight as f64))
        .collect();
    scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(input.max_connected_files);

    // Step 7: Read each connected file
    let mut connected_files: Vec<surgical::ConnectedFileContext> = Vec::new();
    let mut total_lines = primary.line_count;

    for (path, relation, relevance_score) in &scored {
        let resolved = resolve_file_path(path, &state.ingest_roots);
        match std::fs::read_to_string(&resolved) {
            Ok(content) => {
                let all_lines: Vec<&str> = content.lines().collect();
                let file_line_count = all_lines.len() as u32;
                let truncated = file_line_count > input.max_lines_per_file;
                let capped_lines = if truncated {
                    input.max_lines_per_file
                } else {
                    file_line_count
                };
                let source_code: String = all_lines
                    .iter()
                    .take(capped_lines as usize)
                    .cloned()
                    .collect::<Vec<&str>>()
                    .join("\n");

                // Find graph node_id for this file
                let node_id = {
                    let graph = state.graph.read();
                    let nodes = find_nodes_for_file(&graph, &resolved.to_string_lossy());
                    nodes.first().map(|(_, ext)| ext.clone()).unwrap_or_default()
                };

                total_lines += capped_lines;

                connected_files.push(surgical::ConnectedFileContext {
                    file_path: resolved.to_string_lossy().to_string(),
                    source_code,
                    relation: relation.clone(),
                    relevance_score: *relevance_score,
                    line_count: capped_lines,
                    truncated,
                    node_id,
                });
            }
            Err(e) => {
                // Non-fatal: log and skip unreadable files
                eprintln!(
                    "[m1nd] WARNING: surgical_context_v2 cannot read connected file {}: {}",
                    resolved.display(),
                    e
                );
            }
        }
    }

    let total_files = 1 + connected_files.len() as u32;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    state.track_agent(&input.agent_id);

    Ok(surgical::SurgicalContextV2Output {
        primary,
        connected_files,
        total_lines,
        total_files,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// m1nd.apply_batch handler
// ---------------------------------------------------------------------------

/// Handle m1nd.apply_batch.
///
/// Writes multiple files atomically and triggers a single bulk re-ingest.
///
/// Steps:
///   1. Validate: edits list must be non-empty.
///   2. Resolve and validate all file paths (path safety check).
///   3. Read old content for each file (for diff summary).
///   4. ATOMIC mode: write all files to .tmp first.
///      If any .tmp write fails, clean up all .tmp files and return error.
///   5. ATOMIC mode: rename all .tmp files to their targets.
///      NON-ATOMIC mode: write each file independently.
///   6. Compute diff summaries per file.
///   7. If reingest: bulk re-ingest all modified files in one pass.
///   8. Assemble ApplyBatchOutput.
pub fn handle_apply_batch(
    state: &mut SessionState,
    input: surgical::ApplyBatchInput,
) -> M1ndResult<surgical::ApplyBatchOutput> {
    let start = Instant::now();

    // Step 1: Validate
    if input.edits.is_empty() {
        return Err(M1ndError::ApplyBatchEmptyEdits);
    }

    // Step 2: Resolve and validate all paths upfront
    let mut resolved_edits: Vec<(PathBuf, &surgical::SingleEdit, String)> = Vec::new();
    for edit in &input.edits {
        let resolved = resolve_file_path(&edit.file_path, &state.ingest_roots);
        let validated = validate_path_safety(&resolved, &state.ingest_roots)?;

        // Read old content for diff
        let old_content = std::fs::read_to_string(&validated).unwrap_or_default();
        resolved_edits.push((validated, edit, old_content));
    }

    let mut results: Vec<surgical::SingleEditResult> = Vec::new();
    let mut total_lines_added: u32 = 0;
    let mut total_lines_removed: u32 = 0;

    if input.atomic {
        // Step 4: Write all to temp files first
        let mut temp_files: Vec<(PathBuf, PathBuf)> = Vec::new(); // (tmp_path, target_path)
        let mut write_errors: Vec<(usize, String)> = Vec::new();

        for (i, (validated, edit, _old)) in resolved_edits.iter().enumerate() {
            let parent = validated.parent().unwrap_or(Path::new("."));
            let tmp_path = parent.join(format!(
                ".m1nd_batch_{}_{}_.tmp",
                std::process::id(),
                i
            ));

            match std::fs::write(&tmp_path, &edit.new_content) {
                Ok(_) => {
                    temp_files.push((tmp_path, validated.clone()));
                }
                Err(e) => {
                    write_errors.push((i, format!("{}: {}", validated.display(), e)));
                    // Clean up already-written temp files
                    for (tmp, _) in &temp_files {
                        let _ = std::fs::remove_file(tmp);
                    }
                    break;
                }
            }
        }

        if !write_errors.is_empty() {
            let first_error = write_errors[0].1.clone();
            return Err(M1ndError::ApplyBatchAtomicFailure {
                failed_count: write_errors.len(),
                total_count: input.edits.len(),
                first_error,
            });
        }

        // Step 5: Rename all temp files to targets (atomic per-file)
        for (tmp_path, target_path) in &temp_files {
            if let Err(e) = std::fs::rename(tmp_path, target_path) {
                // Rename failure in atomic mode: best-effort cleanup
                for (tmp, _) in &temp_files {
                    let _ = std::fs::remove_file(tmp);
                }
                return Err(M1ndError::ApplyBatchAtomicFailure {
                    failed_count: 1,
                    total_count: input.edits.len(),
                    first_error: format!(
                        "atomic rename failed {} -> {}: {}",
                        tmp_path.display(),
                        target_path.display(),
                        e
                    ),
                });
            }
        }

        // Step 6: Compute diffs for all successfully written files
        for (validated, edit, old_content) in &resolved_edits {
            let (added, removed) = diff_summary(old_content, &edit.new_content);
            total_lines_added += added as u32;
            total_lines_removed += removed.unsigned_abs();

            results.push(surgical::SingleEditResult {
                file_path: validated.to_string_lossy().to_string(),
                success: true,
                bytes_written: edit.new_content.len(),
                lines_added: added,
                lines_removed: removed,
                error: None,
            });
        }
    } else {
        // NON-ATOMIC mode: write each file independently
        for (validated, edit, old_content) in &resolved_edits {
            let parent = validated.parent().unwrap_or(Path::new("."));
            let tmp_path = parent.join(format!(
                ".m1nd_apply_{}.tmp",
                std::process::id()
            ));

            match std::fs::write(&tmp_path, &edit.new_content)
                .and_then(|_| std::fs::rename(&tmp_path, validated))
            {
                Ok(_) => {
                    let (added, removed) = diff_summary(old_content, &edit.new_content);
                    total_lines_added += added as u32;
                    total_lines_removed += removed.unsigned_abs();

                    results.push(surgical::SingleEditResult {
                        file_path: validated.to_string_lossy().to_string(),
                        success: true,
                        bytes_written: edit.new_content.len(),
                        lines_added: added,
                        lines_removed: removed,
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp_path);
                    results.push(surgical::SingleEditResult {
                        file_path: validated.to_string_lossy().to_string(),
                        success: false,
                        bytes_written: 0,
                        lines_added: 0,
                        lines_removed: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    // Step 7: Bulk re-ingest
    let mut updated_node_ids: Vec<String> = Vec::new();
    let reingested = if input.reingest {
        let successful_paths: Vec<String> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.file_path.clone())
            .collect();

        let mut any_ingested = false;
        for path in &successful_paths {
            // Record existing node IDs
            {
                let graph = state.graph.read();
                let existing = find_nodes_for_file(&graph, path);
                for (_, ext_id) in &existing {
                    updated_node_ids.push(ext_id.clone());
                }
            }

            let ingest_input = crate::protocol::IngestInput {
                path: path.clone(),
                agent_id: input.agent_id.clone(),
                mode: "merge".to_string(),
                incremental: true,
                adapter: "code".to_string(),
                namespace: None,
            };

            match crate::tools::handle_ingest(state, ingest_input) {
                Ok(_) => { any_ingested = true; }
                Err(e) => {
                    eprintln!(
                        "[m1nd] WARNING: apply_batch re-ingest failed for {}: {}",
                        path, e
                    );
                }
            }
        }
        any_ingested
    } else {
        false
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    state.track_agent(&input.agent_id);

    Ok(surgical::ApplyBatchOutput {
        results,
        total_files: input.edits.len(),
        total_lines_added,
        total_lines_removed,
        reingested,
        updated_node_ids,
        elapsed_ms,
    })
}
```

---

## Dependencies

No new Cargo.toml dependencies required for V2. The V2 handlers reuse existing:
- `std::fs` for file I/O
- `std::path` for path resolution
- `std::collections::HashMap` for deduplication
- Existing `resolve_file_path`, `validate_path_safety`, `diff_summary`, `find_nodes_for_file` helpers

---

## Checklist for FORGE-SCAFFOLD / FORGE-BUILD

- [ ] Add V2 structs to `m1nd-mcp/src/protocol/surgical.rs` (SurgicalContextV2Input, SurgicalContextV2Output, ConnectedFileContext, ApplyBatchInput, SingleEdit, ApplyBatchOutput, SingleEditResult)
- [ ] Add default helper functions (`default_max_connected_files`, `default_max_lines_per_file`) to `protocol/surgical.rs`
- [ ] Add error variants to `m1nd-core/src/error.rs` (ApplyBatchAtomicFailure, ApplyBatchEmptyEdits, ConnectedFileReadFailed)
- [ ] Add `handle_surgical_context_v2()` to `m1nd-mcp/src/surgical_handlers.rs`
- [ ] Add `handle_apply_batch()` to `m1nd-mcp/src/surgical_handlers.rs`
- [ ] Add `"m1nd.surgical.context.v2"` match arm to `dispatch_core_tool()` in `server.rs`
- [ ] Add `"m1nd.apply.batch"` match arm to `dispatch_core_tool()` in `server.rs`
- [ ] Add JSON schemas to `tool_schemas()` array in `server.rs`
- [ ] Tests: V2 context returns connected sources with correct relation labels
- [ ] Tests: V2 context respects max_connected_files cap
- [ ] Tests: V2 context respects max_lines_per_file truncation
- [ ] Tests: V2 context with include_connected_sources=false returns empty connected_files
- [ ] Tests: apply_batch atomic mode writes all or none
- [ ] Tests: apply_batch atomic mode cleans up temp files on failure
- [ ] Tests: apply_batch non-atomic mode reports per-file success/failure
- [ ] Tests: apply_batch empty edits returns ApplyBatchEmptyEdits error
- [ ] Tests: apply_batch path traversal blocked for all edits
- [ ] Tests: apply_batch re-ingest fires once for all files
