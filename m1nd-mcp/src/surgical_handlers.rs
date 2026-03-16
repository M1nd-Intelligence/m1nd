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
use m1nd_core::types::{EdgeIdx, NodeId, NodeType};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
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
fn validate_path_safety(resolved: &Path, ingest_roots: &[String]) -> M1ndResult<PathBuf> {
    // BUG FIX (E4): Block all writes when no ingest roots configured
    if ingest_roots.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "m1nd_apply".into(),
            detail: format!(
                "path {} cannot be written: no ingest roots configured (run m1nd.ingest first)",
                resolved.display()
            ),
        });
    }

    // Canonicalize the resolved path (follows symlinks, resolves ..)
    // For new files that don't exist yet, canonicalize the parent directory
    let canonical = if resolved.exists() {
        resolved
            .canonicalize()
            .map_err(|e| M1ndError::InvalidParams {
                tool: "m1nd_apply".into(),
                detail: format!("cannot resolve path {}: {}", resolved.display(), e),
            })?
    } else {
        // File doesn't exist yet: canonicalize parent + append filename
        let parent = resolved.parent().unwrap_or(Path::new("."));
        let filename = resolved.file_name().unwrap_or_default();
        let parent_canonical = parent
            .canonicalize()
            .map_err(|e| M1ndError::InvalidParams {
                tool: "m1nd_apply".into(),
                detail: format!(
                    "cannot resolve parent directory {}: {}",
                    parent.display(),
                    e
                ),
            })?;
        parent_canonical.join(filename)
    };

    // BUG FIX (E3): Deny-list for m1nd state files
    if let Some(filename) = canonical.file_name().and_then(|f| f.to_str()) {
        if DENIED_FILENAMES.contains(&filename) {
            return Err(M1ndError::InvalidParams {
                tool: "m1nd_apply".into(),
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
        tool: "m1nd_apply".into(),
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
        let (name, sym_type) = if let Some(rest) = trimmed
            .strip_prefix("pub fn ")
            .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
            .or_else(|| trimmed.strip_prefix("pub(super) fn "))
        {
            (extract_identifier(rest), "function")
        } else if let Some(rest) = trimmed.strip_prefix("fn ") {
            if !trimmed.starts_with("fn ")
                || trimmed.contains("//")
                    && trimmed.find("//").unwrap() < trimmed.find("fn").unwrap_or(0)
            {
                i += 1;
                continue;
            }
            (extract_identifier(rest), "function")
        } else if let Some(rest) = trimmed
            .strip_prefix("pub struct ")
            .or_else(|| trimmed.strip_prefix("pub(crate) struct "))
        {
            (extract_identifier(rest), "struct")
        } else if let Some(rest) = trimmed.strip_prefix("struct ") {
            (extract_identifier(rest), "struct")
        } else if let Some(rest) = trimmed
            .strip_prefix("pub enum ")
            .or_else(|| trimmed.strip_prefix("pub(crate) enum "))
        {
            (extract_identifier(rest), "enum")
        } else if let Some(rest) = trimmed.strip_prefix("enum ") {
            (extract_identifier(rest), "enum")
        } else if let Some(rest) = trimmed
            .strip_prefix("pub trait ")
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
        #[allow(clippy::needless_range_loop)]
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

    #[allow(clippy::needless_range_loop)]
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
                let relation = graph
                    .strings
                    .resolve(graph.csr.relations[edge_pos])
                    .to_string();
                let weight = graph
                    .csr
                    .read_weight(m1nd_core::types::EdgeIdx::new(edge_pos as u32))
                    .get();

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
                let is_test = include_tests
                    && (relation.contains("test")
                        || label.contains("test")
                        || file_path.contains("test")
                        || file_path.contains("spec"));

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
                let relation = graph
                    .strings
                    .resolve(graph.csr.relations[fwd_idx.as_usize()])
                    .to_string();
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

                let is_test = include_tests
                    && (relation.contains("test")
                        || label.contains("test")
                        || file_path.contains("test")
                        || file_path.contains("spec"));

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
    callers.sort_by(|a, b| {
        b.edge_weight
            .partial_cmp(&a.edge_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    callees.sort_by(|a, b| {
        b.edge_weight
            .partial_cmp(&a.edge_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    tests.sort_by(|a, b| {
        b.edge_weight
            .partial_cmp(&a.edge_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
fn find_nodes_for_file(graph: &m1nd_core::graph::Graph, file_path: &str) -> Vec<(NodeId, String)> {
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
// Layer C: BFS blast radius (2-hop reachability via CSR edges)
// ---------------------------------------------------------------------------

/// BFS outward from seed nodes through the CSR graph, collecting all reachable
/// nodes within `max_hops`. Returns unique NodeIds reached (excluding seeds).
fn bfs_reachable(
    graph: &m1nd_core::graph::Graph,
    seeds: &[NodeId],
    max_hops: u32,
) -> HashSet<NodeId> {
    let n = graph.num_nodes() as usize;
    let mut visited: HashSet<NodeId> = seeds.iter().copied().collect();
    let mut queue: VecDeque<(NodeId, u32)> = seeds.iter().map(|&nid| (nid, 0)).collect();
    let mut reachable: HashSet<NodeId> = HashSet::new();

    while let Some((node, depth)) = queue.pop_front() {
        if depth >= max_hops {
            continue;
        }
        let idx = node.as_usize();
        if idx >= n || !graph.finalized {
            continue;
        }

        // Forward edges: node -> targets
        let out_range = graph.csr.out_range(node);
        for edge_pos in out_range {
            let target = graph.csr.targets[edge_pos];
            if !visited.contains(&target) {
                visited.insert(target);
                reachable.insert(target);
                queue.push_back((target, depth + 1));
            }
        }

        // Reverse edges: sources -> node (nodes that depend on this one)
        let in_range = graph.csr.in_range(node);
        for rev_pos in in_range {
            let source = graph.csr.rev_sources[rev_pos];
            if !visited.contains(&source) {
                visited.insert(source);
                reachable.insert(source);
                queue.push_back((source, depth + 1));
            }
        }
    }

    reachable
}

/// Compute Layer C blast radius for a single file: BFS 2-hop from file's nodes,
/// then filter to only OTHER file-level nodes. Returns (reachable_count, top_affected_ids).
fn compute_blast_radius(graph: &m1nd_core::graph::Graph, file_path: &str) -> (usize, Vec<String>) {
    let file_nodes = find_nodes_for_file(graph, file_path);
    if file_nodes.is_empty() {
        return (0, Vec::new());
    }

    let seeds: Vec<NodeId> = file_nodes.iter().map(|(nid, _)| *nid).collect();
    let seed_set: HashSet<NodeId> = seeds.iter().copied().collect();
    let reachable = bfs_reachable(graph, &seeds, 2);

    // Filter to file-level nodes that belong to OTHER files
    let n = graph.num_nodes() as usize;
    let mut other_file_nodes: Vec<String> = Vec::new();

    for &nid in &reachable {
        let idx = nid.as_usize();
        if idx >= n {
            continue;
        }
        // Only count File-type nodes (not functions/structs within the same file)
        if graph.nodes.node_type[idx] != NodeType::File {
            continue;
        }
        // Exclude the file's own nodes
        if seed_set.contains(&nid) {
            continue;
        }
        let ext_id = resolve_external_id(graph, nid);
        other_file_nodes.push(ext_id);
    }

    let count = other_file_nodes.len();
    other_file_nodes.truncate(5); // top 5 for reporting
    (count, other_file_nodes)
}

fn blast_radius_risk(count: usize) -> &'static str {
    if count <= 3 {
        "low"
    } else if count <= 10 {
        "medium"
    } else {
        "high"
    }
}

// ---------------------------------------------------------------------------
// Layer D: Affected test execution
// ---------------------------------------------------------------------------

/// Run a command with a timeout. Returns Ok(output) or Err on timeout/spawn failure.
/// Uses spawn + poll loop with try_wait, killing after `timeout_secs`.
fn run_command_with_timeout(
    mut cmd: Command,
    timeout_secs: u64,
) -> Result<std::process::Output, String> {
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn failed: {}", e))?;

    let deadline = Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let poll_interval = std::time::Duration::from_millis(100);

    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Child exited — collect output
                return child
                    .wait_with_output()
                    .map_err(|e| format!("wait_with_output failed: {}", e));
            }
            Ok(None) => {
                // Still running — check timeout
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait(); // reap zombie
                    return Err(format!("command timed out after {}s", timeout_secs));
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => {
                return Err(format!("try_wait failed: {}", e));
            }
        }
    }
}

/// Detect and run tests for modified files. Returns (tests_run, tests_passed, tests_failed, output_on_failure).
fn run_affected_tests(
    modified_paths: &[PathBuf],
) -> (Option<u32>, Option<u32>, Option<u32>, Option<String>) {
    let mut total_run: u32 = 0;
    let mut total_passed: u32 = 0;
    let mut total_failed: u32 = 0;
    let mut failure_output: Option<String> = None;
    let mut any_tests_found = false;

    for path in modified_paths {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext {
            "rs" => {
                if let Some((run, passed, failed, output)) = run_rust_tests(path) {
                    any_tests_found = true;
                    total_run += run;
                    total_passed += passed;
                    total_failed += failed;
                    if failed > 0 && failure_output.is_none() {
                        failure_output = output;
                    }
                }
            }
            "go" => {
                if let Some((run, passed, failed, output)) = run_go_tests(path) {
                    any_tests_found = true;
                    total_run += run;
                    total_passed += passed;
                    total_failed += failed;
                    if failed > 0 && failure_output.is_none() {
                        failure_output = output;
                    }
                }
            }
            "py" => {
                if let Some((run, passed, failed, output)) = run_python_tests(path) {
                    any_tests_found = true;
                    total_run += run;
                    total_passed += passed;
                    total_failed += failed;
                    if failed > 0 && failure_output.is_none() {
                        failure_output = output;
                    }
                }
            }
            _ => {}
        }
    }

    if any_tests_found {
        (
            Some(total_run),
            Some(total_passed),
            Some(total_failed),
            failure_output,
        )
    } else {
        (None, None, None, None)
    }
}

/// Detect Rust tests: check for #[cfg(test)] in the file or companion _test.rs.
/// Runs `cargo test --lib -p <package> -- <filter>` with 30s timeout.
fn run_rust_tests(path: &Path) -> Option<(u32, u32, u32, Option<String>)> {
    let content = std::fs::read_to_string(path).ok()?;
    let has_inline_tests = content.contains("#[cfg(test)]");
    let stem = path.file_stem()?.to_str()?;
    let parent = path.parent()?;
    let test_file = parent.join(format!("{}_test.rs", stem));
    let has_test_file = test_file.exists();

    if !has_inline_tests && !has_test_file {
        return None;
    }

    // Find the Cargo.toml to determine the package name
    let package = find_cargo_package(path)?;

    // Build the test filter from the file stem
    let filter = stem.replace('-', "_");

    let mut cmd = Command::new("cargo");
    cmd.args(["test", "--lib", "-p", &package, "--", &filter])
        .current_dir(find_cargo_workspace(path)?)
        .env("RUST_BACKTRACE", "0");

    match run_command_with_timeout(cmd, 30) {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            let (run, passed, failed) = parse_cargo_test_output(&combined);
            let fail_output = if failed > 0 {
                Some(combined.chars().take(500).collect())
            } else {
                None
            };
            Some((run, passed, failed, fail_output))
        }
        Err(_) => {
            // Timeout or spawn failure — report as 1 failed test
            Some((
                1,
                0,
                1,
                Some("cargo test timed out (30s limit)".to_string()),
            ))
        }
    }
}

/// Parse cargo test output for pass/fail counts.
/// Looks for lines like: "test result: ok. 5 passed; 0 failed; 0 ignored"
fn parse_cargo_test_output(output: &str) -> (u32, u32, u32) {
    for line in output.lines() {
        if line.starts_with("test result:") {
            let mut passed = 0u32;
            let mut failed = 0u32;
            for part in line.split(';') {
                let trimmed = part.trim();
                if let Some(num_str) = trimmed.strip_suffix(" passed") {
                    let num_str = num_str.trim();
                    // "test result: ok. 5 passed" — extract the number
                    if let Some(n) = num_str.split_whitespace().last() {
                        passed = n.parse().unwrap_or(0);
                    }
                } else if let Some(num_str) = trimmed.strip_suffix(" failed") {
                    let num_str = num_str.trim();
                    if let Some(n) = num_str.split_whitespace().last() {
                        failed = n.parse().unwrap_or(0);
                    }
                }
            }
            return (passed + failed, passed, failed);
        }
    }
    (0, 0, 0)
}

/// Find the cargo package name by walking up to find Cargo.toml and parsing [package] name.
fn find_cargo_package(path: &Path) -> Option<String> {
    let mut dir = path.parent()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml).ok()?;
            // Simple parse: find `name = "..."` under [package]
            let mut in_package = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "[package]" {
                    in_package = true;
                    continue;
                }
                if trimmed.starts_with('[') {
                    in_package = false;
                    continue;
                }
                if in_package {
                    if let Some(rest) = trimmed.strip_prefix("name") {
                        let rest = rest.trim_start();
                        if let Some(rest) = rest.strip_prefix('=') {
                            let name = rest.trim().trim_matches('"').trim_matches('\'');
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }
        dir = dir.parent()?;
    }
}

/// Find the workspace root (directory with Cargo.toml containing [workspace]).
fn find_cargo_workspace(path: &Path) -> Option<PathBuf> {
    let mut dir = path.parent()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(dir.to_path_buf());
                }
            }
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => break,
        }
    }
    // Fallback: use the first Cargo.toml parent
    let mut dir = path.parent()?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Some(dir.to_path_buf());
        }
        match dir.parent() {
            Some(p) if p != dir => dir = p,
            _ => return None,
        }
    }
}

/// Detect Go tests: find _test.go files in the same directory.
/// Runs `go test ./package/...` with 30s timeout.
fn run_go_tests(path: &Path) -> Option<(u32, u32, u32, Option<String>)> {
    let parent = path.parent()?;
    let stem = path.file_stem()?.to_str()?;
    let test_file = parent.join(format!("{}_test.go", stem));

    if !test_file.exists() {
        // Also check for any *_test.go in same dir
        let has_any_test = std::fs::read_dir(parent).ok()?.any(|entry| {
            entry
                .ok()
                .and_then(|e| e.file_name().to_str().map(|s| s.ends_with("_test.go")))
                .unwrap_or(false)
        });
        if !has_any_test {
            return None;
        }
    }

    let pkg_path = format!("./{}", parent.file_name()?.to_str()?);
    let mut cmd = Command::new("go");
    cmd.args(["test", &format!("{}/...", pkg_path)])
        .current_dir(parent.parent()?);

    match run_command_with_timeout(cmd, 30) {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            let (run, passed, failed) = parse_go_test_output(&combined);
            let fail_output = if failed > 0 {
                Some(combined.chars().take(500).collect())
            } else {
                None
            };
            Some((run, passed, failed, fail_output))
        }
        Err(_) => Some((1, 0, 1, Some("go test timed out (30s limit)".to_string()))),
    }
}

/// Parse go test output. Looks for "ok" / "FAIL" lines and "--- PASS" / "--- FAIL".
fn parse_go_test_output(output: &str) -> (u32, u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--- PASS:") {
            passed += 1;
        } else if trimmed.starts_with("--- FAIL:") {
            failed += 1;
        }
    }
    (passed + failed, passed, failed)
}

/// Detect Python tests: find test_*.py or *_test.py nearby.
/// Runs `python3 -m pytest <test_file> -x --tb=short` with 30s timeout.
fn run_python_tests(path: &Path) -> Option<(u32, u32, u32, Option<String>)> {
    let parent = path.parent()?;
    let stem = path.file_stem()?.to_str()?;

    // Look for test files: test_{stem}.py or {stem}_test.py
    let test_file_a = parent.join(format!("test_{}.py", stem));
    let test_file_b = parent.join(format!("{}_test.py", stem));
    // Also check tests/ subdirectory
    let test_file_c = parent.join("tests").join(format!("test_{}.py", stem));

    let test_file = if test_file_a.exists() {
        test_file_a
    } else if test_file_b.exists() {
        test_file_b
    } else if test_file_c.exists() {
        test_file_c
    } else {
        return None;
    };

    let test_file_str = test_file.to_string_lossy().to_string();
    let mut cmd = Command::new("python3");
    cmd.args(["-m", "pytest", &test_file_str, "-x", "--tb=short"])
        .current_dir(parent);

    match run_command_with_timeout(cmd, 30) {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            let (run, passed, failed) = parse_pytest_output(&combined);
            let fail_output = if failed > 0 {
                Some(combined.chars().take(500).collect())
            } else {
                None
            };
            Some((run, passed, failed, fail_output))
        }
        Err(_) => Some((1, 0, 1, Some("pytest timed out (30s limit)".to_string()))),
    }
}

/// Parse pytest output. Looks for summary line like "5 passed, 1 failed" or "3 passed".
fn parse_pytest_output(output: &str) -> (u32, u32, u32) {
    // pytest final line format: "= 5 passed, 2 failed in 1.23s ="
    // or "= 5 passed in 1.23s ="
    for line in output.lines().rev() {
        let trimmed = line.trim().trim_matches('=').trim();
        let mut passed = 0u32;
        let mut failed = 0u32;
        for part in trimmed.split(',') {
            let part = part.trim();
            if let Some(n) = part.strip_suffix(" passed") {
                passed = n.trim().parse().unwrap_or(0);
            } else if let Some(n) = part.strip_suffix(" failed") {
                failed = n.trim().parse().unwrap_or(0);
            }
        }
        if passed > 0 || failed > 0 {
            return (passed + failed, passed, failed);
        }
    }
    (0, 0, 0)
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
    let file_contents =
        std::fs::read_to_string(&resolved_path).map_err(|e| M1ndError::InvalidParams {
            tool: "m1nd_surgical_context".into(),
            detail: format!("cannot read file {}: {}", resolved_path.display(), e),
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
        file_type_node.or(file_nodes.first()).cloned()
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
        symbols
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(sym_name) || s.name == *sym_name)
            .cloned()
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
    let temp_path = parent.join(format!(".m1nd_apply_{}.tmp", std::process::id()));

    // Ensure parent directory exists
    if !parent.exists() {
        std::fs::create_dir_all(parent).map_err(|e| M1ndError::InvalidParams {
            tool: "m1nd_apply".into(),
            detail: format!("cannot create directory {}: {}", parent.display(), e),
        })?;
    }

    // Write to temp file
    std::fs::write(&temp_path, &input.new_content).map_err(|e| M1ndError::InvalidParams {
        tool: "m1nd_apply".into(),
        detail: format!("cannot write temp file {}: {}", temp_path.display(), e),
    })?;

    // Rename (atomic on same filesystem)
    std::fs::rename(&temp_path, &validated_path).map_err(|e| {
        // Clean up temp file on rename failure
        let _ = std::fs::remove_file(&temp_path);
        M1ndError::InvalidParams {
            tool: "m1nd_apply".into(),
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
                                updated_node_ids
                                    .push(format!("file::{}", validated_path.to_string_lossy()));
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
                let excerpt_lines = if truncated {
                    max_lines
                } else {
                    file_line_count
                };
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
            verification: None,
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

    // Pre-write snapshot: capture graph nodes BEFORE writing (for verify graph-diff)
    let pre_nodes: std::collections::HashMap<String, HashSet<String>> = if input.verify {
        let graph = state.graph.read();
        resolved_edits
            .iter()
            .map(|(path, _, _)| {
                let path_str = path.to_string_lossy().to_string();
                let nodes: HashSet<String> = find_nodes_for_file(&graph, &path_str)
                    .into_iter()
                    .map(|(_, ext_id)| ext_id)
                    .collect();
                (path_str, nodes)
            })
            .collect()
    } else {
        std::collections::HashMap::new()
    };

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
                        tool: "m1nd_apply_batch".into(),
                        detail: format!("cannot create directory {}: {}", parent.display(), e),
                    });
                }
            }

            // BUG FIX (B2): unique temp file per edit (pid + batch_id + index)
            let tmp_path = parent.join(format!(".m1nd_batch_{}_{}_{}_.tmp", pid, batch_id, i));

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
                        tool: "m1nd_apply_batch".into(),
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
                    tool: "m1nd_apply_batch".into(),
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
                old_content
                    .lines()
                    .take(3)
                    .map(|l| format!("-{}\n", l))
                    .collect::<String>(),
                edit.new_content
                    .lines()
                    .take(3)
                    .map(|l| format!("+{}\n", l))
                    .collect::<String>(),
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
            let tmp_path = parent.join(format!(".m1nd_batch_{}_{}_{}_.tmp", pid, batch_id, i));

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
                        old_content
                            .lines()
                            .take(3)
                            .map(|l| format!("-{}\n", l))
                            .collect::<String>(),
                        edit.new_content
                            .lines()
                            .take(3)
                            .map(|l| format!("+{}\n", l))
                            .collect::<String>(),
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

    // Step 8: Post-write verification via GRAPH DIFF (verify=true)
    // Compares pre-write graph nodes vs post-write to find what ACTUALLY changed structurally.
    // Also detects anti-patterns in the textual diff (todo!() → empty Ok(), etc.)
    let verification = if input.verify && all_succeeded && reingested {
        let verify_start = Instant::now();
        let mut high_impact_files = Vec::new();
        let mut total_affected = 0usize;
        let mut antibodies_triggered = Vec::new();
        let mut layer_violations = Vec::new();

        // Graph-diff: compare pre vs post nodes per file
        {
            let graph = state.graph.read();
            for result in &results {
                if !result.success {
                    continue;
                }

                // Post-write nodes (after re-ingest)
                let post_nodes: HashSet<String> = find_nodes_for_file(&graph, &result.file_path)
                    .into_iter()
                    .map(|(_, ext_id)| ext_id)
                    .collect();

                // Pre-write nodes (captured before writing)
                let empty_set = HashSet::new();
                let pre = pre_nodes.get(&result.file_path).unwrap_or(&empty_set);

                // Structural diff
                let added: Vec<&String> = post_nodes.difference(pre).collect();
                let removed: Vec<&String> = pre.difference(&post_nodes).collect();
                let changed_count = added.len() + removed.len();
                total_affected += changed_count;

                // Risk based on structural change, not file size
                let risk = if !removed.is_empty() || changed_count > 5 {
                    "high" // removing symbols or many changes = potentially breaking
                } else if changed_count > 0 {
                    "medium"
                } else {
                    "low" // content change within existing symbols
                };

                let mut top_affected: Vec<String> = Vec::new();
                for n in added.iter().take(3) {
                    top_affected.push(format!("+{}", n));
                }
                for n in removed.iter().take(3) {
                    top_affected.push(format!("-{}", n));
                }

                let node_id = post_nodes.iter().next().cloned().unwrap_or_default();

                high_impact_files.push(surgical::VerificationImpact {
                    file_path: result.file_path.clone(),
                    node_id,
                    affected_count: changed_count,
                    risk: risk.to_string(),
                    top_affected,
                });
            }
        }

        // Anti-pattern detection: compare old_content vs new_content directly
        // (The diff field can be unreliable for intra-line changes)
        //
        // Layer A: expanded trivial-return and stub detection.
        // Layer B: post-write compilation check (below this block).

        /// Count occurrences of ALL known trivial return patterns in source text.
        fn count_trivial_returns(src: &str) -> usize {
            const TRIVIAL_PATTERNS: &[&str] = &[
                "Vec::new()",
                "HashMap::new()",
                "BTreeMap::new()",
                "HashSet::new()",
                "String::new()",
                "String::from(\"\")",
                "Default::default()",
                "None",
                "Ok(())",
                "false",
                "true",
            ];
            const TRIVIAL_NUMERIC: &[&str] = &[
                "0,", "0;", "0)", "0\n", // plain 0
                "0.0", "0u8", "0i32", "0usize",
            ];
            let mut total = 0usize;
            for pat in TRIVIAL_PATTERNS {
                total += src.matches(pat).count();
            }
            for pat in TRIVIAL_NUMERIC {
                total += src.matches(pat).count();
            }
            // empty string literal: count `""` occurrences that are NOT `String::from("")`
            // (String::from("") already counted above)
            let empty_str_total = src.matches("\"\"").count();
            let string_from_empty = src.matches("String::from(\"\")").count();
            total += empty_str_total.saturating_sub(string_from_empty);
            total
        }

        /// Heuristic: does `new_code` contain real logic (function calls, control flow)?
        fn has_real_logic(new_code: &str) -> bool {
            // Look for function calls (word followed by parens with args)
            let has_fn_calls = new_code.contains("(") && !new_code.trim().is_empty();
            // Control flow keywords
            let has_control = new_code.contains("if ")
                || new_code.contains("match ")
                || new_code.contains("for ")
                || new_code.contains("while ")
                || new_code.contains("loop ")
                || new_code.contains(".map(")
                || new_code.contains(".filter(")
                || new_code.contains(".iter(")
                || new_code.contains(".and_then(")
                || new_code.contains(".unwrap_or(")
                || new_code.contains("await");
            has_fn_calls && has_control
        }

        for (validated_path, edit, old_content) in &resolved_edits {
            let new_content = &edit.new_content;
            let path_str = validated_path.to_string_lossy().to_string();

            // --- todo!() removal analysis (Layer A: improved stub detection) ---
            let old_todo_count = old_content.matches("todo!(").count();
            let new_todo_count = new_content.matches("todo!(").count();
            let todos_removed = old_todo_count.saturating_sub(new_todo_count);
            if todos_removed > 0 {
                // Count trivial returns in old vs new
                let old_trivial = count_trivial_returns(old_content);
                let new_trivial = count_trivial_returns(new_content);

                // Count net new lines (rough proxy for implementation size)
                let old_line_count = old_content.lines().count();
                let new_line_count = new_content.lines().count();
                let net_new_lines = new_line_count as i64 - old_line_count as i64;

                if new_trivial > old_trivial {
                    // More trivial returns appeared → SILENT OK
                    layer_violations.push(format!(
                        "SILENT OK: {} — {} todo!() replaced with trivial return (+{} trivial patterns detected: Vec::new/Default::default/None/0/Ok(())/etc)",
                        path_str, todos_removed, new_trivial - old_trivial
                    ));
                } else if net_new_lines <= 2 {
                    // todo!() removed but barely any new code → BROKEN
                    layer_violations.push(format!(
                        "BROKEN STUB: {} — {} todo!() removed but only {} net new lines (likely trivial replacement, not real implementation)",
                        path_str, todos_removed, net_new_lines
                    ));
                } else if net_new_lines > 5 && has_real_logic(new_content) {
                    // Substantial new code with real logic → SAFE (just note it)
                    layer_violations.push(format!(
                        "STUB FILLED: {} — {} todo!() stubs implemented with {} net new lines of real logic (verify correctness)",
                        path_str, todos_removed, net_new_lines
                    ));
                } else {
                    // Some code added but unclear quality → warn
                    layer_violations.push(format!(
                        "STUB FILLED (REVIEW): {} — {} todo!() removed, {} net new lines added but no clear control flow detected",
                        path_str, todos_removed, net_new_lines
                    ));
                }
            }

            // --- New .unwrap() calls ---
            let old_unwrap_count = old_content.matches(".unwrap()").count();
            let new_unwrap_count = new_content.matches(".unwrap()").count();
            if new_unwrap_count > old_unwrap_count {
                layer_violations.push(format!(
                    "NEW UNWRAP: {} — {} new .unwrap() calls added (potential panic points)",
                    path_str,
                    new_unwrap_count - old_unwrap_count
                ));
            }

            // --- Error handling removed ---
            let old_question_count =
                old_content.matches("?;").count() + old_content.matches("?)").count();
            let new_question_count =
                new_content.matches("?;").count() + new_content.matches("?)").count();
            if old_question_count > new_question_count + 2 {
                layer_violations.push(format!(
                    "ERROR HANDLING REMOVED: {} — {} fewer error propagation points (?; or ?) detected",
                    path_str, old_question_count - new_question_count
                ));
            }
        }

        // Antibody name/description match in diffs
        for ab in &state.antibodies {
            if !ab.enabled {
                continue;
            }
            for result in &results {
                if result.diff.contains(&ab.name) || result.diff.contains(&ab.description) {
                    antibodies_triggered.push(format!(
                        "MATCH {}: {} (in {})",
                        ab.id, ab.description, result.file_path
                    ));
                }
            }
        }

        // -----------------------------------------------------------------
        // Layer C: Real graph BFS impact (2-hop blast radius)
        // -----------------------------------------------------------------
        let mut blast_radius_entries: Vec<surgical::BlastRadiusEntry> = Vec::new();
        {
            let graph = state.graph.read();
            for result in &results {
                if !result.success {
                    continue;
                }
                let (reachable_count, top_affected_ids) =
                    compute_blast_radius(&graph, &result.file_path);
                let risk = blast_radius_risk(reachable_count);

                // Update the high_impact_files entry with real BFS data:
                // replace the top_affected with real reachable file node IDs
                if let Some(impact) = high_impact_files
                    .iter_mut()
                    .find(|f| f.file_path == result.file_path)
                {
                    impact.top_affected = top_affected_ids.clone();
                    impact.affected_count = impact.affected_count.max(reachable_count);
                    // Upgrade risk if BFS says higher
                    if risk == "high" || (risk == "medium" && impact.risk == "low") {
                        impact.risk = risk.to_string();
                    }
                }

                // Also track the real affected node count
                total_affected = total_affected.max(total_affected + reachable_count);

                blast_radius_entries.push(surgical::BlastRadiusEntry {
                    file_path: result.file_path.clone(),
                    reachable_files: reachable_count,
                    risk: risk.to_string(),
                    top_affected: top_affected_ids,
                });
            }
        }

        // -----------------------------------------------------------------
        // Layer D: Affected test execution
        // -----------------------------------------------------------------
        let modified_paths: Vec<PathBuf> = resolved_edits
            .iter()
            .filter(|(_, _, _)| true)
            .map(|(path, _, _)| path.clone())
            .collect();

        let (tests_run, tests_passed, tests_failed, test_output) =
            run_affected_tests(&modified_paths);

        let tests_broken = tests_failed.unwrap_or(0) > 0;

        // -----------------------------------------------------------------
        // Layer B (compile): Post-write compilation check
        // -----------------------------------------------------------------
        let compile_check: Option<String> = {
            let extensions: HashSet<&str> = resolved_edits
                .iter()
                .filter_map(|(path, _, _)| path.extension().and_then(|e| e.to_str()))
                .collect();
            let ws_root = state
                .workspace_root
                .clone()
                .or_else(|| state.ingest_roots.first().cloned());

            if extensions.contains("rs") {
                let cargo_dir = ws_root.clone().or_else(|| {
                    resolved_edits
                        .iter()
                        .filter(|(p, _, _)| p.extension().and_then(|e| e.to_str()) == Some("rs"))
                        .find_map(|(p, _, _)| {
                            let mut dir = p.parent()?;
                            loop {
                                if dir.join("Cargo.toml").exists() {
                                    return Some(dir.to_string_lossy().to_string());
                                }
                                dir = dir.parent()?;
                            }
                        })
                });
                if let Some(dir) = cargo_dir {
                    match std::process::Command::new("cargo")
                        .arg("check")
                        .arg("--message-format=short")
                        .current_dir(&dir)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                    {
                        Ok(child) => match child.wait_with_output() {
                            Ok(out) if out.status.success() => Some("ok".to_string()),
                            Ok(out) => {
                                let stderr = String::from_utf8_lossy(&out.stderr);
                                let t: String = stderr.chars().take(200).collect();
                                layer_violations
                                    .push(format!("COMPILE ERROR (cargo check): {}", t));
                                Some(format!("error: {}", t))
                            }
                            Err(e) => {
                                let m = format!("cargo check process error: {}", e);
                                layer_violations.push(format!("COMPILE ERROR: {}", m));
                                Some(format!("error: {}", m))
                            }
                        },
                        Err(e) => {
                            let m = format!("failed to spawn cargo: {}", e);
                            layer_violations.push(format!("COMPILE ERROR: {}", m));
                            Some(format!("error: {}", m))
                        }
                    }
                } else {
                    None
                }
            } else if extensions.contains("go") {
                if let Some(dir) = ws_root.clone() {
                    match std::process::Command::new("go")
                        .args(["build", "./..."])
                        .current_dir(&dir)
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .output()
                    {
                        Ok(out) if out.status.success() => Some("ok".to_string()),
                        Ok(out) => {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let t: String = stderr.chars().take(200).collect();
                            layer_violations.push(format!("COMPILE ERROR (go build): {}", t));
                            Some(format!("error: {}", t))
                        }
                        Err(e) => {
                            let m = format!("failed to spawn go: {}", e);
                            layer_violations.push(format!("COMPILE ERROR: {}", m));
                            Some(format!("error: {}", m))
                        }
                    }
                } else {
                    None
                }
            } else if extensions.contains("py") {
                let mut py_ok = true;
                let mut py_err = String::new();
                for (path, _, _) in &resolved_edits {
                    if path.extension().and_then(|e| e.to_str()) != Some("py") {
                        continue;
                    }
                    let path_str = path.to_string_lossy();
                    let script = format!("import ast; ast.parse(open('{}').read())", path_str);
                    match std::process::Command::new("python3")
                        .args(["-c", &script])
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .output()
                    {
                        Ok(out) if !out.status.success() => {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let t: String = stderr.chars().take(200).collect();
                            if py_err.is_empty() {
                                py_err = t.to_string();
                            }
                            layer_violations
                                .push(format!("PARSE ERROR (python3 ast): {} — {}", path_str, t));
                            py_ok = false;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            py_err = format!("failed to spawn python3: {}", e);
                            py_ok = false;
                        }
                    }
                }
                if py_ok {
                    Some("ok".to_string())
                } else {
                    Some(format!("error: {}", py_err))
                }
            } else if extensions.contains("ts") || extensions.contains("tsx") {
                if let Some(dir) = ws_root.clone() {
                    let tsconfig = Path::new(&dir).join("tsconfig.json");
                    if tsconfig.exists() {
                        match std::process::Command::new("tsc")
                            .arg("--noEmit")
                            .current_dir(&dir)
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .output()
                        {
                            Ok(out) if out.status.success() => Some("ok".to_string()),
                            Ok(out) => {
                                let combined = format!(
                                    "{}{}",
                                    String::from_utf8_lossy(&out.stdout),
                                    String::from_utf8_lossy(&out.stderr)
                                );
                                let t: String = combined.chars().take(200).collect();
                                layer_violations.push(format!("TYPE ERROR (tsc --noEmit): {}", t));
                                Some(format!("error: {}", t))
                            }
                            Err(e) => {
                                let m = format!("failed to spawn tsc: {}", e);
                                layer_violations.push(format!("TYPE ERROR: {}", m));
                                Some(format!("error: {}", m))
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
        let compile_broken = compile_check
            .as_deref()
            .map(|s| s.starts_with("error"))
            .unwrap_or(false);

        // -----------------------------------------------------------------
        // Verdict (incorporates all layers: A+B+C+D + compile)
        // -----------------------------------------------------------------
        let has_violations = !layer_violations.is_empty();
        let has_antibodies = !antibodies_triggered.is_empty();
        let has_high_risk = high_impact_files.iter().any(|f| f.risk == "high");
        let has_bfs_high = blast_radius_entries.iter().any(|b| b.risk == "high");

        let verdict = if tests_broken || compile_broken || has_violations || has_antibodies {
            "BROKEN".to_string() // compile/test failure or violations = BROKEN
        } else if has_high_risk || has_bfs_high {
            "RISKY".to_string()
        } else {
            "SAFE".to_string()
        };

        Some(surgical::VerificationReport {
            verdict,
            high_impact_files,
            antibodies_triggered,
            layer_violations,
            total_affected_nodes: total_affected,
            blast_radius: blast_radius_entries,
            tests_run,
            tests_passed,
            tests_failed,
            test_output,
            compile_check,
            verify_elapsed_ms: verify_start.elapsed().as_secs_f64() * 1000.0,
        })
    } else {
        None
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
        verification,
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
        let lines = vec![
            "fn foo() {",
            "    if true {",
            "        bar();",
            "    }",
            "}",
        ];
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
