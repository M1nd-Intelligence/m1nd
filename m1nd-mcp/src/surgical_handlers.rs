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

/// Validate that a path is within allowed workspace roots.
/// Returns Ok(canonical_path) or Err if path traversal is detected.
fn validate_path_safety(
    resolved: &Path,
    ingest_roots: &[String],
) -> M1ndResult<PathBuf> {
    // Canonicalize the resolved path (follows symlinks, resolves ..)
    let canonical = resolved.canonicalize().map_err(|e| M1ndError::InvalidParams {
        tool: "m1nd.apply".into(),
        detail: format!("cannot resolve path {}: {}", resolved.display(), e),
    })?;

    // If no ingest roots configured, allow any path
    if ingest_roots.is_empty() {
        return Ok(canonical);
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
