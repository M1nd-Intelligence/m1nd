// === m1nd-mcp/src/search_handlers.rs ===
//
// v0.4.0: Handlers for m1nd.search and m1nd.help.
// Search: literal/regex/semantic modes with graph context.
// Help: self-documenting tool reference with visual identity.

use m1nd_core::error::{M1ndError, M1ndResult};
use crate::session::SessionState;
use crate::protocol::layers::{
    SearchInput, SearchOutput, SearchResultEntry, SearchMode,
    HelpInput, HelpOutput,
};
use crate::personality;
use std::time::Instant;

// ---------------------------------------------------------------------------
// m1nd.search
// ---------------------------------------------------------------------------

pub fn handle_search(
    state: &mut SessionState,
    input: SearchInput,
) -> M1ndResult<SearchOutput> {
    let start = Instant::now();

    // Validate
    if input.query.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "m1nd.search".into(),
            detail: "query cannot be empty".into(),
        });
    }

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
                    if results.len() < top_k {
                        let (file_path, line_number) = extract_provenance(&graph, ext_id);
                        let (ctx_before, ctx_after) = get_context_lines(&file_path, line_number, context_lines);
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

            // Phase 2: Search file contents on disk (the real grep replacement)
            if results.len() < top_k {
                // Collect unique source files from graph nodes
                let mut seen_files: std::collections::HashSet<String> = std::collections::HashSet::new();
                for (interned, &_nid) in graph.id_to_node.iter() {
                    let ext_id = graph.strings.resolve(*interned);
                    if ext_id.starts_with("file::") {
                        let path = ext_id.strip_prefix("file::").unwrap_or(ext_id);
                        // Only take the file-level nodes (no ::fn:: or ::class::)
                        if !path.contains("::") {
                            seen_files.insert(path.to_string());
                        }
                    }
                }

                for rel_path in &seen_files {
                    if let Some(prefix) = scope {
                        if !rel_path.contains(prefix) {
                            continue;
                        }
                    }
                    if results.len() >= top_k {
                        break;
                    }
                    // Resolve full path via ingest roots OR graph metadata
                    let roots: Vec<&str> = if state.ingest_roots.is_empty() {
                        // Fallback: try common patterns from graph provenance
                        vec![]
                    } else {
                        state.ingest_roots.iter().map(|s| s.as_str()).collect()
                    };

                    let full_path = roots.iter()
                        .map(|root| std::path::Path::new(root).join(rel_path))
                        .find(|p| p.exists())
                        .or_else(|| {
                            // Try the rel_path as absolute
                            let p = std::path::PathBuf::from(rel_path);
                            if p.exists() { Some(p) } else { None }
                        })
                        .unwrap_or_else(|| std::path::PathBuf::from(rel_path));

                    if let Ok(content) = std::fs::read_to_string(&full_path) {
                        for (line_idx, line) in content.lines().enumerate() {
                            let match_line = if input.case_sensitive {
                                line.to_string()
                            } else {
                                line.to_lowercase()
                            };
                            if match_line.contains(&query_pattern) {
                                total_matches += 1;
                                if results.len() < top_k {
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
        }
        SearchMode::Regex => {
            // Regex match using regex crate (ADVERSARY S1: safe linear-time regex only)
            let pattern = if input.case_sensitive {
                input.query.clone()
            } else {
                format!("(?i){}", input.query)
            };

            let re = regex::Regex::new(&pattern).map_err(|e| M1ndError::InvalidParams {
                tool: "m1nd.search".into(),
                detail: format!("invalid regex: {}", e),
            })?;

            for (interned, &_nid) in graph.id_to_node.iter() {
                let ext_id = graph.strings.resolve(*interned);

                // Scope filter
                if let Some(prefix) = scope {
                    if !ext_id.contains(prefix) {
                        continue;
                    }
                }

                if re.is_match(ext_id) {
                    total_matches += 1;
                    if results.len() < top_k {
                        let (file_path, line_number) = extract_provenance(&graph, ext_id);
                        let (ctx_before, ctx_after) = get_context_lines(&file_path, line_number, context_lines);

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
                    let node_id = item.get("node_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let label = item.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    results.push(SearchResultEntry {
                        node_id: node_id.clone(),
                        label: label.clone(),
                        node_type: item.get("node_type").and_then(|v| v.as_str()).unwrap_or("File").to_string(),
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
            });
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    Ok(SearchOutput {
        query: input.query,
        mode: format!("{:?}", input.mode).to_lowercase(),
        results,
        total_matches,
        scope_applied,
        elapsed_ms: elapsed,
    })
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
fn get_context_lines(file_path: &str, line_number: u32, context_lines: u32) -> (Vec<String>, Vec<String>) {
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
    let before: Vec<String> = lines[before_start..line_idx].iter().map(|s| s.to_string()).collect();

    let after_end = (line_idx + 1 + context_lines as usize).min(lines.len());
    let after: Vec<String> = if line_idx + 1 < lines.len() {
        lines[line_idx + 1..after_end].iter().map(|s| s.to_string()).collect()
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
// m1nd.help
// ---------------------------------------------------------------------------

pub fn handle_help(
    _state: &mut SessionState,
    input: HelpInput,
) -> M1ndResult<HelpOutput> {
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
            // Normalize: accept both "activate" and "m1nd.activate"
            let normalized = if name.starts_with("m1nd.") {
                name.to_string()
            } else {
                format!("m1nd.{}", name)
            };

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
                    personality::ANSI_RED, name, personality::ANSI_RESET,
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
