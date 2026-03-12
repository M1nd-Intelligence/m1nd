// === crates/m1nd-ingest/src/resolve.rs ===

use m1nd_core::error::M1ndResult;
use m1nd_core::graph::Graph;
use m1nd_core::types::*;

// ---------------------------------------------------------------------------
// ReferenceResolver — resolve ref:: edges to actual nodes
// FM-ING-008 fix: multi-value index + proximity disambiguation (not dict overwrite).
// Replaces: ingest.py CodebaseIngestor._resolve_references()
// ---------------------------------------------------------------------------

/// Resolves unresolved references (e.g., "ref::Config") to actual graph nodes.
/// FM-ING-008 fix: when multiple nodes match a label, uses proximity
/// disambiguation (same file > same directory > same module) instead of
/// silently shadowing with dict overwrite.
pub struct ReferenceResolver;

/// Resolution outcome for a single reference.
#[derive(Clone, Debug)]
pub struct ResolvedReference {
    pub source: NodeId,
    pub target: NodeId,
    pub relation: InternedStr,
    pub confidence: FiniteF32,
}

/// Summary of resolution results.
#[derive(Clone, Debug)]
pub struct ResolutionStats {
    pub resolved: u64,
    pub unresolved: u64,
    pub ambiguous: u64,
}

impl ReferenceResolver {
    /// Resolve all unresolved references in the graph.
    /// Uses multi-value label index + proximity disambiguation (FM-ING-008).
    /// Task #8: import_hint (4th tuple element) carries the import path for
    /// module-aware disambiguation (e.g., "from foo.bar import Baz" hints "foo/bar").
    /// Replaces: ingest.py CodebaseIngestor._resolve_references()
    pub fn resolve(
        graph: &mut Graph,
        unresolved: &[(String, String, String)], // (source_id, target_label, relation)
    ) -> M1ndResult<ResolutionStats> {
        // Upgrade: also accept optional import hints via resolve_with_hints
        Self::resolve_with_hints(graph, unresolved, &[])
    }

    /// Resolve with optional import-path hints for module-aware disambiguation.
    /// `import_hints` maps (source_id, target_label) -> import_path so that
    /// e.g. "from foo.bar import Baz" prefers the Baz node under foo/bar.
    pub fn resolve_with_hints(
        graph: &mut Graph,
        unresolved: &[(String, String, String)], // (source_id, target_label, relation)
        import_hints: &[(String, String, String)], // (source_id, target_label, import_path)
    ) -> M1ndResult<ResolutionStats> {
        let label_index = Self::build_label_index(graph);
        let mut stats = ResolutionStats {
            resolved: 0,
            unresolved: 0,
            ambiguous: 0,
        };

        // Build a quick lookup for import hints: (source, target_label) -> import_path
        let hint_map: std::collections::HashMap<(&str, &str), &str> = import_hints
            .iter()
            .map(|(s, t, p)| ((s.as_str(), t.as_str()), p.as_str()))
            .collect();

        for (source_id, target_label, relation) in unresolved {
            // Look up source node
            let source = match graph.resolve_id(source_id) {
                Some(id) => id,
                None => {
                    stats.unresolved += 1;
                    continue;
                }
            };

            // Strip "ref::" prefix if present
            let clean_label = target_label
                .strip_prefix("ref::")
                .unwrap_or(target_label);

            // Extract the last segment from import path for matching
            // e.g., "m1nd_core::graph::Graph" -> "Graph"
            let last_segment = clean_label.rsplit("::").next().unwrap_or(clean_label);

            // Check for import path hint (Task #8)
            let import_hint = hint_map.get(&(source_id.as_str(), target_label.as_str())).copied();

            // Look up by label in the graph's string interner
            let label_interned = match graph.strings.lookup(last_segment) {
                Some(id) => id,
                None => {
                    // Try matching by suffix (e.g., "Config" matches "module::Config")
                    let mut found = Vec::new();
                    for i in 0..graph.num_nodes() as usize {
                        let node_label = graph.strings.resolve(graph.nodes.label[i]);
                        if node_label == last_segment
                            || node_label == clean_label
                            || clean_label.ends_with(node_label)
                        {
                            found.push(NodeId::new(i as u32));
                        }
                    }
                    if found.is_empty() {
                        stats.unresolved += 1;
                        continue;
                    }
                    if found.len() > 1 {
                        stats.ambiguous += 1;
                    }
                    // Use first match (or disambiguate if multiple)
                    let target = if found.len() == 1 {
                        found[0]
                    } else if let Some(hint) = import_hint {
                        Self::disambiguate_with_hint(graph, source, &found, hint)
                            .unwrap_or_else(|| Self::disambiguate(graph, source, &found).unwrap_or(found[0]))
                    } else {
                        Self::disambiguate(graph, source, &found).unwrap_or(found[0])
                    };

                    // Add edge
                    let rel = relation.as_str();
                    let _ = graph.add_edge(
                        source,
                        target,
                        rel,
                        FiniteF32::new(0.5),
                        EdgeDirection::Forward,
                        false,
                        FiniteF32::new(0.4),
                    );
                    stats.resolved += 1;
                    continue;
                }
            };

            // Found by exact interned match (using last segment)
            if let Some(candidates) = label_index.get(&label_interned) {
                if candidates.is_empty() {
                    stats.unresolved += 1;
                    continue;
                }
                if candidates.len() > 1 {
                    stats.ambiguous += 1;
                }

                let target = if candidates.len() == 1 {
                    candidates[0]
                } else if let Some(hint) = import_hint {
                    Self::disambiguate_with_hint(graph, source, candidates, hint)
                        .unwrap_or_else(|| Self::disambiguate(graph, source, candidates).unwrap_or(candidates[0]))
                } else {
                    Self::disambiguate(graph, source, candidates).unwrap_or(candidates[0])
                };

                let rel = relation.as_str();
                let _ = graph.add_edge(
                    source,
                    target,
                    rel,
                    FiniteF32::new(0.5),
                    EdgeDirection::Forward,
                    false,
                    FiniteF32::ZERO,
                );
                stats.resolved += 1;
            } else {
                stats.unresolved += 1;
            }
        }

        Ok(stats)
    }

    /// Build label-to-nodes index (multi-value, not single-value).
    /// FM-ING-008 fix: returns Vec of candidates, not single overwrite.
    fn build_label_index(graph: &Graph) -> std::collections::HashMap<InternedStr, Vec<NodeId>> {
        let mut index: std::collections::HashMap<InternedStr, Vec<NodeId>> =
            std::collections::HashMap::new();
        for i in 0..graph.num_nodes() as usize {
            let label = graph.nodes.label[i];
            index.entry(label).or_default().push(NodeId::new(i as u32));
        }
        index
    }

    /// Disambiguate among multiple candidates using proximity.
    /// Priority: same file > same directory > same module > first match.
    fn disambiguate(
        graph: &Graph,
        source: NodeId,
        candidates: &[NodeId],
    ) -> Option<NodeId> {
        if candidates.is_empty() {
            return None;
        }

        // Get source's external ID to compute proximity
        let source_ext_id = Self::find_external_id(graph, source)?;

        // Score each candidate by proximity to source
        let mut best = candidates[0];
        let mut best_score = 0u32;

        for &candidate in candidates {
            if let Some(cand_ext_id) = Self::find_external_id(graph, candidate) {
                let score = Self::proximity_score(&source_ext_id, &cand_ext_id);
                if score > best_score {
                    best_score = score;
                    best = candidate;
                }
            }
        }

        Some(best)
    }

    /// Find external ID string for a node.
    fn find_external_id(graph: &Graph, node: NodeId) -> Option<String> {
        for (interned, &nid) in &graph.id_to_node {
            if nid == node {
                return Some(graph.strings.resolve(*interned).to_string());
            }
        }
        None
    }

    /// Disambiguate among multiple candidates using an import path hint.
    /// If a candidate's external ID contains path segments matching the import hint,
    /// prefer that candidate. E.g., import hint "foo.bar" matches candidate
    /// "file::foo/bar.py::class::Baz".
    fn disambiguate_with_hint(
        graph: &Graph,
        _source: NodeId,
        candidates: &[NodeId],
        import_hint: &str,
    ) -> Option<NodeId> {
        if candidates.is_empty() || import_hint.is_empty() {
            return None;
        }

        // Normalize the import hint: "foo.bar" -> ["foo", "bar"] and also "foo/bar"
        let hint_parts: Vec<&str> = import_hint.split('.').collect();
        let hint_as_path = hint_parts.join("/");
        let hint_as_colons = hint_parts.join("::");

        let mut best: Option<NodeId> = None;
        let mut best_score = 0u32;

        for &candidate in candidates {
            if let Some(cand_ext_id) = Self::find_external_id(graph, candidate) {
                let mut score = 0u32;
                // Check if candidate's ID contains the import path segments
                if cand_ext_id.contains(&hint_as_path) {
                    score += 200;
                }
                if cand_ext_id.contains(&hint_as_colons) {
                    score += 200;
                }
                // Partial match: check individual segments
                for part in &hint_parts {
                    if cand_ext_id.contains(part) {
                        score += 10;
                    }
                }
                if score > best_score {
                    best_score = score;
                    best = Some(candidate);
                }
            }
        }

        // Only return if we actually found a match via the hint
        if best_score > 0 { best } else { None }
    }

    /// Compute proximity score between two external IDs.
    /// Higher = closer. Same file = 100, same dir = 50, same project = 10.
    fn proximity_score(source_id: &str, candidate_id: &str) -> u32 {
        let src_parts: Vec<&str> = source_id.split("::").collect();
        let cand_parts: Vec<&str> = candidate_id.split("::").collect();

        // Count matching prefix segments
        let mut matching = 0;
        for (a, b) in src_parts.iter().zip(cand_parts.iter()) {
            if a == b {
                matching += 1;
            } else {
                break;
            }
        }

        // Score based on matching depth
        match matching {
            0 => 1,        // same project
            1 => 10,       // same top-level
            2 => 50,       // same directory/module
            _ => 100,      // same file or deeper
        }
    }
}
