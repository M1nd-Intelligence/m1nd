// === m1nd-mcp/src/search_handlers.rs ===
//
// v0.5.0: Handlers for m1nd.search, m1nd.glob, and m1nd.help.
// Search: literal/regex/semantic modes with graph context.
//   - v0.5.0: regex mode gets Phase 2 disk search (fixes CRITICAL gap)
//   - v0.5.0: multiline, invert, count_only, filename_pattern support
// Glob: graph-aware file pattern matching (replaces find/glob).
// Help: self-documenting tool reference with visual identity.

use crate::personality;
use crate::protocol::layers::{
    GlobFileEntry, GlobInput, GlobOutput, HelpInput, HelpOutput, SearchInput, SearchMode,
    SearchOutput, SearchResultEntry,
};
use crate::session::SessionState;
use m1nd_core::error::{M1ndError, M1ndResult};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Shared matcher trait for Phase 2 file content search (fixes GAP 2)
// ---------------------------------------------------------------------------

/// Abstraction over literal and regex matching for file content search.
/// This enables Phase 2 (disk file search) to work for BOTH literal and regex modes.
trait LineMatcher {
    /// Returns true if the line matches the pattern.
    fn matches(&self, line: &str) -> bool;
}

/// Literal substring matcher (case-insensitive by default).
struct LiteralMatcher {
    pattern: String,
    case_sensitive: bool,
}

impl LineMatcher for LiteralMatcher {
    fn matches(&self, line: &str) -> bool {
        if self.case_sensitive {
            line.contains(&self.pattern)
        } else {
            line.to_lowercase().contains(&self.pattern)
        }
    }
}

/// Regex line-by-line matcher.
struct RegexMatcher {
    re: regex::Regex,
}

impl LineMatcher for RegexMatcher {
    fn matches(&self, line: &str) -> bool {
        self.re.is_match(line)
    }
}

// ---------------------------------------------------------------------------
// m1nd.search
// ---------------------------------------------------------------------------

pub fn handle_search(state: &mut SessionState, input: SearchInput) -> M1ndResult<SearchOutput> {
    let start = Instant::now();

    // Validate
    if input.query.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "m1nd_search".into(),
            detail: "query cannot be empty".into(),
        });
    }

    // Validate filename_pattern if provided
    let filename_glob = if let Some(ref pat) = input.filename_pattern {
        Some(
            glob::Pattern::new(pat).map_err(|e| M1ndError::InvalidParams {
                tool: "m1nd_search".into(),
                detail: format!("invalid filename pattern '{}': {}", pat, e),
            })?,
        )
    } else {
        None
    };

    // Clamp parameters (ADVERSARY S2: hard cap at 500)
    let top_k = (input.top_k as usize).clamp(1, 500);
    let context_lines = input.context_lines.clamp(0, 10);

    let graph = state.graph.read();
    let scope = input.scope.as_deref();
    let scope_applied = scope.is_some();

    let mut results: Vec<SearchResultEntry> = Vec::new();
    let mut total_matches: usize = 0;

    match input.mode {
        SearchMode::Literal => {
            // Phase 1: Match node labels in graph
            let query_pattern = if input.case_sensitive {
                input.query.clone()
            } else {
                input.query.to_lowercase()
            };

            if !input.invert {
                // Normal (non-inverted) Phase 1: node label matching
                for (interned, &_nid) in graph.id_to_node.iter() {
                    let ext_id = graph.strings.resolve(*interned);

                    if let Some(prefix) = scope {
                        if !ext_id.contains(prefix) {
                            continue;
                        }
                    }

                    let match_target = if input.case_sensitive {
                        ext_id.to_string()
                    } else {
                        ext_id.to_lowercase()
                    };

                    if match_target.contains(&query_pattern) {
                        total_matches += 1;
                        if !input.count_only && results.len() < top_k {
                            let (file_path, line_number) = extract_provenance(&graph, ext_id);
                            let (ctx_before, ctx_after) =
                                get_context_lines(&file_path, line_number, context_lines);
                            results.push(SearchResultEntry {
                                node_id: ext_id.to_string(),
                                label: ext_id.to_string(),
                                node_type: guess_node_type(ext_id),
                                file_path,
                                line_number,
                                matched_line: ext_id.to_string(),
                                context_before: ctx_before,
                                context_after: ctx_after,
                                graph_linked: true,
                            });
                        }
                    }
                }
            }

            // Phase 2: Search file contents on disk (the real grep replacement)
            let matcher = LiteralMatcher {
                pattern: query_pattern,
                case_sensitive: input.case_sensitive,
            };
            search_file_contents(
                state,
                &graph,
                scope,
                &matcher,
                input.invert,
                input.count_only,
                top_k,
                context_lines,
                filename_glob.as_ref(),
                &mut results,
                &mut total_matches,
            );
        }
        SearchMode::Regex => {
            // Build regex (ADVERSARY S1: safe linear-time regex only)
            let pattern = if input.case_sensitive {
                input.query.clone()
            } else {
                format!("(?i){}", input.query)
            };

            // v0.5.0: multiline support via RegexBuilder
            let re = if input.multiline {
                regex::RegexBuilder::new(&pattern)
                    .dot_matches_new_line(true)
                    .multi_line(true)
                    .build()
            } else {
                regex::Regex::new(&pattern)
            }
            .map_err(|e| M1ndError::InvalidParams {
                tool: "m1nd_search".into(),
                detail: format!("invalid regex: {}", e),
            })?;

            // Phase 1: Match node labels in graph (non-inverted only)
            if !input.invert {
                for (interned, &_nid) in graph.id_to_node.iter() {
                    let ext_id = graph.strings.resolve(*interned);

                    if let Some(prefix) = scope {
                        if !ext_id.contains(prefix) {
                            continue;
                        }
                    }

                    if re.is_match(ext_id) {
                        total_matches += 1;
                        if !input.count_only && results.len() < top_k {
                            let (file_path, line_number) = extract_provenance(&graph, ext_id);
                            let (ctx_before, ctx_after) =
                                get_context_lines(&file_path, line_number, context_lines);

                            results.push(SearchResultEntry {
                                node_id: ext_id.to_string(),
                                label: ext_id.to_string(),
                                node_type: guess_node_type(ext_id),
                                file_path,
                                line_number,
                                matched_line: ext_id.to_string(),
                                context_before: ctx_before,
                                context_after: ctx_after,
                                graph_linked: true,
                            });
                        }
                    }
                }
            }

            // v0.5.0 FIX (CRITICAL GAP 2): Phase 2 for regex mode
            // Multiline regex searches whole file content; line-by-line regex uses RegexMatcher
            if input.multiline {
                // Multiline: read entire file as one string, find all matches
                search_file_contents_multiline(
                    state,
                    &graph,
                    scope,
                    &re,
                    input.invert,
                    input.count_only,
                    top_k,
                    context_lines,
                    filename_glob.as_ref(),
                    &mut results,
                    &mut total_matches,
                );
            } else {
                // Line-by-line regex (same as literal but with regex matcher)
                let matcher = RegexMatcher { re };
                search_file_contents(
                    state,
                    &graph,
                    scope,
                    &matcher,
                    input.invert,
                    input.count_only,
                    top_k,
                    context_lines,
                    filename_glob.as_ref(),
                    &mut results,
                    &mut total_matches,
                );
            }
        }
        SearchMode::Semantic => {
            // Delegate to existing seek logic via orchestrator
            drop(graph); // Release read lock before calling orchestrator
            let seek_input = crate::protocol::layers::SeekInput {
                agent_id: input.agent_id.clone(),
                query: input.query.clone(),
                top_k,
                scope: input.scope.clone(),
                node_types: vec![],
                min_score: 0.0,
                graph_rerank: true,
            };
            let seek_result = crate::layer_handlers::handle_seek(state, seek_input)?;

            // Convert seek results to search format
            let seek_json = serde_json::to_value(&seek_result).map_err(M1ndError::Serde)?;
            if let Some(items) = seek_json.get("results").and_then(|v| v.as_array()) {
                total_matches = items.len();
                for item in items.iter().take(top_k) {
                    let node_id = item
                        .get("node_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let label = item
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    results.push(SearchResultEntry {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: item
                            .get("node_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("File")
                            .to_string(),
                        file_path: node_id.clone(),
                        line_number: 1,
                        matched_line: label,
                        context_before: vec![],
                        context_after: vec![],
                        graph_linked: true,
                    });
                }
            }

            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            return Ok(SearchOutput {
                query: input.query,
                mode: "semantic".into(),
                results,
                total_matches,
                scope_applied,
                elapsed_ms: elapsed,
                auto_ingested: false,
                match_count: None,
                auto_ingested_paths: vec![],
            });
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    // v0.5.0: count_only — clear results, set match_count
    let match_count = if input.count_only {
        Some(total_matches)
    } else {
        None
    };
    let final_results = if input.count_only { vec![] } else { results };

    Ok(SearchOutput {
        query: input.query,
        mode: format!("{:?}", input.mode).to_lowercase(),
        results: final_results,
        total_matches,
        scope_applied,
        elapsed_ms: elapsed,
        auto_ingested: false,
        match_count,
        auto_ingested_paths: vec![],
    })
}

// ---------------------------------------------------------------------------
// Phase 2: Shared file content search (fixes GAP 2 — works for literal+regex)
// ---------------------------------------------------------------------------

/// Collect unique file:: nodes from the graph, resolve to disk paths,
/// and search their contents line-by-line using the provided matcher.
/// Supports invert, count_only, filename_pattern filtering.
#[allow(clippy::too_many_arguments)]
fn search_file_contents(
    state: &SessionState,
    graph: &m1nd_core::graph::Graph,
    scope: Option<&str>,
    matcher: &dyn LineMatcher,
    invert: bool,
    count_only: bool,
    top_k: usize,
    context_lines: u32,
    filename_glob: Option<&glob::Pattern>,
    results: &mut Vec<SearchResultEntry>,
    total_matches: &mut usize,
) {
    // Collect unique source files from graph nodes
    let seen_files = collect_graph_files(graph, scope, filename_glob);

    for rel_path in &seen_files {
        if !count_only && results.len() >= top_k {
            break;
        }

        let full_path = resolve_full_path(state, rel_path);

        if let Ok(content) = std::fs::read_to_string(&full_path) {
            for (line_idx, line) in content.lines().enumerate() {
                let is_match = matcher.matches(line);
                let include = if invert { !is_match } else { is_match };

                if include {
                    *total_matches += 1;
                    if !count_only && results.len() < top_k {
                        let ln = (line_idx + 1) as u32;
                        let fp = full_path.to_string_lossy().to_string();
                        let (ctx_before, ctx_after) = get_context_lines(&fp, ln, context_lines);
                        results.push(SearchResultEntry {
                            node_id: format!("file::{}", rel_path),
                            label: rel_path.clone(),
                            node_type: "FileContent".into(),
                            file_path: fp,
                            line_number: ln,
                            matched_line: line.to_string(),
                            context_before: ctx_before,
                            context_after: ctx_after,
                            graph_linked: true,
                        });
                    }
                }
            }
        }
    }
}

/// Multiline regex search: reads entire file content as one string,
/// finds all regex matches that may span multiple lines.
#[allow(clippy::too_many_arguments)]
fn search_file_contents_multiline(
    state: &SessionState,
    graph: &m1nd_core::graph::Graph,
    scope: Option<&str>,
    re: &regex::Regex,
    invert: bool,
    count_only: bool,
    top_k: usize,
    context_lines: u32,
    filename_glob: Option<&glob::Pattern>,
    results: &mut Vec<SearchResultEntry>,
    total_matches: &mut usize,
) {
    let seen_files = collect_graph_files(graph, scope, filename_glob);

    for rel_path in &seen_files {
        if !count_only && results.len() >= top_k {
            break;
        }

        let full_path = resolve_full_path(state, rel_path);

        if let Ok(content) = std::fs::read_to_string(&full_path) {
            if invert {
                // Invert multiline: count lines NOT in any match span
                let match_ranges: Vec<(usize, usize)> = re
                    .find_iter(&content)
                    .map(|m| (m.start(), m.end()))
                    .collect();
                for (line_idx, line) in content.lines().enumerate() {
                    let line_start = content
                        .lines()
                        .take(line_idx)
                        .map(|l| l.len() + 1) // +1 for newline
                        .sum::<usize>();
                    let line_end = line_start + line.len();
                    let in_match = match_ranges
                        .iter()
                        .any(|&(ms, me)| line_start < me && line_end > ms);
                    if !in_match {
                        *total_matches += 1;
                        if !count_only && results.len() < top_k {
                            let ln = (line_idx + 1) as u32;
                            let fp = full_path.to_string_lossy().to_string();
                            let (ctx_before, ctx_after) = get_context_lines(&fp, ln, context_lines);
                            results.push(SearchResultEntry {
                                node_id: format!("file::{}", rel_path),
                                label: rel_path.clone(),
                                node_type: "FileContent".into(),
                                file_path: fp,
                                line_number: ln,
                                matched_line: line.to_string(),
                                context_before: ctx_before,
                                context_after: ctx_after,
                                graph_linked: true,
                            });
                        }
                    }
                }
            } else {
                // Normal multiline: find all matches, report each
                for mat in re.find_iter(&content) {
                    *total_matches += 1;
                    if !count_only && results.len() < top_k {
                        // Calculate start line number
                        let start_byte = mat.start();
                        let line_number =
                            content[..start_byte].chars().filter(|&c| c == '\n').count() as u32 + 1;
                        let matched_text = mat.as_str().to_string();
                        // Truncate very long multiline matches to 500 chars
                        let display_text = if matched_text.len() > 500 {
                            format!("{}...[truncated]", &matched_text[..500])
                        } else {
                            matched_text
                        };
                        let fp = full_path.to_string_lossy().to_string();
                        let (ctx_before, ctx_after) =
                            get_context_lines(&fp, line_number, context_lines);
                        results.push(SearchResultEntry {
                            node_id: format!("file::{}", rel_path),
                            label: rel_path.clone(),
                            node_type: "FileContent".into(),
                            file_path: fp,
                            line_number,
                            matched_line: display_text,
                            context_before: ctx_before,
                            context_after: ctx_after,
                            graph_linked: true,
                        });
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers for file collection and path resolution
// ---------------------------------------------------------------------------

/// Collect unique file-level nodes from the graph, filtered by scope and filename pattern.
fn collect_graph_files(
    graph: &m1nd_core::graph::Graph,
    scope: Option<&str>,
    filename_glob: Option<&glob::Pattern>,
) -> Vec<String> {
    let mut seen_files: Vec<String> = Vec::new();
    let mut seen_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (interned, &_nid) in graph.id_to_node.iter() {
        let ext_id = graph.strings.resolve(*interned);
        if ext_id.starts_with("file::") {
            let path = ext_id.strip_prefix("file::").unwrap_or(ext_id);
            // Only take file-level nodes (no ::fn:: or ::class:: sub-nodes)
            if !path.contains("::") && seen_set.insert(path.to_string()) {
                // Apply scope filter
                if let Some(prefix) = scope {
                    if !path.contains(prefix) {
                        continue;
                    }
                }
                // Apply filename_pattern filter
                if let Some(glob_pat) = filename_glob {
                    let filename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(path);
                    if !glob_pat.matches(filename) {
                        continue;
                    }
                }
                seen_files.push(path.to_string());
            }
        }
    }

    seen_files
}

/// Resolve a relative graph path to a full filesystem path using ingest roots.
fn resolve_full_path(state: &SessionState, rel_path: &str) -> std::path::PathBuf {
    let roots: Vec<&str> = if state.ingest_roots.is_empty() {
        vec![]
    } else {
        state.ingest_roots.iter().map(|s| s.as_str()).collect()
    };

    roots
        .iter()
        .map(|root| std::path::Path::new(root).join(rel_path))
        .find(|p| p.exists())
        .or_else(|| {
            let p = std::path::PathBuf::from(rel_path);
            if p.exists() {
                Some(p)
            } else {
                None
            }
        })
        .unwrap_or_else(|| std::path::PathBuf::from(rel_path))
}

/// Extract file path and line number from a node's external_id / provenance.
fn extract_provenance(graph: &m1nd_core::graph::Graph, ext_id: &str) -> (String, u32) {
    // External IDs are typically like "file::path/to/file.py" or "func::path::name"
    let default_path = if ext_id.starts_with("file::") {
        ext_id.strip_prefix("file::").unwrap_or(ext_id).to_string()
    } else if let Some(pos) = ext_id.find("::") {
        ext_id[pos + 2..].to_string()
    } else {
        ext_id.to_string()
    };

    // Try to get provenance from graph
    if let Some(interned) = graph.strings.lookup(ext_id) {
        if let Some(&nid) = graph.id_to_node.get(&interned) {
            let resolved = graph.resolve_node_provenance(nid);
            let path = resolved.source_path.unwrap_or(default_path.clone());
            let line = resolved.line_start.unwrap_or(1);
            if line > 0 {
                return (path, line);
            }
        }
    }

    (default_path, 1)
}

/// Get context lines around a match from the filesystem.
fn get_context_lines(
    file_path: &str,
    line_number: u32,
    context_lines: u32,
) -> (Vec<String>, Vec<String>) {
    if context_lines == 0 || line_number == 0 {
        return (vec![], vec![]);
    }

    // Try to read the file
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return (vec![], vec![]),
    };

    let lines: Vec<&str> = content.lines().collect();
    let line_idx = (line_number as usize).saturating_sub(1);

    let before_start = line_idx.saturating_sub(context_lines as usize);
    let before: Vec<String> = lines[before_start..line_idx]
        .iter()
        .map(|s| s.to_string())
        .collect();

    let after_end = (line_idx + 1 + context_lines as usize).min(lines.len());
    let after: Vec<String> = if line_idx + 1 < lines.len() {
        lines[line_idx + 1..after_end]
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    };

    (before, after)
}

/// Guess node type from external_id prefix.
fn guess_node_type(ext_id: &str) -> String {
    if ext_id.starts_with("file::") {
        "File".into()
    } else if ext_id.starts_with("func::") || ext_id.starts_with("function::") {
        "Function".into()
    } else if ext_id.starts_with("class::") {
        "Class".into()
    } else if ext_id.starts_with("module::") {
        "Module".into()
    } else {
        "File".into()
    }
}

// ---------------------------------------------------------------------------
// m1nd.glob — Graph-Aware File Glob
// ---------------------------------------------------------------------------

pub fn handle_glob(state: &mut SessionState, input: GlobInput) -> M1ndResult<GlobOutput> {
    let start = Instant::now();

    if input.pattern.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "m1nd_glob".into(),
            detail: "pattern cannot be empty".into(),
        });
    }

    let glob_pattern =
        glob::Pattern::new(&input.pattern).map_err(|e| M1ndError::InvalidParams {
            tool: "m1nd_glob".into(),
            detail: format!("invalid glob pattern '{}': {}", input.pattern, e),
        })?;

    let top_k = (input.top_k as usize).clamp(1, 10_000);
    let scope = input.scope.as_deref();
    let scope_applied = scope.is_some();

    let graph = state.graph.read();

    let mut files: Vec<GlobFileEntry> = Vec::new();
    let mut total_matches: usize = 0;

    // Iterate all file:: nodes in the graph
    for (interned, &nid) in graph.id_to_node.iter() {
        let ext_id = graph.strings.resolve(*interned);
        if !ext_id.starts_with("file::") {
            continue;
        }
        let path = ext_id.strip_prefix("file::").unwrap_or(ext_id);
        // Only file-level nodes (no ::fn:: sub-nodes)
        if path.contains("::") {
            continue;
        }

        // Scope filter
        if let Some(prefix) = scope {
            if !path.starts_with(prefix) && !path.contains(prefix) {
                continue;
            }
        }

        // Glob match against relative path
        if !glob_pattern.matches(path) {
            continue;
        }

        total_matches += 1;

        if files.len() < top_k {
            // Extract metadata from graph
            let extension = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();

            // Check if node has outgoing edges (safe: only if CSR is finalized)
            let has_connections = if !graph.csr.offsets.is_empty() {
                let range = graph.csr.out_range(nid);
                !range.is_empty()
            } else {
                false
            };

            // Try to get line count from provenance metadata
            let line_count = {
                let prov = graph.resolve_node_provenance(nid);
                prov.line_end.unwrap_or(0)
            };

            files.push(GlobFileEntry {
                node_id: ext_id.to_string(),
                file_path: path.to_string(),
                extension,
                line_count,
                has_connections,
            });
        }
    }

    // Sort based on requested order
    match input.sort {
        crate::protocol::layers::GlobSort::Path => {
            files.sort_by(|a, b| a.file_path.cmp(&b.file_path));
        }
        crate::protocol::layers::GlobSort::Activation => {
            // Sort by connection count descending as a proxy for activation
            files.sort_by(|a, b| b.has_connections.cmp(&a.has_connections));
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    Ok(GlobOutput {
        pattern: input.pattern,
        files,
        total_matches,
        scope_applied,
        elapsed_ms: elapsed,
    })
}

// ---------------------------------------------------------------------------
// m1nd.help
// ---------------------------------------------------------------------------

pub fn handle_help(_state: &mut SessionState, input: HelpInput) -> M1ndResult<HelpOutput> {
    let tool_name = input.tool_name.as_deref();

    match tool_name {
        None => {
            // Full index
            let formatted = personality::format_help_index();
            Ok(HelpOutput {
                formatted,
                tool: None,
                found: true,
                suggestions: vec![],
            })
        }
        Some("about") => {
            let formatted = personality::format_about();
            Ok(HelpOutput {
                formatted,
                tool: Some("about".into()),
                found: true,
                suggestions: vec![],
            })
        }
        Some(name) => {
            // Normalize: accept both "activate" and "m1nd_activate",
            // and also underscore aliases like "antibody_scan" -> "m1nd_antibody_scan"
            let with_prefix = if name.starts_with("m1nd_") {
                name.to_string()
            } else {
                format!("m1nd_{}", name)
            };
            let normalized = with_prefix.replace('_', ".");

            let docs = personality::tool_docs();
            if let Some(doc) = docs.iter().find(|d| d.name == normalized) {
                let formatted = personality::format_tool_help(doc);
                Ok(HelpOutput {
                    formatted,
                    tool: Some(normalized),
                    found: true,
                    suggestions: vec![],
                })
            } else {
                // Unknown tool -- find similar (ADVERSARY H2)
                let suggestions = personality::find_similar_tools(name);
                let formatted = format!(
                    "{}tool '{}' not found.{}\n{}did you mean: {}?{}\n",
                    personality::ANSI_RED,
                    name,
                    personality::ANSI_RESET,
                    personality::ANSI_DIM,
                    suggestions.join(", "),
                    personality::ANSI_RESET,
                );
                Ok(HelpOutput {
                    formatted,
                    tool: Some(name.to_string()),
                    found: false,
                    suggestions,
                })
            }
        }
    }
}
