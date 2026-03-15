// === m1nd-mcp/src/surgical_handlers.rs ===
//
// surgical_context and apply tool handlers.
//
// surgical_context: returns complete context for an LLM to edit code surgically --
//   reads the target file, fetches graph neighbours (callers, callees, importers),
//   and packages provenance so the editor has everything it needs in one call.
//
// apply: writes LLM-edited code back to a file and triggers an incremental
//   re-ingest so the graph stays coherent with the changed source.
//
// Pattern: identical to layer_handlers.rs -- parse typed input -> call engine -> return output.

use crate::protocol::surgical;
use crate::session::SessionState;
use m1nd_core::error::{M1ndError, M1ndResult};
use m1nd_core::types::NodeId;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve a file_path input to an absolute path.
/// Handles both absolute paths and workspace-relative paths.
fn resolve_file_path(file_path: &str, ingest_roots: &[String]) -> PathBuf {
    let p = Path::new(file_path);
    if p.is_absolute() {
        p.to_path_buf()
    } else if let Some(root) = ingest_roots.first() {
        Path::new(root).join(file_path)
    } else {
        p.to_path_buf()
    }
}

/// Deny-list: m1nd state files that must never be overwritten by apply/apply_batch.
const DENIED_FILENAMES: &[&str] = &[
    "graph_snapshot.json",
    "plasticity_state.json",
    "antibodies.json",
    "tremor_state.json",
    "trust_state.json",
];

/// Validate that a path is within allowed workspace roots.
/// Returns Ok(canonical_path) or Err if path traversal is detected.
///
/// BUG FIX (E4): When ingest_roots is empty, REFUSE all writes instead of
/// allowing any path. At least one ingest must happen before any apply.
///
/// BUG FIX (E3): Deny-list prevents overwriting m1nd's own state files.
fn validate_path_safety(
    resolved: &Path,
    ingest_roots: &[String],
) -> M1ndResult<PathBuf> {
    // BUG FIX (E4): Block all writes when no ingest roots configured
    if ingest_roots.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "m1nd.apply".into(),
            detail: format!(
                "path {} cannot be written: no ingest roots configured (run m1nd.ingest first)",
                resolved.display()
            ),
        });
    }

    // Canonicalize the resolved path (follows symlinks, resolves ..)
    // For new files that don't exist yet, canonicalize the parent directory
    let canonical = if resolved.exists() {
        resolved.canonicalize().map_err(|e| M1ndError::InvalidParams {
            tool: "m1nd.apply".into(),
            detail: format!("cannot resolve path {}: {}", resolved.display(), e),
        })?
    } else {
        // File doesn't exist yet: canonicalize parent + append filename
        let parent = resolved.parent().unwrap_or(Path::new("."));
        let filename = resolved.file_name().unwrap_or_default();
        let parent_canonical = parent.canonicalize().map_err(|e| M1ndError::InvalidParams {
            tool: "m1nd.apply".into(),
            detail: format!("cannot resolve parent directory {}: {}", parent.display(), e),
        })?;
        parent_canonical.join(filename)
    };

    // BUG FIX (E3): Deny-list for m1nd state files
    if let Some(filename) = canonical.file_name().and_then(|f| f.to_str()) {
        if DENIED_FILENAMES.contains(&filename) {
            return Err(M1ndError::InvalidParams {
                tool: "m1nd.apply".into(),
                detail: format!(
                    "path {} is a protected m1nd state file and cannot be overwritten",
                    resolved.display()
                ),
            });
        }
    }

    // Check that canonical path starts with at least one ingest root
    for root in ingest_roots {
        if let Ok(root_canonical) = Path::new(root).canonicalize() {
            if canonical.starts_with(&root_canonical) {
                return Ok(canonical);
            }
        }
    }

    Err(M1ndError::InvalidParams {
        tool: "m1nd.apply".into(),
        detail: format!(
            "path {} is outside allowed workspace roots",
            resolved.display()
        ),
    })
}

/// Simple line-based diff summary: count added and removed lines.
fn diff_summary(old: &str, new: &str) -> (i32, i32) {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let old_set: HashSet<&str> = old_lines.iter().copied().collect();
    let new_set: HashSet<&str> = new_lines.iter().copied().collect();

    let removed = old_lines.iter().filter(|l| !new_set.contains(**l)).count() as i32;
    let added = new_lines.iter().filter(|l| !old_set.contains(**l)).count() as i32;
    (added, removed)
}

/// Extract symbols from file content (lightweight heuristic parser).
/// Works for Rust, Python, TypeScript/JavaScript, Go.
fn extract_symbols(content: &str, file_path: &str) -> Vec<surgical::SurgicalSymbol> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let lines: Vec<&str> = content.lines().collect();
    let mut symbols = Vec::new();

    match ext {
        "rs" => extract_rust_symbols(&lines, &mut symbols),
        "py" => extract_python_symbols(&lines, &mut symbols),
        "ts" | "tsx" | "js" | "jsx" => extract_ts_symbols(&lines, &mut symbols),
        "go" => extract_go_symbols(&lines, &mut symbols),
        _ => {} // Unknown language, no symbol extraction
    }

    symbols
}

fn extract_rust_symbols(lines: &[&str], symbols: &mut Vec<surgical::SurgicalSymbol>) {
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let line_num = (i + 1) as u32;

        // Match: pub fn, fn, pub struct, struct, pub enum, enum, pub trait, trait, impl
        let (name, sym_type) = if let Some(rest) = trimmed.strip_prefix("pub fn ")
            .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
            .or_else(|| trimmed.strip_prefix("pub(super) fn "))
        {
            (extract_identifier(rest), "function")
        } else if let Some(rest) = trimmed.strip_prefix("fn ") {
            if !trimmed.starts_with("fn ") || trimmed.contains("//") && trimmed.find("//").unwrap() < trimmed.find("fn").unwrap_or(0) {
                i += 1;
                continue;
            }
            (extract_identifier(rest), "function")
        } else if let Some(rest) = trimmed.strip_prefix("pub struct ")
            .or_else(|| trimmed.strip_prefix("pub(crate) struct "))
        {
            (extract_identifier(rest), "struct")
        } else if let Some(rest) = trimmed.strip_prefix("struct ") {
            (extract_identifier(rest), "struct")
        } else if let Some(rest) = trimmed.strip_prefix("pub enum ")
            .or_else(|| trimmed.strip_prefix("pub(crate) enum "))
        {
            (extract_identifier(rest), "enum")
        } else if let Some(rest) = trimmed.strip_prefix("enum ") {
            (extract_identifier(rest), "enum")
        } else if let Some(rest) = trimmed.strip_prefix("pub trait ")
            .or_else(|| trimmed.strip_prefix("pub(crate) trait "))
        {
            (extract_identifier(rest), "trait")
        } else if let Some(rest) = trimmed.strip_prefix("impl ") {
            (extract_identifier(rest), "impl")
        } else {
            i += 1;
            continue;
        };

        if name.is_empty() {
            i += 1;
            continue;
        }

        // Find the end of this symbol: track brace depth
        let line_end = find_brace_end(lines, i);
        let excerpt = build_excerpt(lines, i, line_end);

        symbols.push(surgical::SurgicalSymbol {
            name,
            symbol_type: sym_type.to_string(),
            line_start: line_num,
            line_end: (line_end + 1) as u32,
            excerpt: Some(excerpt),
        });

        i = line_end + 1;
    }
}

fn extract_python_symbols(lines: &[&str], symbols: &mut Vec<surgical::SurgicalSymbol>) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_num = (i + 1) as u32;

        let (name, sym_type) = if let Some(rest) = trimmed.strip_prefix("def ") {
            (extract_identifier(rest), "function")
        } else if let Some(rest) = trimmed.strip_prefix("class ") {
            (extract_identifier(rest), "class")
        } else if let Some(rest) = trimmed.strip_prefix("async def ") {
            (extract_identifier(rest), "function")
        } else {
            continue;
        };

        if name.is_empty() {
            continue;
        }

        // Find end by indentation: next line at same or lower indent level
        let base_indent = line.len() - line.trim_start().len();
        let mut end = i;
        for j in (i + 1)..lines.len() {
            let next = lines[j];
            if next.trim().is_empty() {
                continue;
            }
            let next_indent = next.len() - next.trim_start().len();
            if next_indent <= base_indent {
                break;
            }
            end = j;
        }

        let excerpt = build_excerpt(lines, i, end);
        symbols.push(surgical::SurgicalSymbol {
            name,
            symbol_type: sym_type.to_string(),
            line_start: line_num,
            line_end: (end + 1) as u32,
            excerpt: Some(excerpt),
        });
    }
}

fn extract_ts_symbols(lines: &[&str], symbols: &mut Vec<surgical::SurgicalSymbol>) {
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let line_num = (i + 1) as u32;

        let (name, sym_type) = if trimmed.contains("function ") {
            let after = trimmed.split("function ").nth(1).unwrap_or("");
            (extract_identifier(after), "function")
        } else if trimmed.contains("class ") {
            let after = trimmed.split("class ").nth(1).unwrap_or("");
            (extract_identifier(after), "class")
        } else if trimmed.starts_with("export ") && trimmed.contains("const ") {
            let after = trimmed.split("const ").nth(1).unwrap_or("");
            (extract_identifier(after), "const")
        } else if trimmed.starts_with("interface ") || trimmed.starts_with("export interface ") {
            let after = trimmed.split("interface ").nth(1).unwrap_or("");
            (extract_identifier(after), "interface")
        } else {
            i += 1;
            continue;
        };

        if name.is_empty() {
            i += 1;
            continue;
        }

        let line_end = find_brace_end(lines, i);
        let excerpt = build_excerpt(lines, i, line_end);

        symbols.push(surgical::SurgicalSymbol {
            name,
            symbol_type: sym_type.to_string(),
            line_start: line_num,
            line_end: (line_end + 1) as u32,
            excerpt: Some(excerpt),
        });

        i = line_end + 1;
    }
}

fn extract_go_symbols(lines: &[&str], symbols: &mut Vec<surgical::SurgicalSymbol>) {
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let line_num = (i + 1) as u32;

        let (name, sym_type) = if let Some(rest) = trimmed.strip_prefix("func ") {
            (extract_identifier(rest), "function")
        } else if let Some(rest) = trimmed.strip_prefix("type ") {
            let ident = extract_identifier(rest);
            let remainder = rest.get(ident.len()..).unwrap_or("").trim();
            if remainder.starts_with("struct") {
                (ident, "struct")
            } else if remainder.starts_with("interface") {
                (ident, "interface")
            } else {
                (ident, "type")
            }
        } else {
            i += 1;
            continue;
        };

        if name.is_empty() {
            i += 1;
            continue;
        }

        let line_end = find_brace_end(lines, i);
        let excerpt = build_excerpt(lines, i, line_end);

        symbols.push(surgical::SurgicalSymbol {
            name,
            symbol_type: sym_type.to_string(),
            line_start: line_num,
            line_end: (line_end + 1) as u32,
            excerpt: Some(excerpt),
        });

        i = line_end + 1;
    }
}

/// Extract an identifier from the start of a string.
fn extract_identifier(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Find the line index where a brace-delimited block ends.
/// Returns the line index of the closing brace.
fn find_brace_end(lines: &[&str], start: usize) -> usize {
    let mut depth: i32 = 0;
    let mut found_open = false;

    for i in start..lines.len() {
        for ch in lines[i].chars() {
            if ch == '{' {
                depth += 1;
                found_open = true;
            } else if ch == '}' {
                depth -= 1;
                if found_open && depth == 0 {
                    return i;
                }
            }
        }
    }

    // If no closing brace found, return end of file or start + reasonable range
    (start + 50).min(lines.len().saturating_sub(1))
}

/// Build an excerpt from lines (first 20 lines of the symbol).
fn build_excerpt(lines: &[&str], start: usize, end: usize) -> String {
    let max_lines = 20;
    let actual_end = (start + max_lines).min(end + 1).min(lines.len());
    let excerpt_lines: Vec<&str> = lines[start..actual_end].to_vec();
    let mut result = excerpt_lines.join("\n");
    if actual_end <= end {
        result.push_str("\n    // ... (truncated)");
    }
    result
}

/// Collect graph neighbours of a node within a given BFS radius.
/// Returns (callers, callees, test_neighbours).
fn collect_neighbours(
    state: &SessionState,
    node: NodeId,
    radius: u32,
    include_tests: bool,
) -> (
    Vec<surgical::SurgicalNeighbour>,
    Vec<surgical::SurgicalNeighbour>,
    Vec<surgical::SurgicalNeighbour>,
) {
    let graph = state.graph.read();
    let n = graph.num_nodes() as usize;
    let idx = node.as_usize();

    if idx >= n || !graph.finalized {
        return (vec![], vec![], vec![]);
    }

    let mut callers = Vec::new();
    let mut callees = Vec::new();
    let mut tests = Vec::new();

    // BFS: collect nodes at each radius level
    let mut visited = HashSet::new();
    visited.insert(node);
    let mut current_frontier = vec![node];

    for _depth in 0..radius {
        let mut next_frontier = Vec::new();

        for &frontier_node in &current_frontier {
            let fi = frontier_node.as_usize();
            if fi >= n {
                continue;
            }

            // Forward edges (callees): this node -> target
            let out_range = graph.csr.out_range(frontier_node);
            for edge_pos in out_range {
                let target = graph.csr.targets[edge_pos];
                if visited.contains(&target) {
                    continue;
                }
                visited.insert(target);
                next_frontier.push(target);

                let ti = target.as_usize();
                if ti >= n {
                    continue;
                }

                let label = graph.strings.resolve(graph.nodes.label[ti]).to_string();
                let relation = graph.strings.resolve(graph.csr.relations[edge_pos]).to_string();
                let weight = graph.csr.read_weight(m1nd_core::types::EdgeIdx::new(edge_pos as u32)).get();

                let prov = graph.resolve_node_provenance(target);
                let file_path = prov.source_path.clone().unwrap_or_default();

                let neighbour = surgical::SurgicalNeighbour {
                    node_id: resolve_external_id(&graph, target),
                    label: label.clone(),
                    file_path: file_path.clone(),
                    relation: relation.clone(),
                    edge_weight: weight,
                };

                // Classify: test file or callee
                let is_test = include_tests && (
                    relation.contains("test") ||
                    label.contains("test") ||
                    file_path.contains("test") ||
                    file_path.contains("spec")
                );

                if is_test {
                    tests.push(neighbour);
                } else {
                    callees.push(neighbour);
                }
            }

            // Reverse edges (callers): source -> this node
            let in_range = graph.csr.in_range(frontier_node);
            for rev_pos in in_range {
                let source = graph.csr.rev_sources[rev_pos];
                if visited.contains(&source) {
                    continue;
                }
                visited.insert(source);
                next_frontier.push(source);

                let si = source.as_usize();
                if si >= n {
                    continue;
                }

                let label = graph.strings.resolve(graph.nodes.label[si]).to_string();
                let fwd_idx = graph.csr.rev_edge_idx[rev_pos];
                let relation = graph.strings.resolve(graph.csr.relations[fwd_idx.as_usize()]).to_string();
                let weight = graph.csr.read_weight(fwd_idx).get();

                let prov = graph.resolve_node_provenance(source);
                let file_path = prov.source_path.clone().unwrap_or_default();

                let neighbour = surgical::SurgicalNeighbour {
                    node_id: resolve_external_id(&graph, source),
                    label: label.clone(),
                    file_path: file_path.clone(),
                    relation: relation.clone(),
                    edge_weight: weight,
                };

                let is_test = include_tests && (
                    relation.contains("test") ||
                    label.contains("test") ||
                    file_path.contains("test") ||
                    file_path.contains("spec")
                );

                if is_test {
                    tests.push(neighbour);
                } else {
                    callers.push(neighbour);
                }
            }
        }

        current_frontier = next_frontier;
    }

    // Sort by edge weight descending for relevance
    callers.sort_by(|a, b| b.edge_weight.partial_cmp(&a.edge_weight).unwrap_or(std::cmp::Ordering::Equal));
    callees.sort_by(|a, b| b.edge_weight.partial_cmp(&a.edge_weight).unwrap_or(std::cmp::Ordering::Equal));
    tests.sort_by(|a, b| b.edge_weight.partial_cmp(&a.edge_weight).unwrap_or(std::cmp::Ordering::Equal));

    (callers, callees, tests)
}

/// Resolve the external string ID for a NodeId.
fn resolve_external_id(graph: &m1nd_core::graph::Graph, node: NodeId) -> String {
    for (interned, &nid) in &graph.id_to_node {
        if nid == node {
            return graph.strings.resolve(*interned).to_string();
        }
    }
    format!("node_{}", node.as_usize())
}

/// Find graph nodes whose provenance source_path matches the given file path.
fn find_nodes_for_file(
    graph: &m1nd_core::graph::Graph,
    file_path: &str,
) -> Vec<(NodeId, String)> {
    let n = graph.num_nodes() as usize;
    let mut results = Vec::new();

    // Normalize path for comparison
    let normalized = file_path.replace('\\', "/");

    for i in 0..n {
        let prov = &graph.nodes.provenance[i];
        if let Some(sp) = prov.source_path {
            if let Some(path_str) = graph.strings.try_resolve(sp) {
                let path_normalized = path_str.replace('\\', "/");
                if path_normalized == normalized
                    || path_normalized.ends_with(&normalized)
                    || normalized.ends_with(&path_normalized)
                {
                    let nid = NodeId::new(i as u32);
                    let ext_id = resolve_external_id(graph, nid);
                    results.push((nid, ext_id));
                }
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// m1nd.surgical_context
// ---------------------------------------------------------------------------

/// Handle m1nd.surgical_context.
///
/// Returns everything an LLM needs to edit `file_path` surgically:
///   - full file contents
///   - graph context: callers, callees, importers, test files
///   - provenance: node_ids, line ranges
///   - optional focused symbol slice when `symbol` is provided
///
/// Steps:
///   1. Reading `input.file_path` from disk.
///   2. Finding all graph nodes whose provenance matches this file.
///   3. BFS to radius 1-2 to gather callers / callees / tests.
///   4. Optionally narrowing to a specific symbol via line-range extraction.
///   5. Returning a `SurgicalContextOutput` with all fields populated.
pub fn handle_surgical_context(
    state: &mut SessionState,
    input: surgical::SurgicalContextInput,
) -> M1ndResult<surgical::SurgicalContextOutput> {
    let start = Instant::now();

    // Step 1: Resolve and read the file
    let resolved_path = resolve_file_path(&input.file_path, &state.ingest_roots);
    let file_contents = std::fs::read_to_string(&resolved_path).map_err(|e| {
        M1ndError::InvalidParams {
            tool: "m1nd.surgical_context".into(),
            detail: format!(
                "cannot read file {}: {}",
                resolved_path.display(),
                e
            ),
        }
    })?;

    let line_count = file_contents.lines().count() as u32;

    // Step 2: Extract symbols from file content
    let path_str = resolved_path.to_string_lossy().to_string();
    let symbols = extract_symbols(&file_contents, &path_str);

    // Step 3: Find graph nodes for this file
    let graph = state.graph.read();
    let file_nodes = find_nodes_for_file(&graph, &path_str);
    drop(graph);

    // Pick the primary node (prefer File-type node, otherwise first match)
    let primary_node: Option<(NodeId, String)> = {
        let graph = state.graph.read();
        let file_type_node = file_nodes.iter().find(|(nid, _)| {
            let idx = nid.as_usize();
            idx < graph.num_nodes() as usize
                && graph.nodes.node_type[idx] == m1nd_core::types::NodeType::File
        });
        file_type_node
            .or(file_nodes.first())
            .cloned()
    };

    let node_id_str = primary_node
        .as_ref()
        .map(|(_, ext)| ext.clone())
        .unwrap_or_default();

    // Step 4: Collect graph neighbours via BFS
    let (callers, callees, tests) = if let Some((nid, _)) = &primary_node {
        collect_neighbours(state, *nid, input.radius, input.include_tests)
    } else {
        // No graph node found -- also try collecting from all file nodes
        let mut all_callers = Vec::new();
        let mut all_callees = Vec::new();
        let mut all_tests = Vec::new();
        for (nid, _) in &file_nodes {
            let (c, d, t) = collect_neighbours(state, *nid, input.radius, input.include_tests);
            all_callers.extend(c);
            all_callees.extend(d);
            all_tests.extend(t);
        }
        (all_callers, all_callees, all_tests)
    };

    // Step 5: Focused symbol (if requested)
    let focused_symbol = input.symbol.as_ref().and_then(|sym_name| {
        symbols.iter().find(|s| {
            s.name.eq_ignore_ascii_case(sym_name)
                || s.name == *sym_name
        }).cloned()
    });

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Track agent session
    state.track_agent(&input.agent_id);

    Ok(surgical::SurgicalContextOutput {
        file_path: path_str,
        file_contents,
        line_count,
        node_id: node_id_str,
        symbols,
        focused_symbol,
        callers,
        callees,
        tests,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// m1nd.apply
// ---------------------------------------------------------------------------

/// Handle m1nd.apply.
///
/// Writes LLM-edited code back to `file_path` and triggers an incremental
/// re-ingest so the graph reflects the new source.
///
/// Steps:
///   1. Validating `input.file_path` is within the workspace root (no path traversal).
///   2. Reading old content for diff.
///   3. Atomically writing `input.new_content` to disk (write-then-rename).
///   4. If reingest: performing incremental re-ingest via single-file ingest.
///   5. Returning diff summary + updated node_ids in `ApplyOutput`.
pub fn handle_apply(
    state: &mut SessionState,
    input: surgical::ApplyInput,
) -> M1ndResult<surgical::ApplyOutput> {
    let start = Instant::now();

    // Step 1: Resolve and validate path
    let resolved_path = resolve_file_path(&input.file_path, &state.ingest_roots);
    let validated_path = validate_path_safety(&resolved_path, &state.ingest_roots)?;

    // Step 2: Read old content for diff (if file exists)
    let old_content = std::fs::read_to_string(&validated_path).unwrap_or_default();
    let (lines_added, lines_removed) = diff_summary(&old_content, &input.new_content);
    let bytes_written = input.new_content.len();

    // Step 3: Atomic write -- write to temp file, then rename
    let parent = validated_path.parent().unwrap_or(Path::new("."));
    let temp_path = parent.join(format!(
        ".m1nd_apply_{}.tmp",
        std::process::id()
    ));

    // Ensure parent directory exists
    if !parent.exists() {
        std::fs::create_dir_all(parent).map_err(|e| M1ndError::InvalidParams {
            tool: "m1nd.apply".into(),
            detail: format!("cannot create directory {}: {}", parent.display(), e),
        })?;
    }

    // Write to temp file
    std::fs::write(&temp_path, &input.new_content).map_err(|e| {
        M1ndError::InvalidParams {
            tool: "m1nd.apply".into(),
            detail: format!("cannot write temp file {}: {}", temp_path.display(), e),
        }
    })?;

    // Rename (atomic on same filesystem)
    std::fs::rename(&temp_path, &validated_path).map_err(|e| {
        // Clean up temp file on rename failure
        let _ = std::fs::remove_file(&temp_path);
        M1ndError::InvalidParams {
            tool: "m1nd.apply".into(),
            detail: format!(
                "atomic rename failed {} -> {}: {}",
                temp_path.display(),
                validated_path.display(),
                e
            ),
        }
    })?;

    // Step 4: Incremental re-ingest (if requested)
    let mut updated_node_ids = Vec::new();
    let reingested = if input.reingest {
        // Find existing nodes for this file before re-ingest
        {
            let graph = state.graph.read();
            let path_str = validated_path.to_string_lossy().to_string();
            let existing = find_nodes_for_file(&graph, &path_str);
            for (_, ext_id) in &existing {
                updated_node_ids.push(ext_id.clone());
            }
        }

        // Attempt incremental ingest via single-file code ingest
        let ingest_input = crate::protocol::IngestInput {
            path: validated_path.to_string_lossy().to_string(),
            agent_id: input.agent_id.clone(),
            mode: "merge".to_string(),
            incremental: true,
            adapter: "code".to_string(),
            namespace: None,
        };

        match crate::tools::handle_ingest(state, ingest_input) {
            Ok(result) => {
                // Extract any new node IDs from the ingest result
                if let Some(obj) = result.as_object() {
                    if let Some(nodes) = obj.get("nodes_created") {
                        if let Some(n) = nodes.as_u64() {
                            if n > 0 && updated_node_ids.is_empty() {
                                updated_node_ids.push(format!(
                                    "file::{}",
                                    validated_path.to_string_lossy()
                                ));
                            }
                        }
                    }
                }
                true
            }
            Err(e) => {
                // Re-ingest failure is non-fatal -- file was already written successfully
                eprintln!(
                    "[m1nd] WARNING: apply re-ingest failed for {}: {}",
                    validated_path.display(),
                    e
                );
                false
            }
        }
    } else {
        false
    };

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Track agent session
    state.track_agent(&input.agent_id);

    Ok(surgical::ApplyOutput {
        file_path: validated_path.to_string_lossy().to_string(),
        bytes_written,
        lines_added,
        lines_removed,
        reingested,
        updated_node_ids,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// m1nd.surgical_context_v2
// ---------------------------------------------------------------------------

/// Handle m1nd.surgical_context_v2.
///
/// Returns V1 surgical context for the primary file PLUS source code
/// of connected files (callers, callees, tests), sorted by edge_weight,
/// capped at max_connected_files, truncated at max_lines_per_file.
///
/// Steps:
///   1. Delegate to handle_surgical_context() for the primary file (V1 output).
///   2. Collect unique file paths from primary.callers + callees + tests.
///   3. Deduplicate by file_path (keep highest weight per path).
///   4. Sort by edge_weight descending, take top max_connected_files.
///   5. Read each connected file's source (truncate to max_lines_per_file).
///   6. Assemble SurgicalContextV2Output.
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

    // Step 2: Collect candidate files from neighbourhood
    // Use a HashMap to deduplicate by file_path, keeping highest weight
    let primary_path = primary.file_path.clone();
    let primary_node_id = primary.node_id.clone();
    let mut candidate_map: std::collections::HashMap<String, (String, String, String, f32)> =
        std::collections::HashMap::new(); // path -> (node_id, label, relation, weight)

    for caller in &primary.callers {
        if !caller.file_path.is_empty() && caller.file_path != primary_path {
            let entry = candidate_map.entry(caller.file_path.clone()).or_insert((
                caller.node_id.clone(),
                caller.label.clone(),
                "caller".to_string(),
                caller.edge_weight,
            ));
            if caller.edge_weight > entry.3 {
                *entry = (
                    caller.node_id.clone(),
                    caller.label.clone(),
                    "caller".to_string(),
                    caller.edge_weight,
                );
            }
        }
    }
    for callee in &primary.callees {
        if !callee.file_path.is_empty() && callee.file_path != primary_path {
            let entry = candidate_map.entry(callee.file_path.clone()).or_insert((
                callee.node_id.clone(),
                callee.label.clone(),
                "callee".to_string(),
                callee.edge_weight,
            ));
            if callee.edge_weight > entry.3 {
                *entry = (
                    callee.node_id.clone(),
                    callee.label.clone(),
                    "callee".to_string(),
                    callee.edge_weight,
                );
            }
        }
    }
    for test in &primary.tests {
        if !test.file_path.is_empty() && test.file_path != primary_path {
            let entry = candidate_map.entry(test.file_path.clone()).or_insert((
                test.node_id.clone(),
                test.label.clone(),
                "test".to_string(),
                test.edge_weight,
            ));
            if test.edge_weight > entry.3 {
                *entry = (
                    test.node_id.clone(),
                    test.label.clone(),
                    "test".to_string(),
                    test.edge_weight,
                );
            }
        }
    }

    // Also exclude primary node_id from connected set (circular guard)
    candidate_map.retain(|_, (nid, _, _, _)| *nid != primary_node_id);

    // Step 3: Sort by edge_weight descending, cap at max_connected_files
    let mut scored: Vec<(String, String, String, String, f32)> = candidate_map
        .into_iter()
        .map(|(path, (nid, label, rel, w))| (path, nid, label, rel, w))
        .collect();
    scored.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(input.max_connected_files);

    // Step 4: Read each connected file, build ConnectedFileSource
    let mut connected_files: Vec<surgical::ConnectedFileSource> = Vec::new();
    let mut total_lines = primary.line_count as usize;
    let max_lines = input.max_lines_per_file;

    for (path, node_id, label, relation_type, edge_weight) in &scored {
        let resolved = resolve_file_path(path, &state.ingest_roots);
        match std::fs::read_to_string(&resolved) {
            Ok(content) => {
                let all_lines: Vec<&str> = content.lines().collect();
                let file_line_count = all_lines.len();
                let truncated = file_line_count > max_lines;
                let excerpt_lines = if truncated { max_lines } else { file_line_count };
                let source_excerpt: String = all_lines
                    .iter()
                    .take(excerpt_lines)
                    .cloned()
                    .collect::<Vec<&str>>()
                    .join("\n");

                total_lines += excerpt_lines;

                connected_files.push(surgical::ConnectedFileSource {
                    node_id: node_id.clone(),
                    label: label.clone(),
                    file_path: resolved.to_string_lossy().to_string(),
                    relation_type: relation_type.clone(),
                    edge_weight: *edge_weight,
                    source_excerpt,
                    excerpt_lines,
                    truncated,
                });
            }
            Err(e) => {
                // Non-fatal: skip unreadable/binary files
                eprintln!(
                    "[m1nd] WARNING: surgical_context_v2 cannot read connected file {}: {}",
                    resolved.display(),
                    e
                );
            }
        }
    }

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    state.track_agent(&input.agent_id);

    Ok(surgical::SurgicalContextV2Output {
        file_path: primary.file_path,
        file_contents: primary.file_contents,
        line_count: primary.line_count,
        node_id: primary.node_id,
        symbols: primary.symbols,
        focused_symbol: primary.focused_symbol,
        connected_files,
        total_lines,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// m1nd.apply_batch
// ---------------------------------------------------------------------------

/// Handle m1nd.apply_batch.
///
/// Writes multiple files atomically and triggers a single bulk re-ingest.
///
/// Steps:
///   1. Empty edits = fast-path no-op.
///   2. Resolve and validate all file paths (path safety check) BEFORE any writes.
///   3. Read old content for each file (for diff).
///   4. ATOMIC mode: write all files to unique temp files first.
///      If any temp write fails, clean up all temp files and return error.
///      Then rename all temp files to targets.
///   5. NON-ATOMIC mode: write each file independently via temp+rename.
///   6. Compute diffs per file.
///   7. If reingest: bulk re-ingest all modified files in one pass.
///   8. Assemble ApplyBatchOutput.
pub fn handle_apply_batch(
    state: &mut SessionState,
    input: surgical::ApplyBatchInput,
) -> M1ndResult<surgical::ApplyBatchOutput> {
    let start = Instant::now();

    // Step 1: Empty edits = fast-path no-op
    if input.edits.is_empty() {
        return Ok(surgical::ApplyBatchOutput {
            all_succeeded: true,
            files_written: 0,
            files_total: 0,
            results: Vec::new(),
            reingested: false,
            total_bytes_written: 0,
            elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
        });
    }

    // Step 2: Resolve and validate ALL paths upfront (fail-fast before any writes)
    let mut resolved_edits: Vec<(PathBuf, &surgical::BatchEditItem, String)> = Vec::new();
    for edit in &input.edits {
        let resolved = resolve_file_path(&edit.file_path, &state.ingest_roots);
        let validated = validate_path_safety(&resolved, &state.ingest_roots)?;
        // Read old content for diff (empty string if new file)
        let old_content = std::fs::read_to_string(&validated).unwrap_or_default();
        resolved_edits.push((validated, edit, old_content));
    }

    let mut results: Vec<surgical::BatchEditResult> = Vec::new();
    let mut total_bytes_written: usize = 0;

    if input.atomic {
        // --- ATOMIC MODE: all-or-nothing ---

        // Phase 1: Write all to unique temp files
        let mut temp_files: Vec<(PathBuf, PathBuf)> = Vec::new(); // (tmp_path, target_path)
        let pid = std::process::id();
        let batch_id = start.elapsed().as_nanos(); // unique per batch call

        for (i, (validated, edit, _old)) in resolved_edits.iter().enumerate() {
            let parent = validated.parent().unwrap_or(Path::new("."));

            // Ensure parent directory exists
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    // Clean up temp files written so far
                    for (tmp, _) in &temp_files {
                        let _ = std::fs::remove_file(tmp);
                    }
                    return Err(M1ndError::InvalidParams {
                        tool: "m1nd.apply_batch".into(),
                        detail: format!(
                            "cannot create directory {}: {}",
                            parent.display(),
                            e
                        ),
                    });
                }
            }

            // BUG FIX (B2): unique temp file per edit (pid + batch_id + index)
            let tmp_path = parent.join(format!(
                ".m1nd_batch_{}_{}_{}_.tmp",
                pid, batch_id, i
            ));

            match std::fs::write(&tmp_path, &edit.new_content) {
                Ok(_) => {
                    temp_files.push((tmp_path, validated.clone()));
                }
                Err(e) => {
                    // Clean up already-written temp files
                    for (tmp, _) in &temp_files {
                        let _ = std::fs::remove_file(tmp);
                    }
                    return Err(M1ndError::InvalidParams {
                        tool: "m1nd.apply_batch".into(),
                        detail: format!(
                            "atomic batch failed: cannot write temp file for {}: {}",
                            validated.display(),
                            e
                        ),
                    });
                }
            }
        }

        // Phase 2: Rename all temp files to targets (atomic per-file)
        let mut renamed_files: Vec<(PathBuf, String)> = Vec::new(); // (target, old_content for rollback)
        for (idx, (tmp_path, target_path)) in temp_files.iter().enumerate() {
            if let Err(e) = std::fs::rename(tmp_path, target_path) {
                // Rename failure: rollback already-renamed files by restoring old content
                for (rollback_target, old_content) in &renamed_files {
                    let _ = std::fs::write(rollback_target, old_content);
                }
                // Clean up remaining temp files
                for (tmp, _) in temp_files.iter().skip(idx) {
                    let _ = std::fs::remove_file(tmp);
                }
                return Err(M1ndError::InvalidParams {
                    tool: "m1nd.apply_batch".into(),
                    detail: format!(
                        "atomic rename failed {} -> {}: {}",
                        tmp_path.display(),
                        target_path.display(),
                        e
                    ),
                });
            }
            // Track for potential rollback
            renamed_files.push((
                target_path.clone(),
                resolved_edits[idx].2.clone(), // old_content
            ));
        }

        // Phase 3: Compute diffs for all successfully written files
        for (validated, edit, old_content) in &resolved_edits {
            let (added, removed) = diff_summary(old_content, &edit.new_content);
            let bytes = edit.new_content.len();
            total_bytes_written += bytes;

            // Build a simple unified diff string
            let diff_str = format!(
                "@@ -{},{} +{},{} @@\n{}{}",
                1,
                old_content.lines().count(),
                1,
                edit.new_content.lines().count(),
                old_content.lines().take(3).map(|l| format!("-{}\n", l)).collect::<String>(),
                edit.new_content.lines().take(3).map(|l| format!("+{}\n", l)).collect::<String>(),
            );

            results.push(surgical::BatchEditResult {
                file_path: validated.to_string_lossy().to_string(),
                success: true,
                diff: diff_str,
                lines_added: added,
                lines_removed: removed,
                error: None,
            });
        }
    } else {
        // --- NON-ATOMIC MODE: write each file independently ---
        let pid = std::process::id();
        let batch_id = start.elapsed().as_nanos();

        for (i, (validated, edit, old_content)) in resolved_edits.iter().enumerate() {
            let parent = validated.parent().unwrap_or(Path::new("."));

            // Ensure parent directory exists
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }

            // Unique temp file per edit (same fix as atomic)
            let tmp_path = parent.join(format!(
                ".m1nd_batch_{}_{}_{}_.tmp",
                pid, batch_id, i
            ));

            match std::fs::write(&tmp_path, &edit.new_content)
                .and_then(|_| std::fs::rename(&tmp_path, validated))
            {
                Ok(_) => {
                    let (added, removed) = diff_summary(old_content, &edit.new_content);
                    let bytes = edit.new_content.len();
                    total_bytes_written += bytes;

                    let diff_str = format!(
                        "@@ -{},{} +{},{} @@\n{}{}",
                        1,
                        old_content.lines().count(),
                        1,
                        edit.new_content.lines().count(),
                        old_content.lines().take(3).map(|l| format!("-{}\n", l)).collect::<String>(),
                        edit.new_content.lines().take(3).map(|l| format!("+{}\n", l)).collect::<String>(),
                    );

                    results.push(surgical::BatchEditResult {
                        file_path: validated.to_string_lossy().to_string(),
                        success: true,
                        diff: diff_str,
                        lines_added: added,
                        lines_removed: removed,
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp_path);
                    results.push(surgical::BatchEditResult {
                        file_path: validated.to_string_lossy().to_string(),
                        success: false,
                        diff: String::new(),
                        lines_added: 0,
                        lines_removed: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
    }

    // Step 7: Bulk re-ingest (single pass covering all successfully written files)
    let files_written = results.iter().filter(|r| r.success).count();
    let all_succeeded = files_written == input.edits.len();

    let reingested = if input.reingest && files_written > 0 {
        let successful_paths: Vec<String> = results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.file_path.clone())
            .collect();

        let mut any_ingested = false;
        for path in &successful_paths {
            let ingest_input = crate::protocol::IngestInput {
                path: path.clone(),
                agent_id: input.agent_id.clone(),
                mode: "merge".to_string(),
                incremental: true,
                adapter: "code".to_string(),
                namespace: None,
            };

            match crate::tools::handle_ingest(state, ingest_input) {
                Ok(_) => {
                    any_ingested = true;
                }
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
        all_succeeded,
        files_written,
        files_total: input.edits.len(),
        results,
        reingested,
        total_bytes_written,
        elapsed_ms,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_identifier() {
        assert_eq!(extract_identifier("handle_apply(state)"), "handle_apply");
        assert_eq!(extract_identifier("MyStruct {"), "MyStruct");
        assert_eq!(extract_identifier(""), "");
        // Alphanumeric sequences including leading digits are accepted
        // (the caller context -- e.g. `fn ` prefix -- ensures valid identifiers)
        assert_eq!(extract_identifier("123abc"), "123abc");
        assert_eq!(extract_identifier("(foo)"), "");
    }

    #[test]
    fn test_diff_summary() {
        let old = "line1\nline2\nline3";
        let new = "line1\nline2_modified\nline3\nline4";
        let (added, removed) = diff_summary(old, new);
        assert!(added > 0);
        assert!(removed > 0);
    }

    #[test]
    fn test_diff_summary_identical() {
        let content = "line1\nline2\nline3";
        let (added, removed) = diff_summary(content, content);
        assert_eq!(added, 0);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_find_brace_end_simple() {
        let lines = vec!["fn foo() {", "    bar();", "}"];
        assert_eq!(find_brace_end(&lines, 0), 2);
    }

    #[test]
    fn test_find_brace_end_nested() {
        let lines = vec!["fn foo() {", "    if true {", "        bar();", "    }", "}"];
        assert_eq!(find_brace_end(&lines, 0), 4);
    }

    #[test]
    fn test_extract_rust_symbols_basic() {
        let content = "pub fn handle_apply(\n    state: &mut SessionState,\n) -> Result<()> {\n    todo!()\n}\n";
        let symbols = extract_symbols(content, "test.rs");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "handle_apply");
        assert_eq!(symbols[0].symbol_type, "function");
    }

    #[test]
    fn test_extract_python_symbols() {
        let content = "def my_function():\n    pass\n\nclass MyClass:\n    pass\n";
        let symbols = extract_symbols(content, "test.py");
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "my_function");
        assert_eq!(symbols[0].symbol_type, "function");
        assert_eq!(symbols[1].name, "MyClass");
        assert_eq!(symbols[1].symbol_type, "class");
    }

    #[test]
    fn test_resolve_file_path_absolute() {
        let p = resolve_file_path("/absolute/path/file.rs", &[]);
        assert_eq!(p, PathBuf::from("/absolute/path/file.rs"));
    }

    #[test]
    fn test_resolve_file_path_relative_with_root() {
        let roots = vec!["/workspace".to_string()];
        let p = resolve_file_path("src/main.rs", &roots);
        assert_eq!(p, PathBuf::from("/workspace/src/main.rs"));
    }

    #[test]
    fn test_build_excerpt_truncation() {
        let lines: Vec<&str> = (0..30).map(|_| "code line").collect();
        let excerpt = build_excerpt(&lines, 0, 29);
        assert!(excerpt.contains("truncated"));
    }

    #[test]
    fn test_build_excerpt_short() {
        let lines = vec!["line1", "line2", "line3"];
        let excerpt = build_excerpt(&lines, 0, 2);
        assert!(!excerpt.contains("truncated"));
        assert!(excerpt.contains("line1"));
        assert!(excerpt.contains("line3"));
    }
}
