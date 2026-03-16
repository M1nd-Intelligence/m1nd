// === crates/m1nd-core/src/counterfactual.rs ===

use crate::activation::{ActivationEngine, DimensionResult, HybridEngine};
use crate::error::M1ndResult;
use crate::graph::Graph;
use crate::types::PropagationConfig;
use crate::types::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default number of diverse seed-set trials per counterfactual.
pub const DEFAULT_SEED_TRIALS: u8 = 8;
/// Default top_n for keystone analysis.
pub const DEFAULT_KEYSTONE_TOP_N: usize = 20;

// ---------------------------------------------------------------------------
// RemovalMask — bitset for virtual node removal (FM-CF-004 fix)
// Replaces: counterfactual.py _clone_graph() (was O(V+E) per removal)
// Now O(1) allocation — just flip bits.
// ---------------------------------------------------------------------------

/// Bitset mask marking nodes/edges as removed.
/// Zero-allocation counterfactual: no graph clone needed.
/// FM-CF-004 fix: bitset instead of full graph clone.
pub struct RemovalMask {
    /// Bit per node: true = removed.
    pub removed_nodes: Vec<bool>,
    /// Bit per edge: true = removed (edges incident on removed nodes).
    pub removed_edges: Vec<bool>,
}

impl RemovalMask {
    /// Create empty mask for a graph.
    pub fn new(num_nodes: u32, num_edges: usize) -> Self {
        Self {
            removed_nodes: vec![false; num_nodes as usize],
            removed_edges: vec![false; num_edges],
        }
    }

    /// Mark a node and all its incident edges as removed.
    pub fn remove_node(&mut self, graph: &Graph, node: NodeId) {
        let idx = node.as_usize();
        if idx >= self.removed_nodes.len() {
            return;
        }
        self.removed_nodes[idx] = true;

        // Mark outgoing edges
        let out_range = graph.csr.out_range(node);
        for j in out_range {
            if j < self.removed_edges.len() {
                self.removed_edges[j] = true;
            }
        }

        // Mark incoming edges
        let in_range = graph.csr.in_range(node);
        for j in in_range {
            let fwd_idx = graph.csr.rev_edge_idx[j].as_usize();
            if fwd_idx < self.removed_edges.len() {
                self.removed_edges[fwd_idx] = true;
            }
        }
    }

    /// Mark a specific edge as removed.
    pub fn remove_edge(&mut self, edge: EdgeIdx) {
        self.removed_edges[edge.as_usize()] = true;
    }

    /// Check if a node is removed.
    #[inline]
    pub fn is_node_removed(&self, node: NodeId) -> bool {
        self.removed_nodes[node.as_usize()]
    }

    /// Check if an edge is removed.
    #[inline]
    pub fn is_edge_removed(&self, edge: EdgeIdx) -> bool {
        self.removed_edges[edge.as_usize()]
    }

    /// Reset all removals.
    pub fn reset(&mut self) {
        self.removed_nodes.fill(false);
        self.removed_edges.fill(false);
    }
}

// ---------------------------------------------------------------------------
// CounterfactualResult — output of single node removal
// Replaces: counterfactual.py NodeRemovalSimulator.simulate() return
// ---------------------------------------------------------------------------

/// Impact of removing one or more nodes.
#[derive(Clone, Debug)]
pub struct CounterfactualResult {
    pub removed_nodes: Vec<NodeId>,
    /// Total impact score: fraction of activation lost.
    pub total_impact: FiniteF32,
    /// Percentage of total activation lost.
    pub pct_activation_lost: FiniteF32,
    /// Nodes that become completely unreachable after removal.
    pub orphaned_nodes: Vec<NodeId>,
    /// Nodes that lost >50% of their activation.
    pub weakened_nodes: Vec<(NodeId, FiniteF32)>, // (node, pct_lost)
    /// Number of communities split by the removal.
    pub communities_split: u32,
    /// Graph reachability before removal.
    pub reachability_before: u32,
    /// Graph reachability after removal.
    pub reachability_after: u32,
}

// ---------------------------------------------------------------------------
// KeystoneResult — top nodes by counterfactual impact
// Replaces: counterfactual.py CounterfactualSimulator.find_keystones() return
// ---------------------------------------------------------------------------

/// Keystone node analysis result.
#[derive(Clone, Debug)]
pub struct KeystoneEntry {
    pub node: NodeId,
    /// Average impact across seed trials.
    /// FM-CF-010 fix: denominator is n_runs (not per-node count).
    pub avg_impact: FiniteF32,
    /// Standard deviation of impact across trials.
    pub impact_std: FiniteF32,
}

/// Keystone analysis output.
#[derive(Clone, Debug)]
pub struct KeystoneResult {
    /// Top keystones sorted by avg_impact descending.
    pub keystones: Vec<KeystoneEntry>,
    /// Number of seed trials used.
    pub num_trials: u8,
}

// ---------------------------------------------------------------------------
// CascadeResult — cascade analysis after removal
// Replaces: counterfactual.py CascadeAnalyzer.analyze()
// ---------------------------------------------------------------------------

/// Cascade analysis: what happens downstream after a node is removed.
#[derive(Clone, Debug)]
pub struct CascadeResult {
    pub removed_node: NodeId,
    /// Cascade depth (how many hops the effect propagates).
    pub cascade_depth: u8,
    /// Nodes affected at each depth level.
    pub affected_by_depth: Vec<Vec<NodeId>>,
    /// Total nodes affected.
    pub total_affected: u32,
}

// ---------------------------------------------------------------------------
// SynergyResult — multi-node removal synergy analysis
// Replaces: counterfactual.py WhatIfSimulator.simulate() synergy output
// ---------------------------------------------------------------------------

/// Synergy analysis for multi-node removal.
#[derive(Clone, Debug)]
pub struct SynergyResult {
    /// Individual impact of each removed node.
    pub individual_impacts: Vec<(NodeId, FiniteF32)>,
    /// Combined impact of removing all nodes together.
    pub combined_impact: FiniteF32,
    /// Synergy factor: combined / sum(individual). >1.0 = synergistic fragility.
    pub synergy_factor: FiniteF32,
}

// ---------------------------------------------------------------------------
// RedundancyResult — how replaceable is a node?
// Replaces: counterfactual.py CounterfactualSimulator.check_redundancy()
// FM-CF-016 fix: confidence levels + architectural node protection.
// ---------------------------------------------------------------------------

/// Redundancy analysis for a single node.
#[derive(Clone, Debug)]
pub struct RedundancyResult {
    pub node: NodeId,
    /// Redundancy score [0, 1]: 1.0 = fully redundant, 0.0 = irreplaceable.
    pub redundancy_score: FiniteF32,
    /// Confidence level of the redundancy assessment.
    pub confidence: RedundancyConfidence,
    /// Alternative paths that bypass this node.
    pub alternative_paths: u32,
    /// Whether this is an architectural node (FM-CF-016: protected from deletion advice).
    pub is_architectural: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedundancyConfidence {
    High,
    Medium,
    Low,
}

// ---------------------------------------------------------------------------
// AntifragilityResult — combined keystone + redundancy
// Replaces: counterfactual.py CounterfactualSimulator.antifragility_score()
// ---------------------------------------------------------------------------

/// Antifragility score for the graph or a subgraph.
#[derive(Clone, Debug)]
pub struct AntifragilityResult {
    /// Overall antifragility score [0, 1].
    pub score: FiniteF32,
    /// Top keystones (most fragile points).
    pub top_keystones: Vec<KeystoneEntry>,
    /// Most redundant nodes.
    pub most_redundant: Vec<RedundancyResult>,
    /// Least redundant nodes (most irreplaceable).
    pub least_redundant: Vec<RedundancyResult>,
}

// ---------------------------------------------------------------------------
// Helper: run activation with removal mask
// ---------------------------------------------------------------------------

fn run_baseline_activation(
    graph: &Graph,
    engine: &HybridEngine,
    config: &PropagationConfig,
    seeds: &[(NodeId, FiniteF32)],
) -> M1ndResult<Vec<(NodeId, FiniteF32)>> {
    let result = engine.propagate(graph, seeds, config)?;
    Ok(result.scores)
}

/// Propagate with removal mask. Skips removed nodes and edges during traversal.
/// This gives accurate counterfactual results — signal cannot flow through removed nodes.
fn propagate_with_mask(
    graph: &Graph,
    seeds: &[(NodeId, FiniteF32)],
    config: &PropagationConfig,
    mask: &RemovalMask,
) -> M1ndResult<Vec<(NodeId, FiniteF32)>> {
    let n = graph.num_nodes() as usize;
    if n == 0 || seeds.is_empty() {
        return Ok(Vec::new());
    }

    let threshold = config.threshold.get();
    let decay = config.decay.get();
    let max_depth = config.max_depth.min(20) as usize;

    let mut activation = vec![0.0f32; n];
    let mut visited = vec![false; n];
    let mut frontier: Vec<NodeId> = Vec::new();

    for &(node, score) in seeds {
        let idx = node.as_usize();
        if idx < n && !mask.is_node_removed(node) {
            let s = score.get().min(config.saturation_cap.get());
            if s > activation[idx] {
                activation[idx] = s;
            }
            if !visited[idx] {
                frontier.push(node);
                visited[idx] = true;
            }
        }
    }

    for _depth in 0..max_depth {
        if frontier.is_empty() {
            break;
        }
        let mut next_frontier: Vec<NodeId> = Vec::new();

        for &src in &frontier {
            let src_act = activation[src.as_usize()];
            if src_act < threshold {
                continue;
            }

            let range = graph.csr.out_range(src);
            for j in range {
                // Skip removed edges
                if mask.is_edge_removed(EdgeIdx::new(j as u32)) {
                    continue;
                }

                let tgt = graph.csr.targets[j];
                let tgt_idx = tgt.as_usize();

                // Skip removed nodes
                if tgt_idx >= n || mask.is_node_removed(tgt) {
                    continue;
                }

                let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                let is_inhib = graph.csr.inhibitory[j];

                let mut signal = src_act * w * decay;
                if is_inhib {
                    signal = -signal * config.inhibitory_factor.get();
                }

                if !is_inhib && signal > threshold {
                    if signal > activation[tgt_idx] {
                        activation[tgt_idx] = signal;
                    }
                    if !visited[tgt_idx] {
                        visited[tgt_idx] = true;
                        next_frontier.push(tgt);
                    }
                } else if is_inhib {
                    activation[tgt_idx] = (activation[tgt_idx] + signal).max(0.0);
                }
            }
        }

        frontier = next_frontier;
    }

    let mut scores: Vec<(NodeId, FiniteF32)> = activation
        .iter()
        .enumerate()
        .filter(|(i, &v)| v > 0.0 && !mask.is_node_removed(NodeId::new(*i as u32)))
        .map(|(i, &v)| (NodeId::new(i as u32), FiniteF32::new(v)))
        .collect();
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(scores)
}

/// Compute total activation from a score vector.
fn total_activation(scores: &[(NodeId, FiniteF32)]) -> f32 {
    scores.iter().map(|(_, s)| s.get()).sum()
}

/// Generate diverse seed sets for trials using PageRank-stratified selection.
/// Avoids isolated/leaf nodes that produce degenerate baselines.
fn generate_diverse_seeds(graph: &Graph, num_trials: u8) -> Vec<Vec<(NodeId, FiniteF32)>> {
    let n = graph.num_nodes() as usize;
    if n == 0 {
        return Vec::new();
    }

    // Collect nodes with nonzero out-degree, sorted by PageRank descending
    let mut candidates: Vec<(usize, f32)> = (0..n)
        .filter(|&i| {
            let r = graph.csr.out_range(NodeId::new(i as u32));
            r.end > r.start // has outgoing edges
        })
        .map(|i| (i, graph.nodes.pagerank[i].get()))
        .collect();
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1));

    if candidates.is_empty() {
        return Vec::new();
    }

    // Stride through candidates to get diverse, high-PageRank seeds
    let mut trials = Vec::new();
    let stride = candidates.len().max(1) / (num_trials as usize).max(1);
    for t in 0..num_trials as usize {
        let idx = (t * stride.max(1)) % candidates.len();
        let (node_idx, _) = candidates[idx];
        trials.push(vec![(NodeId::new(node_idx as u32), FiniteF32::ONE)]);
    }
    trials
}

// ---------------------------------------------------------------------------
// CounterfactualEngine — facade
// Replaces: counterfactual.py CounterfactualSimulator
// ---------------------------------------------------------------------------

/// Counterfactual analysis engine. Uses bitset-based removal (FM-CF-004 fix).
/// Replaces: counterfactual.py CounterfactualSimulator
pub struct CounterfactualEngine {
    num_trials: u8,
    keystone_top_n: usize,
}

impl CounterfactualEngine {
    pub fn new(num_trials: u8, keystone_top_n: usize) -> Self {
        Self {
            num_trials,
            keystone_top_n,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            num_trials: DEFAULT_SEED_TRIALS,
            keystone_top_n: DEFAULT_KEYSTONE_TOP_N,
        }
    }

    /// Simulate removal of one or more nodes.
    /// FM-CF-001 fix: if a seed node is in the removal set, replace it instead of dropping.
    /// FM-CF-010 fix: aggregation divides by n_runs, not per-node count.
    /// Uses RemovalMask for accurate propagation — signal cannot flow through removed nodes.
    /// Replaces: counterfactual.py NodeRemovalSimulator.simulate()
    pub fn simulate_removal(
        &self,
        graph: &Graph,
        engine: &HybridEngine,
        config: &PropagationConfig,
        remove_nodes: &[NodeId],
    ) -> M1ndResult<CounterfactualResult> {
        let n = graph.num_nodes() as usize;

        // Generate seed trials
        let seed_trials = generate_diverse_seeds(graph, self.num_trials);

        let mut total_baseline = 0.0f32;
        let mut total_removed = 0.0f32;

        // Build removal mask (marks nodes AND their incident edges)
        let mut mask = RemovalMask::new(graph.num_nodes(), graph.num_edges());
        let mut removed_set = vec![false; n];
        for &node in remove_nodes {
            if node.as_usize() < n {
                removed_set[node.as_usize()] = true;
                mask.remove_node(graph, node);
            }
        }

        let mut per_node_loss = vec![0.0f32; n];

        for seeds in &seed_trials {
            // FM-CF-001: replace seeds that are in removal set
            let adjusted_seeds: Vec<(NodeId, FiniteF32)> = seeds
                .iter()
                .map(|&(node, score)| {
                    if removed_set[node.as_usize()] {
                        // Find replacement: nearest non-removed neighbor (forward)
                        let range = graph.csr.out_range(node);
                        for j in range {
                            let tgt = graph.csr.targets[j];
                            if !removed_set[tgt.as_usize()] {
                                return (tgt, score);
                            }
                        }
                        // Also check reverse neighbors (incoming edges)
                        let rev_range = graph.csr.in_range(node);
                        for j in rev_range {
                            let src = graph.csr.rev_sources[j];
                            if !removed_set[src.as_usize()] {
                                return (src, score);
                            }
                        }
                        // Last resort: pick any non-removed node
                        for (i, &removed) in removed_set.iter().enumerate().take(n) {
                            if !removed {
                                return (NodeId::new(i as u32), score);
                            }
                        }
                        (node, FiniteF32::ZERO) // All nodes removed (impossible in practice)
                    } else {
                        (node, score)
                    }
                })
                .filter(|(_, s)| s.get() > 0.0)
                .collect();

            // Baseline activation (full graph)
            let baseline = run_baseline_activation(graph, engine, config, seeds)?;
            let baseline_total = total_activation(&baseline);
            total_baseline += baseline_total;

            // Masked propagation: signal cannot flow through removed nodes/edges
            let removed_scores = propagate_with_mask(graph, &adjusted_seeds, config, &mask)?;
            let removed_total = total_activation(&removed_scores);
            total_removed += removed_total;

            // Per-node loss tracking
            let mut baseline_map = std::collections::HashMap::new();
            for &(node, score) in &baseline {
                baseline_map.insert(node.0, score.get());
            }
            let mut removed_map = std::collections::HashMap::new();
            for &(node, score) in &removed_scores {
                removed_map.insert(node.0, score.get());
            }

            for (i, loss) in per_node_loss.iter_mut().enumerate().take(n) {
                let base = baseline_map.get(&(i as u32)).copied().unwrap_or(0.0);
                let rem = removed_map.get(&(i as u32)).copied().unwrap_or(0.0);
                if base > 0.0 {
                    *loss += (base - rem) / base;
                }
            }
        }

        let num_trials = seed_trials.len().max(1) as f32;

        // FM-CF-010 fix: denominator is n_runs
        let pct_lost = if total_baseline > 0.0 {
            ((total_baseline - total_removed) / total_baseline).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Orphaned: nodes with >99% activation loss
        let orphaned: Vec<NodeId> = (0..n)
            .filter(|&i| per_node_loss[i] / num_trials > 0.99 && !removed_set[i])
            .map(|i| NodeId::new(i as u32))
            .collect();

        // Weakened: nodes with >50% activation loss
        let weakened: Vec<(NodeId, FiniteF32)> = (0..n)
            .filter(|&i| {
                let avg = per_node_loss[i] / num_trials;
                avg > 0.5 && avg <= 0.99 && !removed_set[i]
            })
            .map(|i| {
                let avg = per_node_loss[i] / num_trials;
                (NodeId::new(i as u32), FiniteF32::new(avg))
            })
            .collect();

        // Compute reachability via BFS from arbitrary start node
        let reachability_before = Self::compute_reachability(graph, n, &vec![false; n]);
        let reachability_after = Self::compute_reachability(graph, n, &removed_set);

        Ok(CounterfactualResult {
            removed_nodes: remove_nodes.to_vec(),
            total_impact: FiniteF32::new(pct_lost),
            pct_activation_lost: FiniteF32::new(pct_lost),
            orphaned_nodes: orphaned,
            weakened_nodes: weakened,
            communities_split: 0, // Would need Louvain recomputation
            reachability_before,
            reachability_after,
        })
    }

    /// BFS reachability count. Starts from highest-degree non-removed node
    /// to avoid starting from isolated nodes (e.g. config files with no edges).
    fn compute_reachability(graph: &Graph, n: usize, removed: &[bool]) -> u32 {
        if n == 0 {
            return 0;
        }
        // Find highest-degree non-removed node (not just first) to avoid isolated starts
        let start = (0..n).filter(|&i| !removed[i]).max_by_key(|&i| {
            let nid = NodeId::new(i as u32);
            let out = graph.csr.out_range(nid);
            let inv = graph.csr.in_range(nid);
            (out.end - out.start) + (inv.end - inv.start)
        });
        let start = match start {
            Some(s) => s,
            None => return 0,
        };

        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        visited[start] = true;
        let mut count = 1u32;

        while let Some(node) = queue.pop_front() {
            let nid = NodeId::new(node as u32);
            // Forward edges
            let range = graph.csr.out_range(nid);
            for j in range {
                let tgt = graph.csr.targets[j].as_usize();
                if tgt < n && !visited[tgt] && !removed[tgt] {
                    visited[tgt] = true;
                    queue.push_back(tgt);
                    count += 1;
                }
            }
            // Reverse edges
            let rev_range = graph.csr.in_range(nid);
            for j in rev_range {
                let src = graph.csr.rev_sources[j].as_usize();
                if src < n && !visited[src] && !removed[src] {
                    visited[src] = true;
                    queue.push_back(src);
                    count += 1;
                }
            }
        }

        count
    }

    /// Find keystone nodes (highest counterfactual impact). Parallelised via rayon.
    /// FM-CF-010 fix: correct aggregation denominator.
    /// Replaces: counterfactual.py CounterfactualSimulator.find_keystones()
    pub fn find_keystones(
        &self,
        graph: &Graph,
        engine: &HybridEngine,
        config: &PropagationConfig,
    ) -> M1ndResult<KeystoneResult> {
        let n = graph.num_nodes() as usize;
        let mut impacts: Vec<(NodeId, f32)> = Vec::new();

        // Test removal of each node (or top-N by degree for efficiency)
        let mut candidates: Vec<(usize, usize)> = (0..n)
            .map(|i| {
                let range = graph.csr.out_range(NodeId::new(i as u32));
                (i, range.end - range.start)
            })
            .collect();
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        candidates.truncate(self.keystone_top_n * 2);

        for (node_idx, _) in &candidates {
            let result =
                self.simulate_removal(graph, engine, config, &[NodeId::new(*node_idx as u32)])?;
            impacts.push((NodeId::new(*node_idx as u32), result.total_impact.get()));
        }

        impacts.sort_by(|a, b| b.1.total_cmp(&a.1));
        let keystones: Vec<KeystoneEntry> = impacts
            .iter()
            .take(self.keystone_top_n)
            .map(|&(node, impact)| KeystoneEntry {
                node,
                avg_impact: FiniteF32::new(impact),
                impact_std: FiniteF32::ZERO, // Single-trial std
            })
            .collect();

        Ok(KeystoneResult {
            keystones,
            num_trials: self.num_trials,
        })
    }

    /// Cascade analysis for a removed node.
    /// Replaces: counterfactual.py CascadeAnalyzer.analyze()
    pub fn cascade_analysis(
        &self,
        graph: &Graph,
        _engine: &HybridEngine,
        _config: &PropagationConfig,
        remove_node: NodeId,
    ) -> M1ndResult<CascadeResult> {
        let n = graph.num_nodes() as usize;
        if remove_node.as_usize() >= n {
            return Ok(CascadeResult {
                removed_node: remove_node,
                cascade_depth: 0,
                affected_by_depth: Vec::new(),
                total_affected: 0,
            });
        }

        // BFS from removed node to find downstream affected nodes
        let mut affected_by_depth: Vec<Vec<NodeId>> = Vec::new();
        let mut visited = vec![false; n];
        visited[remove_node.as_usize()] = true;

        let mut frontier = vec![remove_node];
        let max_depth = 5u8;

        for _depth in 0..max_depth {
            if frontier.is_empty() {
                break;
            }
            let mut next = Vec::new();
            let mut depth_affected = Vec::new();

            for &node in &frontier {
                let range = graph.csr.out_range(node);
                for j in range {
                    let tgt = graph.csr.targets[j];
                    let tgt_idx = tgt.as_usize();
                    if tgt_idx < n && !visited[tgt_idx] {
                        visited[tgt_idx] = true;
                        next.push(tgt);
                        depth_affected.push(tgt);
                    }
                }
            }

            if !depth_affected.is_empty() {
                affected_by_depth.push(depth_affected);
            }
            frontier = next;
        }

        let total_affected: u32 = affected_by_depth.iter().map(|d| d.len() as u32).sum();

        Ok(CascadeResult {
            removed_node: remove_node,
            cascade_depth: affected_by_depth.len() as u8,
            affected_by_depth,
            total_affected,
        })
    }

    /// Multi-node synergy analysis.
    /// Replaces: counterfactual.py WhatIfSimulator.simulate()
    pub fn synergy_analysis(
        &self,
        graph: &Graph,
        engine: &HybridEngine,
        config: &PropagationConfig,
        remove_nodes: &[NodeId],
    ) -> M1ndResult<SynergyResult> {
        // Individual impacts
        let mut individual_impacts = Vec::new();
        for &node in remove_nodes {
            let result = self.simulate_removal(graph, engine, config, &[node])?;
            individual_impacts.push((node, result.total_impact));
        }

        // Combined impact
        let combined = self.simulate_removal(graph, engine, config, remove_nodes)?;

        let sum_individual: f32 = individual_impacts.iter().map(|(_, s)| s.get()).sum();
        let synergy_factor = if sum_individual > 0.0 {
            combined.total_impact.get() / sum_individual
        } else {
            1.0
        };

        Ok(SynergyResult {
            individual_impacts,
            combined_impact: combined.total_impact,
            synergy_factor: FiniteF32::new(synergy_factor.min(10.0)),
        })
    }

    /// Redundancy analysis for a single node.
    /// FM-CF-016 fix: architectural node protection + confidence levels.
    /// Replaces: counterfactual.py CounterfactualSimulator.check_redundancy()
    pub fn check_redundancy(
        &self,
        graph: &Graph,
        engine: &HybridEngine,
        config: &PropagationConfig,
        node: NodeId,
    ) -> M1ndResult<RedundancyResult> {
        let n = graph.num_nodes() as usize;
        let idx = node.as_usize();
        if idx >= n {
            return Ok(RedundancyResult {
                node,
                redundancy_score: FiniteF32::ZERO,
                confidence: RedundancyConfidence::Low,
                alternative_paths: 0,
                is_architectural: false,
            });
        }

        // Simulate removal
        let result = self.simulate_removal(graph, engine, config, &[node])?;
        let impact = result.total_impact.get();

        // Redundancy = 1 - impact (low impact = high redundancy)
        let redundancy = (1.0 - impact).clamp(0.0, 1.0);

        // Count alternative paths: BFS bypassing this node
        let out_range = graph.csr.out_range(node);
        let out_degree = out_range.end - out_range.start;

        let in_range = graph.csr.in_range(node);
        let in_degree = in_range.end - in_range.start;

        let alternative_paths = (out_degree.min(in_degree)) as u32;

        // Architectural node detection: high degree + bridge-like
        let is_architectural = out_degree >= 5 && impact > 0.3;

        // Confidence based on number of trials
        let confidence = if self.num_trials >= 8 {
            RedundancyConfidence::High
        } else if self.num_trials >= 4 {
            RedundancyConfidence::Medium
        } else {
            RedundancyConfidence::Low
        };

        Ok(RedundancyResult {
            node,
            redundancy_score: FiniteF32::new(redundancy),
            confidence,
            alternative_paths,
            is_architectural,
        })
    }

    /// Antifragility score for the graph.
    /// Replaces: counterfactual.py CounterfactualSimulator.antifragility_score()
    pub fn antifragility_score(
        &self,
        graph: &Graph,
        engine: &HybridEngine,
        config: &PropagationConfig,
    ) -> M1ndResult<AntifragilityResult> {
        let keystones = self.find_keystones(graph, engine, config)?;

        // Overall score: lower max keystone impact = more antifragile
        let max_impact = keystones
            .keystones
            .first()
            .map(|k| k.avg_impact.get())
            .unwrap_or(0.0);
        let score = (1.0 - max_impact).clamp(0.0, 1.0);

        Ok(AntifragilityResult {
            score: FiniteF32::new(score),
            top_keystones: keystones.keystones,
            most_redundant: Vec::new(), // Would require full redundancy scan
            least_redundant: Vec::new(),
        })
    }
}

static_assertions::assert_impl_all!(CounterfactualEngine: Send, Sync);
