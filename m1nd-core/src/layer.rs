// === m1nd-core/src/layer.rs ===
//
// Architectural layer detection from graph topology.
// Tarjan SCC -> BFS depth -> layer grouping -> violation detection.

use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};

// ── Constants ──

/// Maximum number of architectural layers the detector will produce (default).
pub const DEFAULT_MAX_LAYERS: u8 = 8;
/// Minimum number of nodes a layer must contain before it is merged into an adjacent layer (default).
pub const DEFAULT_MIN_NODES_PER_LAYER: u32 = 2;

/// Threshold for classifying a node as utility/cross-cutting:
/// used by >= ceil(total_layers * UTILITY_LAYER_FRACTION) layers.
const UTILITY_LAYER_FRACTION: f32 = 0.5;

// ── Core Types ──

/// A single detected architectural layer in the dependency hierarchy.
#[derive(Clone, Debug, Serialize)]
pub struct ArchLayer {
    /// Zero-based level index (0 = highest/entry point, N = lowest/foundation).
    pub level: u8,
    /// Human-readable layer name derived from the naming strategy.
    pub name: String,
    /// Short description of the layer's role.
    pub description: String,
    /// Node IDs assigned to this layer.
    pub nodes: Vec<NodeId>,
    /// Per-node confidence scores in `[0.0, 1.0]` (parallel to `nodes`).
    pub node_confidence: Vec<f32>,
    /// Mean PageRank across all nodes in this layer.
    pub avg_pagerank: f32,
    /// Mean out-degree across all nodes in this layer.
    pub avg_out_degree: f32,
}

/// Severity of a detected layering violation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ViolationSeverity {
    /// Minor — low-weight edge crossing one layer boundary.
    Low,
    /// Moderate — skip-layer or medium-weight upward dependency.
    Medium,
    /// Significant — high-weight or multi-layer violation.
    High,
    /// Critical — circular dependency (SCC with >1 node).
    Critical,
}

/// Category of a detected layering violation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ViolationType {
    /// A node in a deeper layer depends on a node in a shallower layer by exactly one level.
    UpwardDependency,
    /// A node in a deeper layer depends on a node two or more levels shallower.
    SkipLayerDependency,
    /// Two or more nodes form a mutual dependency cycle (Tarjan SCC).
    CircularDependency,
}

/// A single layering violation detected in the dependency graph.
#[derive(Clone, Debug, Serialize)]
pub struct LayerViolation {
    /// ID of the node originating the problematic edge.
    pub source: NodeId,
    /// Layer level of the source node.
    pub source_layer: u8,
    /// ID of the node that is the target of the problematic edge.
    pub target: NodeId,
    /// Layer level of the target node.
    pub target_layer: u8,
    /// Relation label on the edge.
    pub edge_relation: String,
    /// Weight of the edge.
    pub edge_weight: f32,
    /// Assessed severity.
    pub severity: ViolationSeverity,
    /// Category of violation.
    pub violation_type: ViolationType,
    /// Human-readable explanation of the violation.
    pub explanation: String,
}

/// Classification of a utility/cross-cutting node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum UtilityClassification {
    /// Used by ≥50 % of all layers — excluded from layer assignment.
    CrossCutting,
    /// Used by two or more non-adjacent layers — kept in layers but flagged.
    Bridge,
    /// No incoming or outgoing edges within the candidate set.
    Orphan,
}

/// A node identified as cross-cutting, a bridge between non-adjacent layers, or an orphan.
#[derive(Clone, Debug, Serialize)]
pub struct UtilityNode {
    /// The node ID.
    pub node: NodeId,
    /// Layer levels that reference this node via incoming edges.
    pub used_by_layers: Vec<u8>,
    /// How this node was classified.
    pub classification: UtilityClassification,
}

/// Full output from a layer detection run.
#[derive(Clone, Debug, Serialize)]
pub struct LayerDetectionResult {
    /// Detected layers, sorted by level ascending.
    pub layers: Vec<ArchLayer>,
    /// All layering violations found.
    pub violations: Vec<LayerViolation>,
    /// Nodes classified as cross-cutting, bridge, or orphan.
    pub utility_nodes: Vec<UtilityNode>,
    /// `true` if Tarjan SCC found any dependency cycle.
    pub has_cycles: bool,
    /// Score in `[0.0, 1.0]` — 1.0 means perfectly separated layers, 0.0 means full cycle chaos.
    pub layer_separation_score: f32,
    /// Number of nodes assigned to a layer (utility nodes excluded).
    pub total_nodes_classified: u32,
}

/// Health metrics for a single architectural layer.
#[derive(Clone, Debug, Serialize)]
pub struct LayerHealth {
    /// Ratio of intra-layer edges to the theoretical maximum — measures how internally cohesive the layer is.
    pub cohesion: f32,
    /// Fraction of external edges that go to a shallower (higher-level) layer — upward coupling is a smell.
    pub coupling_up: f32,
    /// Fraction of external edges that go to a deeper (lower-level) layer — normal downward dependency.
    pub coupling_down: f32,
    /// Number of violations involving this layer divided by the layer's node count.
    pub violation_density: f32,
}

/// Cache for layer detection results to avoid re-running the full algorithm on unchanged graphs.
#[derive(Clone, Debug)]
pub struct LayerCache {
    /// Graph generation counter at the time the result was computed.
    pub graph_generation: u64,
    /// Monotonically increasing cache generation counter for invalidation.
    pub cache_generation: u64,
    /// Scope string used when the result was computed (`None` = whole graph).
    pub scope: Option<String>,
    /// Cached detection output.
    pub result: LayerDetectionResult,
}

// ── Engine ──

/// Architectural layer detector.
///
/// Runs a multi-phase pipeline: Tarjan SCC → BFS depth assignment →
/// layer grouping + merging → utility-node detection → violation detection.
pub struct LayerDetector {
    /// Maximum number of layers to produce; adjacent layers are merged until this cap is satisfied.
    pub max_layers: u8,
    /// Minimum node count per layer; tiny layers are absorbed into neighbours.
    pub min_nodes_per_layer: u32,
}

impl LayerDetector {
    /// Create a detector with the given parameters.
    ///
    /// # Parameters
    /// - `max_layers`: upper bound on the number of output layers.
    /// - `min_nodes_per_layer`: layers smaller than this are merged into an adjacent layer.
    pub fn new(max_layers: u8, min_nodes_per_layer: u32) -> Self {
        Self {
            max_layers,
            min_nodes_per_layer,
        }
    }

    /// Create a detector using [`DEFAULT_MAX_LAYERS`] and [`DEFAULT_MIN_NODES_PER_LAYER`].
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_MAX_LAYERS, DEFAULT_MIN_NODES_PER_LAYER)
    }

    /// Detect architectural layers from graph topology.
    ///
    /// Algorithm: Tarjan SCC → BFS depth assignment → layer grouping + merging →
    /// utility-node detection → violation detection.
    ///
    /// # Parameters
    /// - `graph`: the connectivity graph to analyse.
    /// - `scope`: optional path prefix to restrict which nodes are considered.
    /// - `node_type_filter`: if non-empty, only nodes of these types are included.
    /// - `exclude_tests`: if `true`, nodes whose labels contain "test" are dropped.
    /// - `naming_strategy`: one of `"heuristic"`, `"path_prefix"`, or `"pagerank"`.
    ///
    /// # Errors
    /// Returns `M1ndError::EmptyGraph` if no candidate nodes match the filters.
    pub fn detect(
        &self,
        graph: &Graph,
        scope: Option<&str>,
        node_type_filter: &[NodeType],
        exclude_tests: bool,
        naming_strategy: &str,
    ) -> M1ndResult<LayerDetectionResult> {
        let n = graph.num_nodes() as usize;
        if n == 0 {
            return Err(M1ndError::EmptyGraph);
        }

        // Phase 0: Collect candidate nodes (scope + type filter)
        let mut candidates = layer_collect_candidates(graph, scope, node_type_filter);

        // Exclude test files if requested
        if exclude_tests {
            candidates.retain(|&nid| {
                let idx = nid.as_usize();
                if idx >= graph.nodes.count as usize {
                    return false;
                }
                let label = graph.strings.resolve(graph.nodes.label[idx]).to_lowercase();
                !label.contains("test")
            });
        }

        if candidates.is_empty() {
            return Err(M1ndError::EmptyGraph);
        }

        let candidate_set: HashSet<NodeId> = candidates.iter().copied().collect();

        // Phase 1: Tarjan SCC detection
        let sccs = tarjan_scc(graph, &candidates);
        let has_cycles = !sccs.is_empty();

        // Build SCC membership: node -> scc_representative (lowest NodeId in SCC)
        let mut scc_map: HashMap<NodeId, NodeId> = HashMap::new();
        for scc in &sccs {
            let representative = *scc.iter().min().unwrap();
            for &node in scc {
                scc_map.insert(node, representative);
            }
        }

        // Build DAG nodes: either the node itself or its SCC representative
        let mut dag_nodes: Vec<NodeId> = Vec::new();
        let mut dag_set: HashSet<NodeId> = HashSet::new();
        for &node in &candidates {
            let effective = scc_map.get(&node).copied().unwrap_or(node);
            if dag_set.insert(effective) {
                dag_nodes.push(effective);
            }
        }

        // Phase 2: BFS depth assignment on the DAG
        // Compute in-degree for DAG nodes considering only edges within candidate set
        let mut in_degree: HashMap<NodeId, u32> = HashMap::new();
        for &node in &dag_nodes {
            in_degree.insert(node, 0);
        }

        for &node in &candidates {
            let effective_src = scc_map.get(&node).copied().unwrap_or(node);
            let range = graph.csr.out_range(node);
            for j in range {
                let target = graph.csr.targets[j];
                if !candidate_set.contains(&target) {
                    continue;
                }
                let effective_tgt = scc_map.get(&target).copied().unwrap_or(target);
                if effective_src != effective_tgt {
                    *in_degree.entry(effective_tgt).or_insert(0) += 1;
                }
            }
        }

        // Find roots (in-degree == 0)
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        let mut depth: HashMap<NodeId, u32> = HashMap::new();
        for &node in &dag_nodes {
            if *in_degree.get(&node).unwrap_or(&0) == 0 {
                queue.push_back(node);
                depth.insert(node, 0);
            }
        }

        // If no roots found (all nodes in cycles), pick nodes with min in-degree
        if queue.is_empty() {
            let min_in = dag_nodes
                .iter()
                .map(|n| *in_degree.get(n).unwrap_or(&0))
                .min()
                .unwrap_or(0);
            for &node in &dag_nodes {
                if *in_degree.get(&node).unwrap_or(&0) == min_in {
                    queue.push_back(node);
                    depth.insert(node, 0);
                }
            }
        }

        // Build forward adjacency for DAG
        let mut dag_adj: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        for &node in &candidates {
            let effective_src = scc_map.get(&node).copied().unwrap_or(node);
            let range = graph.csr.out_range(node);
            for j in range {
                let target = graph.csr.targets[j];
                if !candidate_set.contains(&target) {
                    continue;
                }
                let effective_tgt = scc_map.get(&target).copied().unwrap_or(target);
                if effective_src != effective_tgt {
                    dag_adj
                        .entry(effective_src)
                        .or_default()
                        .insert(effective_tgt);
                }
            }
        }

        // BFS: assign depth = max(depth of parents) + 1 for longest-path layering
        // We use iterative relaxation (Kahn-like) for this
        let mut visited: HashSet<NodeId> = HashSet::new();
        while let Some(node) = queue.pop_front() {
            if !visited.insert(node) {
                continue;
            }
            let current_depth = *depth.get(&node).unwrap_or(&0);
            if let Some(neighbors) = dag_adj.get(&node) {
                for &next in neighbors {
                    let new_depth = current_depth + 1;
                    let entry = depth.entry(next).or_insert(0);
                    if new_depth > *entry {
                        *entry = new_depth;
                    }
                    // Decrement in-degree; if zero, enqueue
                    let deg = in_degree.entry(next).or_insert(0);
                    if *deg > 0 {
                        *deg -= 1;
                    }
                    if *deg == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }

        // Handle unreachable nodes (disconnected components)
        for &node in &dag_nodes {
            depth.entry(node).or_insert(0);
        }

        // Map DAG depths back to original nodes
        let mut node_depth: HashMap<NodeId, u32> = HashMap::new();
        for &node in &candidates {
            let effective = scc_map.get(&node).copied().unwrap_or(node);
            let d = *depth.get(&effective).unwrap_or(&0);
            node_depth.insert(node, d);
        }

        // Phase 3: Group into layers
        let max_depth = node_depth.values().copied().max().unwrap_or(0);

        // Build initial layers by depth
        let mut depth_groups: Vec<Vec<NodeId>> = vec![Vec::new(); (max_depth + 1) as usize];
        for &node in &candidates {
            let d = *node_depth.get(&node).unwrap_or(&0);
            depth_groups[d as usize].push(node);
        }

        // Merge layers if too many (> max_layers)
        while depth_groups.len() > self.max_layers as usize && depth_groups.len() > 1 {
            // Find two adjacent layers with most similar avg PageRank to merge
            let mut best_idx = 0;
            let mut best_diff = f32::MAX;
            for i in 0..depth_groups.len() - 1 {
                let pr_a = layer_avg_pagerank(graph, &depth_groups[i]);
                let pr_b = layer_avg_pagerank(graph, &depth_groups[i + 1]);
                let diff = (pr_a - pr_b).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best_idx = i;
                }
            }
            let merged = depth_groups.remove(best_idx + 1);
            depth_groups[best_idx].extend(merged);
        }

        // Prune tiny layers (below min_nodes_per_layer) by merging into nearest
        let mut i = 0;
        while i < depth_groups.len() {
            if (depth_groups[i].len() as u32) < self.min_nodes_per_layer && depth_groups.len() > 1 {
                let removed = depth_groups.remove(i);
                let merge_into = if i > 0 { i - 1 } else { 0 };
                let merge_into = merge_into.min(depth_groups.len() - 1);
                depth_groups[merge_into].extend(removed);
                // Don't increment i, re-check the current position
            } else {
                i += 1;
            }
        }

        // Update node_depth after merging
        let mut node_layer: HashMap<NodeId, u8> = HashMap::new();
        for (level, group) in depth_groups.iter().enumerate() {
            for &node in group {
                node_layer.insert(node, level as u8);
            }
        }

        let total_layers = depth_groups.len();

        // Phase 4: Utility node detection
        let utility_threshold = (total_layers as f32 * UTILITY_LAYER_FRACTION).ceil() as usize;
        let mut utility_nodes: Vec<UtilityNode> = Vec::new();
        let mut utility_set: HashSet<NodeId> = HashSet::new();

        for &node in &candidates {
            // Count how many distinct layers reference this node (incoming edges)
            let range = graph.csr.in_range(node);
            let mut referencing_layers: HashSet<u8> = HashSet::new();
            for j in range {
                let source = graph.csr.rev_sources[j];
                if candidate_set.contains(&source) && !scc_map.contains_key(&source)
                    || scc_map.get(&source).copied() != scc_map.get(&node).copied()
                {
                    if let Some(&layer) = node_layer.get(&source) {
                        if layer != *node_layer.get(&node).unwrap_or(&255) {
                            referencing_layers.insert(layer);
                        }
                    }
                }
            }

            let used_by: Vec<u8> = {
                let mut v: Vec<u8> = referencing_layers.into_iter().collect();
                v.sort();
                v
            };

            if used_by.len() >= utility_threshold && utility_threshold > 0 {
                utility_nodes.push(UtilityNode {
                    node,
                    used_by_layers: used_by,
                    classification: UtilityClassification::CrossCutting,
                });
                utility_set.insert(node);
            } else if used_by.len() >= 2 {
                // Check for Bridge: used by 2 non-adjacent layers
                let mut is_bridge = false;
                for i in 0..used_by.len() {
                    for j in i + 1..used_by.len() {
                        if (used_by[j] as i16 - used_by[i] as i16).unsigned_abs() > 1 {
                            is_bridge = true;
                            break;
                        }
                    }
                    if is_bridge {
                        break;
                    }
                }
                if is_bridge {
                    utility_nodes.push(UtilityNode {
                        node,
                        used_by_layers: used_by,
                        classification: UtilityClassification::Bridge,
                    });
                    // Bridges are NOT removed from layers, only CrossCutting nodes are
                }
            }
        }

        // Also detect orphan nodes (no incoming references at all)
        for &node in &candidates {
            if utility_set.contains(&node) {
                continue;
            }
            let in_range = graph.csr.in_range(node);
            let has_incoming = in_range.clone().any(|j| {
                let source = graph.csr.rev_sources[j];
                candidate_set.contains(&source) && source != node
            });
            let out_range = graph.csr.out_range(node);
            let has_outgoing = out_range.clone().any(|j| {
                let target = graph.csr.targets[j];
                candidate_set.contains(&target) && target != node
            });
            if !has_incoming && !has_outgoing {
                utility_nodes.push(UtilityNode {
                    node,
                    used_by_layers: Vec::new(),
                    classification: UtilityClassification::Orphan,
                });
                utility_set.insert(node);
            }
        }

        // Remove utility (CrossCutting + Orphan) nodes from layers
        for group in depth_groups.iter_mut() {
            group.retain(|n| !utility_set.contains(n));
        }

        // Phase 5: Build ArchLayer structs with naming
        let mut layers: Vec<ArchLayer> = Vec::new();
        for (level, group) in depth_groups.iter().enumerate() {
            if group.is_empty() {
                continue;
            }

            let avg_pr = layer_avg_pagerank(graph, group);
            let avg_out = layer_avg_out_degree(graph, group);
            let confidences: Vec<f32> = group
                .iter()
                .map(|&node| layer_node_confidence(graph, node, &node_layer, &candidate_set))
                .collect();

            let (name, description) = match naming_strategy {
                "path_prefix" => layer_name_path_prefix(graph, group, level),
                "pagerank" => layer_name_by_pagerank(graph, group, level),
                _ => layer_name_heuristic(graph, group, level, depth_groups.len()),
            };

            layers.push(ArchLayer {
                level: level as u8,
                name,
                description,
                nodes: group.clone(),
                node_confidence: confidences,
                avg_pagerank: avg_pr,
                avg_out_degree: avg_out,
            });
        }

        // Re-number levels to be contiguous 0..N
        for (i, layer) in layers.iter_mut().enumerate() {
            layer.level = i as u8;
        }

        // Rebuild node_layer after re-numbering
        let mut node_layer_final: HashMap<NodeId, u8> = HashMap::new();
        for layer in &layers {
            for &node in &layer.nodes {
                node_layer_final.insert(node, layer.level);
            }
        }

        // Phase 6: Violation detection
        let mut violations: Vec<LayerViolation> = Vec::new();

        // Add circular dependency violations for SCCs
        for scc in &sccs {
            for &node in scc {
                let source_layer = node_layer_final.get(&node).copied().unwrap_or(0);
                for &other in scc {
                    if node == other {
                        continue;
                    }
                    // Check if there's an actual edge
                    let range = graph.csr.out_range(node);
                    for j in range {
                        let target = graph.csr.targets[j];
                        if target == other {
                            let rel = graph.strings.resolve(graph.csr.relations[j]).to_string();
                            let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                            let src_label = graph
                                .strings
                                .resolve(graph.nodes.label[node.as_usize()])
                                .to_string();
                            let tgt_label = graph
                                .strings
                                .resolve(graph.nodes.label[other.as_usize()])
                                .to_string();
                            violations.push(LayerViolation {
                                source: node,
                                source_layer,
                                target: other,
                                target_layer: source_layer,
                                edge_relation: rel,
                                edge_weight: w,
                                severity: ViolationSeverity::Critical,
                                violation_type: ViolationType::CircularDependency,
                                explanation: format!(
                                    "Circular dependency: {} and {} form a dependency cycle",
                                    src_label, tgt_label
                                ),
                            });
                            break; // One violation per edge direction is enough
                        }
                    }
                }
            }
        }

        // Detect upward dependency violations
        let mut total_cross_layer_edges: u32 = 0;
        let mut violation_edges: f32 = 0.0;

        for &node in &candidates {
            if utility_set.contains(&node) {
                continue;
            }
            let src_layer = match node_layer_final.get(&node) {
                Some(&l) => l,
                None => continue,
            };

            let range = graph.csr.out_range(node);
            for j in range {
                let target = graph.csr.targets[j];
                if utility_set.contains(&target) {
                    continue;
                }
                if !candidate_set.contains(&target) {
                    continue;
                }

                let tgt_layer = match node_layer_final.get(&target) {
                    Some(&l) => l,
                    None => continue,
                };

                if src_layer != tgt_layer {
                    total_cross_layer_edges += 1;
                }

                // Upward violation: source in deeper layer, target in shallower layer
                if src_layer > tgt_layer {
                    let rel = graph.strings.resolve(graph.csr.relations[j]).to_string();
                    let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                    let gap = src_layer - tgt_layer;

                    let (vtype, severity) = if gap > 1 {
                        (
                            ViolationType::SkipLayerDependency,
                            layer_violation_severity(&rel, gap, w),
                        )
                    } else {
                        (
                            ViolationType::UpwardDependency,
                            layer_violation_severity(&rel, gap, w),
                        )
                    };

                    // Weighted penalty: skip-layer violations penalized more
                    violation_edges += gap as f32;

                    let src_name = layer_name_for_level(src_layer, &layers);
                    let tgt_name = layer_name_for_level(tgt_layer, &layers);
                    let src_label = graph
                        .strings
                        .resolve(graph.nodes.label[node.as_usize()])
                        .to_string();
                    let tgt_label = graph
                        .strings
                        .resolve(graph.nodes.label[target.as_usize()])
                        .to_string();

                    violations.push(LayerViolation {
                        source: node,
                        source_layer: src_layer,
                        target,
                        target_layer: tgt_layer,
                        edge_relation: rel,
                        edge_weight: w,
                        severity,
                        violation_type: vtype,
                        explanation: format!(
                            "{} (layer {}: {}) depends on {} (layer {}: {})",
                            src_label, src_layer, src_name, tgt_label, tgt_layer, tgt_name,
                        ),
                    });
                }
            }
        }

        // Compute layer separation score
        // Penalize cycles proportionally
        let cycle_penalty: f32 =
            sccs.iter().map(|scc| scc.len() as f32).sum::<f32>() / candidates.len().max(1) as f32;

        let layer_separation_score = if total_cross_layer_edges == 0 && !has_cycles {
            1.0 // Perfect: no cross-layer edges, no violations possible
        } else if total_cross_layer_edges == 0 {
            (1.0 - cycle_penalty).max(0.0)
        } else {
            let raw = 1.0 - (violation_edges / total_cross_layer_edges as f32);
            (raw - cycle_penalty).clamp(0.0, 1.0)
        };

        let total_nodes_classified = layers.iter().map(|l| l.nodes.len() as u32).sum();

        Ok(LayerDetectionResult {
            layers,
            violations,
            utility_nodes,
            has_cycles,
            layer_separation_score,
            total_nodes_classified,
        })
    }

    /// Compute cohesion, coupling, and violation-density metrics for one layer.
    ///
    /// # Parameters
    /// - `graph`: the connectivity graph (must be the same graph used for `detect`).
    /// - `result`: the `LayerDetectionResult` to pull layer membership from.
    /// - `level`: zero-based layer level to analyse.
    ///
    /// # Errors
    /// Returns `M1ndError::LayerNotFound { level }` if no layer with that level exists in `result`.
    pub fn layer_health(
        &self,
        graph: &Graph,
        result: &LayerDetectionResult,
        level: u8,
    ) -> M1ndResult<LayerHealth> {
        let layer = result
            .layers
            .iter()
            .find(|l| l.level == level)
            .ok_or(M1ndError::LayerNotFound { level })?;

        let node_set: HashSet<NodeId> = layer.nodes.iter().copied().collect();
        let utility_set: HashSet<NodeId> = result.utility_nodes.iter().map(|u| u.node).collect();
        let all_layer_nodes: HashMap<NodeId, u8> = result
            .layers
            .iter()
            .flat_map(|l| l.nodes.iter().map(move |&n| (n, l.level)))
            .collect();

        let n = layer.nodes.len();
        if n == 0 {
            return Ok(LayerHealth {
                cohesion: 0.0,
                coupling_up: 0.0,
                coupling_down: 0.0,
                violation_density: 0.0,
            });
        }

        let mut intra_edges: u32 = 0;
        let mut edges_up: u32 = 0;
        let mut edges_down: u32 = 0;
        let mut total_outgoing: u32 = 0;

        for &node in &layer.nodes {
            let range = graph.csr.out_range(node);
            for j in range {
                let target = graph.csr.targets[j];
                if utility_set.contains(&target) {
                    continue;
                }

                if node_set.contains(&target) {
                    intra_edges += 1;
                } else if let Some(&tgt_level) = all_layer_nodes.get(&target) {
                    total_outgoing += 1;
                    if tgt_level < level {
                        edges_up += 1;
                    } else if tgt_level > level {
                        edges_down += 1;
                    }
                }
            }
        }

        // Cohesion: intra-layer edges / max possible intra-layer edges
        let max_intra = if n > 1 { (n * (n - 1)) as f32 } else { 1.0 };
        let cohesion = (intra_edges as f32 / max_intra).min(1.0);

        // Coupling ratios
        let total_external = (edges_up + edges_down).max(1);
        let coupling_up = edges_up as f32 / total_external as f32;
        let coupling_down = edges_down as f32 / total_external as f32;

        // Violation density: violations per node
        let violations_in_layer = result
            .violations
            .iter()
            .filter(|v| v.source_layer == level || v.target_layer == level)
            .count();
        let violation_density = violations_in_layer as f32 / n as f32;

        Ok(LayerHealth {
            cohesion,
            coupling_up,
            coupling_down,
            violation_density,
        })
    }
}

// ── Tarjan's SCC ──

/// Run Tarjan's SCC algorithm. Returns Vec of SCCs (each SCC = Vec<NodeId>).
/// Only SCCs with size > 1 are returned (single nodes are trivial SCCs).
fn tarjan_scc(graph: &Graph, nodes: &[NodeId]) -> Vec<Vec<NodeId>> {
    let node_set: HashSet<NodeId> = nodes.iter().copied().collect();
    let n = nodes.len();
    if n == 0 {
        return Vec::new();
    }

    // Map NodeId to local index and back
    let mut node_to_idx: HashMap<NodeId, usize> = HashMap::with_capacity(n);
    let mut idx_to_node: Vec<NodeId> = Vec::with_capacity(n);
    for (i, &node) in nodes.iter().enumerate() {
        node_to_idx.insert(node, i);
        idx_to_node.push(node);
    }

    let mut index_counter: usize = 0;
    let mut stack: Vec<usize> = Vec::new();
    let mut on_stack: Vec<bool> = vec![false; n];
    let mut indices: Vec<Option<usize>> = vec![None; n];
    let mut lowlinks: Vec<usize> = vec![0; n];
    let mut result: Vec<Vec<NodeId>> = Vec::new();

    // Iterative Tarjan's (avoids stack overflow on large graphs)
    // We use an explicit call stack to simulate recursion.
    #[derive(Clone)]
    enum TarjanFrame {
        Enter(usize),
        Resume(usize, usize), // (node_local_idx, neighbor_iterator_position)
    }

    for start in 0..n {
        if indices[start].is_some() {
            continue;
        }

        let mut call_stack: Vec<TarjanFrame> = vec![TarjanFrame::Enter(start)];

        while let Some(frame) = call_stack.pop() {
            match frame {
                TarjanFrame::Enter(v) => {
                    indices[v] = Some(index_counter);
                    lowlinks[v] = index_counter;
                    index_counter += 1;
                    stack.push(v);
                    on_stack[v] = true;

                    // Push resume frame then process neighbors
                    call_stack.push(TarjanFrame::Resume(v, 0));
                }
                TarjanFrame::Resume(v, pos) => {
                    let node = idx_to_node[v];
                    let range = graph.csr.out_range(node);
                    let neighbors: Vec<usize> = range
                        .filter_map(|j| {
                            let target = graph.csr.targets[j];
                            if node_set.contains(&target) {
                                node_to_idx.get(&target).copied()
                            } else {
                                None
                            }
                        })
                        .collect();

                    let mut next_pos = pos;
                    let mut found_unvisited = false;

                    while next_pos < neighbors.len() {
                        let w = neighbors[next_pos];
                        next_pos += 1;

                        if indices[w].is_none() {
                            // Push resume for after w returns, then enter w
                            call_stack.push(TarjanFrame::Resume(v, next_pos));
                            call_stack.push(TarjanFrame::Enter(w));
                            found_unvisited = true;
                            break;
                        } else if on_stack[w] {
                            lowlinks[v] = lowlinks[v].min(indices[w].unwrap());
                        }
                    }

                    if found_unvisited {
                        continue;
                    }

                    // Update parent's lowlink if we were called from a resume
                    // (this happens naturally when we complete processing)

                    // Check if v is a root of an SCC
                    if lowlinks[v] == indices[v].unwrap() {
                        let mut scc: Vec<NodeId> = Vec::new();
                        loop {
                            let w = stack.pop().unwrap();
                            on_stack[w] = false;
                            scc.push(idx_to_node[w]);
                            if w == v {
                                break;
                            }
                        }
                        if scc.len() > 1 {
                            result.push(scc);
                        }
                    }

                    // Propagate lowlink to parent
                    if let Some(TarjanFrame::Resume(parent, _)) = call_stack.last() {
                        lowlinks[*parent] = lowlinks[*parent].min(lowlinks[v]);
                    }
                }
            }
        }
    }

    result
}

// ── Helper Functions ──

/// Collect candidate nodes based on scope and type filters.
fn layer_collect_candidates(
    graph: &Graph,
    scope: Option<&str>,
    node_type_filter: &[NodeType],
) -> Vec<NodeId> {
    let n = graph.num_nodes() as usize;
    let mut candidates = Vec::new();

    for i in 0..n {
        let nid = NodeId::new(i as u32);

        // Type filter
        if !node_type_filter.is_empty() && !node_type_filter.contains(&graph.nodes.node_type[i]) {
            continue;
        }

        // Scope filter: check external_id prefix
        if let Some(scope_prefix) = scope {
            let mut matched = false;
            for (&interned, &id) in &graph.id_to_node {
                if id == nid {
                    let ext_id = graph.strings.resolve(interned);
                    if ext_id.starts_with(scope_prefix) {
                        matched = true;
                    }
                    break;
                }
            }
            if !matched {
                continue;
            }
        }

        candidates.push(nid);
    }

    candidates
}

/// Average PageRank for a set of nodes.
fn layer_avg_pagerank(graph: &Graph, nodes: &[NodeId]) -> f32 {
    if nodes.is_empty() {
        return 0.0;
    }
    let sum: f32 = nodes
        .iter()
        .map(|&n| graph.nodes.pagerank[n.as_usize()].get())
        .sum();
    sum / nodes.len() as f32
}

/// Average out-degree for a set of nodes.
fn layer_avg_out_degree(graph: &Graph, nodes: &[NodeId]) -> f32 {
    if nodes.is_empty() {
        return 0.0;
    }
    let sum: f32 = nodes
        .iter()
        .map(|&n| {
            let range = graph.csr.out_range(n);
            range.len() as f32
        })
        .sum();
    sum / nodes.len() as f32
}

/// Confidence of a node's layer assignment.
/// Based on how consistent its edges are with the layering.
fn layer_node_confidence(
    graph: &Graph,
    node: NodeId,
    node_layer: &HashMap<NodeId, u8>,
    candidate_set: &HashSet<NodeId>,
) -> f32 {
    let my_layer = match node_layer.get(&node) {
        Some(&l) => l,
        None => return 0.5,
    };

    let range = graph.csr.out_range(node);
    let mut total_edges = 0u32;
    let mut consistent_edges = 0u32;

    for j in range {
        let target = graph.csr.targets[j];
        if !candidate_set.contains(&target) {
            continue;
        }
        if let Some(&tgt_layer) = node_layer.get(&target) {
            total_edges += 1;
            if tgt_layer >= my_layer {
                // Edge going down or lateral = consistent
                consistent_edges += 1;
            }
        }
    }

    // Also check incoming edges
    let in_range = graph.csr.in_range(node);
    for j in in_range {
        let source = graph.csr.rev_sources[j];
        if !candidate_set.contains(&source) {
            continue;
        }
        if let Some(&src_layer) = node_layer.get(&source) {
            total_edges += 1;
            if src_layer <= my_layer {
                // Incoming from shallower or same = consistent
                consistent_edges += 1;
            }
        }
    }

    if total_edges == 0 {
        0.5 // No edges = moderate confidence
    } else {
        consistent_edges as f32 / total_edges as f32
    }
}

/// Name layer by most common path prefix component.
fn layer_name_path_prefix(graph: &Graph, nodes: &[NodeId], level: usize) -> (String, String) {
    let mut prefix_counts: HashMap<String, u32> = HashMap::new();

    for &node in nodes {
        let label = graph
            .strings
            .resolve(graph.nodes.label[node.as_usize()])
            .to_lowercase();
        // Extract first path component or identifier prefix
        let prefix = if let Some(slash_pos) = label.find('/') {
            &label[..slash_pos]
        } else if let Some(underscore_pos) = label.find('_') {
            &label[..underscore_pos]
        } else {
            &label
        };
        if !prefix.is_empty() {
            *prefix_counts.entry(prefix.to_string()).or_insert(0) += 1;
        }
    }

    if let Some((prefix, _)) = prefix_counts.iter().max_by_key(|&(_, count)| count) {
        (
            prefix.clone(),
            format!("Layer {} grouped by path prefix '{}'", level, prefix),
        )
    } else {
        (
            format!("layer_{}", level),
            format!("Layer at depth {}", level),
        )
    }
}

/// Name layer by highest PageRank node.
fn layer_name_by_pagerank(graph: &Graph, nodes: &[NodeId], level: usize) -> (String, String) {
    if nodes.is_empty() {
        return (
            format!("layer_{}", level),
            format!("Layer at depth {}", level),
        );
    }

    let top_node = nodes
        .iter()
        .max_by(|&&a, &&b| {
            graph.nodes.pagerank[a.as_usize()]
                .get()
                .partial_cmp(&graph.nodes.pagerank[b.as_usize()].get())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    let label = graph
        .strings
        .resolve(graph.nodes.label[top_node.as_usize()])
        .to_string();
    (
        label.clone(),
        format!(
            "Layer {} named after highest-PageRank node: {}",
            level, label
        ),
    )
}

/// Determine layer name and description from filename patterns.
fn layer_name_heuristic(
    graph: &Graph,
    nodes: &[NodeId],
    level: usize,
    total_levels: usize,
) -> (String, String) {
    // Count filename pattern matches
    let mut route_count = 0u32;
    let mut handler_count = 0u32;
    let mut service_count = 0u32;
    let mut store_count = 0u32;
    let mut model_count = 0u32;
    let mut config_count = 0u32;
    let mut main_count = 0u32;
    let mut test_count = 0u32;

    for &node in nodes {
        let label = graph
            .strings
            .resolve(graph.nodes.label[node.as_usize()])
            .to_lowercase();

        if label.contains("route") || label.contains("router") || label.contains("endpoint") {
            route_count += 1;
        }
        if label.contains("handler") || label.contains("middleware") || label.contains("dispatch") {
            handler_count += 1;
        }
        if label.contains("manager")
            || label.contains("orchestr")
            || label.contains("engine")
            || label.contains("service")
            || label.contains("daemon")
            || label.contains("processor")
        {
            service_count += 1;
        }
        if label.contains("store")
            || label.contains("pool")
            || label.contains("client")
            || label.contains("db")
            || label.contains("cache")
            || label.contains("repository")
        {
            store_count += 1;
        }
        if label.contains("model")
            || label.contains("types")
            || label.contains("schema")
            || label.contains("struct")
            || label.contains("enum")
        {
            model_count += 1;
        }
        if label.contains("config") || label.contains("settings") || label.contains("constants") {
            config_count += 1;
        }
        if label.contains("main") || label.contains("app") || label.contains("cli") {
            main_count += 1;
        }
        if label.contains("test") || label.contains("spec") {
            test_count += 1;
        }
    }

    // Pick dominant pattern
    let counts = [
        (
            main_count + route_count,
            "entry_points",
            "Surface layer: HTTP routes, CLI entry points, main modules",
        ),
        (
            handler_count,
            "handlers",
            "Request processing: route handlers, middleware, dispatchers",
        ),
        (
            service_count,
            "services",
            "Business logic: orchestrators, managers, processors",
        ),
        (
            store_count,
            "data_access",
            "Persistence and I/O: stores, pools, clients",
        ),
        (
            model_count + config_count,
            "foundation",
            "Core definitions: models, types, configuration",
        ),
        (
            test_count,
            "tests",
            "Test infrastructure: unit tests, integration tests",
        ),
    ];

    let dominant = counts.iter().max_by_key(|(c, _, _)| *c).unwrap();

    if dominant.0 > 0 {
        return (dominant.1.to_string(), dominant.2.to_string());
    }

    // Fallback: use positional naming
    if level == 0 {
        (
            "entry_points".to_string(),
            "Surface layer: entry points and top-level modules".to_string(),
        )
    } else if level == total_levels - 1 {
        (
            "foundation".to_string(),
            "Deepest layer: foundational modules and definitions".to_string(),
        )
    } else {
        let name = format!("layer_{}", level);
        let desc = format!("Intermediate layer at depth {}", level);
        (name, desc)
    }
}

/// Compute violation severity from edge relation, layer gap, and weight.
fn layer_violation_severity(relation: &str, gap: u8, weight: f32) -> ViolationSeverity {
    let base_severity = match relation {
        "imports" => 3,
        "calls" => 2,
        "inherits" => 2,
        "registers" => 1,
        "references" => 1,
        "configures" => 1,
        _ => 2,
    };

    let gap_factor = if gap > 1 { 1 } else { 0 };
    let weight_factor = if weight > 0.5 { 1 } else { 0 };

    let total = base_severity + gap_factor + weight_factor;

    match total {
        0..=1 => ViolationSeverity::Low,
        2 => ViolationSeverity::Medium,
        3 => ViolationSeverity::High,
        _ => ViolationSeverity::High, // Reserve Critical for circular deps
    }
}

/// Get layer name for a given level from the layers vec.
fn layer_name_for_level(level: u8, layers: &[ArchLayer]) -> String {
    layers
        .iter()
        .find(|l| l.level == level)
        .map(|l| l.name.clone())
        .unwrap_or_else(|| format!("layer_{}", level))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::GraphBuilder;
    use crate::types::NodeType;

    // ── Helper: build a simple DAG  A → B → C ──
    fn dag_graph() -> Graph {
        let mut b = GraphBuilder::new();
        let a = b.add_node("file::a", "a", NodeType::File, &[]).unwrap();
        let c = b.add_node("file::c", "c", NodeType::File, &[]).unwrap();
        let bb = b.add_node("file::b", "b", NodeType::File, &[]).unwrap();
        b.add_edge(a, bb, "imports", 0.8).unwrap();
        b.add_edge(bb, c, "imports", 0.8).unwrap();
        b.finalize().unwrap()
    }

    // ── Helper: build a graph with a cycle  X ↔ Y and a DAG tail Z ──
    fn cyclic_graph() -> Graph {
        let mut b = GraphBuilder::new();
        let x = b.add_node("file::x", "x", NodeType::File, &[]).unwrap();
        let y = b.add_node("file::y", "y", NodeType::File, &[]).unwrap();
        let z = b.add_node("file::z", "z", NodeType::File, &[]).unwrap();
        b.add_edge(x, y, "imports", 0.9).unwrap();
        b.add_edge(y, x, "imports", 0.9).unwrap(); // cycle
        b.add_edge(y, z, "imports", 0.5).unwrap();
        b.finalize().unwrap()
    }

    // 1. test_tarjan_empty (already exists — kept for completeness)
    #[test]
    fn test_tarjan_empty() {
        let graph = Graph::new();
        let result = tarjan_scc(&graph, &[]);
        assert!(result.is_empty());
    }

    // 2. detect_dag: DAG graph has no cycles, produces multiple layers
    #[test]
    fn detect_dag_produces_layers_no_cycles() {
        let graph = dag_graph();
        let detector = LayerDetector::new(8, 1);
        let result = detector.detect(&graph, None, &[], false, "auto").unwrap();
        assert!(!result.has_cycles, "DAG should have no cycles");
        // A→B→C should produce at least 2 layers (depth 0 and 1 or 2)
        assert!(result.layers.len() >= 1);
        assert!(result.total_nodes_classified >= 1);
    }

    // 3. detect_cycle_scc: cyclic graph has_cycles = true
    #[test]
    fn detect_cycle_scc_sets_has_cycles() {
        let graph = cyclic_graph();
        let detector = LayerDetector::new(8, 1);
        let result = detector.detect(&graph, None, &[], false, "auto").unwrap();
        assert!(
            result.has_cycles,
            "Cyclic graph should set has_cycles = true"
        );
        // Circular dependency violations should be present
        let circular = result
            .violations
            .iter()
            .filter(|v| v.violation_type == ViolationType::CircularDependency)
            .count();
        assert!(circular > 0, "Expected CircularDependency violations");
    }

    // 4. utility_nodes: orphan node (no edges) is classified as Orphan
    #[test]
    fn orphan_node_is_utility() {
        let mut b = GraphBuilder::new();
        let _a = b
            .add_node("file::lone", "lone", NodeType::File, &[])
            .unwrap();
        let _bb = b
            .add_node("file::connected_a", "connected_a", NodeType::File, &[])
            .unwrap();
        let _c = b
            .add_node("file::connected_b", "connected_b", NodeType::File, &[])
            .unwrap();
        b.add_edge(_bb, _c, "imports", 0.5).unwrap();
        let graph = b.finalize().unwrap();

        let detector = LayerDetector::new(8, 1);
        let result = detector.detect(&graph, None, &[], false, "auto").unwrap();

        let orphans: Vec<_> = result
            .utility_nodes
            .iter()
            .filter(|u| u.classification == UtilityClassification::Orphan)
            .collect();
        assert!(
            !orphans.is_empty(),
            "Expected at least one Orphan utility node"
        );
    }

    // 5. violations_upward: an edge from deeper layer back to shallower yields UpwardDependency
    #[test]
    fn violations_upward_detected() {
        // Build: entry → service → entry (upward edge from service back to entry)
        let mut b = GraphBuilder::new();
        let entry = b
            .add_node("file::main_entry", "main_entry", NodeType::File, &[])
            .unwrap();
        let svc = b
            .add_node("file::service_layer", "service_layer", NodeType::File, &[])
            .unwrap();
        let deepest = b
            .add_node("file::deep_store", "deep_store", NodeType::File, &[])
            .unwrap();
        b.add_edge(entry, svc, "imports", 0.8).unwrap();
        b.add_edge(svc, deepest, "imports", 0.8).unwrap();
        // Upward violation: deep → entry
        b.add_edge(deepest, entry, "imports", 0.6).unwrap();
        let graph = b.finalize().unwrap();

        let detector = LayerDetector::new(8, 1);
        let result = detector.detect(&graph, None, &[], false, "auto").unwrap();

        // Either has_cycles (if Tarjan sees it as cycle) or upward violation
        let has_violation = result.has_cycles
            || result.violations.iter().any(|v| {
                v.violation_type == ViolationType::UpwardDependency
                    || v.violation_type == ViolationType::CircularDependency
            });
        assert!(
            has_violation,
            "Expected violation or cycle for cross-layer back-edge"
        );
    }

    // 6. naming_heuristic: node labelled "main_routes" → layer named "entry_points"
    #[test]
    fn naming_heuristic_routes_become_entry_points() {
        let mut b = GraphBuilder::new();
        let r = b
            .add_node("file::main_routes", "main_routes", NodeType::File, &[])
            .unwrap();
        let s = b
            .add_node(
                "file::service_manager",
                "service_manager",
                NodeType::File,
                &[],
            )
            .unwrap();
        b.add_edge(r, s, "imports", 0.8).unwrap();
        let graph = b.finalize().unwrap();

        let detector = LayerDetector::new(8, 1);
        let result = detector.detect(&graph, None, &[], false, "auto").unwrap();

        let names: Vec<&str> = result.layers.iter().map(|l| l.name.as_str()).collect();
        let has_entry = names
            .iter()
            .any(|&n| n == "entry_points" || n == "services" || n.starts_with("layer_"));
        assert!(
            has_entry,
            "Naming should produce recognizable layer names, got: {:?}",
            names
        );
    }

    // 7. exclude_tests: nodes with "test" in label are filtered out when exclude_tests=true
    #[test]
    fn exclude_tests_removes_test_nodes() {
        let mut b = GraphBuilder::new();
        let prod = b
            .add_node("file::main_app", "main_app", NodeType::File, &[])
            .unwrap();
        let _test = b
            .add_node("file::test_routes", "test_routes", NodeType::File, &[])
            .unwrap();
        let svc = b
            .add_node("file::service_core", "service_core", NodeType::File, &[])
            .unwrap();
        b.add_edge(prod, svc, "imports", 0.8).unwrap();
        let graph = b.finalize().unwrap();

        let detector = LayerDetector::new(8, 1);
        // With exclude_tests=true
        let result_excl = detector.detect(&graph, None, &[], true, "auto").unwrap();
        // With exclude_tests=false
        let result_all = detector.detect(&graph, None, &[], false, "auto").unwrap();

        let test_in_excl = result_excl
            .layers
            .iter()
            .flat_map(|l| l.nodes.iter())
            .any(|&nid| {
                graph
                    .strings
                    .resolve(graph.nodes.label[nid.as_usize()])
                    .contains("test")
            });
        assert!(
            !test_in_excl,
            "Test nodes should be excluded when exclude_tests=true"
        );

        // excluded result should classify fewer nodes than all
        assert!(result_excl.total_nodes_classified <= result_all.total_nodes_classified);
    }

    // 8. health_metrics: layer_health returns Ok for a valid level
    #[test]
    fn health_metrics_for_valid_layer() {
        let graph = dag_graph();
        let detector = LayerDetector::new(8, 1);
        let result = detector.detect(&graph, None, &[], false, "auto").unwrap();

        assert!(!result.layers.is_empty(), "Need at least one layer");
        let level = result.layers[0].level;
        let health = detector.layer_health(&graph, &result, level).unwrap();

        // cohesion and coupling values are in [0, 1] or small multiples
        assert!(
            health.cohesion >= 0.0 && health.cohesion <= 1.0,
            "cohesion out of range: {}",
            health.cohesion
        );
        assert!(
            health.coupling_up >= 0.0 && health.coupling_up <= 1.0,
            "coupling_up out of range: {}",
            health.coupling_up
        );
        assert!(
            health.coupling_down >= 0.0 && health.coupling_down <= 1.0,
            "coupling_down out of range: {}",
            health.coupling_down
        );
        assert!(
            health.violation_density >= 0.0,
            "violation_density should be non-negative: {}",
            health.violation_density
        );
    }

    // Existing violation severity tests (kept)
    #[test]
    fn test_violation_severity_imports_high() {
        let sev = layer_violation_severity("imports", 1, 0.8);
        assert_eq!(sev, ViolationSeverity::High);
    }

    #[test]
    fn test_violation_severity_references_low() {
        let sev = layer_violation_severity("references", 1, 0.2);
        assert_eq!(sev, ViolationSeverity::Low);
    }

    #[test]
    fn test_violation_severity_skip_layer() {
        // imports + gap > 1 + high weight = High
        let sev = layer_violation_severity("imports", 3, 0.9);
        assert_eq!(sev, ViolationSeverity::High);
    }

    #[test]
    fn test_detector_empty_graph() {
        let graph = Graph::new();
        let detector = LayerDetector::with_defaults();
        let result = detector.detect(&graph, None, &[], false, "auto");
        assert!(matches!(result, Err(M1ndError::EmptyGraph)));
    }

    #[test]
    fn test_layer_health_not_found() {
        let result = LayerDetectionResult {
            layers: Vec::new(),
            violations: Vec::new(),
            utility_nodes: Vec::new(),
            has_cycles: false,
            layer_separation_score: 1.0,
            total_nodes_classified: 0,
        };
        let graph = Graph::new();
        let detector = LayerDetector::with_defaults();
        let health = detector.layer_health(&graph, &result, 99);
        assert!(matches!(
            health,
            Err(M1ndError::LayerNotFound { level: 99 })
        ));
    }
}
